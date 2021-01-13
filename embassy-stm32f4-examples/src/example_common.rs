#![macro_use]

use defmt_rtt as _; // global logger
use panic_probe as _;

pub use defmt::*;

use core::sync::atomic::{AtomicUsize, Ordering};

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}
