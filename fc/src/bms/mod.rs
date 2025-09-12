use num_traits::float::Float;
use embassy_stm32::adc::{Adc, AnyAdcChannel, Resolution, SampleTime};
use embassy_stm32::peripherals::ADC1;
use embassy_time::{Duration, Instant, Ticker};
use crate::telemetry::{BatteryLevel, BatteryStatus, DroneBatteryLevelSender, DroneBatteryStatusSender};

const ADC_MAX: u32 = 4095;         // 12-bit
const VDDA_MV: u32 = 3300;         // assume 3.3V

// Resistor values for voltage divider.
const R_TOP: u32 = 20_000;
const R_BOT: u32 = 10_000;

const BATTERY_CUTOFF_MV: u32 = 7_000; // 3500mV per cell for a 2S LiPo.
const BATTERY_CRITICAL_MV:u32 = 7_300;
const BATTERY_LOW_MV: u32 = 7_500;
const BATTERY_MAX_MV: u32 = 8_400;
const BATTERY_RANGE_MV: u32 = BATTERY_MAX_MV - BATTERY_CUTOFF_MV;

#[embassy_executor::task]
pub async fn run(mut adc_channel: AnyAdcChannel<ADC1>, mut adc: Adc<'static, ADC1>, level_sender: DroneBatteryLevelSender, status_sender: DroneBatteryStatusSender) {
    adc.set_resolution(Resolution::BITS12);
    adc.set_sample_time(SampleTime::CYCLES112);

    let mut ticker = Ticker::every(Duration::from_millis(500));
    loop {
        let raw = adc.blocking_read(&mut adc_channel) as u32;
        let pin_mv = raw * VDDA_MV / ADC_MAX;
        let mut battery_mv = pin_mv * (R_TOP + R_BOT) / R_BOT;
        if battery_mv < BATTERY_CUTOFF_MV {
            battery_mv = BATTERY_CUTOFF_MV;
        }
        // TODO: Just for testing. Remove this!!
        battery_mv = 8_000;
        let voltage_range = (battery_mv - BATTERY_CUTOFF_MV) as f32 / BATTERY_RANGE_MV as f32;
        let level = BatteryLevel((voltage_range.clamp(0.0, 1.0) * 100.0).ceil() as u8);

        let status = if battery_mv <= BATTERY_CUTOFF_MV {
            BatteryStatus::Cutoff
        } else if battery_mv <= BATTERY_CRITICAL_MV {
            BatteryStatus::Critical
        } else if battery_mv <= BATTERY_LOW_MV {
            BatteryStatus::Low
        }
        else {
            BatteryStatus::Ok
        };

        level_sender.send((Instant::now(), level));
        status_sender.send((Instant::now(), status));

        ticker.next().await;
    }
}