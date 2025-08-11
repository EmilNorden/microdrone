#![no_std]
#![no_main]

mod radio;

use core::mem::ManuallyDrop;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::SPI1;
use embassy_time::{Delay, Timer};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal_bus::spi::AtomicDevice;
use embedded_hal_bus::util::AtomicCell;
use nrf24_rs::config::NrfConfig;
use nrf24_rs::Nrf24l01;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<Async>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Flight controller starting...");

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    /*let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA2_CH3, p.DMA2_CH2, spi_config);
    let spi_bus = Mutex::new(spi);
    let spi_bus = SPI_BUS.init(spi_bus);

    // NRF24L01+
    let radio_cs = Output::new(p.PB13, Level::High, Speed::Low);
    let radio_device = SpiDevice::new(spi_bus, radio_cs);

    let radio_ce = Output::new(p.PB12, Level::High, Speed::Low);
    spawner.spawn(radio::run(radio_device, radio_ce)).unwrap();*/


    let spi = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);
    let shared_bus = AtomicCell::new(spi);
    let shared_bus = ManuallyDrop::new(shared_bus);
    let local_shared_bus:  &'static AtomicCell<Spi<Async>> = unsafe { core::mem::transmute(&shared_bus) };

    let radio_cs = Output::new(p.PB13, Level::High, Speed::Low);
    let radio_ce = Output::new(p.PB12, Level::High, Speed::Low);
    let radio_irq = ExtiInput::new(p.PB1, p.EXTI1, Pull::Up);
    let radio_device = AtomicDevice::new(local_shared_bus, radio_cs, Delay{}).unwrap();

   spawner.spawn(radio::run(radio_device, radio_ce, radio_irq)).unwrap();


    info!("Flight controller started!");
    loop {
        Timer::after_millis(300).await;
    }
}
