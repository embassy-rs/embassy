//! Quadrature decoder using a timer.

use stm32_metapac::timer::vals::{self, Sms};

use super::low_level::Timer;
pub use super::{Ch1, Ch2};
use super::{GeneralInstance4Channel, TimerPin};
use crate::Peri;
use crate::gpio::{AfType, Flex, Pull};
use crate::timer::TimerChannel;

/// Qei driver config.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct Config {
    /// Configures the internal pull up/down resistor for Qei's channel 1 pin.
    pub ch1_pull: Pull,
    /// Configures the internal pull up/down resistor for Qei's channel 2 pin.
    pub ch2_pull: Pull,
    /// Specifies the encoder mode to use for the Qei peripheral.
    pub mode: QeiMode,
}

impl Default for Config {
    /// Arbitrary defaults to preserve backwards compatibility
    fn default() -> Self {
        Self {
            ch1_pull: Pull::None,
            ch2_pull: Pull::None,
            mode: QeiMode::Mode3,
        }
    }
}

/// See STMicro AN4013 for §2.3 for more information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub enum QeiMode {
    /// Direct alias for [`Sms::ENCODER_MODE_1`]
    Mode1,
    /// Direct alias for [`Sms::ENCODER_MODE_2`]
    Mode2,
    /// Direct alias for [`Sms::ENCODER_MODE_3`]
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

/// Counting direction
pub enum Direction {
    /// Counting up.
    Upcounting,
    /// Counting down.
    Downcounting,
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
    _ch1: Flex<'d>,
    _ch2: Flex<'d>,
}

impl<'d, T: GeneralInstance4Channel> Qei<'d, T> {
    /// Create a new quadrature decoder driver, with a given [`Config`].
    #[allow(unused)]
    pub fn new<CH1: QeiChannel, CH2: QeiChannel, #[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: Peri<'d, if_afio!(impl TimerPin<T, CH1, A>)>,
        ch2: Peri<'d, if_afio!(impl TimerPin<T, CH2, A>)>,
        config: Config,
    ) -> Self {
        // Configure the pins to be used for the QEI peripheral.
        critical_section::with(|_| {
            ch1.set_low();
            set_as_af!(ch1, AfType::input(config.ch1_pull));

            ch2.set_low();
            set_as_af!(ch2, AfType::input(config.ch2_pull));
        });

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
            w.set_sms(config.mode.into());
        });

        r.arr().modify(|w| w.set_arr(u16::MAX));
        r.cr1().modify(|w| w.set_cen(true));

        Self {
            inner,
            _ch1: Flex::new(ch1),
            _ch2: Flex::new(ch2),
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
