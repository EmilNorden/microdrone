mod state;

use embassy_time::{Duration, Ticker};
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Event, Input, Output};
use esp_hal::spi::master::Spi;
use esp_hal::Async;
use fc_common::{DroneStatus, FlightInput, SignalBase, DRONE_STATUS_SIZE, FLIGHT_INPUT_SIZE};
use nrf24_rs::config::{NrfConfig, PALevel, PayloadSize};
use nrf24_rs::{Nrf24l01, MAX_PAYLOAD_SIZE};
use zerocopy::{FromBytes, IntoBytes};

use crate::moving_sum::MovingSum;
use crate::signal::{
    DroneAltitudeEmitter, DroneBatteryLevelEmitter, InputSignal, RadioEmitter, RadioLinkQualityEmitter, RadioStatus,
};

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
    mut irq: Input<'static>,
    mut input_signal: InputSignal,
    mut radio_status_emitter: RadioEmitter,
    mut drone_altitude_emitter: DroneAltitudeEmitter,
    mut drone_battery_emitter: DroneBatteryLevelEmitter,
    mut radio_link_quality_emitter: RadioLinkQualityEmitter,
) {
    const {
        assert!(
            FLIGHT_INPUT_SIZE < MAX_PAYLOAD_SIZE as usize,
            "FlightInput size exceeds max payload size"
        );
    }
    radio_status_emitter.emit(RadioStatus { connected: false });
    irq.listen(Event::FallingEdge);

    esp_println::println!("TX Radio init");
    let config = NrfConfig::default()
        .channel(76)
        .pa_level(PALevel::Min)
        .payload_size(PayloadSize::Dynamic)
        .ack_payloads_enabled(true);

    let mut delay = Delay::new();
    let mut radio = match Nrf24l01::new_blocking(spi_device, ce, &mut delay, config) {
        Ok(radio) => radio,
        Err(e) => {
            esp_println::println!("NRF24 Error : {:?}", e);
            return;
        }
    };

    if !radio.is_connected().unwrap() {
        esp_println::println!("!!! Radio not connected!");
    }
    esp_println::println!("TX Radio connected");
    radio.open_writing_pipe(b"Node1").unwrap();

    esp_println::println!("Radio 1 started!");

    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut i = 0;
    let mut moving_sum: MovingSum<u8, u16, 50> = MovingSum::new();
    let mut quality_update_ticker = 0;
    const QUALITY_UPDATE_FREQUENCY: usize = 5;
    let mut total_failures = 0;
    loop {
        ticker.next().await;
        esp_println::println!("tick! {}   fail: {}", i, total_failures);
        i = i + 1;

        let input: FlightInput = input_signal.get().into();

        match radio.write(&mut delay, input.as_bytes()) {
            Ok(_) => {
                moving_sum.push(radio.retries_in_last_transmission().unwrap());
                quality_update_ticker += 1;
                if quality_update_ticker > QUALITY_UPDATE_FREQUENCY {
                    quality_update_ticker = 0;
                    let link_score = 1.0 - (moving_sum.average() / 15.0);
                    radio_link_quality_emitter.emit(link_score);
                }

                irq.wait_for_low().await;

                let status = radio.status().unwrap();
                radio.reset_status().unwrap();

                if status.reached_max_retries() {
                    esp_println::println!("MAX_RT");
                    total_failures += 1;
                    radio.flush_tx().unwrap();
                } else if let Some(ack) = read_ack(&mut radio).await {
                    radio_status_emitter.emit_if_changed(RadioStatus { connected: true });
                    //drone_status_emitter.emit(drone_status);
                    drone_altitude_emitter.emit(ack.altitude);
                    drone_battery_emitter.emit(ack.battery_level);
                } else {
                    total_failures += 1;
                }
            }
            Err(e) => {
                radio.reset_status().unwrap();
                total_failures += 1;
                esp_println::println!("ERR: Radio write error: {:?}", e);
            }
        }
    }
}

async fn read_ack<SPI, CE>(radio: &mut Nrf24l01<SPI, CE, nrf24_rs::Sync>) -> Option<DroneStatus>
where
    SPI: SpiDevice,
    CE: OutputPin,
{
    let mut ack_buffer = [0; 32];
    match radio.read(&mut ack_buffer) {
        Ok(len) => {
            if len != DRONE_STATUS_SIZE {
                /* After connection has been re-established between controller and drone, it seems like
                there may be ACKs of larger size than expected. I choose to just log these for now */
                esp_println::println!("Received ACK of size {}", len);
                None
            } else {
                if let Ok(drone_status) = DroneStatus::read_from_bytes(&ack_buffer[0..len]) {
                    esp_println::println!("ACK received ({}) {:?}", len, drone_status);
                    Some(drone_status)
                } else {
                    esp_println::println!("Unable to parse ACK");
                    None
                }
            }
        }
        Err(e) => {
            esp_println::println!("Error reading ACK {:?}", e);
            None
        }
    }
}
