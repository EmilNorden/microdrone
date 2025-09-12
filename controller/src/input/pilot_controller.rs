use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use fc_common::FlightInput;

use crate::input::gamepad::HIDReport;

pub struct ControllerState<B, U> {
    pub connected: B,
    pub battery: U,
}

type ControllerStateAtomic = ControllerState<AtomicBool, AtomicU8>;
type ControllerStateNonAtomic = ControllerState<bool, u8>;

static CONTROLLER_STATE: ControllerStateAtomic = ControllerStateAtomic {
    connected: AtomicBool::new(false),
    battery: AtomicU8::new(0),
};

pub struct PilotInputState<T> {
    pub left_stick_x: T,
    pub left_stick_y: T,
    pub right_stick_x: T,
    pub right_stick_y: T,
    pub left_trigger: T,
    pub right_trigger: T,
    pub buttons: T,
    pub buttons_latch: T,
}

impl Into<FlightInput> for PilotInputState<u8> {
    fn into(self) -> FlightInput {
        FlightInput {
            left_stick_x: self.left_stick_x,
            left_stick_y: self.left_stick_y,
            right_stick_x: self.right_stick_x,
            right_stick_y: self.right_stick_y,
            left_trigger: self.left_trigger,
            right_trigger: self.right_trigger,
            buttons: self.buttons_latch,
        }
    }
}

type PilotInputStateAtomic = PilotInputState<AtomicU8>;
type PilotInputStateNonAtomic = PilotInputState<u8>;

static INPUT: PilotInputStateAtomic = PilotInputStateAtomic {
    left_stick_x: AtomicU8::new(127),
    left_stick_y: AtomicU8::new(127),
    right_stick_x: AtomicU8::new(127),
    right_stick_y: AtomicU8::new(127),
    left_trigger: AtomicU8::new(0),
    right_trigger: AtomicU8::new(0),
    buttons: AtomicU8::new(0),
    buttons_latch: AtomicU8::new(0),
};

pub fn get_input_state() -> PilotInputStateNonAtomic {
    PilotInputStateNonAtomic {
        left_stick_x: INPUT.left_stick_x.load(Ordering::Relaxed),
        left_stick_y: INPUT.left_stick_y.load(Ordering::Relaxed),
        right_stick_x: INPUT.right_stick_x.load(Ordering::Relaxed),
        right_stick_y: INPUT.right_stick_y.load(Ordering::Relaxed),
        left_trigger: INPUT.left_trigger.load(Ordering::Relaxed),
        right_trigger: INPUT.right_trigger.load(Ordering::Relaxed),
        buttons: INPUT.buttons.load(Ordering::Relaxed),
        buttons_latch: INPUT.buttons_latch.load(Ordering::Relaxed),
    }
}

pub fn reset_buttons_latch() {
    INPUT.buttons_latch.store(0, Ordering::Relaxed);
}

pub fn get_controller_state() -> ControllerStateNonAtomic {
    ControllerStateNonAtomic {
        connected: CONTROLLER_STATE.connected.load(Ordering::Relaxed),
        battery: CONTROLLER_STATE.battery.load(Ordering::Relaxed),
    }
}
pub fn update_from_hid_report(report: HIDReport) {
    INPUT.left_stick_x.store(report.left_stick_x, Ordering::Relaxed);
    INPUT.left_stick_y.store(report.left_stick_y, Ordering::Relaxed);
    INPUT.right_stick_x.store(report.right_stick_x, Ordering::Relaxed);
    INPUT.right_stick_y.store(report.right_stick_y, Ordering::Relaxed);
    INPUT.left_trigger.store(report.left_trigger, Ordering::Relaxed);
    INPUT.right_trigger.store(report.right_trigger, Ordering::Relaxed);
    INPUT.buttons.store(report.buttons, Ordering::Relaxed);
    INPUT.buttons_latch.fetch_or(report.buttons, Ordering::Relaxed);
}

pub fn update_battery(battery: u8) {
    CONTROLLER_STATE.battery.store(battery, Ordering::Relaxed);
}

pub fn update_connected(connected: bool) {
    CONTROLLER_STATE.connected.store(connected, Ordering::Relaxed);
}
