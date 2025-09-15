//! Quadrature decoder using a timer.

use core::marker::PhantomData;

use stm32_metapac::timer::vals;

use super::low_level::Timer;
pub use super::{Ch1, Ch2};
use super::{GeneralInstance4Channel, TimerPin};
use crate::gpio::{AfType, AnyPin, Pull};
use crate::timer::TimerChannel;
use crate::Peri;

/// Counting direction
pub enum Direction {
    /// Counting up.
    Upcounting,
    /// Counting down.
    Downcounting,
}

/// Wrapper for using a pin with QEI.
pub struct QeiPin<'d, T, Channel, #[cfg(afio)] A> {
    #[allow(unused)]
    pin: Peri<'d, AnyPin>,
    phantom: PhantomData<if_afio!((T, Channel, A))>,
}

impl<'d, T: GeneralInstance4Channel, C: QeiChannel, #[cfg(afio)] A> if_afio!(QeiPin<'d, T, C, A>) {
    /// Create a new QEI pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl TimerPin<T, C, A>)>) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            set_as_af!(pin, AfType::input(Pull::None));
        });
        QeiPin {
            pin: pin.into(),
            phantom: PhantomData,
        }
    }
}

trait SealedQeiChannel: TimerChannel {}

/// Marker trait for a timer channel eligible for use with QEI.
#[expect(private_bounds)]
pub trait QeiChannel: SealedQeiChannel {}

impl QeiChannel for Ch1 {}
impl QeiChannel for Ch2 {}

impl SealedQeiChannel for Ch1 {}
impl SealedQeiChannel for Ch2 {}

/// Quadrature decoder driver.
pub struct Qei<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> Qei<'d, T> {
    /// Create a new quadrature decoder driver.
    #[allow(unused)]
    pub fn new<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: if_afio!(QeiPin<'d, T, Ch1, A>),
        ch2: if_afio!(QeiPin<'d, T, Ch2, A>),
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: Peri<'d, T>) -> Self {
        let inner = Timer::new(tim);
        let r = inner.regs_gp16();

        // Configure TxC1 and TxC2 as captures
        r.ccmr_input(0).modify(|w| {
            w.set_ccs(0, vals::CcmrInputCcs::TI4);
            w.set_ccs(1, vals::CcmrInputCcs::TI4);
        });

        // enable and configure to capture on rising edge
        r.ccer().modify(|w| {
            w.set_cce(0, true);
            w.set_cce(1, true);

            w.set_ccp(0, false);
            w.set_ccp(1, false);
        });

        r.smcr().modify(|w| {
            w.set_sms(vals::Sms::ENCODER_MODE_3);
        });

        r.arr().modify(|w| w.set_arr(u16::MAX));
        r.cr1().modify(|w| w.set_cen(true));

        Self { inner }
    }

    /// Get direction.
    pub fn read_direction(&self) -> Direction {
        match self.inner.regs_gp16().cr1().read().dir() {
            vals::Dir::DOWN => Direction::Downcounting,
            vals::Dir::UP => Direction::Upcounting,
        }
    }

    /// Get count.
    pub fn count(&self) -> u16 {
        self.inner.regs_gp16().cnt().read().cnt()
    }
}
