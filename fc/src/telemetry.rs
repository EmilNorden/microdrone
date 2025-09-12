use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use fc_common::{telemetry_type, Telemetry};

#[derive(Debug, Clone)]
pub struct BatteryLevel(pub u8);
telemetry_type!(DroneBatteryLevel, BatteryLevel, 1, BatteryLevel(0));

#[derive(Debug, Clone)]
pub enum BatteryStatus {
    Ok,
    Low, // 7.5 V
    Critical, // 7.3 V
    Cutoff, // 7.0 V
}
telemetry_type!(DroneBatteryStatus, BatteryStatus, 2, BatteryStatus::Critical);