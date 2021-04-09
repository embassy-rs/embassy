use core::convert::Infallible;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

use embassy::util::PeripheralBorrow;
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
    #[cfg(any(feature = "52833", feature = "52840"))]
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
    pub fn new(pin: impl PeripheralBorrow<Target = T> + 'd, pull: Pull) -> Self {
        unborrow!(pin);

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
        pin: impl PeripheralBorrow<Target = T> + 'd,
        initial_output: Level,
        drive: OutputDrive,
    ) -> Self {
        unborrow!(pin);

        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

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

pub(crate) mod sealed {
    use super::*;

    pub trait Pin {
        fn pin_port(&self) -> u8;

        #[inline]
        fn _pin(&self) -> u8 {
            #[cfg(any(feature = "52833", feature = "52840"))]
            {
                self.pin_port() % 32
            }

            #[cfg(not(any(feature = "52833", feature = "52840")))]
            {
                self.pin_port()
            }
        }

        #[inline]
        fn block(&self) -> &gpio::RegisterBlock {
            unsafe {
                match self.pin_port() / 32 {
                    0 => &*pac::P0::ptr(),
                    #[cfg(any(feature = "52833", feature = "52840"))]
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
            #[cfg(any(feature = "52833", feature = "52840"))]
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
        impl Pin for peripherals::$type {}
        impl sealed::Pin for peripherals::$type {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }
    };
}

impl_pin!(P0_00, 0, 0);
impl_pin!(P0_01, 0, 1);
impl_pin!(P0_02, 0, 2);
impl_pin!(P0_03, 0, 3);
impl_pin!(P0_04, 0, 4);
impl_pin!(P0_05, 0, 5);
impl_pin!(P0_06, 0, 6);
impl_pin!(P0_07, 0, 7);
impl_pin!(P0_08, 0, 8);
impl_pin!(P0_09, 0, 9);
impl_pin!(P0_10, 0, 10);
impl_pin!(P0_11, 0, 11);
impl_pin!(P0_12, 0, 12);
impl_pin!(P0_13, 0, 13);
impl_pin!(P0_14, 0, 14);
impl_pin!(P0_15, 0, 15);
impl_pin!(P0_16, 0, 16);
impl_pin!(P0_17, 0, 17);
impl_pin!(P0_18, 0, 18);
impl_pin!(P0_19, 0, 19);
impl_pin!(P0_20, 0, 20);
impl_pin!(P0_21, 0, 21);
impl_pin!(P0_22, 0, 22);
impl_pin!(P0_23, 0, 23);
impl_pin!(P0_24, 0, 24);
impl_pin!(P0_25, 0, 25);
impl_pin!(P0_26, 0, 26);
impl_pin!(P0_27, 0, 27);
impl_pin!(P0_28, 0, 28);
impl_pin!(P0_29, 0, 29);
impl_pin!(P0_30, 0, 30);
impl_pin!(P0_31, 0, 31);

#[cfg(any(feature = "52833", feature = "52840"))]
mod _p1 {
    use super::*;
    impl_pin!(P1_00, 1, 0);
    impl_pin!(P1_01, 1, 1);
    impl_pin!(P1_02, 1, 2);
    impl_pin!(P1_03, 1, 3);
    impl_pin!(P1_04, 1, 4);
    impl_pin!(P1_05, 1, 5);
    impl_pin!(P1_06, 1, 6);
    impl_pin!(P1_07, 1, 7);
    impl_pin!(P1_08, 1, 8);
    impl_pin!(P1_09, 1, 9);
    impl_pin!(P1_10, 1, 10);
    impl_pin!(P1_11, 1, 11);
    impl_pin!(P1_12, 1, 12);
    impl_pin!(P1_13, 1, 13);
    impl_pin!(P1_14, 1, 14);
    impl_pin!(P1_15, 1, 15);
}
