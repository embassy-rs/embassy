#![macro_use]

use defmt_rtt as _; // global logger
use nrf52840_hal as _;
use panic_probe as _;

pub use defmt::{info, intern};

use core::sync::atomic::{AtomicUsize, Ordering};

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}

macro_rules! depanic {
    ($( $i:expr ),*) => {
        {
            defmt::error!($( $i ),*);
            panic!();
        }
    }
}

pub trait Dewrap<T> {
    /// dewrap = defmt unwrap
    fn dewrap(self) -> T;

    /// dexpect = defmt expect
    fn dexpect<M: defmt::Format>(self, msg: M) -> T;
}

impl<T> Dewrap<T> for Option<T> {
    fn dewrap(self) -> T {
        match self {
            Some(t) => t,
            None => depanic!("Dewrap failed: enum is none"),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Some(t) => t,
            None => depanic!("Unexpected None: {:?}", msg),
        }
    }
}

impl<T, E: defmt::Format> Dewrap<T> for Result<T, E> {
    fn dewrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("Dewrap failed: {:?}", e),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("Unexpected error: {:?}: {:?}", msg, e),
        }
    }
}
