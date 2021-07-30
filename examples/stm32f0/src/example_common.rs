#![macro_use]

use defmt_rtt as _; // global logger
use panic_probe as _;

pub use defmt::*;

use core::sync::atomic::{AtomicUsize, Ordering};
use embassy_stm32::rcc;
use embassy_stm32::Config;

pub fn config() -> Config {
    let mut rcc_config = rcc::Config::default();
    rcc_config.enable_debug_wfe = true;
    Config::default().rcc(rcc_config)
}

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}
