//! Watchdog Timer (IWDG, WWDG)
use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use stm32_metapac::iwdg::vals::{Key, Pr};

use crate::Peri;
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
    pub fn new(_instance: Peri<'d, T>, timeout_us: u32) -> Self {
        // Find lowest prescaler value, which makes watchdog period longer or equal to timeout.
        // This iterates from 4 (2^2) to 256 (2^8).
        let psc_power = unwrap!((2..=8).find(|psc_power| {
            let psc = 2u16.pow(*psc_power);
            timeout_us <= get_timeout_us(psc, MAX_RL)
        }));

        // Prescaler value
        let psc = 2u16.pow(psc_power);

        #[cfg(not(iwdg_v3))]
        assert!(psc <= 256, "IWDG prescaler should be no more than 256");
        #[cfg(iwdg_v3)] // H5, U5, WBA
        assert!(psc <= 1024, "IWDG prescaler should be no more than 1024");

        // Convert prescaler power to PR register value
        let pr = psc_power as u8 - 2;

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

trait SealedInstance {
    fn regs() -> crate::pac::iwdg::Iwdg;
}

/// IWDG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

foreach_peripheral!(
    (iwdg, $inst:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::iwdg::Iwdg {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);

// ============================================================
// WWDG (Window Watchdog)
// ============================================================

/// Returns `ceil(duration_us * pclk1_hz / (prescaler_mul * 4096 * 1_000_000))`.
///
/// Uses `u64` arithmetic throughout to prevent overflow.
#[cfg(any(wwdg, test))]
fn wwdg_ticks(duration_us: u32, pclk1_hz: u32, prescaler_mul: u32) -> u64 {
    let num = duration_us as u64 * pclk1_hz as u64;
    let den = prescaler_mul as u64 * 4096 * 1_000_000;
    (num + den - 1) / den
}

#[cfg(wwdg)]
use stm32_metapac::wwdg::vals::Wdgtb;

/// Window watchdog (WWDG) driver.
///
/// Once activated via [`WindowWatchdog::new`], the WWDG cannot be stopped
/// without a system reset.
///
/// The counter counts from `T` down to 0x3F (63), triggering a reset when it
/// reaches 0x3F. Petting the watchdog while the counter is still above the
/// window register `W` (the *closed window*) also causes an immediate reset.
///
/// ```text
/// T_initial ──count down──▶ W ──count down──▶ 0x40 ──▶ 0x3F (RESET)
/// |◄──── closed window ────►|◄──── open window ────►|
/// ```
#[cfg(wwdg)]
pub struct WindowWatchdog<'d, T: WwdgInstance> {
    wdg: PhantomData<&'d mut T>,
    /// Counter value written to CR on every [`pet`](WindowWatchdog::pet) call.
    counter: u8,
}

#[cfg(wwdg)]
impl<'d, T: WwdgInstance> WindowWatchdog<'d, T> {
    /// Creates and immediately starts the window watchdog.
    ///
    /// - `timeout_us`: total watchdog period in microseconds (counter-to-reset time).
    /// - `window_us`: closed-window duration in microseconds. During this initial
    ///   portion of the period, petting the watchdog causes a reset. Pass `0` to
    ///   disable the window restriction (allow petting at any time within the period).
    ///   Must be strictly less than `timeout_us`.
    pub fn new(_instance: Peri<'d, T>, timeout_us: u32, window_us: u32) -> Self {
        assert!(window_us < timeout_us, "window_us must be less than timeout_us");

        crate::rcc::enable_and_reset::<T>();

        let pclk1 = T::frequency().0;

        // Select the smallest prescaler such that ticks falls in [1, 64].
        // wwdg_v1 has a 2-bit WDGTB field (DIV1–DIV8); wwdg_v2 has a 3-bit field (DIV1–DIV128).
        #[cfg(wwdg_v2)]
        const PRESCALER_MULS: &[u32] = &[1, 2, 4, 8, 16, 32, 64, 128];
        #[cfg(not(wwdg_v2))]
        const PRESCALER_MULS: &[u32] = &[1, 2, 4, 8];
        let (prescaler_mul, ticks) = unwrap!(
            PRESCALER_MULS.iter().find_map(|&mul| {
                let t = wwdg_ticks(timeout_us, pclk1, mul);
                if (1..=64).contains(&t) { Some((mul, t)) } else { None }
            }),
            "WWDG: timeout_us is out of range for all prescalers"
        );

        // T = 63 + ticks; T is in [0x40, 0x7F].
        let t_val = 63u8 + ticks as u8;

        // W = T − floor(window_us * pclk1 / (prescaler_mul * 4096 * 1_000_000)).
        // When window_us == 0 the closed window is empty and W == T.
        let den = prescaler_mul as u64 * 4096 * 1_000_000;
        let closed_ticks = (window_us as u64 * pclk1 as u64) / den;
        let w_val = t_val - closed_ticks as u8;

        // WDGTB bits are log2(prescaler_mul): DIV1=0, DIV2=1, DIV4=2, ...
        let wdgtb = Wdgtb::from_bits(prescaler_mul.trailing_zeros() as u8);

        let regs = T::regs();

        // Write CFR before CR: prescaler and window must be set before activation.
        regs.cfr().write(|cfr| {
            cfr.set_wdgtb(wdgtb);
            cfr.set_w(w_val);
        });

        // Activate watchdog (WDGA = 1 is hardware-irreversible).
        regs.cr().write(|cr| {
            cr.set_t(t_val);
            cr.set_wdga(true);
        });

        trace!(
            "WWDG configured: timeout={}us window={}us pclk1={} prescaler=x{} T={} W={}",
            timeout_us, window_us, pclk1, prescaler_mul, t_val, w_val,
        );

        WindowWatchdog {
            wdg: PhantomData,
            counter: t_val,
        }
    }

    /// Pet (reload) the watchdog.
    ///
    /// Must be called while the counter has fallen into the open window
    /// (counter ≤ W). Calling too early (counter > W) causes an immediate reset.
    pub fn pet(&mut self) {
        T::regs().cr().write(|cr| {
            cr.set_t(self.counter);
            cr.set_wdga(true);
        });
    }
}

#[cfg(wwdg)]
trait WwdgSealedInstance {
    fn regs() -> crate::pac::wwdg::Wwdg;
}

/// WWDG instance trait.
#[cfg(wwdg)]
#[allow(private_bounds)]
pub trait WwdgInstance: WwdgSealedInstance + PeripheralType + crate::rcc::RccPeripheral {}

#[cfg(wwdg)]
foreach_peripheral!(
    (wwdg, $inst:ident) => {
        impl WwdgSealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::wwdg::Wwdg {
                crate::pac::$inst
            }
        }

        impl WwdgInstance for crate::peripherals::$inst {}
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

#[cfg(test)]
mod wwdg_tests {
    use super::wwdg_ticks;

    // Typical PCLK1 frequency for STM32WBA65 at 64 MHz.
    const PCLK1: u32 = 64_000_000;

    #[test]
    fn ticks_rounded_up() {
        // 1000 µs * 64 MHz / (1 * 4096 * 1e6) = 15.625 → ceil = 16
        assert_eq!(16, wwdg_ticks(1_000, PCLK1, 1));
    }

    #[test]
    fn ticks_exact() {
        // 1024 µs * 64 MHz / (1 * 4096 * 1e6) = 16.0 exactly
        assert_eq!(16, wwdg_ticks(1_024, PCLK1, 1));
    }

    #[test]
    fn ticks_just_above_exact() {
        // 1025 µs → ceil(16.015625) = 17
        assert_eq!(17, wwdg_ticks(1_025, PCLK1, 1));
    }

    #[test]
    fn ticks_large_prescaler() {
        // 100 ms, prescaler 128:
        // 100_000 * 64_000_000 / (128 * 4096 * 1e6) ≈ 12.207 → ceil = 13
        assert_eq!(13, wwdg_ticks(100_000, PCLK1, 128));
    }

    #[test]
    fn ticks_minimum_duration() {
        // 1 µs: ceil(64_000_000 / 4_096_000_000) = 1
        assert_eq!(1, wwdg_ticks(1, PCLK1, 1));
    }
}
