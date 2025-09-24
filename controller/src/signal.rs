use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use fc_common::{define_signal, FlightInput, Signal, SignalBase, SignalEmitter};

define_signal!(Radio, RadioStatus, 1);
define_signal!(ControllerConnected, bool, 1);
define_signal!(Battery, ControllerBattery, 2);
define_signal!(Input, ControllerInput, 1);
define_signal!(DroneBatteryLevel, u8, 1);
define_signal!(DroneAltitude, u8, 1);
define_signal!(RadioLinkQuality, f32, 1);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioStatus {
    pub connected: bool,
}

impl Default for RadioStatus {
    fn default() -> Self {
        RadioStatus { connected: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControllerBattery {
    pub level: u8,
}

impl Default for ControllerBattery {
    fn default() -> Self {
        ControllerBattery { level: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControllerInput {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub buttons: u8,
}

impl Default for ControllerInput {
    fn default() -> Self {
        Self {
            left_stick_x: 0x7F,
            left_stick_y: 0x7F,
            right_stick_x: 0x7F,
            right_stick_y: 0x7F,
            left_trigger: 0x00,
            right_trigger: 0x00,
            buttons: 0x00,
        }
    }
}

impl Into<FlightInput> for ControllerInput {
    fn into(self) -> FlightInput {
        FlightInput {
            left_stick_x: self.left_stick_x,
            left_stick_y: self.left_stick_y,
            right_stick_x: self.right_stick_x,
            right_stick_y: self.right_stick_y,
            left_trigger: self.left_trigger,
            right_trigger: self.right_trigger,
            buttons: self.buttons,
        }
    }
}
