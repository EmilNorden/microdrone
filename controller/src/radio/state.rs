use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub struct RadioState<B> {
    connected: B,
}

type RadioStateAtomic = RadioState<AtomicBool>;
type RadioStateNonAtomic = RadioState<bool>;

static RADIO_STATE: RadioStateAtomic = RadioStateAtomic {
    connected: AtomicBool::new(false),
};

pub fn update_connected(connected: bool) {
    RADIO_STATE.connected.store(connected, Ordering::Relaxed);
}

/*
static CONTROLLER_STATE: ControllerStateAtomic = ControllerStateAtomic {
    connected: AtomicBool::new(false),
    battery: AtomicU8::new(0),
};
*/
