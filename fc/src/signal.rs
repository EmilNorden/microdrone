use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use fc_common::{define_signal, Signal, SignalBase, SignalEmitter};

define_signal!(DroneBatteryLevel, BatteryLevel, 1);
define_signal!(DroneBatteryStatus, BatteryStatus, 2);
define_signal!(Altitude, uom::si::f32::Length, 1);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatteryLevel(pub u8);

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Ok,
    Low,      // 7.5 V
    Critical, // 7.3 V
    Cutoff,   // 7.0 V
}

impl Default for BatteryStatus {
    fn default() -> Self {
        BatteryStatus::Critical
    }
}
