use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::AtomicDevice;
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel};
use nrf24_rs::Nrf24l01;
use defmt::*;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
) {
    info!("Radio init");
    let mut delay = Delay{};
    let message = b"Ping!";
    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Min)
        .payload_size(message.len() as u8);

    let mut radio = match Nrf24l01::new(spi_device, ce, &mut delay, config) {
        Ok(radio) => radio,
        Err(_) => {
            info!("NRF24 Error");
            return;
        }
    };

    if !radio.is_connected().unwrap() {
        info!("!!! RX Radio not connected!");
    }
    info!("RX Radio connected");

    radio.open_reading_pipe(DataPipe::DP0, b"Node1").unwrap();
    radio.start_listening().unwrap();

    info!("Radio RX started!");
    loop {
        while !radio.data_available().unwrap() {
            // No data available, wait 50ms, then check again
            Timer::after(Duration::from_millis(50)).await;

        }

        let mut buffer = [0; b"Ping!".len()];
        radio.read(&mut buffer).unwrap();

        info!("Received from NRF24: {:?}!", str::from_utf8(&buffer).unwrap());

        Timer::after(Duration::from_millis(50)).await;
    }
}