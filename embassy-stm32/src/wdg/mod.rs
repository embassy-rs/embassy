//! Watchdog Timer (IWDG, WWDG)
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use stm32_metapac::iwdg::vals::{Key, Pr};

use crate::rcc::LSI_FREQ;

/// Independent watchdog (IWDG) driver.
pub struct IndependentWatchdog<'d, T: Instance> {
    wdg: PhantomData<&'d mut T>,
}

// 12-bit counter
const MAX_RL: u16 = 0xFFF;

/// Calculates maximum watchdog timeout in us (RL = 0xFFF) for a given prescaler
const fn get_timeout_us(prescaler: u16, reload_value: u16) -> u32 {
    1_000_000 * (reload_value + 1) as u32 / (LSI_FREQ.0 / prescaler as u32)
}

/// Calculates watchdog reload value for the given prescaler and desired timeout
const fn reload_value(prescaler: u16, timeout_us: u32) -> u16 {
    (timeout_us / prescaler as u32 * LSI_FREQ.0 / 1_000_000) as u16 - 1
}

impl<'d, T: Instance> IndependentWatchdog<'d, T> {
    /// Creates an IWDG (Independent Watchdog) instance with a given timeout value in microseconds.
    ///
    /// [Self] has to be started with [Self::unleash()].
    /// Once timer expires, MCU will be reset. To prevent this, timer must be reloaded by repeatedly calling [Self::pet()] within timeout interval.
    pub fn new(_instance: impl Peripheral<P = T> + 'd, timeout_us: u32) -> Self {
        into_ref!(_instance);

        // Find lowest prescaler value, which makes watchdog period longer or equal to timeout.
        // This iterates from 4 (2^2) to 256 (2^8).
        let psc_power = unwrap!((2..=8).find(|psc_power| {
            let psc = 2u16.pow(*psc_power);
            timeout_us <= get_timeout_us(psc, MAX_RL)
        }));

        // Prescaler value
        let psc = 2u16.pow(psc_power);

        // Convert prescaler power to PR register value
        let pr = psc_power as u8 - 2;
        assert!(pr <= 0b110);

        // Reload value
        let rl = reload_value(psc, timeout_us);

        let wdg = T::regs();
        wdg.kr().write(|w| w.set_key(Key::ENABLE));
        wdg.pr().write(|w| w.set_pr(Pr::from_bits(pr)));
        wdg.rlr().write(|w| w.set_rl(rl));

        trace!(
            "Watchdog configured with {}us timeout, desired was {}us (PR={}, RL={})",
            get_timeout_us(psc, rl),
            timeout_us,
            pr,
            rl
        );

        IndependentWatchdog { wdg: PhantomData }
    }

    /// Unleash (start) the watchdog.
    pub fn unleash(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::START));
    }

    /// Pet (reload, refresh) the watchdog.
    pub fn pet(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::RESET));
    }
}

mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::iwdg::Iwdg;
    }
}

/// IWDG instance trait.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_compute_timeout_us() {
        assert_eq!(125, get_timeout_us(4, 0));
        assert_eq!(512_000, get_timeout_us(4, MAX_RL));

        assert_eq!(8_000, get_timeout_us(256, 0));
        assert_eq!(32_768_000, get_timeout_us(256, MAX_RL));

        assert_eq!(8_000_000, get_timeout_us(64, 3999));
    }

    #[test]
    fn can_compute_reload_value() {
        assert_eq!(0xFFF, reload_value(4, 512_000));
        assert_eq!(0xFFF, reload_value(256, 32_768_000));

        assert_eq!(3999, reload_value(64, 8_000_000));
    }
}
