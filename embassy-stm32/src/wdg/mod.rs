use core::marker::PhantomData;

use embassy::time::Duration;
use embassy_hal_common::{unborrow, Unborrow};
use stm32_metapac::iwdg::vals::{Key, Pr};

use crate::rcc::LSI_FREQ;
use crate::time::Hertz;

pub struct IndependentWatchdog<'d, T: Instance> {
    wdg: PhantomData<&'d mut T>,
}

// 12-bit counter
const MAX_RL: u16 = 0xFFF;

/// Calculates maximum watchdog timeout (RL = 0xFFF) for a given prescaler
const fn max_timeout(prescaler: u8) -> Duration {
    Duration::from_micros(1_000_000 / (LSI_FREQ.0 / prescaler as u32) as u64 * MAX_RL as u64)
}

/// Calculates watchdog reload value for the given prescaler and desired timeout
const fn reload_value(prescaler: u8, timeout: Duration) -> u16 {
    ((LSI_FREQ.0 / prescaler as u32) as u64 * timeout.as_micros() / 1_000_000) as u16
}

impl<'d, T: Instance> IndependentWatchdog<'d, T> {
    pub fn new(_instance: impl Unborrow<Target = T> + 'd, timeout: Duration) -> Self {
        unborrow!(_instance);

        // Find lowest prescaler value, which makes watchdog period longer or equal to timeout.
        // This iterates from 4 (2^2) to 256 (2^8).
        let psc_power = unwrap!((2..=8).find(|psc_power| {
            let psc = 2u8.pow(*psc_power);
            timeout <= max_timeout(psc)
        }));

        // Prescaler value
        let psc = 2u8.pow(psc_power);

        // Convert prescaler power to PR register value
        let pr = psc_power as u8 - 2;
        assert!(pr <= 0b110);

        // Reload value
        let rl = reload_value(psc, timeout);

        let wdg = T::regs();
        unsafe {
            wdg.kr().write(|w| w.set_key(Key::ENABLE));
            wdg.pr().write(|w| w.set_pr(Pr(pr)));
            wdg.rlr().write(|w| w.set_rl(rl));
        }

        IndependentWatchdog {
            wdg: PhantomData::default(),
        }
    }

    pub unsafe fn unleash(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::START));
    }

    pub unsafe fn pet(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::RESET));
    }
}

mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::iwdg::Iwdg;
    }
}

pub trait Instance: sealed::Instance {}

foreach_peripheral!(
    (iwdg, $inst:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            fn regs() -> crate::pac::iwdg::Iwdg {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);
