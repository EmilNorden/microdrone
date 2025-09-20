#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;
use core::mem::ManuallyDrop;

use controller::signal::{
    battery_signal, controller_connected_signal, drone_status_signal, input_signal, new_battery_signal_emitter,
    new_controller_connected_signal_emitter, new_drone_status_signal_emitter, new_input_signal_emitter,
    new_radio_signal_emitter, radio_signal,
};
use controller::{gui, input, radio};
use embassy_executor::Spawner;
use embedded_hal_bus::spi::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Async;
use esp_wifi::EspWifiController;

#[panic_handler]
fn panic(p: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic occurred: {:?}", p);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

    let sclk = peripherals.GPIO37;
    let mosi = peripherals.GPIO38;
    let miso = peripherals.GPIO39;

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default().with_frequency(Rate::from_mhz(4)).with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .into_async();
    let shared_bus = AtomicCell::new(spi);
    let shared_bus = ManuallyDrop::new(shared_bus);
    let local_shared_bus: &'static AtomicCell<Spi<Async>> = unsafe { core::mem::transmute(&shared_bus) };

    let display_cs = Output::new(peripherals.GPIO36, Level::High, OutputConfig::default());
    let display_rst = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let display_dc = Output::new(peripherals.GPIO35, Level::Low, OutputConfig::default());
    let display_device = AtomicDevice::new(local_shared_bus, display_cs, Delay::new()).unwrap();

    let radio_csn = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let radio_ce = Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default());
    let radio_device = AtomicDevice::new(local_shared_bus, radio_csn, Delay::new()).unwrap();
    let radio_irq = Input::new(peripherals.GPIO12, InputConfig::default().with_pull(Pull::Up));

    let battery_emitter = new_battery_signal_emitter();
    let input_emitter = new_input_signal_emitter();
    let controller_emitter = new_controller_connected_signal_emitter();
    let radio_status_emitter = new_radio_signal_emitter();
    let drone_status_emitter = new_drone_status_signal_emitter();

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
            drone_status_signal(),
        ))
        .unwrap();
    spawner
        .spawn(radio::run(
            radio_device,
            radio_ce,
            radio_irq,
            input_signal(),
            radio_status_emitter,
            drone_status_emitter,
        ))
        .unwrap();

    core::future::pending::<()>().await;
}
