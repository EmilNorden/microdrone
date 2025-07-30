use core::sync::atomic::{AtomicU8, Ordering};
use crate::input::gamepad::GamepadState;


pub struct PilotControllerState<T> {
    pub throttle: T,
    pub yaw: T, // rotate drone left/right,
    pub pitch: T, // tilt drone forward/backward
    pub roll: T, // tilts drone left/right
}

type PilotControllerStateAtomic = PilotControllerState<AtomicU8>;
type PilotControllerStateNonAtomic = PilotControllerState<u8>;

static CONTROLLER: PilotControllerStateAtomic = PilotControllerStateAtomic {
    throttle: AtomicU8::new(0),
    yaw: AtomicU8::new(0),
    pitch: AtomicU8::new(0),
    roll: AtomicU8::new(0),
};

pub fn get_controller_state() -> PilotControllerStateNonAtomic {
    PilotControllerStateNonAtomic {
        throttle: CONTROLLER.throttle.load(Ordering::Relaxed),
        yaw: CONTROLLER.yaw.load(Ordering::Relaxed),
        pitch: CONTROLLER.pitch.load(Ordering::Relaxed),
        roll: CONTROLLER.roll.load(Ordering::Relaxed),
    }
}
pub fn update_from_gamepad_state(gamepad: GamepadState) {
    let final_throttle = gamepad.right_trigger - gamepad.left_trigger;
    CONTROLLER.throttle.fetch_add(final_throttle, Ordering::Relaxed);
    CONTROLLER.yaw.store(gamepad.left_stick_x, Ordering::Relaxed);
    CONTROLLER.pitch.store(gamepad.right_stick_y, Ordering::Relaxed);
    CONTROLLER.roll.store(gamepad.right_stick_x, Ordering::Relaxed);
}
