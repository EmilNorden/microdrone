use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Duration, Instant, Timer};
use fc_common::{FLIGHT_INPUT_SIZE, FlightInput, DroneStatus};
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::{MAX_PAYLOAD_SIZE, Nrf24l01};
use zerocopy::{FromBytes, IntoBytes};
use crate::telemetry::DroneBatteryLevelTelemetry;

#[embassy_executor::task]
pub async fn run(
    spi_device: SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>,
    ce: Output<'static>,
    mut irq: ExtiInput<'static>,
    mut battery_level_receiver: DroneBatteryLevelTelemetry,
) {
    info!("Radio init");
    let mut delay = Delay {};
    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Min)
        .payload_size(PayloadSize::Dynamic)
        .ack_payloads_enabled(true);

    let mut radio = match Nrf24l01::new_async(spi_device, ce, &mut delay, config).await {
        Ok(radio) => radio,
        Err(_) => {
            info!("NRF24 Error");
            return;
        }
    };

    if !radio.is_connected().await.unwrap() {
        info!("!!! RX Radio not connected!");
    }
    info!("RX Radio connected");

    radio
        .open_reading_pipe(DataPipe::DP0, b"Node1")
        .await
        .unwrap();
    radio.start_listening().await.unwrap();

    radio
        .write_ack_payload(DataPipe::DP0, b"Pong!")
        .await
        .unwrap();

    info!("Radio RX started!");
    let mut i = 0u32;
    loop {
        while irq.is_low() {
            let status = radio.status().await.unwrap();

            if status.data_ready() {
                // Drain RX FIFO
                while !radio.rx_fifo_empty().await.unwrap() {
                    let mut buf = [0u8; FLIGHT_INPUT_SIZE];
                    match radio.read(&mut buf).await {
                        Ok(len) => {
                            let input = FlightInput::read_from_bytes(&buf[0..len]).unwrap();
                            // TODO COmment below 1 line back in
                            info!("{} - RX {:?}", i, input);
                            //info!("{} RX {} bytes: {:?}", i, len, core::str::from_utf8(&buf[..len]).unwrap());
                            i = i.wrapping_add(1);
                            let drone_status = DroneStatus {
                                battery_level: battery_level_receiver.get().1.0,
                                altitude: 0,
                                temp: 0
                            };
                            radio
                                .write_ack_payload(DataPipe::DP0, drone_status.as_bytes())
                                .await
                                .unwrap();
                        }
                        Err(_) => break,
                    }
                }
            }

            radio.reset_status().await.unwrap();
        }

        irq.wait_for_falling_edge().await;
    }
}
