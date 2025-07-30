#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;
use controller::{gui, input};
use core::mem::ManuallyDrop;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use esp_hal::{
    spi::Mode,
    spi::master::{Config, Spi},
    gpio::{Level, Output, OutputConfig},
    clock::CpuClock,
    time::Rate,
    timer::systimer::SystemTimer,
    timer::timg::TimerGroup,
    Async,
    delay::Delay,
};
use esp_wifi::EspWifiController;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
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

    esp_println::println!("Init WIFI");
    let wifi_init = esp_wifi::init(
            timg0.timer0,
            esp_hal::rng::Rng::new(peripherals.RNG))
        .expect("Failed to initialize WIFI/BLE controller");

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let local_wifi = ManuallyDrop::new(wifi_init);
    let local_wifi:  &'static EspWifiController<'static> = unsafe { core::mem::transmute(&local_wifi) };
    //let local_wifi:  &'static EspWifiController<'static> = unsafe { &*(&*local_wifi as *const _) };

    let sclk = peripherals.GPIO37;
    let mosi = peripherals.GPIO38;
    let miso = peripherals.GPIO39;

    let cs = Output::new(peripherals.GPIO36, Level::Low, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO35, Level::Low, OutputConfig::default());

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(Mode::_3),
    ).unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .into_async();

    let shared_bus = AtomicCell::new(spi);
    let shared_bus = ManuallyDrop::new(shared_bus);
    let local_shared_bus:  &'static AtomicCell<Spi<Async>> = unsafe { core::mem::transmute(&shared_bus) };

    let atomic_device = AtomicDevice::new(local_shared_bus, cs, Delay::new()).unwrap();

    spawner.spawn(input::run(local_wifi, peripherals.BT)).unwrap();

    spawner.spawn(gui::run(atomic_device, rst, dc)).unwrap();

    loop {

        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}