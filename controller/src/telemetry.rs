use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use fc_common::{telemetry_type, FlightInput, Telemetry};
/*
/// Creates a Watch and type aliases for a new telemetry.
/// Takes three arguments:
/// - Telemetry name
/// - The datatype of the telemetry.
/// - The total number of receivers.
///
/// ## Example
/// ```rust
/// telemetry_type!(Foo, u8, 2);
/// ```
/// Would result in:
///
/// ```rust
/// const FOO_SUBSCRIBERS: usize = 2;
///
/// pub static FOO_WATCH = Watch<CriticalSectionRawMutex, (Instant, u8), FOO_SUBSCRIBERS> = Watch::new();
///
/// pub type FooSender = Sender<'static, CriticalSectionRawMutex, (Instant, u8), FOO_SUBSCRIBERS>;
///
/// pub type FooReceiver = Receiver<'static, CriticalSectionRawMutex, (Instant, u8), FOO_SUBSCRIBERS>;
/// ```
macro_rules! telemetry_type {
    ($NAME:ident, $Ty:ty, $subs:expr) => {
        ::paste::paste! {
            const [<$NAME:upper _SUBSCRIBERS>]: usize = $subs;

            pub static [<$NAME:upper _WATCH>]: embassy_sync::watch::Watch<CriticalSectionRawMutex, (embassy_time::Instant, $Ty), [<$NAME:upper _SUBSCRIBERS>]> = embassy_sync::watch::Watch::new();

            pub type [<$NAME:camel Sender>] = embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, (embassy_time::Instant, $Ty), [<$NAME:upper _SUBSCRIBERS>]>;

            pub type [<$NAME:camel Receiver>] = embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, (embassy_time::Instant, $Ty), [<$NAME:upper _SUBSCRIBERS>]>;
        }
    };
}
*/

#[derive(Clone, Copy, Debug)]
pub struct ControllerBattery {
    pub level: u8,
}
telemetry_type!(Battery, ControllerBattery, 2, ControllerBattery { level: 0 });

#[derive(Clone, Copy, Debug)]
pub struct ControllerInput {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub buttons: u8,
}
telemetry_type!(Input, ControllerInput, 1, ControllerInput::default());

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
