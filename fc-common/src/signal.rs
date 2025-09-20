use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::{Receiver, Sender};

pub struct Signal<T: Clone + Default + PartialEq + 'static, const N: usize> {
    receiver: Receiver<'static, CriticalSectionRawMutex, T, N>,
    last_value: T,
}

impl<T: Clone + Default + PartialEq + 'static, const N: usize> Signal<T, N> {
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

pub struct SignalEmitter<T: Clone + Default + PartialEq + 'static, const N: usize> {
    sender: Sender<'static, CriticalSectionRawMutex, T, N>,
    last_emitted_value: T,
}

impl<T: Clone + Default + PartialEq + 'static, const N: usize> SignalEmitter<T, N> {
    pub fn new(sender: Sender<'static, CriticalSectionRawMutex, T, N>) -> Self {
        Self { sender, last_emitted_value: T::default() }
    }

    pub fn emit(&mut self, value: T) {
        self.last_emitted_value = value.clone();
        self.sender.send(value);
    }

    pub fn emit_if_changed(&mut self, value: T) {
        if self.last_emitted_value != value {
            self.last_emitted_value = value.clone();
            self.sender.send(value);
        }
    }
}

/// Creates a Watch and type aliases for a new signal.
/// Takes three arguments:
/// - Signal name
/// - The datatype of the signal.
/// - The total number of receivers.
///
/// ## Example
/// ```rust
/// define_signal!(Foo, u8, 2);
/// ```
/// Would result in:
///
/// ```rust
/// const FOO_SUBSCRIBERS: usize = 2;
///
/// pub static FOO_WATCH: Watch<CriticalSectionRawMutex, u8, FOO_SUBSCRIBERS> = Watch::new();
///
/// pub type FooSender = Sender<'static, CriticalSectionRawMutex, u8, FOO_SUBSCRIBERS>;
///
/// pub type FooReceiver = Receiver<'static, CriticalSectionRawMutex, u8, FOO_SUBSCRIBERS>;
/// ```
#[macro_export]
macro_rules! define_signal {
    ($NAME:ident, $Ty:ty, $subs:expr) => {
        ::paste::paste! {
            const [<$NAME:snake:upper _SUBSCRIBERS>]: usize = $subs;

            static [<$NAME:snake:upper _WATCH>]: embassy_sync::watch::Watch<CriticalSectionRawMutex, $Ty, [<$NAME:snake:upper _SUBSCRIBERS>]> = embassy_sync::watch::Watch::new();

            pub type [<$NAME:camel Emitter>] = SignalEmitter<$Ty, [<$NAME:snake:upper _SUBSCRIBERS>]>;

            pub type  [<$NAME:camel Signal>] = Signal<$Ty, [<$NAME:snake:upper _SUBSCRIBERS>]>;

            pub fn [<$NAME:snake _signal>]() -> [<$NAME:camel Signal>] {
               Signal::new([<$NAME:snake:upper _WATCH>].receiver().unwrap(), $Ty::default())
            }

            pub fn [<new_$NAME:snake _signal_emitter>]() -> [<$NAME:camel Emitter>] {
                SignalEmitter::new([<$NAME:snake:upper _WATCH>].sender())
            }
        }
    };
}