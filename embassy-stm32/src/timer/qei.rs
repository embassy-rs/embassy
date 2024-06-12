//! Quadrature decoder using a timer.

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::timer::vals;

use super::low_level::Timer;
use super::{Channel1Pin, Channel2Pin, GeneralInstance4Channel};
use crate::gpio::{AFType, AnyPin};
use crate::Peripheral;

/// Counting direction
pub enum Direction {
    /// Counting up.
    Upcounting,
    /// Counting down.
    Downcounting,
}

/// Channel 1 marker type.
pub enum Ch1 {}
/// Channel 2 marker type.
pub enum Ch2 {}

/// Wrapper for using a pin with QEI.
pub struct QeiPin<'d, T, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, Channel)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: GeneralInstance4Channel> QeiPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " QEI pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AFType::Input);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                QeiPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }
    };
}

channel_impl!(new_ch1, Ch1, Channel1Pin);
channel_impl!(new_ch2, Ch2, Channel2Pin);

/// Quadrature decoder driver.
pub struct Qei<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> Qei<'d, T> {
    /// Create a new quadrature decoder driver.
    pub fn new(tim: impl Peripheral<P = T> + 'd, _ch1: QeiPin<'d, T, Ch1>, _ch2: QeiPin<'d, T, Ch2>) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd) -> Self {
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
