use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Receiver;

pub struct Telemetry<T: Clone + 'static, const N: usize> {
    receiver: Receiver<'static, CriticalSectionRawMutex, T, N>,
    last_value: T,
}

impl<T: Clone + 'static, const N: usize> Telemetry<T, N> {
    pub fn new(receiver: Receiver<'static, CriticalSectionRawMutex, T, N>, default_value: T) -> Self {
        Self {
            receiver,
            last_value: default_value,
        }
    }

    pub async fn next_value(&mut self) -> T {
        self.last_value = self.receiver.changed().await;
        self.last_value.clone()
    }

    pub fn get(&mut self) -> T {
        if let Some(value) = self.receiver.try_get() {
            self.last_value = value;
        }

        self.last_value.clone()
    }
}

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
#[macro_export]
macro_rules! telemetry_type {
    ($NAME:ident, $Ty:ty, $subs:expr, $default_value:expr) => {
        ::paste::paste! {
            const [<$NAME:snake:upper _SUBSCRIBERS>]: usize = $subs;

            static [<$NAME:snake:upper _WATCH>]: embassy_sync::watch::Watch<CriticalSectionRawMutex, (embassy_time::Instant, $Ty), [<$NAME:snake:upper _SUBSCRIBERS>]> = embassy_sync::watch::Watch::new();

            pub type [<$NAME:camel Sender>] = embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, (embassy_time::Instant, $Ty), [<$NAME:snake:upper _SUBSCRIBERS>]>;

            pub type  [<$NAME:camel Telemetry>] = Telemetry<(embassy_time::Instant, $Ty), [<$NAME:snake:upper _SUBSCRIBERS>]>;

            pub fn [<$NAME:snake _telemetry>]() -> [<$NAME:camel Telemetry>] {
               Telemetry::new([<$NAME:snake:upper _WATCH>].receiver().unwrap(), (embassy_time::Instant::now(), $default_value))
            }

            pub fn [<$NAME:snake _telemetry_sender>]() -> [<$NAME:camel Sender>] {
               [<$NAME:snake:upper _WATCH>].sender()
            }
        }
    };
}