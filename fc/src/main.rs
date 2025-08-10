#![no_std]
#![no_main]

mod radio;

use core::mem::ManuallyDrop;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::SPI1;
use embassy_time::{Delay, Timer};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embedded_hal_bus::spi::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use nrf24_rs::config::NrfConfig;
use nrf24_rs::Nrf24l01;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Flight controller starting...");

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);
    let shared_bus = AtomicCell::new(spi);
    let shared_bus = ManuallyDrop::new(shared_bus);
    let local_shared_bus:  &'static AtomicCell<Spi<Async>> = unsafe { core::mem::transmute(&shared_bus) };


    let radio_cs = Output::new(p.PB13, Level::High, Speed::Low);
    let radio_ce = Output::new(p.PB12, Level::High, Speed::Low);
    let radio_device = AtomicDevice::new(local_shared_bus, radio_cs, Delay{}).unwrap();

    spawner.spawn(radio::run(radio_device, radio_ce)).unwrap();

    info!("Flight controller started!");
    loop {
        Timer::after_millis(300).await;
    }
}
