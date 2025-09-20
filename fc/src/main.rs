#![no_std]
#![no_main]
mod bms;
mod env;
mod radio;
mod signal;

use crate::signal::{
    drone_battery_level_signal, drone_battery_status_signal, new_drone_battery_level_signal_emitter,
    new_drone_battery_status_signal_emitter, BatteryStatus,
};
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_stm32::adc::{Adc, AdcChannel, VREF_CALIB_MV};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<Async>>> = StaticCell::new();

fn read_calibrated_vdda(adc: &mut Adc<ADC1>) -> u32 {
    let mut vref = adc.enable_vrefint();

    let vref_raw = adc.blocking_read(&mut vref);

    let vref_cal = unsafe { core::ptr::read_volatile(0x1FFF_7A2A as *const u16) };

    VREF_CALIB_MV * (vref_cal as u32) / (vref_raw as u32)
}

async fn timeout<A: Future>(duration: Duration, awaitable: A) -> Option<A::Output> {
    match select(Timer::after(duration), awaitable).await {
        Either::First(_) => None,
        Either::Second(x) => Some(x),
    }
}

async fn battery_power_on_self_test() {
    let mut battery_status = drone_battery_status_signal();
    let battery_status = timeout(Duration::from_secs(1), battery_status.next_value()).await;

    match battery_status {
        None => defmt::panic!("Battery status timeout"),
        Some(BatteryStatus::Critical) => defmt::panic!("Battery level critical."),
        Some(_) => {}
    };
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Flight controller starting.");

    // Start-up BMS (Battery Management Subsystem) first
    spawner
        .spawn(bms::run(
            p.PA0.degrade_adc(),
            Adc::new(p.ADC1),
            new_drone_battery_level_signal_emitter(),
            new_drone_battery_status_signal_emitter(),
        ))
        .unwrap();

    // Halt start-up if battery level is critical
    battery_power_on_self_test().await;

    // Setup SPU
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi = Spi::new(
        p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA2_CH3, p.DMA2_CH2, spi_config,
    );
    let spi_bus = Mutex::new(spi);
    let spi_bus = SPI_BUS.init(spi_bus);

    // NRF24L01+
    let radio_cs = Output::new(p.PB13, Level::High, Speed::Low);
    let radio_device = SpiDevice::new(spi_bus, radio_cs);

    let radio_ce = Output::new(p.PB12, Level::High, Speed::Low);
    let radio_irq = ExtiInput::new(p.PB1, p.EXTI1, Pull::Up);
    spawner
        .spawn(radio::run(
            radio_device,
            radio_ce,
            radio_irq,
            drone_battery_level_signal(),
        ))
        .unwrap();

    let bmp390_cs = Output::new(p.PB14, Level::High, Speed::Low);
    let bmp390_device = SpiDevice::new(spi_bus, bmp390_cs);
    let bmp390_irq = ExtiInput::new(p.PB6, p.EXTI6, Pull::Up);
    /*
    let r = adc.blocking_read(&mut battery);

    let mut buf = [0u16; 32];

    // Enable DMA2 clock
    embassy_stm32::pac::RCC.ahb1enr().modify(|w| w.set_dma2en(true));

    // Disable stream
    embassy_stm32::pac::DMA2.st(0).cr().modify(|w| w.set_en(false));
    while embassy_stm32::pac::DMA2.st(0).cr().read().en() {}

    embassy_stm32::pac::DMA2.ifcr(0).write(|w| {
        w.set_tcif(0, true);
        w.set_htif(0, true);
        w.set_teif(0, true);
        w.set_dmeif(0, true);
        w.set_feif(0, true);
    });*/

    // Set up TIM3
    //embassy_stm32::pac::RCC.apb1enr().modify(|w| w.set_tim3en(true));
    /*
        let timer = embassy_stm32::timer::low_level::Timer::new(p.TIM3);
        timer.set_frequency(Hertz(400));
        // Set TRGO to trigger on update event. This has no API in the embassy low level driver, so I have to go through PAC.
        embassy_stm32::pac::TIM3.cr2().modify(|w| w.set_mms(Mms::UPDATE));
        timer.start();
    */

    info!("Flight controller started!");
    core::future::pending::<()>().await;
}
