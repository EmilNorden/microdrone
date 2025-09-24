use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::{Receiver, Sender};

pub trait SignalBase<T> {
    /// Awaits and returns the next value emitted on this Signal.

    async fn next_value(&mut self) -> T;

    /// Awaits and returns the next *distinct* value emitted on this Signal.
    ///
    /// For instance, if the source emits the values `1, 2, 2, 3`, any consumers using this method will only see `1, 2, 3`.
    async fn next_distinct(&mut self) -> T;
    fn get(&mut self) -> T;
}

pub struct Signal<T: Clone + Default + PartialEq + 'static, const N: usize> {
    receiver: Receiver<'static, CriticalSectionRawMutex, T, N>,
    last_value: T,
}

impl<T: Clone + Default + PartialEq + 'static, const N: usize> SignalBase<T> for Signal<T, N> {
    async fn next_value(&mut self) -> T {
        self.last_value = self.receiver.changed().await;
        self.last_value.clone()
    }

    async fn next_distinct(&mut self) -> T {
        loop {
            let value = self.receiver.changed().await;
            if value != self.last_value {
                self.last_value = value.clone();
                return value;
            }
        }
    }

    fn get(&mut self) -> T {
        if let Some(value) = self.receiver.try_get() {
            self.last_value = value;
        }

        self.last_value.clone()
    }
}

impl<T: Clone + Default + PartialEq + 'static, const N: usize> Signal<T, N> {
    pub fn new(
        receiver: Receiver<'static, CriticalSectionRawMutex, T, N>,
        default_value: T,
    ) -> Self {
        Self {
            receiver,
            last_value: default_value,
        }
    }
}

pub struct SignalEmitter<T: Clone + Default + PartialEq + 'static, const N: usize> {
    sender: Sender<'static, CriticalSectionRawMutex, T, N>,
    last_emitted_value: T,
}

impl<T: Clone + Default + PartialEq + 'static, const N: usize> SignalEmitter<T, N> {
    pub fn new(sender: Sender<'static, CriticalSectionRawMutex, T, N>) -> Self {
        Self {
            sender,
            last_emitted_value: T::default(),
        }
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
/// ```rust,ignore
/// define_signal!(Foo, u8, 2);
/// ```
/// Would result in:
///
/// ```rust
/// # use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
/// # use embassy_sync::watch::{Watch, Receiver, Sender};
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

            pub struct [<$NAME:camel Emitter>](SignalEmitter<$Ty, [<$NAME:snake:upper _SUBSCRIBERS>]>);

            impl [<$NAME:camel Emitter>]{
                pub fn emit(&mut self, value: $Ty) {
                    self.0.emit(value);
                }

                pub fn emit_if_changed(&mut self, value: $Ty) {
                    self.0.emit_if_changed(value);
                }
            }

            pub struct [<$NAME:camel Signal>](Signal<$Ty, [<$NAME:snake:upper _SUBSCRIBERS>]>);

            impl SignalBase<$Ty> for [<$NAME:camel Signal>]{
                async fn next_value(&mut self) -> $Ty {
                    self.0.next_value().await
                }

                async fn next_distinct(&mut self) -> $Ty {
                    self.0.next_distinct().await
                }

                fn get(&mut self) -> $Ty {
                    self.0.get()
                }
            }

            pub fn [<$NAME:snake _signal>]() -> [<$NAME:camel Signal>] {
               let signal = Signal::new([<$NAME:snake:upper _WATCH>].receiver().unwrap(), $Ty::default());
                [<$NAME:camel Signal>](signal)
            }

            pub fn [<new_$NAME:snake _signal_emitter>]() -> [<$NAME:camel Emitter>] {
                let emitter = SignalEmitter::new([<$NAME:snake:upper _WATCH>].sender());
                [<$NAME:camel Emitter>](emitter)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;
    use tokio::time::timeout;

    extern crate std;

    define_signal!(Test1, u8, 1);
    #[tokio::test]
    async fn signal_next_value_when_none_emitted() {
        let mut signal = test1_signal();
        match timeout(Duration::from_millis(100), signal.next_value()).await {
            Ok(_) => {
                panic!("next_value returned a value when not expected to");
            }
            Err(_) => {}
        }
    }

    define_signal!(Test2, u8, 1);
    #[tokio::test]
    async fn signal_next_value() {
        let mut emitter = new_test2_signal_emitter();
        let mut signal = test2_signal();

        emitter.emit(0);
        assert_eq!(signal.next_value().await, 0);

        emitter.emit(1);
        assert_eq!(signal.next_value().await, 1);

        emitter.emit(2);
        emitter.emit(3);
        assert_eq!(signal.next_value().await, 3);
    }

    define_signal!(Test3, u8, 1);
    #[tokio::test]
    async fn signal_next_distinct_when_none_emitted() {
        let mut signal = test3_signal();

        match timeout(Duration::from_millis(100), signal.next_distinct()).await {
            Ok(_) => {
                panic!("next_distinct returned a value when not expected to");
            }
            Err(_) => {}
        }
    }

    define_signal!(Test4, u8, 1);
    #[tokio::test]
    async fn signal_next_distinct() {
        let mut emitter = new_test4_signal_emitter();
        let mut signal = test4_signal();

        emitter.emit(5);
        assert_eq!(signal.next_distinct().await, 5);

        emitter.emit(5);
        match timeout(Duration::from_millis(100), signal.next_distinct()).await {
            Ok(_) => {
                panic!("next_distinct returned a value when not expected to");
            }
            Err(_) => {}
        }

        emitter.emit(6);
        assert_eq!(signal.next_distinct().await, 6);
    }
}
