//! CTimer-based PWM driver

use embassy_hal_internal::Peri;
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};

use super::{Channel, Info, Instance, OutputPin};
use crate::gpio::{AnyPin, SealedPin};

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
    period_ch: Channel<'d>,
    match_ch: Channel<'d>,
    pin: Peri<'d, AnyPin>,
    freq: u16,
    max_period: u16,
}

impl<'d> Pwm<'d> {
    /// Create Pwm driver with a single pin as output.
    ///
    /// Upon `Drop`, the external `pin` will be placed into `Disabled`
    /// state.
    pub fn new_single_output<T: Instance>(
        period_ch: Channel<'d>,
        match_ch: Channel<'d>,
        pin: Peri<'d, impl OutputPin<T>>,
        config: Config,
    ) -> Result<Self, PwmError> {
        if period_ch.number > 3 || match_ch.number > 3 {
            return Err(PwmError::InvalidChannel);
        }

        if pin.number() != match_ch.number {
            return Err(PwmError::ChannelMismatch);
        }

        pin.mux();

        let mut inst = Self {
            info: period_ch.info,
            period_ch,
            match_ch,
            freq: config.freq,
            pin: pin.into(),
            max_period: 0,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn enable(&mut self) {
        self.info.regs().tcr().modify(|_, w| w.cen().enabled());
    }

    fn disable(&mut self) {
        self.info.regs().tcr().modify(|_, w| w.cen().disabled());
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), PwmError> {
        // Enable PWM mode on the match channel
        self.info.regs().pwmc().modify(|_, w| {
            match self.match_ch.number {
                0 => {
                    w.pwmen0().pwm();
                }
                1 => {
                    w.pwmen1().pwm();
                }
                2 => {
                    w.pwmen2().pwm();
                }
                3 => {
                    w.pwmen3().pwm();
                }
                _ => unreachable!(),
            }
            w
        });

        self.info.regs().mcr().modify(|_, w| {
            // Clear stop, reset, and interrupt bits for the PWM channel
            match self.match_ch.number {
                0 => {
                    w.mr0i().clear_bit().mr0r().clear_bit().mr0s().clear_bit();
                }
                1 => {
                    w.mr1i().clear_bit().mr1r().clear_bit().mr1s().clear_bit();
                }
                2 => {
                    w.mr2i().clear_bit().mr2r().clear_bit().mr2s().clear_bit();
                }
                3 => {
                    w.mr3i().clear_bit().mr3r().clear_bit().mr3s().clear_bit();
                }
                _ => unreachable!(),
            }

            match self.period_ch.number {
                0 => {
                    w.mr0r().set_bit();
                }
                1 => {
                    w.mr1r().set_bit();
                }
                2 => {
                    w.mr2r().set_bit();
                }
                3 => {
                    w.mr3r().set_bit();
                }
                _ => unreachable!(),
            }

            w
        });

        // Configure PWM period
        let period = self.period_ch.freq / u32::from(self.freq) - 1;
        self.max_period = period as u16;
        self.info
            .regs()
            .mr(self.period_ch.number)
            .write(|w| unsafe { w.match_().bits(period) });

        // Configure PWM duty cycle
        let duty_cycle = ((period + 1) * (100 - u32::from(config.duty_cycle))) / 100;

        self.info
            .regs()
            .mr(self.match_ch.number)
            .write(|w| unsafe { w.match_().bits(u32::from(duty_cycle)) });

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
            .mr(usize::from(self.match_ch.number))
            .write(|w| unsafe { w.match_().bits(u32::from(duty)) });

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
