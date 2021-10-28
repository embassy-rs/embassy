#![macro_use]

use defmt_rtt as _;
use embassy_stm32::time::U32Ext;
use embassy_stm32::Config;
// global logger
use panic_probe as _;

pub use defmt::*;

use core::sync::atomic::{AtomicUsize, Ordering};

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

#[allow(unused)]
pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(200.mhz().into());
    config
}
