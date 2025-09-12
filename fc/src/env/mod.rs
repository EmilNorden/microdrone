use bmp390_rs::ResetPolicy;
use bmp390_rs::register::osr::{OsrCfg, Oversampling};
use bmp390_rs::typestate::Bmp390Builder;
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Timer};
use libm::powf;

#[embassy_executor::task]
pub async fn run(
    spi_device: SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>,
    irq: ExtiInput<'static>,
) {
    info!("Altimeter init");
    let mut device = Bmp390Builder::new()
        .use_spi(spi_device)
        .use_irq(irq)
        .enable_pressure()
        .enable_temperature()
        .into_forced()
        .build(ResetPolicy::Soft, Delay {})
        .await
        .unwrap();

    device
        .set_oversampling_config(&OsrCfg {
            osr_p: Oversampling::X8,
            osr_t: Oversampling::X1,
        })
        .await
        .unwrap();

    let initial_measurement = device.read_measurement().await.unwrap();
    let reference_pressure = initial_measurement.pressure_pascal();

    loop {
        let measurement = device.read_measurement().await.unwrap();
        let pressure = measurement.pressure_pascal();
        let altitude = 44330.0 * (1.0 - powf(pressure / reference_pressure, 1.0 / 5.255));
        info!("Altitude: {}", altitude);

        Timer::after_millis(500).await;
    }
}
