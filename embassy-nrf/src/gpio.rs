#![macro_use]

use core::convert::Infallible;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::{impl_unborrow, unborrow};
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use gpio::pin_cnf::DRIVE_A;

use crate::pac;
use crate::pac::p0 as gpio;
use crate::peripherals;

/// A GPIO port with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0, available on all nRF52 and nRF51 MCUs.
    Port0,

    /// Port 1, only available on some nRF52 MCUs.
    #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
    Port1,
}

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    None,
    Up,
    Down,
}

/// GPIO input driver.
pub struct Input<'d, T: Pin> {
    pub(crate) pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Input<'d, T> {
    pub fn new(pin: impl Unborrow<Target = T> + 'd, pull: Pull) -> Self {
        unborrow!(pin);

        init_input(&pin, pull);

        Self {
            pin,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Pin> Drop for Input<'d, T> {
    fn drop(&mut self) {
        self.pin.conf().reset();
    }
}

impl<'d, T: Pin> InputPin for Input<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.block().in_.read().bits() & (1 << self.pin.pin()) == 0)
    }
}

/// Digital input or output level.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    Low,
    High,
}

// These numbers match DRIVE_A exactly so hopefully the compiler will unify them.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OutputDrive {
    /// Standard '0', standard '1'
    Standard = 0,
    /// High drive '0', standard '1'
    HighDrive0Standard1 = 1,
    /// Standard '0', high drive '1'
    Standard0HighDrive1 = 2,
    /// High drive '0', high 'drive '1'
    HighDrive = 3,
    /// Disconnect '0' standard '1' (normally used for wired-or connections)
    Disconnect0Standard1 = 4,
    /// Disconnect '0', high drive '1' (normally used for wired-or connections)
    Disconnect0HighDrive1 = 5,
    /// Standard '0'. disconnect '1' (also known as "open drain", normally used for wired-and connections)
    Standard0Disconnect1 = 6,
    /// High drive '0', disconnect '1' (also known as "open drain", normally used for wired-and connections)
    HighDrive0Disconnect1 = 7,
}

/// GPIO output driver.
pub struct Output<'d, T: Pin> {
    pub(crate) pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Output<'d, T> {
    pub fn new(
        pin: impl Unborrow<Target = T> + 'd,
        initial_output: Level,
        drive: OutputDrive,
    ) -> Self {
        unborrow!(pin);

        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        init_output(&pin, drive);

        Self {
            pin,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Pin> Drop for Output<'d, T> {
    fn drop(&mut self) {
        self.pin.conf().reset();
    }
}

impl<'d, T: Pin> OutputPin for Output<'d, T> {
    type Error = Infallible;

    /// Set the output as high.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.pin
                .block()
                .outset
                .write(|w| w.bits(1u32 << self.pin.pin()));
        }
        Ok(())
    }

    /// Set the output as low.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.pin
                .block()
                .outclr
                .write(|w| w.bits(1u32 << self.pin.pin()));
        }
        Ok(())
    }
}

impl<'d, T: Pin> StatefulOutputPin for Output<'d, T> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.block().out.read().bits() & (1 << self.pin.pin()) == 0)
    }
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct FlexPin<'d, T: Pin> {
    pub(crate) pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> FlexPin<'d, T> {
    /// Wrap the pin in a `FlexPin`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified, but can be changed
    /// before the pin is put into output mode.
    pub fn new(pin: impl Unborrow<Target = T> + 'd) -> Self {
        unborrow!(pin);
        // Pin will be in disconnected state.
        Self {
            pin,
            phantom: PhantomData,
        }
    }

    /// Put the pin into input mode.
    pub fn set_as_input(&mut self, pull: Pull) {
        init_input(&self.pin, pull);
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    pub fn set_as_output(&mut self, drive: OutputDrive) {
        init_output(&self.pin, drive);
    }

    /// Put the pin into disconnected mode.
    pub fn set_as_disconnected(&mut self) {
        self.pin.conf().reset();
    }
}

impl<'d, T: Pin> Drop for FlexPin<'d, T> {
    fn drop(&mut self) {
        self.pin.conf().reset();
    }
}

/// Implement [`InputPin`] for [`FlexPin`];
///
/// If the pin is not in input mode the result is unspecified.
impl<'d, T: Pin> InputPin for FlexPin<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.block().in_.read().bits() & (1 << self.pin.pin()) == 0)
    }
}

impl<'d, T: Pin> OutputPin for FlexPin<'d, T> {
    type Error = Infallible;

    /// Set the output as high.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.pin
                .block()
                .outset
                .write(|w| w.bits(1u32 << self.pin.pin()));
        }
        Ok(())
    }

    /// Set the output as low.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.pin
                .block()
                .outclr
                .write(|w| w.bits(1u32 << self.pin.pin()));
        }
        Ok(())
    }
}

impl<'d, T: Pin> StatefulOutputPin for FlexPin<'d, T> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.block().out.read().bits() & (1 << self.pin.pin()) == 0)
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin {
        fn pin_port(&self) -> u8;

        #[inline]
        fn _pin(&self) -> u8 {
            #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
            {
                self.pin_port() % 32
            }

            #[cfg(not(any(feature = "nrf52833", feature = "nrf52840")))]
            {
                self.pin_port()
            }
        }

        #[inline]
        fn block(&self) -> &gpio::RegisterBlock {
            unsafe {
                match self.pin_port() / 32 {
                    0 => &*pac::P0::ptr(),
                    #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
                    1 => &*pac::P1::ptr(),
                    _ => unreachable_unchecked(),
                }
            }
        }

        #[inline]
        fn conf(&self) -> &gpio::PIN_CNF {
            &self.block().pin_cnf[self._pin() as usize]
        }

        /// Set the output as high.
        #[inline]
        fn set_high(&self) {
            unsafe {
                self.block().outset.write(|w| w.bits(1u32 << self._pin()));
            }
        }

        /// Set the output as low.
        #[inline]
        fn set_low(&self) {
            unsafe {
                self.block().outclr.write(|w| w.bits(1u32 << self._pin()));
            }
        }
    }

    pub trait OptionalPin {}
}

pub trait Pin: sealed::Pin + Sized {
    /// Number of the pin within the port (0..31)
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Port of the pin
    #[inline]
    fn port(&self) -> Port {
        match self.pin_port() / 32 {
            0 => Port::Port0,
            #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
            1 => Port::Port1,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    fn psel_bits(&self) -> u32 {
        self.pin_port() as u32
    }

    /// Convert from concrete pin type PX_XX to type erased `AnyPin`.
    #[inline]
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_port: self.pin_port(),
        }
    }
}

// Type-erased GPIO pin
pub struct AnyPin {
    pin_port: u8,
}

impl AnyPin {
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Self {
        Self { pin_port }
    }
}

impl_unborrow!(AnyPin);
impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

// =====================

/// Set up a pin for input
#[inline]
fn init_input<T: Pin>(pin: &T, pull: Pull) {
    pin.conf().write(|w| {
        w.dir().input();
        w.input().connect();
        match pull {
            Pull::None => {
                w.pull().disabled();
            }
            Pull::Up => {
                w.pull().pullup();
            }
            Pull::Down => {
                w.pull().pulldown();
            }
        }
        w.drive().s0s1();
        w.sense().disabled();
        w
    });
}

/// Set up a pin for output
#[inline]
fn init_output<T: Pin>(pin: &T, drive: OutputDrive) {
    let drive = match drive {
        OutputDrive::Standard => DRIVE_A::S0S1,
        OutputDrive::HighDrive0Standard1 => DRIVE_A::H0S1,
        OutputDrive::Standard0HighDrive1 => DRIVE_A::S0H1,
        OutputDrive::HighDrive => DRIVE_A::H0H1,
        OutputDrive::Disconnect0Standard1 => DRIVE_A::D0S1,
        OutputDrive::Disconnect0HighDrive1 => DRIVE_A::D0H1,
        OutputDrive::Standard0Disconnect1 => DRIVE_A::S0D1,
        OutputDrive::HighDrive0Disconnect1 => DRIVE_A::H0D1,
    };

    pin.conf().write(|w| {
        w.dir().output();
        w.input().disconnect();
        w.pull().disabled();
        w.drive().variant(drive);
        w.sense().disabled();
        w
    });
}

// ====================

pub trait OptionalPin: sealed::OptionalPin + Sized {
    type Pin: Pin;
    fn pin(&self) -> Option<&Self::Pin>;
    fn pin_mut(&mut self) -> Option<&mut Self::Pin>;

    #[inline]
    fn psel_bits(&self) -> u32 {
        self.pin().map_or(1u32 << 31, |pin| Pin::psel_bits(pin))
    }

    /// Convert from concrete pin type PX_XX to type erased `Option<AnyPin>`.
    #[inline]
    fn degrade_optional(mut self) -> Option<AnyPin> {
        self.pin_mut()
            .map(|pin| unsafe { core::ptr::read(pin) }.degrade())
    }
}

impl<T: Pin> sealed::OptionalPin for T {}
impl<T: Pin> OptionalPin for T {
    type Pin = T;

    #[inline]
    fn pin(&self) -> Option<&T> {
        Some(self)
    }

    #[inline]
    fn pin_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NoPin;
impl_unborrow!(NoPin);
impl sealed::OptionalPin for NoPin {}
impl OptionalPin for NoPin {
    type Pin = AnyPin;

    #[inline]
    fn pin(&self) -> Option<&AnyPin> {
        None
    }

    #[inline]
    fn pin_mut(&mut self) -> Option<&mut AnyPin> {
        None
    }
}

// ====================

macro_rules! impl_pin {
    ($type:ident, $port_num:expr, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$type {}
        impl crate::gpio::sealed::Pin for peripherals::$type {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }
    };
}
