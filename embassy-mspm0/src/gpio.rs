#![macro_use]

use core::convert::Infallible;
use core::future::Future;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::pac::gpio::vals::*;
use crate::pac::gpio::{self};
#[cfg(all(feature = "rt", feature = "mspm0c110x"))]
use crate::pac::interrupt;
use crate::pac::{self};

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logical low.
    Low,
    /// Logical high.
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

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// Represents a pull setting for an input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Port {
    /// Port A.
    PortA = 0,

    /// Port B.
    #[cfg(gpio_pb)]
    PortB = 1,

    /// Port C.
    #[cfg(gpio_pc)]
    PortC = 2,
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        // Pin will be in disconnected state.
        Self { pin: pin.into() }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pipd(matches!(pull, Pull::Down));
            w.set_pipu(matches!(pull, Pull::Up));
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(false);
            w.set_pc(true);
            w.set_inena(true);
        });

        self.pin.block().doeclr31_0().write(|w| {
            w.set_dio(self.pin.bit_index(), true);
        });
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(false);
            w.set_pc(true);
            w.set_inena(false);
        });

        self.pin.block().doeset31_0().write(|w| {
            w.set_dio(self.pin.bit_index(), true);
        });
    }

    /// Put the pin into input + open-drain output mode.
    ///
    /// The hardware will drive the line low if you set it to low, and will leave it floating if you set
    /// it to high, in which case you can read the input to figure out whether another device
    /// is driving the line low.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    ///
    /// The internal weak pull-up and pull-down resistors will be disabled.
    #[inline]
    pub fn set_as_input_output(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(GPIO_PF);
            w.set_hiz1(true);
            w.set_pc(true);
            w.set_inena(false);
        });

        self.set_pull(Pull::None);
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the internal weak pull-up and pull-down resistors.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(DISCONNECT_PF);
            w.set_hiz1(false);
            w.set_pc(false);
            w.set_inena(false);
        });

        self.set_pull(Pull::None);
        self.set_inversion(false);
    }

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to both the input and output path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_inv(invert);
        });
    }

    // TODO: drive strength, hysteresis, wakeup enable, wakeup compare

    /// Put the pin into the PF mode, unchecked.
    ///
    /// This puts the pin into the PF mode, with the request number. This is completely unchecked,
    /// it can attach the pin to literally any peripheral, so use with care. In addition the pin
    /// peripheral is connected in the iomux.
    ///
    /// The peripheral attached to the pin depends on the part in use. Consult the datasheet
    /// or technical reference manual for additional details.
    #[inline]
    pub fn set_pf_unchecked(&mut self, pf: u8) {
        // Per SLAU893 and SLAU846B, PF is only 6 bits
        assert_eq!(pf & 0xC0, 0, "PF is out of range");

        let pincm = pac::IOMUX.pincm(self.pin.pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(pf);
            // If the PF is manually set, connect the pin
            w.set_pc(true);
        });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.block().din31_0().read().dio(self.pin.bit_index())
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.block().doutset31_0().write(|w| {
            w.set_dio(self.pin.bit_index() as usize, true);
        });
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.block().doutclr31_0().write(|w| {
            w.set_dio(self.pin.bit_index(), true);
        });
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.block().douttgl31_0().write(|w| {
            w.set_dio(self.pin.bit_index(), true);
        })
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_high().into()
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        (self.pin.block().dout31_0().read().0 & self.pin.bit_index() as u32) == 0
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        if self.is_high() {
            return;
        }

        self.wait_for_rising_edge().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        if self.is_low() {
            return;
        }

        self.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), Polarity::RISE).await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), Polarity::FALL).await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), Polarity::RISE_FALL).await
    }
}

impl<'d> Drop for Flex<'d> {
    #[inline]
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

/// GPIO input driver.
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// GPIO output driver.
///
/// Note that pins will **return to their floating state** when `Output` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `Output`, or pass it to [`core::mem::forget`].
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_output();
        pin.set_level(initial_output);
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }

    /// Is the output pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }
}

/// GPIO output open-drain driver.
///
/// Note that pins will **return to their floating state** when `OutputOpenDrain` is dropped.
/// If pins should retain their state indefinitely, either keep ownership of the
/// `OutputOpenDrain`, or pass it to [`core::mem::forget`].
pub struct OutputOpenDrain<'d> {
    pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create a new GPIO open drain output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_level(initial_output);
        pin.set_as_input_output();
        Self { pin }
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        !self.pin.is_low()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level);
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Configure the logic inversion of this pin.
    ///
    /// Logic inversion applies to the input path of this pin.
    #[inline]
    pub fn set_inversion(&mut self, invert: bool) {
        self.pin.set_inversion(invert)
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pub(crate) pin_port: u8,
}

impl AnyPin {
    /// Create an [AnyPin] for a specific pin.
    ///
    /// # Safety
    /// - `pin_port` should not in use by another driver.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// The index of this pin in PINCM (pin control management) registers.
    #[inline]
    fn pin_cm(&self) -> u8 {
        self._pin_cm()
    }
}

impl<'d> embedded_hal::digital::ErrorType for Flex<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::InputPin for Flex<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal::digital::OutputPin for Flex<'d> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d> embedded_hal::digital::StatefulOutputPin for Flex<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Flex<'d> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

impl<'d> embedded_hal::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::InputPin for Input<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Input<'d> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal::digital::ErrorType for OutputOpenDrain<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal::digital::InputPin for OutputOpenDrain<'d> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal::digital::OutputPin for OutputOpenDrain<'d> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }
}

impl<'d> embedded_hal::digital::StatefulOutputPin for OutputOpenDrain<'d> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for OutputOpenDrain<'d> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct PfType {
    pull: Pull,
    input: bool,
    invert: bool,
}

impl PfType {
    pub const fn input(pull: Pull, invert: bool) -> Self {
        Self {
            pull,
            input: true,
            invert,
        }
    }

    pub const fn output(pull: Pull, invert: bool) -> Self {
        Self {
            pull,
            input: false,
            invert,
        }
    }
}

/// The pin function to disconnect peripherals from the pin.
///
/// This is also the pin function used to connect to analog peripherals, such as an ADC.
const DISCONNECT_PF: u8 = 0;

/// The pin function for the GPIO peripheral.
///
/// This is fixed to `1` for every part.
const GPIO_PF: u8 = 1;

macro_rules! impl_pin {
    ($name: ident, $port: expr, $pin_num: expr) => {
        impl crate::gpio::Pin for crate::peripherals::$name {}
        impl crate::gpio::SealedPin for crate::peripherals::$name {
            #[inline]
            fn pin_port(&self) -> u8 {
                ($port as u8) * 32 + $pin_num
            }
        }

        impl From<crate::peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: crate::peripherals::$name) -> Self {
                Self {
                    pin_port: crate::gpio::SealedPin::pin_port(&val),
                }
            }
        }
    };
}

// TODO: Possible micro-op for C110X, not every pin is instantiated even on the 20 pin parts.
//       This would mean cfg guarding to just cfg guarding every pin instance.
static PORTA_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio_pb)]
static PORTB_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];
#[cfg(gpio_pc)]
static PORTC_WAKERS: [AtomicWaker; 32] = [const { AtomicWaker::new() }; 32];

pub(crate) trait SealedPin {
    fn pin_port(&self) -> u8;

    fn port(&self) -> Port {
        match self.pin_port() / 32 {
            0 => Port::PortA,
            #[cfg(gpio_pb)]
            1 => Port::PortB,
            #[cfg(gpio_pc)]
            2 => Port::PortC,
            _ => unreachable!(),
        }
    }

    fn waker(&self) -> &AtomicWaker {
        match self.port() {
            Port::PortA => &PORTA_WAKERS[self.bit_index()],
            #[cfg(gpio_pb)]
            Port::PortB => &PORTB_WAKERS[self.bit_index()],
            #[cfg(gpio_pc)]
            Port::PortC => &PORTC_WAKERS[self.bit_index()],
        }
    }

    fn _pin_cm(&self) -> u8 {
        // Some parts like the MSPM0L222x have pincm mappings all over the place.
        crate::gpio_pincm(self.pin_port())
    }

    fn bit_index(&self) -> usize {
        (self.pin_port() % 32) as usize
    }

    #[inline]
    fn set_as_analog(&self) {
        let pincm = pac::IOMUX.pincm(self._pin_cm() as usize);

        pincm.modify(|w| {
            w.set_pf(DISCONNECT_PF);
            w.set_pipu(false);
            w.set_pipd(false);
        });
    }

    fn update_pf(&self, ty: PfType) {
        let pincm = pac::IOMUX.pincm(self._pin_cm() as usize);
        let pf = pincm.read().pf();

        set_pf(self._pin_cm() as usize, pf, ty);
    }

    fn set_as_pf(&self, pf: u8, ty: PfType) {
        set_pf(self._pin_cm() as usize, pf, ty)
    }

    /// Set the pin as "disconnected", ie doing nothing and consuming the lowest
    /// amount of power possible.
    ///
    /// This is currently the same as [`Self::set_as_analog()`] but is semantically different
    /// really. Drivers should `set_as_disconnected()` pins when dropped.
    ///
    /// Note that this also disables the internal weak pull-up and pull-down resistors.
    #[inline]
    fn set_as_disconnected(&self) {
        self.set_as_analog();
    }

    #[inline]
    fn block(&self) -> gpio::Gpio {
        match self.pin_port() / 32 {
            0 => pac::GPIOA,
            #[cfg(gpio_pb)]
            1 => pac::GPIOB,
            #[cfg(gpio_pc)]
            2 => pac::GPIOC,
            _ => unreachable!(),
        }
    }
}

#[inline(never)]
fn set_pf(pincm: usize, pf: u8, ty: PfType) {
    pac::IOMUX.pincm(pincm).modify(|w| {
        w.set_pf(pf);
        w.set_pc(true);
        w.set_pipu(ty.pull == Pull::Up);
        w.set_pipd(ty.pull == Pull::Down);
        w.set_inena(ty.input);
        w.set_inv(ty.invert);
    });
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> InputFuture<'d> {
    fn new(pin: Peri<'d, AnyPin>, polarity: Polarity) -> Self {
        let block = pin.block();

        // Before clearing any previous edge events, we must disable events.
        //
        // If we don't do this, it is possible that after we clear the interrupt, the current event
        // the hardware is listening for may not be the same event we will configure. This may result
        // in RIS being set. Then when interrupts are unmasked and RIS is set, we may get the wrong event
        // causing an interrupt.
        //
        // Selecting which polarity events happen is a RMW operation.
        critical_section::with(|_cs| {
            if pin.bit_index() >= 16 {
                block.polarity31_16().modify(|w| {
                    w.set_dio(pin.bit_index() - 16, Polarity::DISABLE);
                });
            } else {
                block.polarity15_0().modify(|w| {
                    w.set_dio(pin.bit_index(), Polarity::DISABLE);
                });
            };
        });

        // First clear the bit for this event. Otherwise previous edge events may be recorded.
        block.cpu_int().iclr().write(|w| {
            w.set_dio(pin.bit_index(), true);
        });

        // Selecting which polarity events happen is a RMW operation.
        critical_section::with(|_cs| {
            // Tell the hardware which pin event we want to receive.
            if pin.bit_index() >= 16 {
                block.polarity31_16().modify(|w| {
                    w.set_dio(pin.bit_index() - 16, polarity);
                });
            } else {
                block.polarity15_0().modify(|w| {
                    w.set_dio(pin.bit_index(), polarity);
                });
            };
        });

        Self { pin }
    }
}

impl<'d> Future for InputFuture<'d> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        let waker = self.pin.waker();
        waker.register(cx.waker());

        // The interrupt handler will mask the interrupt if the event has occurred.
        if self.pin.block().cpu_int().ris().read().dio(self.pin.bit_index()) {
            return Poll::Ready(());
        }

        // Unmasking the interrupt is a RMW operation.
        //
        // Guard with a critical section in case two different threads try to unmask at the same time.
        critical_section::with(|_cs| {
            self.pin.block().cpu_int().imask().modify(|w| {
                w.set_dio(self.pin.bit_index(), true);
            });
        });

        Poll::Pending
    }
}

pub(crate) fn init(gpio: gpio::Gpio) {
    gpio.gprcm().rstctl().write(|w| {
        w.set_resetstkyclr(true);
        w.set_resetassert(true);
        w.set_key(ResetKey::KEY);
    });

    gpio.gprcm().pwren().write(|w| {
        w.set_enable(true);
        w.set_key(PwrenKey::KEY);
    });

    gpio.evt_mode().modify(|w| {
        // The CPU will clear it's own interrupts
        w.set_cpu_cfg(EvtCfg::SOFTWARE);
    });
}

#[cfg(feature = "rt")]
fn irq_handler(gpio: gpio::Gpio, wakers: &[AtomicWaker; 32]) {
    // Only consider pins which have interrupts unmasked.
    let bits = gpio.cpu_int().mis().read().0;

    for i in BitIter(bits) {
        wakers[i as usize].wake();

        // Notify the future that an edge event has occurred by masking the interrupt for this pin.
        gpio.cpu_int().imask().modify(|w| {
            w.set_dio(i as usize, false);
        });
    }
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

// C110x has a dedicated interrupt just for GPIOA, as it does not have a GROUP1 interrupt.
#[cfg(all(feature = "rt", feature = "mspm0c110x"))]
#[interrupt]
fn GPIOA() {
    gpioa_interrupt();
}

#[cfg(feature = "rt")]
pub(crate) fn gpioa_interrupt() {
    irq_handler(pac::GPIOA, &PORTA_WAKERS);
}

#[cfg(all(feature = "rt", gpio_pb))]
pub(crate) fn gpiob_interrupt() {
    irq_handler(pac::GPIOB, &PORTB_WAKERS);
}

#[cfg(all(feature = "rt", gpio_pc))]
pub(crate) fn gpioc_interrupt() {
    irq_handler(pac::GPIOC, &PORTC_WAKERS);
}
