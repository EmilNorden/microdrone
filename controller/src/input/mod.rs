use esp_wifi::EspWifiController;

mod gamepad;
mod pilot_controller;

pub use pilot_controller::{get_controller_state, get_input_state, reset_buttons_latch};

use crate::signal::{BatteryEmitter, ControllerConnectedEmitter, InputEmitter};

#[embassy_executor::task]
pub async fn run(
    wifi: &'static EspWifiController<'static>,
    bt: esp_hal::peripherals::BT<'static>,
    battery_emitter: BatteryEmitter,
    input_emitter: InputEmitter,
    controller_emitter: ControllerConnectedEmitter,
) {
    gamepad::run(wifi, bt, battery_emitter, input_emitter, controller_emitter).await;
}
