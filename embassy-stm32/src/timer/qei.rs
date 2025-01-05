//! Quadrature decoder using a timer.

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::timer::vals::{self, Sms};

use super::low_level::Timer;
use super::{Channel1Pin, Channel2Pin, GeneralInstance4Channel};
use crate::gpio::{AfType, AnyPin, Pull};
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

#[doc = "See STMicro AN4013 for §2.3 for more information"]
#[derive(Clone, Eq, PartialEq, Copy, Debug)]
pub enum QeiMode {
    #[doc = "Direct alias for Sms::ENCODER_MODE_1"]
    Mode1,
    #[doc = "Direct alias for Sms::ENCODER_MODE_2"]
    Mode2,
    #[doc = "Direct alias for Sms::ENCODER_MODE_3"]
    Mode3,
}

impl From<QeiMode> for Sms {
    fn from(mode: QeiMode) -> Self {
        match mode {
            QeiMode::Mode1 => Sms::ENCODER_MODE_1,
            QeiMode::Mode2 => Sms::ENCODER_MODE_2,
            QeiMode::Mode3 => Sms::ENCODER_MODE_3,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct QeiConfig {
    /// Encoder Mode
    pub encoder_mode: QeiMode,

    /// Set the pull configuration for the RX pin.
    pub pull: Pull,
}

impl QeiConfig {
    /// Default constructor
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the encoder mode
    pub fn with_encoder_mode(mut self, mode: QeiMode) -> Self {
        self.encoder_mode = mode;
        self
    }

    /// Set the pull configuration
    pub fn with_pull(mut self, pull: Pull) -> Self {
        self.pull = pull;
        self
    }
}

impl Default for QeiConfig {
    #[doc = "Arbitrary defaults to preserve backwards compatibility"]
    fn default() -> Self {
        Self {
            encoder_mode: QeiMode::Mode3,
            pull: Pull::None,
        }
    }
}

/// Quadrature decoder driver.
pub struct Qei<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
    _ch1: PeripheralRef<'d, AnyPin>,
    _ch2: PeripheralRef<'d, AnyPin>,
}

impl<'d, T: GeneralInstance4Channel> Qei<'d, T> {
    /// Create a new quadrature decoder driver.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        ch2: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
    ) -> Self {
        Self::new_inner(tim, ch1, ch2, QeiConfig::default())
    }

    /// Create new quadrature encoder driver with non-default config.
    pub fn new_with_config(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        ch2: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
        config: QeiConfig,
    ) -> Self {
        Self::new_inner(tim, ch1, ch2, config)
    }

    fn new_inner(
        tim: impl Peripheral<P = T> + 'd,
        ch1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        ch2: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
        config: QeiConfig,
    ) -> Self {
        let inner = Timer::new(tim);
        let r = inner.regs_gp16();

        // Due to generics and specific typesig of Peripheral<P> repeating
        // this block of code twice is less bad than the alternative
        // extra types and traits.
        //
        // Use of into_ref!() is destructive, and the resulting ref
        // is later used when constructing the return type of this fn
        into_ref!(ch1);
        critical_section::with(|_| {
            ch1.set_low();
            ch1.set_as_af(ch1.af_num(), AfType::input(config.pull));
        });
        into_ref!(ch2);
        critical_section::with(|_| {
            ch2.set_low();
            ch2.set_as_af(ch2.af_num(), AfType::input(config.pull));
        });

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
            w.set_sms(config.encoder_mode.into());
        });

        r.arr().modify(|w| w.set_arr(u16::MAX));
        r.cr1().modify(|w| w.set_cen(true));

        Self {
            inner,
            _ch1: ch1.map_into(),
            _ch2: ch2.map_into(),
        }
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
