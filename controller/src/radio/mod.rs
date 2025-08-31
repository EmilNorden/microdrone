mod state;

use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_hal_bus::spi::AtomicDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Event, Input, Output};
use esp_hal::spi::master::Spi;
use esp_hal::Async;
use fc_common::{FlightInput, FLIGHT_INPUT_SIZE};
use nrf24_rs::config::{NrfConfig, PALevel, PayloadSize};
use nrf24_rs::{Nrf24l01, MAX_PAYLOAD_SIZE};
use zerocopy::{IntoBytes};
use crate::input;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
    mut irq: Input<'static>
) {
    const { assert!(FLIGHT_INPUT_SIZE < MAX_PAYLOAD_SIZE as usize, "FlightInput size exceeds max payload size"); }

    irq.listen(Event::FallingEdge);

    esp_println::println!("TX Radio init");
    let config = NrfConfig::default()
        .channel(8)               
        .pa_level(PALevel::Min)
        .payload_size(PayloadSize::Dynamic)
        .ack_payloads_enabled(true);


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

    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut timestamp = Instant::now();

    loop {
        ticker.next().await;
        radio.reset_status().unwrap();
        let input: FlightInput = input::get_input_state().into();

        match radio.write(&mut delay, input.as_bytes()) {
            Ok(_) => {
                match select(
                    async { Timer::after(Duration::from_millis(1000)).await },
                    async { irq.wait_for(Event::FallingEdge).await }
                ).await {
                    Either::First(_) => {
                        esp_println::print!("No reply from drone.");
                        radio.reset_status().unwrap();
                    }
                    Either::Second(_) => {
                        let status = radio.status().unwrap();

                        if status.reached_max_retries() {
                            esp_println::println!("MAX_RT");
                            radio.reset_status().unwrap();
                        } else {
                            input::reset_buttons_latch();
                            esp_println::println!(
                                "data sent: {} data ready: {} - ",
                                status.data_sent(),
                                status.data_ready()
                            );
                            if status.data_ready() {
                                let mut ack_buffer = [0; MAX_PAYLOAD_SIZE as usize];
                                let len = radio.read(&mut ack_buffer).unwrap();
                                radio.reset_status().unwrap();

                                esp_println::println!("ACK received ({}) {}", len, str::from_utf8(&ack_buffer[..len]).unwrap());
                            }

                            radio.reset_status().unwrap();
                        }
                    },
                }
            },
            Err(e) => {
                esp_println::print!("Radio write error: {:?}", e);
            }
        }
        radio.reset_status().unwrap();
    }
}
