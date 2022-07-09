use core::convert::Infallible;
use core::marker::PhantomData;

use embassy_hal_common::{unborrow, unsafe_impl_unborrow};

use crate::pac::common::{Reg, RW};
use crate::pac::SIO;
use crate::{pac, peripherals, Unborrow};

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

/// Represents a pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Bank {
    Bank0 = 0,
    Qspi = 1,
}

pub struct Input<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> Input<'d, T> {
    pub fn new(pin: impl Unborrow<Target = T> + 'd, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }
}

pub struct Output<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> Output<'d, T> {
    #[inline]
    pub fn new(pin: impl Unborrow<Target = T> + 'd, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        pin.set_as_output();
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

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }
}

/// GPIO output open-drain.
pub struct OutputOpenDrain<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> OutputOpenDrain<'d, T> {
    #[inline]
    pub fn new(pin: impl Unborrow<Target = T> + 'd, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_low();
        match initial_output {
            Level::High => pin.set_as_input(),
            Level::Low => pin.set_as_output(),
        }
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        // For Open Drain High, disable the output pin.
        self.pin.set_as_input()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        // For Open Drain Low, enable the output pin.
        self.pin.set_as_output()
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_as_output()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle_set_as_output()
    }
}

/// GPIO flexible pin.
///
/// This pin can be either an input or output pin. The output level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d, T: Pin> {
    pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Flex<'d, T> {
    #[inline]
    pub fn new(pin: impl Unborrow<Target = T> + 'd) -> Self {
        unborrow!(pin);

        unsafe {
            pin.pad_ctrl().write(|w| {
                w.set_ie(true);
            });

            pin.io().ctrl().write(|w| {
                w.set_funcsel(pac::io::vals::Gpio0CtrlFuncsel::SIO_0.0);
            });
        }

        Self {
            pin,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn bit(&self) -> u32 {
        1 << self.pin.pin()
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        unsafe {
            self.pin.pad_ctrl().write(|w| {
                w.set_ie(true);
                match pull {
                    Pull::Up => w.set_pue(true),
                    Pull::Down => w.set_pde(true),
                    Pull::None => {}
                }
            });
        }
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        unsafe { self.pin.sio_oe().value_clr().write_value(self.bit()) }
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        unsafe { self.pin.sio_oe().value_set().write_value(self.bit()) }
    }

    #[inline]
    fn is_set_as_output(&self) -> bool {
        unsafe { (self.pin.sio_oe().value().read() & self.bit()) != 0 }
    }

    #[inline]
    pub fn toggle_set_as_output(&mut self) {
        unsafe { self.pin.sio_oe().value_xor().write_value(self.bit()) }
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        unsafe { self.pin.sio_in().read() & self.bit() == 0 }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        unsafe { self.pin.sio_out().value_set().write_value(self.bit()) }
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        unsafe { self.pin.sio_out().value_clr().write_value(self.bit()) }
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        unsafe { (self.pin.sio_out().value().read() & self.bit()) == 0 }
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        unsafe { self.pin.sio_out().value_xor().write_value(self.bit()) }
    }
}

impl<'d, T: Pin> Drop for Flex<'d, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.pin.pad_ctrl().write(|_| {});
            self.pin.io().ctrl().write(|w| {
                w.set_funcsel(pac::io::vals::Gpio0CtrlFuncsel::NULL.0);
            });
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin: Sized {
        fn pin_bank(&self) -> u8;

        #[inline]
        fn pin(&self) -> u8 {
            self.pin_bank() & 0x1f
        }

        #[inline]
        fn bank(&self) -> Bank {
            if self.pin_bank() & 0x20 == 0 {
                Bank::Bank0
            } else {
                Bank::Qspi
            }
        }

        fn io(&self) -> pac::io::Gpio {
            let block = match self.bank() {
                Bank::Bank0 => crate::pac::IO_BANK0,
                Bank::Qspi => crate::pac::IO_QSPI,
            };
            block.gpio(self.pin() as _)
        }

        fn pad_ctrl(&self) -> Reg<pac::pads::regs::GpioCtrl, RW> {
            let block = match self.bank() {
                Bank::Bank0 => crate::pac::PADS_BANK0,
                Bank::Qspi => crate::pac::PADS_QSPI,
            };
            block.gpio(self.pin() as _)
        }

        fn sio_out(&self) -> pac::sio::Gpio {
            SIO.gpio_out(self.bank() as _)
        }

        fn sio_oe(&self) -> pac::sio::Gpio {
            SIO.gpio_oe(self.bank() as _)
        }

        fn sio_in(&self) -> Reg<u32, RW> {
            SIO.gpio_in(self.bank() as _)
        }
    }
}

pub trait Pin: Unborrow<Target = Self> + sealed::Pin {
    /// Degrade to a generic pin struct
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_bank: self.pin_bank(),
        }
    }
}

pub struct AnyPin {
    pin_bank: u8,
}
unsafe_impl_unborrow!(AnyPin);
impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    fn pin_bank(&self) -> u8 {
        self.pin_bank
    }
}

// ==========================

macro_rules! impl_pin {
    ($name:ident, $bank:expr, $pin_num:expr) => {
        impl Pin for peripherals::$name {}
        impl sealed::Pin for peripherals::$name {
            fn pin_bank(&self) -> u8 {
                ($bank as u8) * 32 + $pin_num
            }
        }
    };
}

impl_pin!(PIN_0, Bank::Bank0, 0);
impl_pin!(PIN_1, Bank::Bank0, 1);
impl_pin!(PIN_2, Bank::Bank0, 2);
impl_pin!(PIN_3, Bank::Bank0, 3);
impl_pin!(PIN_4, Bank::Bank0, 4);
impl_pin!(PIN_5, Bank::Bank0, 5);
impl_pin!(PIN_6, Bank::Bank0, 6);
impl_pin!(PIN_7, Bank::Bank0, 7);
impl_pin!(PIN_8, Bank::Bank0, 8);
impl_pin!(PIN_9, Bank::Bank0, 9);
impl_pin!(PIN_10, Bank::Bank0, 10);
impl_pin!(PIN_11, Bank::Bank0, 11);
impl_pin!(PIN_12, Bank::Bank0, 12);
impl_pin!(PIN_13, Bank::Bank0, 13);
impl_pin!(PIN_14, Bank::Bank0, 14);
impl_pin!(PIN_15, Bank::Bank0, 15);
impl_pin!(PIN_16, Bank::Bank0, 16);
impl_pin!(PIN_17, Bank::Bank0, 17);
impl_pin!(PIN_18, Bank::Bank0, 18);
impl_pin!(PIN_19, Bank::Bank0, 19);
impl_pin!(PIN_20, Bank::Bank0, 20);
impl_pin!(PIN_21, Bank::Bank0, 21);
impl_pin!(PIN_22, Bank::Bank0, 22);
impl_pin!(PIN_23, Bank::Bank0, 23);
impl_pin!(PIN_24, Bank::Bank0, 24);
impl_pin!(PIN_25, Bank::Bank0, 25);
impl_pin!(PIN_26, Bank::Bank0, 26);
impl_pin!(PIN_27, Bank::Bank0, 27);
impl_pin!(PIN_28, Bank::Bank0, 28);
impl_pin!(PIN_29, Bank::Bank0, 29);

impl_pin!(PIN_QSPI_SCLK, Bank::Qspi, 0);
impl_pin!(PIN_QSPI_SS, Bank::Qspi, 1);
impl_pin!(PIN_QSPI_SD0, Bank::Qspi, 2);
impl_pin!(PIN_QSPI_SD1, Bank::Qspi, 3);
impl_pin!(PIN_QSPI_SD2, Bank::Qspi, 4);
impl_pin!(PIN_QSPI_SD3, Bank::Qspi, 5);

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Pin> embedded_hal_02::digital::v2::InputPin for Input<'d, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for Output<'d, T> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for OutputOpenDrain<'d, T> {
        type Error = Infallible;

        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for OutputOpenDrain<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for OutputOpenDrain<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::InputPin for Flex<'d, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for Flex<'d, T> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Input<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::InputPin for Input<'d, T> {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Output<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::OutputPin for Output<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::StatefulOutputPin for Output<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::ToggleableOutputPin for Output<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for OutputOpenDrain<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::OutputPin for OutputOpenDrain<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::StatefulOutputPin for OutputOpenDrain<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::ToggleableOutputPin for OutputOpenDrain<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Flex<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::InputPin for Flex<'d, T> {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::OutputPin for Flex<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::StatefulOutputPin for Flex<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::blocking::ToggleableOutputPin for Flex<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }
}
