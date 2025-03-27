//! General purpose input/output (GPIO) driver.
#![macro_use]

use core::convert::Infallible;
use core::hint::unreachable_unchecked;

use cfg_if::cfg_if;
use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};

use crate::pac;
use crate::pac::common::{Reg, RW};
use crate::pac::gpio;
use crate::pac::gpio::vals;
#[cfg(not(feature = "_nrf51"))]
use crate::pac::shared::{regs::Psel, vals::Connect};

/// A GPIO port with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0, available on nRF9160 and all nRF52 and nRF51 MCUs.
    Port0,

    /// Port 1, only available on some MCUs.
    #[cfg(feature = "_gpio-p1")]
    Port1,

    /// Port 2, only available on some MCUs.
    #[cfg(feature = "_gpio-p2")]
    Port2,
}

/// Pull setting for an input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// GPIO input driver.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input(pull);

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

    /// Get the pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }
}

/// Digital input or output level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Drive strength settings for a given output level.
// These numbers match vals::Drive exactly so hopefully the compiler will unify them.
#[cfg(feature = "_nrf54l")]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LevelDrive {
    /// Disconnect (do not drive the output at all)
    Disconnect = 2,
    /// Standard
    Standard = 0,
    /// High drive
    High = 1,
    /// Extra high drive
    ExtraHigh = 3,
}

/// Drive strength settings for an output pin.
///
/// This is a combination of two drive levels, used when the pin is set
/// low and high respectively.
#[cfg(feature = "_nrf54l")]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutputDrive {
    low: LevelDrive,
    high: LevelDrive,
}

#[cfg(feature = "_nrf54l")]
#[allow(non_upper_case_globals)]
impl OutputDrive {
    /// Standard '0', standard '1'
    pub const Standard: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::Standard,
    };
    /// High drive '0', standard '1'
    pub const HighDrive0Standard1: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::Standard,
    };
    /// Standard '0', high drive '1'
    pub const Standard0HighDrive1: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::High,
    };
    /// High drive '0', high 'drive '1'
    pub const HighDrive: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::High,
    };
    /// Disconnect '0' standard '1' (normally used for wired-or connections)
    pub const Disconnect0Standard1: Self = Self {
        low: LevelDrive::Disconnect,
        high: LevelDrive::Standard,
    };
    /// Disconnect '0', high drive '1' (normally used for wired-or connections)
    pub const Disconnect0HighDrive1: Self = Self {
        low: LevelDrive::Disconnect,
        high: LevelDrive::High,
    };
    /// Standard '0'. disconnect '1' (also known as "open drain", normally used for wired-and connections)
    pub const Standard0Disconnect1: Self = Self {
        low: LevelDrive::Standard,
        high: LevelDrive::Disconnect,
    };
    /// High drive '0', disconnect '1' (also known as "open drain", normally used for wired-and connections)
    pub const HighDrive0Disconnect1: Self = Self {
        low: LevelDrive::High,
        high: LevelDrive::Disconnect,
    };
}

/// Drive strength settings for an output pin.
// These numbers match vals::Drive exactly so hopefully the compiler will unify them.
#[cfg(not(feature = "_nrf54l"))]
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
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] and [OutputDriver] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level, drive: OutputDrive) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }
        pin.set_as_output(drive);

        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
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
}

pub(crate) fn convert_drive(w: &mut pac::gpio::regs::PinCnf, drive: OutputDrive) {
    #[cfg(not(feature = "_nrf54l"))]
    {
        let drive = match drive {
            OutputDrive::Standard => vals::Drive::S0S1,
            OutputDrive::HighDrive0Standard1 => vals::Drive::H0S1,
            OutputDrive::Standard0HighDrive1 => vals::Drive::S0H1,
            OutputDrive::HighDrive => vals::Drive::H0H1,
            OutputDrive::Disconnect0Standard1 => vals::Drive::D0S1,
            OutputDrive::Disconnect0HighDrive1 => vals::Drive::D0H1,
            OutputDrive::Standard0Disconnect1 => vals::Drive::S0D1,
            OutputDrive::HighDrive0Disconnect1 => vals::Drive::H0D1,
        };
        w.set_drive(drive);
    }

    #[cfg(feature = "_nrf54l")]
    {
        fn convert(d: LevelDrive) -> vals::Drive {
            match d {
                LevelDrive::Disconnect => vals::Drive::D,
                LevelDrive::Standard => vals::Drive::S,
                LevelDrive::High => vals::Drive::H,
                LevelDrive::ExtraHigh => vals::Drive::E,
            }
        }

        w.set_drive0(convert(drive.low));
        w.set_drive0(convert(drive.high));
    }
}

fn convert_pull(pull: Pull) -> vals::Pull {
    match pull {
        Pull::None => vals::Pull::DISABLED,
        Pull::Up => vals::Pull::PULLUP,
        Pull::Down => vals::Pull::PULLDOWN,
    }
}

/// GPIO flexible pin.
///
/// This pin can either be a disconnected, input, or output pin, or both. The level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d> {
    pub(crate) pin: Peri<'d, AnyPin>,
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

    /// Put the pin into input mode.
    #[inline]
    pub fn set_as_input(&mut self, pull: Pull) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::INPUT);
            w.set_input(vals::Input::CONNECT);
            w.set_pull(convert_pull(pull));
            convert_drive(w, OutputDrive::Standard);
            w.set_sense(vals::Sense::DISABLED);
        });
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self, drive: OutputDrive) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::OUTPUT);
            w.set_input(vals::Input::DISCONNECT);
            w.set_pull(vals::Pull::DISABLED);
            convert_drive(w, drive);
            w.set_sense(vals::Sense::DISABLED);
        });
    }

    /// Put the pin into input + output mode.
    ///
    /// This is commonly used for "open drain" mode. If you set `drive = Standard0Disconnect1`,
    /// the hardware will drive the line low if you set it to low, and will leave it floating if you set
    /// it to high, in which case you can read the input to figure out whether another device
    /// is driving the line low.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_input_output(&mut self, pull: Pull, drive: OutputDrive) {
        self.pin.conf().write(|w| {
            w.set_dir(vals::Dir::OUTPUT);
            w.set_input(vals::Input::CONNECT);
            w.set_pull(convert_pull(pull));
            convert_drive(w, drive);
            w.set_sense(vals::Sense::DISABLED);
        });
    }

    /// Put the pin into disconnected mode.
    #[inline]
    pub fn set_as_disconnected(&mut self) {
        self.pin.conf().write(|w| {
            w.set_input(vals::Input::DISCONNECT);
        });
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.block().in_().read().pin(self.pin.pin() as _)
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Get the pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Toggle the output level.
    #[inline]
    pub fn toggle(&mut self) {
        if self.is_set_low() {
            self.set_high()
        } else {
            self.set_low()
        }
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.pin.set_low(),
            Level::High => self.pin.set_high(),
        }
    }

    /// Get whether the output level is set to high.
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.block().out().read().pin(self.pin.pin() as _)
    }

    /// Get whether the output level is set to low.
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }
}

impl<'d> Drop for Flex<'d> {
    fn drop(&mut self) {
        self.set_as_disconnected();
    }
}

pub(crate) trait SealedPin {
    fn pin_port(&self) -> u8;

    #[inline]
    fn _pin(&self) -> u8 {
        cfg_if! {
            if #[cfg(feature = "_gpio-p1")] {
                self.pin_port() % 32
            } else {
                self.pin_port()
            }
        }
    }

    #[inline]
    fn block(&self) -> gpio::Gpio {
        match self.pin_port() / 32 {
            #[cfg(feature = "_nrf51")]
            0 => pac::GPIO,
            #[cfg(not(feature = "_nrf51"))]
            0 => pac::P0,
            #[cfg(feature = "_gpio-p1")]
            1 => pac::P1,
            #[cfg(feature = "_gpio-p2")]
            2 => pac::P2,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    fn conf(&self) -> Reg<gpio::regs::PinCnf, RW> {
        self.block().pin_cnf(self._pin() as usize)
    }

    /// Set the output as high.
    #[inline]
    fn set_high(&self) {
        self.block().outset().write(|w| w.set_pin(self._pin() as _, true))
    }

    /// Set the output as low.
    #[inline]
    fn set_low(&self) {
        self.block().outclr().write(|w| w.set_pin(self._pin() as _, true))
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
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
            #[cfg(feature = "_gpio-p1")]
            1 => Port::Port1,
            #[cfg(feature = "_gpio-p2")]
            2 => Port::Port2,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// Peripheral port register value
    #[inline]
    #[cfg(not(feature = "_nrf51"))]
    fn psel_bits(&self) -> pac::shared::regs::Psel {
        pac::shared::regs::Psel(self.pin_port() as u32)
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

// ====================

#[cfg(not(feature = "_nrf51"))]
#[cfg_attr(feature = "_nrf54l", allow(unused))] // TODO
pub(crate) trait PselBits {
    fn psel_bits(&self) -> pac::shared::regs::Psel;
}

#[cfg(not(feature = "_nrf51"))]
impl<'a, P: Pin> PselBits for Option<Peri<'a, P>> {
    #[inline]
    fn psel_bits(&self) -> pac::shared::regs::Psel {
        match self {
            Some(pin) => pin.psel_bits(),
            None => DISCONNECTED,
        }
    }
}

#[cfg(not(feature = "_nrf51"))]
#[cfg_attr(feature = "_nrf54l", allow(unused))] // TODO
pub(crate) const DISCONNECTED: Psel = Psel(1 << 31);

#[cfg(not(feature = "_nrf51"))]
#[allow(dead_code)]
pub(crate) fn deconfigure_pin(psel: Psel) {
    if psel.connect() == Connect::DISCONNECTED {
        return;
    }
    unsafe { AnyPin::steal(psel.0 as _) }.conf().write(|w| {
        w.set_input(vals::Input::DISCONNECT);
    })
}

// ====================

macro_rules! impl_pin {
    ($type:ident, $port_num:expr, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$type {}
        impl crate::gpio::SealedPin for peripherals::$type {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }

        impl From<peripherals::$type> for crate::gpio::AnyPin {
            fn from(_val: peripherals::$type) -> Self {
                Self {
                    pin_port: $port_num * 32 + $pin_num,
                }
            }
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'d> embedded_hal_02::digital::v2::InputPin for Input<'d> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::OutputPin for Output<'d> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'d> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }

    /// Implement [`embedded_hal_02::digital::v2::InputPin`] for [`Flex`];
    ///
    /// If the pin is not in input mode the result is unspecified.
    impl<'d> embedded_hal_02::digital::v2::InputPin for Flex<'d> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::OutputPin for Flex<'d> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle();
            Ok(())
        }
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::InputPin for Input<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::OutputPin for Output<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for Flex<'d> {
    type Error = Infallible;
}

/// Implement [`InputPin`] for [`Flex`];
///
/// If the pin is not in input mode the result is unspecified.
impl<'d> embedded_hal_1::digital::InputPin for Flex<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<'d> embedded_hal_1::digital::OutputPin for Flex<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> embedded_hal_1::digital::StatefulOutputPin for Flex<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}
