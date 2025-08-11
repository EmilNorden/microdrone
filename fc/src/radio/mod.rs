use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_time::{Delay, Duration, Instant, Timer};
use embedded_hal_bus::spi::AtomicDevice;
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::{Nrf24l01, MAX_PAYLOAD_SIZE};
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::exti::ExtiInput;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_nrf24l01_async::{Configuration, CrcMode, DataRate};
use fc_common::FlightInput;
use nrf24_rs::error::TransceiverError;
use nrf24_rs::status::Interrupts;
use zerocopy::FromBytes;

#[embassy_executor::task]
pub async fn run(
    spi_device: AtomicDevice<'static, Spi<'static, Async>, Output<'static>, Delay>,
    ce: Output<'static>,
    mut irq: ExtiInput<'static>,
) {
    info!("Radio init");
    let mut delay = Delay{};
    let message = b"Ping!";
    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Min)
        .payload_size(PayloadSize::Dynamic)
        .ack_payloads_enabled(true);

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

    radio.write_ack_payload(DataPipe::DP0, b"Pong!").unwrap();

    info!("Radio RX started!");
    let mut i = 0u32;
    loop {
        while irq.is_low() {
            let status =  radio.status().unwrap();

            if status.data_ready() {
                // Drain RX FIFO
                while !radio.rx_fifo_empty().unwrap() {
                    let mut buf = [0u8; MAX_PAYLOAD_SIZE as usize];
                    match radio.read(&mut buf) {
                        Ok(len) => {
                            info!("{} RX {} bytes: {:?}", i, len, core::str::from_utf8(&buf[..len]).unwrap());
                            i = i.wrapping_add(1);
                            radio.write_ack_payload(DataPipe::DP0, b"Pong!").unwrap();
                        },
                        Err(_) => break,
                    }
                }
            }

            radio.reset_status().unwrap();
        }

        irq.wait_for_falling_edge().await;
    }
}