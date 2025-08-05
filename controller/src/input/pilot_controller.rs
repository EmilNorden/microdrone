use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use crate::input::gamepad::HIDReport;

pub struct ControllerState<B, U> {
    pub connected: B,
    pub battery: U
}

type ControllerStateAtomic = ControllerState<AtomicBool, AtomicU8>;
type ControllerStateNonAtomic = ControllerState<bool, u8>;

static CONTROLLER_STATE: ControllerStateAtomic = ControllerStateAtomic {
    connected: AtomicBool::new(false),
    battery: AtomicU8::new(0),
};

pub struct PilotInputState<T> {
    pub throttle: T,
    pub yaw: T, // rotate drone left/right,
    pub pitch: T, // tilt drone forward/backward
    pub roll: T, // tilts drone left/right
}

type PilotInputStateAtomic = PilotInputState<AtomicU8>;
type PilotInputStateNonAtomic = PilotInputState<u8>;

static INPUT: PilotInputStateAtomic = PilotInputStateAtomic {
    throttle: AtomicU8::new(0),
    yaw: AtomicU8::new(0),
    pitch: AtomicU8::new(0),
    roll: AtomicU8::new(0),
};

pub fn get_input_state() -> PilotInputStateNonAtomic {
    PilotInputStateNonAtomic {
        throttle: INPUT.throttle.load(Ordering::Relaxed),
        yaw: INPUT.yaw.load(Ordering::Relaxed),
        pitch: INPUT.pitch.load(Ordering::Relaxed),
        roll: INPUT.roll.load(Ordering::Relaxed),
    }
}

pub fn get_controller_state() -> ControllerStateNonAtomic {
    ControllerStateNonAtomic {
        connected: CONTROLLER_STATE.connected.load(Ordering::Relaxed),
        battery: CONTROLLER_STATE.battery.load(Ordering::Relaxed),
    }
}
pub fn update_from_hid_report(report: HIDReport) {
    let final_throttle = report.right_trigger - report.left_trigger;
    INPUT.throttle.fetch_add(final_throttle, Ordering::Relaxed);
    INPUT.yaw.store(report.left_stick_x, Ordering::Relaxed);
    INPUT.pitch.store(report.right_stick_y, Ordering::Relaxed);
    INPUT.roll.store(report.right_stick_x, Ordering::Relaxed);
}

pub fn update_battery(battery: u8) {
    CONTROLLER_STATE.battery.store(battery, Ordering::Relaxed);
}

pub fn update_connected(connected: bool) {
    CONTROLLER_STATE.connected.store(connected, Ordering::Relaxed);
}
