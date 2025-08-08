use embassy_time::{Duration, Timer};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::Async;
use esp_hal::delay::Delay;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use nrf24_rs::config::{NrfConfig, PALevel};
use nrf24_rs::Nrf24l01;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
) {
    /*esp_println::println!("TX Radio init");
    let mut s = spi_device;
    let mut ce = ce;

    ce.set_low();

    Timer::after(Duration::from_millis(10)).await;


    esp_println::println!("CE LOW: {:?}", ce.is_set_low());

    let write_buff = [0xFF; 1];
    let mut read_buff = [0u8; 1];
    s.write(&write_buff).unwrap();
    s.read(&mut read_buff).unwrap();

    esp_println::println!("Received {}", read_buff[0]);
*/
    esp_println::println!("TX Radio init");
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
        esp_println::println!("!!! Radio not connected!");
    }
    esp_println::println!("TX Radio connected");
    radio.open_writing_pipe(b"Node1").unwrap();



    esp_println::println!("Radio 1 started!");
    loop {
        match radio.write(&mut delay, message) {
            Ok(_) => {
                esp_println::println!("Radio write sent!");
            },
            Err(e) => {
                esp_println::println!("Radio write error: {:?}", e);
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
    

}