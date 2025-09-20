use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use fc_common::{define_signal, Signal, SignalEmitter};

define_signal!(DroneBatteryLevel, BatteryLevel, 1);
define_signal!(DroneBatteryStatus, BatteryStatus, 2);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatteryLevel(pub u8);

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Ok,
    Low, // 7.5 V
    Critical, // 7.3 V
    Cutoff, // 7.0 V
}

impl Default for BatteryStatus {
    fn default() -> Self {
        BatteryStatus::Critical
    }
}