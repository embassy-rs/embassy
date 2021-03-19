use core::convert::Infallible;
use core::hint::unreachable_unchecked;

use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

use crate::pac;
use crate::pac::p0 as gpio;
use crate::peripherals;

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

/// A GPIO port with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    /// Port 0, available on all nRF52 and nRF51 MCUs.
    Port0,

    /// Port 1, only available on some nRF52 MCUs.
    #[cfg(any(feature = "52833", feature = "52840"))]
    Port1,
}

pub struct Input<T: Pin> {
    pin: T,
}

impl<T: Pin> Input<T> {
    pub fn new(pin: T, pull: Pull) -> Self {
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

        Self { pin }
    }
}

impl<T: Pin> Drop for Input<T> {
    fn drop(&mut self) {
        self.pin.conf().reset();
    }
}

impl<T: Pin> InputPin for Input<T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.block().in_.read().bits() & (1 << self.pin.pin()) == 0)
    }
}

pub struct Output<T: Pin> {
    pin: T,
}

impl<T: Pin> Output<T> {
    // TODO opendrain
    pub fn new(pin: T, initial_output: Level) -> Self {
        pin.conf().write(|w| {
            w.dir().output();
            w.input().disconnect();
            w.pull().disabled();
            w.drive().s0s1();
            w.sense().disabled();
            w
        });

        Self { pin }
    }
}

impl<T: Pin> Drop for Output<T> {
    fn drop(&mut self) {
        self.pin.conf().reset();
    }
}

impl<T: Pin> OutputPin for Output<T> {
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

impl<T: Pin> StatefulOutputPin for Output<T> {
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

        fn conf(&self) -> &gpio::PIN_CNF {
            &self.block().pin_cnf[self._pin() as usize]
        }

        /// Set the output as high.
        fn set_high(&self) {
            unsafe {
                self.block().outset.write(|w| w.bits(1u32 << self._pin()));
            }
        }

        /// Set the output as low.
        fn set_low(&self) {
            unsafe {
                self.block().outclr.write(|w| w.bits(1u32 << self._pin()));
            }
        }
    }
}

pub trait Pin: sealed::Pin + Sized {
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    #[inline]
    fn port(&self) -> Port {
        match self.pin_port() / 32 {
            1 => Port::Port0,
            #[cfg(any(feature = "52833", feature = "52840"))]
            1 => Port::Port1,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    fn psel_bits(&self) -> u32 {
        self.pin_port() as u32
    }

    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_port: self.pin_port(),
        }
    }
}

pub struct AnyPin {
    pin_port: u8,
}

impl AnyPin {
    pub unsafe fn from_psel_bits(psel_bits: u32) -> Self {
        Self {
            pin_port: psel_bits as u8,
        }
    }
}

impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

macro_rules! make_impl {
    ($type:ident, $port_num:expr, $pin_num:expr) => {
        impl Pin for peripherals::$type {}
        impl sealed::Pin for peripherals::$type {
            fn pin_port(&self) -> u8 {
                $port_num * 32 + $pin_num
            }
        }
    };
}

make_impl!(P0_00, 0, 0);
make_impl!(P0_01, 0, 1);
make_impl!(P0_02, 0, 2);
make_impl!(P0_03, 0, 3);
make_impl!(P0_04, 0, 4);
make_impl!(P0_05, 0, 5);
make_impl!(P0_06, 0, 6);
make_impl!(P0_07, 0, 7);
make_impl!(P0_08, 0, 8);
make_impl!(P0_09, 0, 9);
make_impl!(P0_10, 0, 10);
make_impl!(P0_11, 0, 11);
make_impl!(P0_12, 0, 12);
make_impl!(P0_13, 0, 13);
make_impl!(P0_14, 0, 14);
make_impl!(P0_15, 0, 15);
make_impl!(P0_16, 0, 16);
make_impl!(P0_17, 0, 17);
make_impl!(P0_18, 0, 18);
make_impl!(P0_19, 0, 19);
make_impl!(P0_20, 0, 20);
make_impl!(P0_21, 0, 21);
make_impl!(P0_22, 0, 22);
make_impl!(P0_23, 0, 23);
make_impl!(P0_24, 0, 24);
make_impl!(P0_25, 0, 25);
make_impl!(P0_26, 0, 26);
make_impl!(P0_27, 0, 27);
make_impl!(P0_28, 0, 28);
make_impl!(P0_29, 0, 29);
make_impl!(P0_30, 0, 30);
make_impl!(P0_31, 0, 31);

#[cfg(any(feature = "52833", feature = "52840"))]
mod _p1 {
    use super::*;
    make_impl!(P1_00, 1, 0);
    make_impl!(P1_01, 1, 1);
    make_impl!(P1_02, 1, 2);
    make_impl!(P1_03, 1, 3);
    make_impl!(P1_04, 1, 4);
    make_impl!(P1_05, 1, 5);
    make_impl!(P1_06, 1, 6);
    make_impl!(P1_07, 1, 7);
    make_impl!(P1_08, 1, 8);
    make_impl!(P1_09, 1, 9);
    make_impl!(P1_10, 1, 10);
    make_impl!(P1_11, 1, 11);
    make_impl!(P1_12, 1, 12);
    make_impl!(P1_13, 1, 13);
    make_impl!(P1_14, 1, 14);
    make_impl!(P1_15, 1, 15);
}
