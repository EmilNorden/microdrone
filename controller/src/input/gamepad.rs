use core::fmt::Debug;
use bt_hci::controller::ExternalController;
use bt_hci::param::{AddrKind, BdAddr};
use embassy_futures::join::{join, join3};
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use embassy_time::{Duration, Timer};
use esp_hal::efuse::Efuse;
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::EspWifiController;
use heapless::Vec;
use trouble_host::attribute::{Characteristic, Uuid};
use trouble_host::connection::{ConnectConfig, ScanConfig};
use trouble_host::gatt::GattClient;
use trouble_host::prelude::DefaultPacketPool;
use trouble_host::{Address, Controller, Host, HostResources};
use crate::input::pilot_controller;

pub struct GamepadState {
    pub dpad: u8,
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub right_trigger: u8,
    pub left_trigger: u8,
    pub buttons: u8,
}

pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    L4,
    R4
}

impl GamepadState {
    pub fn button_pressed(&self, button: GamepadButton) -> bool {
        match button {
            GamepadButton::A => (self.buttons & 1) != 0,
            GamepadButton::B => (self.buttons & 2) != 0,
            GamepadButton::X => (self.buttons & 8) != 0,
            GamepadButton::Y => (self.buttons & 16) != 0,
            GamepadButton::LB => (self.buttons & 64) != 0,
            GamepadButton::RB => (self.buttons & 128) != 0,
            GamepadButton::L4 => (self.buttons & 4) != 0,
            GamepadButton::R4 => (self.buttons & 32) != 0,
        }
    }
}

impl Debug for GamepadState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GamepadState")
            .field("dpad", &self.dpad)
            .field("left_stick_x", &self.left_stick_x)
            .field("left_stick_y", &self.left_stick_y)
            .field("right_stick_x", &self.right_stick_x)
            .field("right_stick_y", &self.right_stick_y)
            .field("left_trigger", &self.left_trigger)
            .field("right_trigger", &self.right_trigger)
            .field("buttons", &self.buttons)
            .finish()
    }
}

impl GamepadState {
    pub fn from_hid_report(report: &[u8]) -> Self {
        GamepadState {
            dpad: report[0],
            left_stick_x: report[1],
            left_stick_y: report[2],
            right_stick_x: report[3],
            right_stick_y: report[4],
            right_trigger: report[5],
            left_trigger: report[6],
            buttons: report[7],
        }
    }
}

//TODO: Kolla dokumentation for EspWifiController. Det kan hända att det går lite långsamt pga extensiv loggning

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 3; // Signal + att + CoC

fn get_bluetooth_mac_address() -> [u8; 6] {
    // Read_base_mac_address gives the base MAC address. To get BT MAC, I have to offset it by 2.
    let mut base_mac = Efuse::read_base_mac_address();
    base_mac[5] = base_mac[5].wrapping_add(2);

    base_mac
}

pub async fn run(wifi: &'static EspWifiController<'static>, bt: esp_hal::peripherals::BT<'static>) {

    esp_println::println!("Init Bluetooth...");
    let transport = BleConnector::new(wifi, bt);
    let ble_controller = ExternalController::<_, 20>::new(transport);

    let address: Address = Address::random(get_bluetooth_mac_address());

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(ble_controller, &mut resources).set_random_address(address);

    let Host {
        mut central,
        mut runner,
        ..
    } = stack.build();

    loop {
        let cancel_watch: Watch<CriticalSectionRawMutex, (), 4> = Watch::new();

        let cancel_sender = cancel_watch.sender();

        let runner_task = async {
            let mut cancel_signal = cancel_watch.receiver().unwrap();
            'outer: loop {
                let fut = runner.run();
                match select(async { cancel_signal.changed().await }, fut).await {
                    Either::First(_) => {
                        esp_println::println!("Cancelled main");
                        break 'outer;
                    }
                    Either::Second(_) => (),
                }
            }
        };

        let _ = join(runner_task, async {
            esp_println::println!("Connecting");

            let target = Address {
                kind: AddrKind::PUBLIC,
                addr: BdAddr::new([47, 2, 54, 216, 23, 228]),
            };

            let config = ConnectConfig {
                connect_params: Default::default(),
                scan_config: ScanConfig {
                    filter_accept_list: &[(target.kind, &target.addr)],
                    timeout: Duration::from_secs(5),
                    ..Default::default()
                },
            };

            let conn = match central.connect(&config).await {
                Ok(c) => {
                    esp_println::println!("Connected");
                    c
                }
                Err(_) => {
                    esp_println::println!("Error!");
                    panic!("oh no");
                }
            };

            esp_println::println!("Connected, creating gatt client");

            let client = GattClient::<ExternalController<BleConnector, 20>, DefaultPacketPool, 10>::new(&stack, &conn)
                .await
                .unwrap();

            let client_task = async {
                let mut cancel_signal = cancel_watch.receiver().unwrap();
                'outer: loop {
                    let fut = client.task();
                    match select(async { cancel_signal.changed().await }, fut).await {
                        Either::First(_) => {
                            esp_println::println!("Cancelled GATT");
                            break 'outer;
                        }
                        Either::Second(_) => (),
                    }
                }
            };

            let _ = join(client_task, async {
                esp_println::println!("Looking for hid service");

                let hid_services = client
                    .services_by_uuid(&Uuid::new_short(0x1812))
                    .await
                    .unwrap();
                let hid_service = hid_services.first().unwrap().clone();
                let all_characteristics: Vec<Characteristic<u8>, 20> = client
                    .discover_all_characteristics(&hid_service)
                    .await
                    .unwrap();

                let hid_report_index = discover_hid_report_characteristic(&client, &all_characteristics).await;

                if hid_report_index.is_none() {
                    esp_println::println!("No hid report characteristic found!");
                    return;
                }

                let hid_report_characteristic = &all_characteristics[hid_report_index.unwrap()];

                esp_println::println!("Looking for battery service");
                let battery_services = client
                    .services_by_uuid(&Uuid::new_short(0x180f))
                    .await
                    .unwrap();
                esp_println::println!("Found {} battery services", battery_services.len());

                let battery_service = battery_services.first().unwrap().clone();

                let all_characteristics: Vec<Characteristic<u8>, 10> = client
                    .discover_all_characteristics(&battery_service)
                    .await
                    .unwrap();
                for x in &all_characteristics {
                    esp_println::println!(
                        "battery characteristic: {:x?}, {:?}",
                        x.uuid.as_raw(),
                        x.cccd_handle
                    )
                }

                let battery_level_characteristic: Characteristic<u8> = client
                    .characteristic_by_uuid(&battery_service, &Uuid::new_short(0x2A19))
                    .await
                    .unwrap();

                esp_println::println!("Starting the stuff");
                let mut buttons_listener = client
                    .subscribe(&hid_report_characteristic, true)
                    .await
                    .unwrap();
                let mut battery_listener =
                    match client.subscribe(&battery_level_characteristic, true).await {
                        Ok(b) => b,
                        Err(e) => {
                            esp_println::println!("Error: {:?}", e);
                            return;
                        }
                    };

                let buttons = async {
                    let mut cancel_signal = cancel_watch.receiver().unwrap();
                    let mut i = 0u32;
                    'outer: loop {
                        let fut = buttons_listener.next();
                        match select(async { cancel_signal.changed().await }, fut).await {
                            Either::First(_) => {
                                esp_println::println!("cancelling buttons");
                                break 'outer;
                            }
                            Either::Second(notification) => {
                                let gamepad_state = GamepadState::from_hid_report(notification.as_ref());
                                esp_println::println!("{} {:?}", i, gamepad_state);
                                pilot_controller::update_from_gamepad_state(gamepad_state);
                                i = i.wrapping_add(1);
                            }
                        }
                    }
                };

                let battery = async {
                    let mut cancel_signal = cancel_watch.receiver().unwrap();
                    let mut data = [0; 1];
                    client
                        .read_characteristic(&battery_level_characteristic, &mut data)
                        .await
                        .unwrap();
                    esp_println::println!("battery level: {}", data[0]);
                    'outer: loop {
                        let fut = battery_listener.next();
                        match select(async { cancel_signal.changed().await }, fut).await {
                            Either::First(_) => {
                                esp_println::println!("cancelling battery");
                                break 'outer;
                            }
                            Either::Second(notification) => {
                                esp_println::println!(
                                    "Got notification: {:?} (val: {})",
                                    notification.as_ref(),
                                    notification.as_ref()[0]
                                );
                            }
                        }
                    }
                };

                let _ = join3(battery, buttons, async {
                    loop {
                        if !conn.is_connected() {
                            esp_println::println!("Connection lost!");
                            cancel_sender.send(());
                            break;
                        }
                        Timer::after(Duration::from_secs(2)).await;
                    }
                })
                    .await;
            })
                .await;
        })
            .await;
    }
}

async fn discover_hid_report_characteristic<C, const N: usize>(
    client: &GattClient<'_, C, DefaultPacketPool, 10>,
    all_characteristics: &Vec<Characteristic<u8>, N>,
) -> Option<usize>
where
    C: Controller,
{
    for i in 0..all_characteristics.len() {
        if all_characteristics[i].uuid == Uuid::new_short(0x2A4D) {
            if i < all_characteristics.len() - 1 {
                let descriptors: Vec<trouble_host::gatt::Descriptor, 5> = client
                    .get_descriptor_for_range(
                        all_characteristics[i].handle + 1,
                        all_characteristics[i + 1].handle - 1,
                    )
                    .await
                    .unwrap();

                for x in &descriptors {
                    if x.attribute_type == Uuid::new_short(0x2908)
                    // Report reference
                    {
                        let (report_id, report_type) = client
                            .read_descriptor(x, |data| (data[0], data[1]))
                            .await
                            .unwrap();

                        if report_id == 1 && report_type == 1 {
                            return Some(i);
                        }
                    }
                }
            }
        }
    }

    None
}