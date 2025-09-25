#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;

use core::cell::RefCell;
use core::mem::ManuallyDrop;

use controller::signal::{
    battery_signal, controller_connected_signal, drone_altitude_signal, drone_battery_level_signal, input_signal,
    new_battery_signal_emitter, new_controller_connected_signal_emitter, new_drone_altitude_signal_emitter,
    new_drone_battery_level_signal_emitter, new_input_signal_emitter, new_radio_link_quality_signal_emitter,
    new_radio_signal_emitter, radio_link_quality_signal, radio_signal,
};
use controller::{gui, input, radio};
use embassy_embedded_hal::shared_bus::{asynch, blocking};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::mutex::Mutex;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Async, Blocking};
use esp_wifi::EspWifiController;
use static_cell::StaticCell;

#[panic_handler]
fn panic(p: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic occurred: {:?}", p);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static SPI2_BUS: StaticCell<Mutex<NoopRawMutex, Spi<Async>>> = StaticCell::new();
//static SPI3_BUS: StaticCell<BlockingMutex<NoopRawMutex, Spi<Blocking>>> = StaticCell::new();
static SPI3_BUS: StaticCell<NoopMutex<RefCell<Spi<Blocking>>>> = StaticCell::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_println::println!("Init WIFI!");
    let wifi_init = esp_wifi::init(timg0.timer0, esp_hal::rng::Rng::new(peripherals.RNG))
        .expect("Failed to initialize WIFI/BLE controller");

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let local_wifi = ManuallyDrop::new(wifi_init);
    let local_wifi: &'static EspWifiController<'static> = unsafe { core::mem::transmute(&local_wifi) };
    //let local_wifi:  &'static EspWifiController<'static> = unsafe { &*(&*local_wifi as *const _) };

    // Initialize SPI2
    let spi2 = Spi::new(
        peripherals.SPI2,
        Config::default().with_frequency(Rate::from_mhz(4)).with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO37)
    .with_mosi(peripherals.GPIO38)
    .with_miso(peripherals.GPIO39)
    .into_async();

    let spi2_bus = Mutex::new(spi2);
    let spi2_bus = SPI2_BUS.init(spi2_bus);

    // Initialize SPI3
    let spi3 = Spi::new(
        peripherals.SPI3,
        Config::default().with_frequency(Rate::from_mhz(4)).with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO16)
    .with_mosi(peripherals.GPIO17)
    .with_miso(peripherals.GPIO18);

    let spi3_bus = SPI3_BUS.init(NoopMutex::new(RefCell::new(spi3)));

    let display_cs = Output::new(peripherals.GPIO36, Level::High, OutputConfig::default());
    let display_rst = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let display_dc = Output::new(peripherals.GPIO35, Level::Low, OutputConfig::default());
    let display_device = blocking::spi::SpiDevice::new(spi3_bus, display_cs);

    let radio_cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let radio_ce = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let radio_device = asynch::spi::SpiDevice::new(spi2_bus, radio_cs);
    let radio_irq = Input::new(peripherals.GPIO12, InputConfig::default().with_pull(Pull::Up));

    /* Create signal emitters */
    let battery_emitter = new_battery_signal_emitter();
    let input_emitter = new_input_signal_emitter();
    let controller_emitter = new_controller_connected_signal_emitter();
    let radio_status_emitter = new_radio_signal_emitter();
    let drone_battery_emitter = new_drone_battery_level_signal_emitter();
    let drone_altitude_emitter = new_drone_altitude_signal_emitter();
    let radio_link_quality_emitter = new_radio_link_quality_signal_emitter();

    /* Start up sub-systems */
    spawner
        .spawn(input::run(
            local_wifi,
            peripherals.BT,
            battery_emitter,
            input_emitter,
            controller_emitter,
        ))
        .unwrap();
    spawner
        .spawn(gui::run(
            display_device,
            display_rst,
            display_dc,
            battery_signal(),
            radio_signal(),
            controller_connected_signal(),
            drone_battery_level_signal(),
            drone_altitude_signal(),
            radio_link_quality_signal(),
        ))
        .unwrap();
    spawner
        .spawn(radio::run(
            radio_device,
            radio_ce,
            radio_irq,
            input_signal(),
            radio_status_emitter,
            drone_altitude_emitter,
            drone_battery_emitter,
            radio_link_quality_emitter,
        ))
        .unwrap();

    core::future::pending::<()>().await;
}
