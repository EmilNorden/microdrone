use esp_wifi::EspWifiController;

mod gamepad;
mod pilot_controller;

pub use pilot_controller::{get_controller_state, get_input_state, reset_buttons_latch};

use crate::telemetry::{BatterySender, InputSender};

#[embassy_executor::task]
pub async fn run(
    wifi: &'static EspWifiController<'static>,
    bt: esp_hal::peripherals::BT<'static>,
    battery_sender: BatterySender,
    input_sender: InputSender,
) {
    gamepad::run(wifi, bt, battery_sender, input_sender).await;
}
