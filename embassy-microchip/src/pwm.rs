//! Pulse Width Modulation (PWM)

use embassy_hal_internal::{Peri, PeripheralType};
pub use embedded_hal_1::pwm::SetDutyCycle;
use embedded_hal_1::pwm::{Error, ErrorKind, ErrorType};

use crate::gpio::{AnyPin, Pin, SealedPin};
use crate::pac::pwm0::Pwm0;
use crate::{pac, peripherals};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
/// Clock selection
pub enum Clock {
    _48MHz,
    _100kHz,
}

impl From<Clock> for bool {
    fn from(clk: Clock) -> Self {
        match clk {
            Clock::_48MHz => true,
            Clock::_100kHz => false,
        }
    }
}

/// The configuration of a PWM slice.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Inverts the PWM output signal.
    pub invert: bool,

    /// Enables the PWM slice, allowing it to generate an output.
    pub enable: bool,

    /// PWM target frequency
    pub frequency: u32,

    /// PWM target duty cycle
    pub duty_cycle: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            invert: false,
            enable: true, // differs from reset
            frequency: 100_000,
            duty_cycle: 32767,
        }
    }
}

/// PWM error.
#[derive(Debug)]
pub enum PwmError {
    /// Invalid Duty Cycle.
    InvalidDutyCycle,
}

impl Error for PwmError {
    fn kind(&self) -> ErrorKind {
        match self {
            PwmError::InvalidDutyCycle => ErrorKind::Other,
        }
    }
}

/// PWM driver.
pub struct Pwm<'d, T: Instance> {
    _peri: Peri<'d, T>,
    _pin: Peri<'d, AnyPin>,
}

impl<'d, T: Instance> ErrorType for Pwm<'d, T> {
    type Error = PwmError;
}

impl<'d, T: Instance> Pwm<'d, T> {
    const HIGHEST_DIVIDER: u32 = 15;
    const HIGHEST_ON_OFF: u32 = 65_535;
    const HIGH_CLK_FREQ: u32 = 48_000_000;
    const LOW_CLK_FREQ: u32 = 100_000;
    const MIN_HIGH_CLK_FREQ: u32 = Self::HIGH_CLK_FREQ / (2 * (Self::HIGHEST_ON_OFF + 1) * (Self::HIGHEST_DIVIDER + 1));

    pub fn new(peri: Peri<'d, T>, pin: Peri<'d, impl PwmPin<T>>, config: Config) -> Self {
        Self::new_inner(peri, pin, config)
    }

    fn new_inner(_peri: Peri<'d, T>, _pin: Peri<'d, impl PwmPin<T>>, config: Config) -> Self {
        _pin.setup();
        Self::configure(config);
        Self {
            _peri,
            _pin: _pin.into(),
        }
    }

    fn configure(config: Config) {
        let (div, on, off, clk) = Self::compute_parameters(config.frequency, config.duty_cycle);

        T::regs().cfg().write(|w| {
            w.set_inv(config.invert);
            w.set_clk_sel(clk.into());
            w.set_clk_pre_div(div);
            w.set_pwm_en(config.enable);
        });

        T::regs().cnt_on().write_value(on);
        T::regs().cnt_off().write_value(off);
    }

    fn compute_parameters(target_freq: u32, target_duty_cycle: u16) -> (u8, u32, u32, Clock) {
        let high = target_freq > Self::MIN_HIGH_CLK_FREQ;

        // We assume that if target_freq is not greater than high
        // clock minimum frequency, then it must fit within the low
        // clock, even if with high error.

        let (clk, freq) = if high {
            (Clock::_48MHz, Self::HIGH_CLK_FREQ)
        } else {
            (Clock::_100kHz, Self::LOW_CLK_FREQ)
        };

        let (div, _) = (1..=16).fold((0, u32::MAX), |(best_div, best_error), d| {
            let candidate = freq / d;
            let error = target_freq.abs_diff(candidate);

            if error < best_error {
                (candidate, error)
            } else {
                (best_div, best_error)
            }
        });

        let (on, off) = Self::compute_on_off(freq, target_freq, target_duty_cycle);
        (div as u8, on, off, clk)
    }

    fn compute_on_off(freq: u32, target_freq: u32, target_duty_cycle: u16) -> (u32, u32) {
        let total = freq * 10 / target_freq;
        let on = ((total * u32::from(target_duty_cycle)) / 100_000) - 1;
        let off = total - on - 2;

        (on, off)
    }
}

impl<'d, T: Instance> SetDutyCycle for Pwm<'d, T> {
    fn max_duty_cycle(&self) -> u16 {
        u16::MAX
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let max_duty = self.max_duty_cycle();

        if duty > max_duty {
            return Err(PwmError::InvalidDutyCycle);
        }

        let off = u32::from(duty) * u32::from(u16::MAX) / u32::from(max_duty);
        let on = u32::from(u16::MAX) - off;

        T::regs().cnt_on().write_value(on.into());
        T::regs().cnt_off().write_value(off.into());

        Ok(())
    }
}

trait SealedInstance {
    fn regs() -> Pwm0;
}

/// PWM Instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {}

macro_rules! impl_instance {
    ($($peri:ident),*) => {
	$(
	    impl SealedInstance for peripherals::$peri {
		#[inline(always)]
		fn regs() -> Pwm0 {
		    pac::$peri
		}
	    }

	    impl Instance for peripherals::$peri {}
	)*
    }
}

impl_instance!(PWM0, PWM1, PWM2, PWM3, PWM4, PWM5, PWM6, PWM7, PWM8, PWM9, PWM10, PWM11);

pub trait PwmPin<T: Instance>: Pin + PeripheralType {
    fn setup(&self);
}

macro_rules! impl_pin {
    ($peri:ident, $($pin:ident),*) => {
	$(
	    impl PwmPin<peripherals::$peri> for peripherals::$pin {
                #[inline(always)]
                fn setup(&self) {
                    critical_section::with(|_| {
                        self.regs().ctrl1.modify(|w| {
                            w.set_mux_ctrl(pac::Function::F1);
                        })
                    });
                }
            }
	)*
    }
}

impl_pin!(PWM0, GPIO53, GPIO241);
impl_pin!(PWM1, GPIO54);
impl_pin!(PWM2, GPIO55, GPIO45);
impl_pin!(PWM3, GPIO56);
impl_pin!(PWM4, GPIO11, GPIO1);
impl_pin!(PWM5, GPIO2);
impl_pin!(PWM6, GPIO14, GPIO63);
impl_pin!(PWM7, GPIO15);
impl_pin!(PWM8, GPIO35, GPIO175);
impl_pin!(PWM9, GPIO133);
impl_pin!(PWM10, GPIO134);
impl_pin!(PWM11, GPIO160, GPIO222);
