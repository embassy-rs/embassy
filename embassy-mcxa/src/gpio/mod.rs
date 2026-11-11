//! MCX GPIO support with shared value types, data operations and driver wrappers.
//! Family modules own pin control, type erasure and interrupt capabilities.

use crate::pac::gpio::{Ptco, Ptso};
use core::convert::Infallible;
use core::marker::PhantomData;
use embassy_hal_internal::Peri;

#[cfg(not(feature = "_mcxn"))]
mod mcxa;
#[cfg(not(feature = "_mcxn"))]
pub use mcxa::{AnyPin, Async, GpioPin, HasGpioInstance, Instance, InterruptHandler, PeriGpioExt};
#[cfg(not(feature = "_mcxn"))]
pub(crate) use mcxa::{Info, SealedInstance, SealedPin};
#[cfg(feature = "_mcxn")]
mod mcxn;
#[cfg(feature = "_mcxn")]
pub(crate) use mcxn::SealedPin;
#[cfg(feature = "_mcxn")]
pub use mcxn::{AnyPin, GpioPin};

/// Open-drain for GPIO pins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDrain {
    /// Output is push-pull (not open-drain)
    No,
    /// Output is open-drain
    Yes,
}

/// Logical level for GPIO pins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    Low,
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Pull {
    Disabled,
    Up,
    Down,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SlewRate {
    Fast,
    Slow,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DriveStrength {
    Normal,
    Double,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Inverter {
    Disabled,
    Enabled,
}

pub type Gpio = crate::peripherals::GPIO0;

mod sealed {
    pub trait Sealed {}
}
pub trait Mode: sealed::Sealed {}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NoIrqBound;

pub struct Blocking {}
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// A flexible pin that can be configured as input or output.
pub struct Flex<'d, M: Mode = Blocking> {
    pin: Peri<'d, AnyPin>,
    _phantom: PhantomData<&'d mut M>,
}

impl<'d, M: Mode> Flex<'d, M> {
    #[inline]
    fn gpio(&self) -> crate::pac::gpio::Gpio {
        self.pin.gpio()
    }

    /// Set output level to High.
    #[inline]
    pub fn set_high(&mut self) {
        self.gpio()
            .psor()
            .write(|w| w.set_ptso(self.pin.pin_index() as usize, Ptso::Ptso1));
    }

    /// Set output level to Low.
    #[inline]
    pub fn set_low(&mut self) {
        self.gpio()
            .pcor()
            .write(|w| w.set_ptco(self.pin.pin_index() as usize, Ptco::Ptco1));
    }

    /// Set output level to the given `Level`.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }
    }

    /// Toggle output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.gpio()
            .ptor()
            .write(|w| w.set_ptto(self.pin.pin_index() as usize, true));
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.gpio().pdir().read().pdi(self.pin.pin_index() as usize)
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Configure the pin pull up/down level.
    pub fn set_pull(&mut self, pull_select: Pull) {
        self.pin.set_pull(pull_select);
    }

    /// Configure the pin drive strength.
    pub fn set_drive_strength(&mut self, strength: DriveStrength) {
        self.pin.set_drive_strength(strength.into());
    }

    /// Configure the pin slew rate.
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate.into());
    }

    /// Enable input buffer for the pin.
    pub fn set_enable_input_buffer(&mut self, buffer_enabled: bool) {
        self.pin.set_enable_input_buffer(buffer_enabled);
    }

    /// Get pin level.
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }
}

impl<'d, M: Mode> Drop for Flex<'d, M> {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_as_disabled();
    }
}

/// GPIO output driver that owns a `Flex` pin.
pub struct Output<'d> {
    flex: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create a GPIO output driver for a [GpioPin] with the provided [Level].
    pub fn new(pin: Peri<'d, impl GpioPin>, initial: Level, strength: DriveStrength, slew_rate: SlewRate) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_level(initial);
        flex.set_as_output();
        flex.set_drive_strength(strength);
        flex.set_slew_rate(slew_rate);
        flex.set_open_drain(OpenDrain::No);
        Self { flex }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.flex.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.flex.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.flex.set_level(level);
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.flex.toggle();
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.flex.is_set_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.flex.is_set_low()
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d> {
        self.flex
    }

    /// Convert this output pin into an open-drain output pin.
    #[inline]
    pub fn into_open_drain(mut self) -> OutputOpenDrain<'d> {
        self.flex.set_open_drain(OpenDrain::Yes);
        OutputOpenDrain { flex: self.flex }
    }
}

/// GPIO output open-drain driver that owns a `Flex` pin.
pub struct OutputOpenDrain<'d> {
    flex: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a GPIO output open-drain driver for a [GpioPin] with the provided [Level].
    pub fn new(pin: Peri<'d, impl GpioPin>, initial: Level, strength: DriveStrength, slew_rate: SlewRate) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_level(initial);
        flex.set_as_output();
        flex.set_drive_strength(strength);
        flex.set_slew_rate(slew_rate);
        flex.set_enable_input_buffer(true);
        flex.set_open_drain(OpenDrain::Yes);
        Self { flex }
    }

    /// Get whether the pin level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.flex.is_high()
    }

    /// Get whether the pin level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.is_low()
    }

    /// Set the output as high (open-drain high is just letting go of the line).
    #[inline]
    pub fn set_high(&mut self) {
        self.flex.set_high();
    }

    /// Set the output as low (open-drain low is driving the line low).
    #[inline]
    pub fn set_low(&mut self) {
        self.flex.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.flex.set_level(level);
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.flex.get_level()
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        if self.flex.is_set_low() {
            self.set_high();
        } else {
            self.set_low();
        }
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: Inverter) {
        self.flex.set_input_inversion(invert)
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    #[inline]
    pub fn into_flex(self) -> Flex<'d> {
        self.flex
    }

    /// Convert this output pin into an push-pull output pin.
    #[inline]
    pub fn into_push_pull(mut self) -> Output<'d> {
        self.flex.set_open_drain(OpenDrain::No);
        Output { flex: self.flex }
    }
}

/// GPIO input driver that owns a `Flex` pin.
pub struct Input<'d, M: Mode = Blocking> {
    flex: Flex<'d, M>,
}

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a [GpioPin].
    ///
    pub fn new(pin: Peri<'d, impl GpioPin>, pull_select: Pull) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input();
        flex.set_pull(pull_select);
        Self { flex }
    }
}

impl<'d, M: Mode> Input<'d, M> {
    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.flex.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.is_low()
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    ///
    /// Since Drive Strength and Slew Rate are not set when creating the Input
    /// pin, they need to be set when converting
    #[inline]
    pub fn into_flex(mut self, strength: DriveStrength, slew_rate: SlewRate) -> Flex<'d, M> {
        self.flex.set_drive_strength(strength);
        self.flex.set_slew_rate(slew_rate);
        self.flex
    }

    /// Configure the input logic inversion of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: Inverter) {
        self.flex.set_input_inversion(invert)
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.flex.get_level()
    }
}

// Both embedded_hal 0.2 and 1.0 must be supported by embassy HALs.
impl<M: Mode> embedded_hal_02::digital::v2::InputPin for Flex<'_, M> {
    // GPIO operations on this block cannot fail, therefore we set the error type
    // to Infallible to guarantee that we can only produce Ok variants.
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::InputPin for Input<'_, M> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl embedded_hal_02::digital::v2::OutputPin for Output<'_> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::OutputPin for Flex<'_, M> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'_, M> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<M: Mode> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'_, M> {
    type Error = Infallible;

    #[inline]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<M: Mode> embedded_hal_1::digital::ErrorType for Flex<'_, M> {
    type Error = Infallible;
}

impl<M: Mode> embedded_hal_1::digital::ErrorType for Input<'_, M> {
    type Error = Infallible;
}

impl embedded_hal_1::digital::ErrorType for Output<'_> {
    type Error = Infallible;
}

impl<M: Mode> embedded_hal_1::digital::InputPin for Input<'_, M> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl embedded_hal_1::digital::OutputPin for Output<'_> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<M: Mode> embedded_hal_1::digital::OutputPin for Flex<'_, M> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<M: Mode> embedded_hal_1::digital::StatefulOutputPin for Flex<'_, M> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}
