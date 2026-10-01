//! General-purpose Input/Output (GPIO).

#![macro_use]

use core::convert::Infallible;

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};

use crate::pac::GPIO;
// GPIO routes to a different register-block version per chip (gpio_s2v1 on
// MG22, gpio_s2v7 on MG26, ...). The port register newtypes (PortDout /
// PortModel / PortModeh) have identical shapes across them; the metapac's
// version-neutral `gpio` alias names the chip's version.
use crate::pac::gpio as gpio_mod;

/// Logic level.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low.
    Low,
    /// High.
    High,
}

impl From<bool> for Level {
    fn from(b: bool) -> Self {
        if b { Level::High } else { Level::Low }
    }
}
impl From<Level> for bool {
    fn from(l: Level) -> Self {
        matches!(l, Level::High)
    }
}

/// Pull setting for an input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull.
    None,
    /// Pull-up.
    Up,
    /// Pull-down.
    Down,
}

/// Output drive characteristics.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    /// Low.
    Low,
    /// Medium.
    Medium,
    /// High
    High,
}

use gpio_mod::vals::Mode;

/// Type-erased GPIO pin reference.
pub struct AnyPin {
    pin_port: u8,
}

impl_peripheral!(AnyPin);

impl AnyPin {
    /// Creates an `AnyPin` for `pin_port` without ownership.
    ///
    /// # Safety
    ///
    /// No other `AnyPin` for the same pin may be in scope at the same time.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }

    #[inline]
    pub(crate) fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

/// GPIO output driver.
pub struct Output<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] and [Speed] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level, _speed: Speed) -> Self {
        let pin = pin.into();
        match initial {
            Level::High => write_dout_bit(&pin, true),
            Level::Low => write_dout_bit(&pin, false),
        }
        write_mode(&pin, Mode::Pushpull);
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        write_dout_bit(&self.pin, true);
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        write_dout_bit(&self.pin, false);
    }

    /// Sets the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        write_dout_bit(&self.pin, level.into());
    }

    /// Toggles the output.
    #[inline]
    pub fn toggle(&mut self) {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = self.pin.pin_port() & 0xF;
        GPIO.p_dout_tgl(port).write_value(gpio_mod::regs::PortDout(1 << n));
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = (self.pin.pin_port() & 0xF) as u32;
        if GPIO.p_dout(port).read().0 & (1 << n) != 0 {
            Level::High
        } else {
            Level::Low
        }
    }
}

impl<'d> Drop for Output<'d> {
    fn drop(&mut self) {
        write_mode(&self.pin, Mode::Disabled);
    }
}

/// GPIO input.
pub struct Input<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let pin = pin.into();
        // In `INPUTPULL` mode, `DOUT` 1 selects pull-up and 0 selects pull-down. In `INPUT` mode,
        // `DOUT` 1 turns on the glitch filter.
        match pull {
            Pull::None => write_mode(&pin, Mode::Input),
            Pull::Up => {
                write_dout_bit(&pin, true);
                write_mode(&pin, Mode::Inputpull);
            }
            Pull::Down => {
                write_dout_bit(&pin, false);
                write_mode(&pin, Mode::Inputpull);
            }
        }
        Self { pin }
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        if self.is_high() { Level::High } else { Level::Low }
    }

    /// Is the input pin high?
    #[inline]
    pub fn is_high(&self) -> bool {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = (self.pin.pin_port() & 0xF) as u32;
        GPIO.p_din(port).read().0 & (1 << n) != 0
    }

    /// Is the input pin low?
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }
}

impl<'d> Drop for Input<'d> {
    fn drop(&mut self) {
        write_mode(&self.pin, Mode::Disabled);
    }
}

mod sealed {
    pub trait SealedPin {
        fn pin_port(&self) -> u8;
    }
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + sealed::SealedPin {}

impl sealed::SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}
impl Pin for AnyPin {}

foreach_pin!(
    ($name:ident, $pin_port:expr) => {
        impl sealed::SealedPin for crate::peripherals::$name {
            #[inline]
            fn pin_port(&self) -> u8 {
                $pin_port
            }
        }
        impl Pin for crate::peripherals::$name {}
        impl From<crate::peripherals::$name> for AnyPin {
            #[inline]
            fn from(_: crate::peripherals::$name) -> Self {
                Self { pin_port: $pin_port }
            }
        }
    };
);

#[inline]
fn write_mode(pin: &AnyPin, mode: Mode) {
    let port = (pin.pin_port() >> 4) as usize;
    let n = (pin.pin_port() & 0xF) as usize;

    // A 4-bit mode field has no SET/CLR form, so read-modify-write it.
    critical_section::with(|_| {
        if n < 8 {
            GPIO.p_model(port).modify(|w| w.set_mode(n, mode));
        } else {
            GPIO.p_modeh(port).modify(|w| w.set_mode(n - 8, mode));
        }
    });
}

#[inline]
fn write_dout_bit(pin: &AnyPin, high: bool) {
    let port = (pin.pin_port() >> 4) as usize;
    let bit = gpio_mod::regs::PortDout(1 << (pin.pin_port() & 0xF));
    // `P_DOUT_SET` and `P_DOUT_CLR` change only the written bits, so no critical section.
    if high {
        GPIO.p_dout_set(port).write_value(bit);
    } else {
        GPIO.p_dout_clr(port).write_value(bit);
    }
}

/// Sets `pin` as a push-pull output for a peripheral, such as EUSART TX.
/// Sets the line idle-high before switching to push-pull so a UART idle
/// level is presented immediately.
#[cfg(eusart)]
#[inline]
pub(crate) fn set_as_alternate_output(pin: &AnyPin) {
    write_dout_bit(pin, true);
    write_mode(pin, Mode::Pushpull);
}

/// Sets `pin` as a floating input for a peripheral, such as EUSART RX.
#[cfg(eusart)]
#[inline]
pub(crate) fn set_as_alternate_input(pin: &AnyPin) {
    write_mode(pin, Mode::Input);
}

/// Puts `pin` back in its reset state (`DISABLED`).
///
/// `DOUT` goes low first, because `DOUT` high in `DISABLED` mode turns on the pull-up.
#[cfg(eusart)]
#[inline]
pub(crate) fn set_as_disabled(pin: &AnyPin) {
    write_dout_bit(pin, false);
    write_mode(pin, Mode::Disabled);
}

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}
impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Output::set_low(self);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Output::set_high(self);
        Ok(())
    }
}
impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(matches!(Output::get_output_level(self), Level::High))
    }
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(matches!(Output::get_output_level(self), Level::Low))
    }
}
impl<'d> embedded_hal::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}
impl<'d> embedded_hal::digital::InputPin for Input<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_high(self))
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_low(self))
    }
}
