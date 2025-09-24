#![no_std]

mod signal;
pub use signal::{Signal, SignalBase, SignalEmitter};

use zerocopy::{FromBytes, Immutable, IntoBytes};

#[derive(IntoBytes, FromBytes, Immutable)]
#[repr(C, packed)]
pub struct FlightInput {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub buttons: u8,
}
pub const FLIGHT_INPUT_SIZE: usize = size_of::<FlightInput>();

impl defmt::Format for FlightInput {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "left_stick_x({:x}) left_stick_y({:x}) right_stick_x({:x})  right_stick_y({:x}) left_trigger({:x}) right_trigger({:x}) buttons({:x})",
            self.left_stick_x,
            self.left_stick_y,
            self.right_stick_x,
            self.right_stick_y,
            self.left_trigger,
            self.right_trigger,
            self.buttons,
        )
    }
}

#[derive(IntoBytes, FromBytes, Immutable, Debug, PartialEq, Default, Clone)]
#[repr(C, packed)]
pub struct DroneStatus {
    pub battery_level: u8,
    /// This is the altitude of the drone in 25cm increments. So 1 = 25cm, 2 = 50cm etc.
    pub altitude: u8,
    pub temp: u8,
}
pub const DRONE_STATUS_SIZE: usize = size_of::<DroneStatus>();

/*pub async fn timeout<A: Future>(duration: Duration, awaitable: A) -> Option<A::Output> {
    match select(Timer::after(duration), awaitable).await {
        Either::First(_) => None,
        Either::Second(x) => Some(x),
    }
}*/
