use esp_wifi::EspWifiController;

mod pilot_controller;
mod gamepad;

pub use pilot_controller::{get_input_state, get_controller_state, reset_buttons_latch};


#[embassy_executor::task]
pub async fn run(wifi: &'static EspWifiController<'static>, bt: esp_hal::peripherals::BT<'static>) {
    gamepad::run(wifi, bt).await;
}