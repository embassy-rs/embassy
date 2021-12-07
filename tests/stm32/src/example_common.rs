#![macro_use]

use defmt_rtt as _;
#[allow(unused)]
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
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

pub fn config() -> Config {
    #[allow(unused_mut)]
    let mut config = Config::default();

    #[cfg(feature = "stm32h755zi")]
    {
        config.rcc.sys_ck = Some(Hertz(400_000_000));
        config.rcc.pll1.q_ck = Some(Hertz(100_000_000));
    }

    config
}
