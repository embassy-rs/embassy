//! CTimer-based PWM driver

use embassy_hal_internal::Peri;
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};

use super::{AnyChannel, CTimer, Info, Instance, OutputPin, PwmChannel};
use crate::gpio::{AnyPin, SealedPin};
use crate::pac::ctimer::vals::{
    Mr0i, Mr0r, Mr0s, Mr1i, Mr1r, Mr1s, Mr2i, Mr2r, Mr2s, Mr3i, Mr3r, Mr3s, Pwmen0, Pwmen1, Pwmen2, Pwmen3,
};

/// PWM error.
#[derive(Debug)]
pub enum PwmError {
    /// Invalid Duty Cycle.
    InvalidDutyCycle,
    /// Invalid Channel Number.
    InvalidChannel,
    /// Channel mismatch
    ChannelMismatch,
}

impl Error for PwmError {
    fn kind(&self) -> ErrorKind {
        match self {
            PwmError::InvalidDutyCycle => ErrorKind::Other,
            PwmError::InvalidChannel => ErrorKind::Other,
            PwmError::ChannelMismatch => ErrorKind::Other,
        }
    }
}

/// Pwm Configuration
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// The point at which the counter wraps around.
    ///
    /// This value represents the maximum possible period.
    pub freq: u16,

    /// Duty cycle.
    pub duty_cycle: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            freq: 20_000,
            duty_cycle: 0,
        }
    }
}

/// Pwm driver
pub struct Pwm<'d> {
    info: &'static Info,
    period_ch: Peri<'d, AnyChannel>,
    match_ch: Peri<'d, AnyChannel>,
    pin: Peri<'d, AnyPin>,
    source_freq: u32,
    pwm_freq: u16,
    max_period: u16,
}

impl<'d> Pwm<'d> {
    /// Create Pwm driver with a single pin as output.
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new<T: Instance, MATCH: PwmChannel<T>, PIN: OutputPin<T>>(
        ctimer: CTimer<'d>,
        period_ch: Peri<'d, impl PwmChannel<T>>,
        match_ch: Peri<'d, MATCH>,
        pin: Peri<'d, PIN>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, MATCH, PIN): ValidMatchConfig,
    {
        if period_ch.number() > 3 || match_ch.number() > 3 {
            return Err(PwmError::InvalidChannel);
        }

        if pin.number() != match_ch.number() {
            return Err(PwmError::ChannelMismatch);
        }

        pin.mux();

        let mut inst = Self {
            info: T::info(),
            period_ch: period_ch.into(),
            match_ch: match_ch.into(),
            source_freq: ctimer._freq,
            pwm_freq: config.freq,
            pin: pin.into(),
            max_period: 0,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn enable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(true));
    }

    fn disable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(false));
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        // Enable PWM mode on the match channel
        self.info.regs().pwmc().modify(|w| match self.match_ch.number() {
            0 => {
                w.set_pwmen0(Pwmen0::PWM);
            }
            1 => {
                w.set_pwmen1(Pwmen1::PWM);
            }
            2 => {
                w.set_pwmen2(Pwmen2::PWM);
            }
            3 => {
                w.set_pwmen3(Pwmen3::PWM);
            }
            _ => unreachable!(),
        });

        self.info.regs().mcr().modify(|w| {
            // Clear stop, reset, and interrupt bits for the PWM channel
            match self.match_ch.number() {
                0 => {
                    w.set_mr0i(Mr0i::MR0I_0);
                    w.set_mr0r(Mr0r::MR0R_0);
                    w.set_mr0s(Mr0s::MR0S_0);
                }
                1 => {
                    w.set_mr1i(Mr1i::MR1I_0);
                    w.set_mr1r(Mr1r::MR1R_0);
                    w.set_mr1s(Mr1s::MRIS_0);
                }
                2 => {
                    w.set_mr2i(Mr2i::MR2I_0);
                    w.set_mr2r(Mr2r::MR2R_0);
                    w.set_mr2s(Mr2s::MR2S_0);
                }
                3 => {
                    w.set_mr3i(Mr3i::MR3I_0);
                    w.set_mr3r(Mr3r::MR3R_0);
                    w.set_mr3s(Mr3s::MR3S_0);
                }
                _ => unreachable!(),
            }

            match self.period_ch.number() {
                0 => {
                    w.set_mr0r(Mr0r::MR0R_1);
                }
                1 => {
                    w.set_mr1r(Mr1r::MR1R_1);
                }
                2 => {
                    w.set_mr2r(Mr2r::MR2R_1);
                }
                3 => {
                    w.set_mr3r(Mr3r::MR3R_1);
                }
                _ => unreachable!(),
            }
        });

        // Configure PWM period
        let period = self.source_freq / u32::from(self.pwm_freq) - 1;
        self.max_period = period as u16;
        self.info
            .regs()
            .mr(self.period_ch.number())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;

        self.info
            .regs()
            .mr(self.match_ch.number())
            .write(|w| w.set_match_(u32::from(duty_cycle)));

        // REVISIT: do we need interrupts?

        // Start CTimer
        self.enable();

        Ok(())
    }
}

impl<'d> Drop for Pwm<'d> {
    fn drop(&mut self) {
        self.disable();
        self.pin.set_as_disabled();
    }
}

impl<'d> ErrorType for Pwm<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for Pwm<'d> {
    fn max_duty_cycle(&self) -> u16 {
        self.max_period
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let max_duty = self.max_duty_cycle();

        if duty > max_duty {
            return Err(PwmError::InvalidDutyCycle);
        }

        self.info
            .regs()
            .mr(usize::from(self.match_ch.number()))
            .write(|w| w.set_match_(u32::from(duty)));

        Ok(())
    }
}

impl<'d> embassy_embedded_hal::SetConfig for Pwm<'d> {
    type Config = Config;
    type ConfigError = PwmError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}

trait SealedValidMatchConfig {}

/// Valid match channel + pin configuration marker trait
#[allow(private_bounds)]
pub trait ValidMatchConfig: SealedValidMatchConfig {}

macro_rules! impl_valid_match {
    ($peri:ident, $ch:ident, $pin:ident, $n:literal) => {
        impl SealedValidMatchConfig
            for (
                crate::peripherals::$peri,
                crate::peripherals::$ch,
                crate::peripherals::$pin,
            )
        {
        }

        impl ValidMatchConfig
            for (
                crate::peripherals::$peri,
                crate::peripherals::$ch,
                crate::peripherals::$pin,
            )
        {
        }
    };
}

// CTIMER0 match channels
#[cfg(feature = "swd-swo-as-gpio")]
impl_valid_match!(CTIMER0, CTIMER0_CH0, P0_2, 0);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_valid_match!(CTIMER0, CTIMER0_CH1, P0_3, 1);

impl_valid_match!(CTIMER0, CTIMER0_CH0, P0_16, 0);
impl_valid_match!(CTIMER0, CTIMER0_CH1, P0_17, 1);
impl_valid_match!(CTIMER0, CTIMER0_CH2, P0_18, 2);
impl_valid_match!(CTIMER0, CTIMER0_CH3, P0_19, 3);

impl_valid_match!(CTIMER0, CTIMER0_CH0, P0_22, 0);
impl_valid_match!(CTIMER0, CTIMER0_CH1, P0_23, 1);
impl_valid_match!(CTIMER0, CTIMER0_CH2, P1_0, 2);
impl_valid_match!(CTIMER0, CTIMER0_CH3, P1_1, 3);

#[cfg(feature = "sosc-as-gpio")]
impl_valid_match!(CTIMER0, CTIMER0_CH2, P3_30, 2);
#[cfg(feature = "sosc-as-gpio")]
impl_valid_match!(CTIMER0, CTIMER0_CH3, P3_31, 3);

// CTIMER1 match channels
impl_valid_match!(CTIMER1, CTIMER1_CH0, P1_2, 0);
impl_valid_match!(CTIMER1, CTIMER1_CH1, P1_3, 1);
impl_valid_match!(CTIMER1, CTIMER1_CH2, P1_4, 2);
impl_valid_match!(CTIMER1, CTIMER1_CH3, P1_5, 3);

impl_valid_match!(CTIMER1, CTIMER1_CH0, P2_4, 0);
impl_valid_match!(CTIMER1, CTIMER1_CH1, P2_5, 1);
impl_valid_match!(CTIMER1, CTIMER1_CH2, P2_6, 2);
impl_valid_match!(CTIMER1, CTIMER1_CH3, P2_7, 3);

impl_valid_match!(CTIMER1, CTIMER1_CH0, P3_10, 0);
impl_valid_match!(CTIMER1, CTIMER1_CH1, P3_11, 1);
impl_valid_match!(CTIMER1, CTIMER1_CH2, P3_12, 2);
impl_valid_match!(CTIMER1, CTIMER1_CH3, P3_13, 3);

// CTIMER2 match channels
impl_valid_match!(CTIMER2, CTIMER2_CH0, P1_10, 0);
impl_valid_match!(CTIMER2, CTIMER2_CH1, P1_11, 1);
impl_valid_match!(CTIMER2, CTIMER2_CH2, P1_12, 2);
impl_valid_match!(CTIMER2, CTIMER2_CH3, P1_13, 3);

impl_valid_match!(CTIMER2, CTIMER2_CH0, P2_0, 0);
impl_valid_match!(CTIMER2, CTIMER2_CH1, P2_1, 1);
impl_valid_match!(CTIMER2, CTIMER2_CH2, P2_2, 2);
impl_valid_match!(CTIMER2, CTIMER2_CH3, P2_3, 3);

impl_valid_match!(CTIMER2, CTIMER2_CH0, P2_20, 0);
impl_valid_match!(CTIMER2, CTIMER2_CH1, P2_21, 1);
impl_valid_match!(CTIMER2, CTIMER2_CH3, P2_23, 3);

impl_valid_match!(CTIMER2, CTIMER2_CH0, P3_18, 0);
impl_valid_match!(CTIMER2, CTIMER2_CH1, P3_19, 1);
impl_valid_match!(CTIMER2, CTIMER2_CH2, P3_20, 2);
impl_valid_match!(CTIMER2, CTIMER2_CH3, P3_21, 3);

// CTIMER3 match channels
impl_valid_match!(CTIMER3, CTIMER3_CH0, P1_14, 0);
impl_valid_match!(CTIMER3, CTIMER3_CH1, P1_15, 1);
impl_valid_match!(CTIMER3, CTIMER3_CH2, P2_10, 2);
impl_valid_match!(CTIMER3, CTIMER3_CH3, P2_11, 3);

impl_valid_match!(CTIMER3, CTIMER3_CH0, P2_16, 0);
impl_valid_match!(CTIMER3, CTIMER3_CH1, P2_17, 1);
impl_valid_match!(CTIMER3, CTIMER3_CH2, P2_19, 3);

impl_valid_match!(CTIMER3, CTIMER3_CH0, P3_27, 1);
impl_valid_match!(CTIMER3, CTIMER3_CH2, P3_28, 2);
#[cfg(feature = "dangerous-reset-as-gpio")]
impl_valid_match!(CTIMER3, CTIMER3_CH3, P3_29, 3);

// CTIMER4 match channels
impl_valid_match!(CTIMER4, CTIMER4_CH0, P1_6, 0);
impl_valid_match!(CTIMER4, CTIMER4_CH1, P1_7, 1);

impl_valid_match!(CTIMER4, CTIMER4_CH0, P2_12, 0);
impl_valid_match!(CTIMER4, CTIMER4_CH1, P2_13, 1);
impl_valid_match!(CTIMER4, CTIMER4_CH3, P2_15, 3);

impl_valid_match!(CTIMER4, CTIMER4_CH0, P3_2, 0);
impl_valid_match!(CTIMER4, CTIMER4_CH2, P3_6, 2);
impl_valid_match!(CTIMER4, CTIMER4_CH3, P3_7, 3);

impl_valid_match!(CTIMER4, CTIMER4_CH0, P4_2, 0);
impl_valid_match!(CTIMER4, CTIMER4_CH1, P4_3, 1);
impl_valid_match!(CTIMER4, CTIMER4_CH2, P4_4, 2);
impl_valid_match!(CTIMER4, CTIMER4_CH3, P4_5, 3);
