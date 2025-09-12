use embassy_time::Instant;

pub struct Altitude {
    pub ts: Instant,
    pub meters: f32,
}
pub struct Temperature {
    pub ts: Instant,
    pub temp_c: f32,
}

pub enum Telemetry {
    Altitude(Altitude),
    Temperature(Temperature),
}
