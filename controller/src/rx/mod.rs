use embassy_time::{Duration, Timer};
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::Async;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel};
use nrf24_rs::Nrf24l01;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
) {

    esp_println::println!("RX Radio init");
    let message = b"Ping!";
    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Min)
        .payload_size(message.len() as u8);

    let mut delay = Delay::new();
    let mut radio = match Nrf24l01::new(spi_device, ce, &mut delay, config) {
        Ok(radio) => radio,
        Err(e) => {
            esp_println::println!("NRF24 Error : {:?}", e);
            return;
        }
    };

    if !radio.is_connected().unwrap() {
        esp_println::println!("!!! RX Radio not connected!");
    }
    esp_println::println!("RX Radio connected");

    radio.open_reading_pipe(DataPipe::DP0, b"Node1").unwrap();
    radio.start_listening().unwrap();

    esp_println::println!("Radio RX started!");
    loop {
        while !radio.data_available().unwrap() {
            // No data available, wait 50ms, then check again
            Timer::after(Duration::from_millis(50)).await;

        }

        let mut buffer = [0; b"Ping!".len()];
        radio.read(&mut buffer).unwrap();

        esp_println::println!("Received from NRF24: {:?}!", str::from_utf8(&buffer).unwrap());

        Timer::after(Duration::from_millis(50)).await;
    }


}