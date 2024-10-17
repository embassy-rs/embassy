//! Quadrature decoder using a timer.

use stm32_metapac::timer::vals;

use super::low_level::Timer;
use super::raw::{RawTimer, RawTimerPin};
use super::{Ch1, Ch2, General4ChInstance, General4ChTim, TimerPin};
use crate::gpio::{AfType, Pull};
use crate::Peripheral;

/// Counting direction
pub enum Direction {
    /// Counting up.
    Upcounting,
    /// Counting down.
    Downcounting,
}

/// Quadrature decoder driver.
pub struct Qei<'d> {
    inner: Timer<'d, General4ChTim>,
    _pins: [RawTimerPin<'d>; 2],
}

impl<'d> Qei<'d> {
    /// Create a new quadrature decoder driver.
    pub fn new<T: General4ChInstance>(
        tim: impl Peripheral<P = T> + 'd,
        ch1_pin: impl Peripheral<P = impl TimerPin<T, Ch1>> + 'd,
        ch2_pin: impl Peripheral<P = impl TimerPin<T, Ch2>> + 'd,
    ) -> Self {
        let raw = RawTimer::new_general_4ch(tim);
        let ch1_pin = RawTimerPin::new(ch1_pin, AfType::input(Pull::None));
        let ch2_pin = RawTimerPin::new(ch2_pin, AfType::input(Pull::None));
        Self::new_inner(raw, [ch1_pin, ch2_pin])
    }

    fn new_inner(raw: RawTimer<'d, General4ChTim>, pins: [RawTimerPin<'d>; 2]) -> Self {
        let inner = Timer::new(raw);

        // Configure TxC1 and TxC2 as captures
        inner.raw.ccmr_input_1ch(0).modify(|w| {
            w.set_ccs(0, vals::CcmrInputCcs::TI4);
            w.set_ccs(1, vals::CcmrInputCcs::TI4);
        });

        // enable and configure to capture on rising edge
        inner.raw.ccer_1ch().modify(|w| {
            w.set_cce(0, true);
            w.set_cce(1, true);

            w.set_ccp(0, false);
            w.set_ccp(1, false);
        });

        inner.raw.smcr_4ch().modify(|w| {
            w.set_sms(vals::Sms::ENCODER_MODE_3);
        });

        inner.raw.arr().modify(|w| w.set_arr(u16::MAX));
        inner.raw.cr1_core().modify(|w| w.set_cen(true));

        Self { inner, _pins: pins }
    }

    /// Get direction.
    pub fn direction(&self) -> Direction {
        match self.inner.raw.cr1_4ch().read().dir() {
            vals::Dir::DOWN => Direction::Downcounting,
            vals::Dir::UP => Direction::Upcounting,
        }
    }

    /// Get count.
    pub fn count(&self) -> u16 {
        self.inner.raw.cnt().read().cnt()
    }
}
