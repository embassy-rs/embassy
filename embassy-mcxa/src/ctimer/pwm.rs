//! CTimer-based PWM driver

use embassy_hal_internal::Peri;
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};

use super::{AnyChannel, CTimer, CTimerChannel, Channel, Info, Instance, OutputPin};
use crate::gpio::{AnyPin, SealedPin};
use crate::pac::ctimer::vals::{Mri, Mrr, Mrrl, Mrs, Pwmen};

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

/// Representation of a single PWM channel.
///
/// This PWM representation can change duty cycle, but not frequency
/// of the PWM.
pub struct Pwm<'d> {
    info: &'static Info,
    duty_ch: Peri<'d, AnyChannel>,
    pin: Peri<'d, AnyPin>,
    source_freq: u32,
    pwm_freq: u16,
    max_period: u16,
}

impl<'d> Pwm<'d> {
    fn enable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(true));
    }

    fn disable(&mut self) {
        self.info.regs().tcr().modify(|w| w.set_cen(false));
    }

    fn set_pwm_mode(&self) {
        self.info.regs().pwmc().modify(|w| match self.duty_ch.number() {
            Channel::Zero => {
                w.set_pwmen0(Pwmen::PWM);
            }
            Channel::One => {
                w.set_pwmen1(Pwmen::PWM);
            }
            Channel::Two => {
                w.set_pwmen2(Pwmen::PWM);
            }
            Channel::Three => {
                w.set_pwmen3(Pwmen::PWM);
            }
        });
    }

    fn clear_status(&self) {
        self.info.regs().mcr().modify(|w| {
            // Clear stop, reset, and interrupt bits for the PWM channel
            match self.duty_ch.number() {
                Channel::Zero => {
                    w.set_mr0i(Mri::MRI0);
                    w.set_mr0r(Mrr::MRR0);
                    w.set_mr0s(Mrs::MRS0);
                }
                Channel::One => {
                    w.set_mr1i(Mri::MRI0);
                    w.set_mr1r(Mrr::MRR0);
                    w.set_mr1s(Mrs::MRS0);
                }
                Channel::Two => {
                    w.set_mr2i(Mri::MRI0);
                    w.set_mr2r(Mrr::MRR0);
                    w.set_mr2s(Mrs::MRS0);
                }
                Channel::Three => {
                    w.set_mr3i(Mri::MRI0);
                    w.set_mr3r(Mrr::MRR0);
                    w.set_mr3s(Mrs::MRS0);
                }
            }

            match self.duty_ch.number() {
                Channel::Zero => {
                    w.set_mr0rl(Mrrl::MRRL1);
                }
                Channel::One => {
                    w.set_mr1rl(Mrrl::MRRL1);
                }
                Channel::Two => {
                    w.set_mr2rl(Mrrl::MRRL1);
                }
                Channel::Three => {
                    w.set_mr3rl(Mrrl::MRRL1);
                }
            }
        });
    }

    fn configure_duty_cycle(&self, duty_cycle: u32) {
        self.info
            .regs()
            .mr(self.duty_ch.number().into())
            .write(|w| w.set_match_(duty_cycle));
        self.info
            .regs()
            .msr(self.duty_ch.number().into())
            .write(|w| w.set_match_shadow(duty_cycle));
    }
}

/// Single channel PWM driver
///
/// A single channel is used for Duty Cycle and a single channel is
/// used for PWM period match.
pub struct SinglePwm<'d> {
    pwm: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> SinglePwm<'d> {
    /// Create Pwm driver with a single pin as output.
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new<T: Instance, DUTY: CTimerChannel<T>, PIN: OutputPin<T>>(
        ctimer: CTimer<'d>,
        duty_ch: Peri<'d, DUTY>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin: Peri<'d, PIN>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY, PIN): ValidMatchConfig,
    {
        pin.mux();

        let mut inst = Self {
            pwm: Pwm {
                info: T::info(),
                duty_ch: duty_ch.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin.into(),
                max_period: 0,
            },
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Degrade `self` into the underlying PWM representation.
    ///
    /// Upon calling this method, changing frequency will be disallowed.
    pub fn degrade(self) -> Pwm<'d> {
        self.pwm
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm.disable();
        self.pwm.set_pwm_mode();
        self.pwm.clear_status();

        self.pwm.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(Mrr::MRR1);
            }
            Channel::One => {
                w.set_mr1r(Mrr::MRR1);
            }
            Channel::Two => {
                w.set_mr2r(Mrr::MRR1);
            }
            Channel::Three => {
                w.set_mr3r(Mrr::MRR1);
            }
        });

        // Configure PWM period
        let period = self.pwm.source_freq / u32::from(self.pwm.pwm_freq) - 1;
        self.pwm.max_period = period as u16;
        self.pwm
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm.enable();

        Ok(())
    }
}

/// Dual channel PWM driver.
///
/// A single period match channel is shared for two independent PWM
/// outputs. That is, both PWM output channels run on the same
/// frequency, with optionally different duty cycles.
pub struct DualPwm<'d> {
    pub pwm0: Pwm<'d>,
    pub pwm1: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> DualPwm<'d> {
    /// Create Pwm driver with a two pins for two PWM outputs.
    ///
    /// Upon `Drop`, all external pins will be placed into `Disabled`
    /// state.
    pub fn new<T: Instance, DUTY0: CTimerChannel<T>, DUTY1: CTimerChannel<T>, PIN0: OutputPin<T>, PIN1: OutputPin<T>>(
        ctimer: CTimer<'d>,
        duty_ch0: Peri<'d, DUTY0>,
        duty_ch1: Peri<'d, DUTY1>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin0: Peri<'d, PIN0>,
        pin1: Peri<'d, PIN1>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY0, PIN0): ValidMatchConfig,
        (T, DUTY1, PIN1): ValidMatchConfig,
    {
        pin0.mux();
        pin1.mux();

        let mut inst = Self {
            pwm0: Pwm {
                info: T::info(),
                duty_ch: duty_ch0.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin0.into(),
                max_period: 0,
            },
            pwm1: Pwm {
                info: T::info(),
                duty_ch: duty_ch1.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin1.into(),
                max_period: 0,
            },
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Split `self` into its underlying channels.
    ///
    /// Upon calling this method, changing PWM frequency will be
    /// disallowed. Only duty cycles can be changed.
    pub fn split(self) -> (Pwm<'d>, Pwm<'d>) {
        (self.pwm0, self.pwm1)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm0.disable();

        self.pwm0.set_pwm_mode();
        self.pwm1.set_pwm_mode();

        self.pwm0.clear_status();
        self.pwm1.clear_status();

        self.pwm0.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(Mrr::MRR1);
            }
            Channel::One => {
                w.set_mr1r(Mrr::MRR1);
            }
            Channel::Two => {
                w.set_mr2r(Mrr::MRR1);
            }
            Channel::Three => {
                w.set_mr3r(Mrr::MRR1);
            }
        });

        // Configure PWM period
        let period = self.pwm0.source_freq / u32::from(self.pwm0.pwm_freq) - 1;

        self.pwm0.max_period = period as u16;
        self.pwm1.max_period = period as u16;

        self.pwm0
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm0.configure_duty_cycle(duty_cycle);
        self.pwm1.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm0.enable();

        Ok(())
    }
}

/// Triple channel PWM driver.
///
/// A single period match channel is shared for three independent PWM
/// outputs. That is, all three PWM output channels run on the same
/// frequency, with optionally different duty cycles.
pub struct TriplePwm<'d> {
    pub pwm0: Pwm<'d>,
    pub pwm1: Pwm<'d>,
    pub pwm2: Pwm<'d>,
    period_ch: Peri<'d, AnyChannel>,
}

impl<'d> TriplePwm<'d> {
    /// Create Pwm driver using three pins for three PWM outputs.
    ///
    /// Upon `Drop`, all external pins will be placed into `Disabled`
    /// state.
    pub fn new<
        T: Instance,
        DUTY0: CTimerChannel<T>,
        DUTY1: CTimerChannel<T>,
        DUTY2: CTimerChannel<T>,
        PIN0: OutputPin<T>,
        PIN1: OutputPin<T>,
        PIN2: OutputPin<T>,
    >(
        ctimer: CTimer<'d>,
        duty_ch0: Peri<'d, DUTY0>,
        duty_ch1: Peri<'d, DUTY1>,
        duty_ch2: Peri<'d, DUTY2>,
        period_ch: Peri<'d, impl CTimerChannel<T>>,
        pin0: Peri<'d, PIN0>,
        pin1: Peri<'d, PIN1>,
        pin2: Peri<'d, PIN2>,
        config: Config,
    ) -> Result<Self, PwmError>
    where
        (T, DUTY0, PIN0): ValidMatchConfig,
        (T, DUTY1, PIN1): ValidMatchConfig,
        (T, DUTY2, PIN2): ValidMatchConfig,
    {
        pin0.mux();
        pin1.mux();
        pin2.mux();

        let mut inst = Self {
            pwm0: Pwm {
                info: T::info(),
                duty_ch: duty_ch0.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin0.into(),
                max_period: 0,
            },
            pwm1: Pwm {
                info: T::info(),
                duty_ch: duty_ch1.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin1.into(),
                max_period: 0,
            },
            pwm2: Pwm {
                info: T::info(),
                duty_ch: duty_ch2.into(),
                source_freq: ctimer._freq,
                pwm_freq: config.freq,
                pin: pin2.into(),
                max_period: 0,
            },
            period_ch: period_ch.into(),
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    /// Split `self` into its underlying channels.
    ///
    /// Upon calling this method, changing PWM frequency will be
    /// disallowed. Only duty cycles can be changed.
    pub fn split(self) -> (Pwm<'d>, Pwm<'d>, Pwm<'d>) {
        (self.pwm0, self.pwm1, self.pwm2)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        self.pwm0.disable();

        self.pwm0.set_pwm_mode();
        self.pwm1.set_pwm_mode();
        self.pwm2.set_pwm_mode();

        self.pwm0.clear_status();
        self.pwm1.clear_status();
        self.pwm2.clear_status();

        self.pwm0.info.regs().mcr().modify(|w| match self.period_ch.number() {
            Channel::Zero => {
                w.set_mr0r(Mrr::MRR1);
            }
            Channel::One => {
                w.set_mr1r(Mrr::MRR1);
            }
            Channel::Two => {
                w.set_mr2r(Mrr::MRR1);
            }
            Channel::Three => {
                w.set_mr3r(Mrr::MRR1);
            }
        });

        // Configure PWM period
        let period = self.pwm0.source_freq / u32::from(self.pwm0.pwm_freq) - 1;

        self.pwm0.max_period = period as u16;
        self.pwm1.max_period = period as u16;
        self.pwm2.max_period = period as u16;

        self.pwm0
            .info
            .regs()
            .mr(self.period_ch.number().into())
            .write(|w| w.set_match_(period));

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;
        self.pwm0.configure_duty_cycle(duty_cycle);
        self.pwm1.configure_duty_cycle(duty_cycle);
        self.pwm2.configure_duty_cycle(duty_cycle);

        // Start CTimer
        self.pwm0.enable();

        Ok(())
    }
}

impl<'d> Drop for Pwm<'d> {
    fn drop(&mut self) {
        self.disable();
        self.pin.set_as_disabled();
    }
}

impl<'d> ErrorType for SinglePwm<'d> {
    type Error = PwmError;
}

impl<'d> SetDutyCycle for SinglePwm<'d> {
    fn max_duty_cycle(&self) -> u16 {
        self.pwm.max_period
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.pwm.set_duty_cycle(duty)
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
            .msr(self.duty_ch.number().into())
            .write(|w| w.set_match_shadow(u32::from(duty)));

        Ok(())
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
