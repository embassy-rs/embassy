use core::convert::Infallible;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

use embassy::util::PeripheralBorrow;
use embassy_extras::{impl_unborrow, unborrow};
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use gpio::vals;

use crate::pac::gpio_v2 as gpio;
use crate::peripherals;

/// A GPIO port with up to 16 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Port {
    PortA,
    PortB,
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

        cortex_m::interrupt::free(|_| unsafe {
            let r = pin.block();
            let n = pin.pin() as usize;
            let val = match pull {
                Pull::None => vals::Pupdr::FLOATING,
                Pull::Up => vals::Pupdr::PULLUP,
                Pull::Down => vals::Pupdr::PULLDOWN,
            };
            r.pupdr().modify(|w| w.set_pupdr(n, val));
            r.otyper().modify(|w| w.set_ot(n, vals::Ot::PUSHPULL));
            r.moder().modify(|w| w.set_moder(n, vals::Moder::INPUT));
        });

        Self {
            pin,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Pin> Drop for Input<'d, T> {
    fn drop(&mut self) {
        cortex_m::interrupt::free(|_| unsafe {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            r.pupdr().modify(|w| w.set_pupdr(n, vals::Pupdr::FLOATING));
        });
    }
}

impl<'d, T: Pin> InputPin for Input<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let state = unsafe { self.pin.block().idr().read().idr(self.pin.pin() as _) };
        Ok(state == vals::Idr::LOW)
    }
}

/// Digital input or output level.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    Low,
    High,
}

/// GPIO output driver.
pub struct Output<'d, T: Pin> {
    pub(crate) pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Output<'d, T> {
    pub fn new(pin: impl PeripheralBorrow<Target = T> + 'd, initial_output: Level) -> Self {
        unborrow!(pin);

        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        cortex_m::interrupt::free(|_| unsafe {
            let r = pin.block();
            let n = pin.pin() as usize;
            r.pupdr().modify(|w| w.set_pupdr(n, vals::Pupdr::FLOATING));
            r.moder().modify(|w| w.set_moder(n, vals::Moder::OUTPUT));
        });

        Self {
            pin,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Pin> Drop for Output<'d, T> {
    fn drop(&mut self) {
        cortex_m::interrupt::free(|_| unsafe {
            let r = self.pin.block();
            let n = self.pin.pin() as usize;
            r.pupdr().modify(|w| w.set_pupdr(n, vals::Pupdr::FLOATING));
            r.moder().modify(|w| w.set_moder(n, vals::Moder::INPUT));
        });
    }
}

impl<'d, T: Pin> OutputPin for Output<'d, T> {
    type Error = Infallible;

    /// Set the output as high.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }

    /// Set the output as low.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
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
        let state = unsafe { self.pin.block().odr().read().odr(self.pin.pin() as _) };
        Ok(state == vals::Odr::LOW)
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin {
        fn pin_port(&self) -> u8;

        #[inline]
        fn _pin(&self) -> u8 {
            self.pin_port() % 16
        }
        #[inline]
        fn _port(&self) -> u8 {
            self.pin_port() / 16
        }

        #[inline]
        fn block(&self) -> gpio::Gpio {
            let p = 0x4002_0000 + (self._port() as u32) * 0x400;
            gpio::Gpio(p as *mut u8)
        }

        /// Set the output as high.
        #[inline]
        fn set_high(&self) {
            unsafe {
                self.block()
                    .bsrr()
                    .write(|w| w.set_bs(self._pin() as _, true));
            }
        }

        /// Set the output as low.
        #[inline]
        fn set_low(&self) {
            unsafe {
                self.block()
                    .bsrr()
                    .write(|w| w.set_br(self._pin() as _, true));
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
        match self.pin_port() / 16 {
            0 => Port::PortA,
            1 => Port::PortB,
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

// Uninhabited enum, so it's actually impossible to create a DummyPin value.
#[doc(hidden)]
pub enum DummyPin {}
impl Pin for DummyPin {}
impl sealed::Pin for DummyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NoPin;
impl_unborrow!(NoPin);
impl sealed::OptionalPin for NoPin {}
impl OptionalPin for NoPin {
    type Pin = DummyPin;

    #[inline]
    fn pin(&self) -> Option<&DummyPin> {
        None
    }

    #[inline]
    fn pin_mut(&mut self) -> Option<&mut DummyPin> {
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
                $port_num * 16 + $pin_num
            }
        }
    };
}

impl_pin!(PA0, 0, 0);
impl_pin!(PA1, 0, 1);
impl_pin!(PA2, 0, 2);
impl_pin!(PA3, 0, 3);
impl_pin!(PA4, 0, 4);
impl_pin!(PA5, 0, 5);
impl_pin!(PA6, 0, 6);
impl_pin!(PA7, 0, 7);
impl_pin!(PA8, 0, 8);
impl_pin!(PA9, 0, 9);
impl_pin!(PA10, 0, 10);
impl_pin!(PA11, 0, 11);
impl_pin!(PA12, 0, 12);
impl_pin!(PA13, 0, 13);
impl_pin!(PA14, 0, 14);
impl_pin!(PA15, 0, 15);
impl_pin!(PB0, 1, 0);
impl_pin!(PB1, 1, 1);
impl_pin!(PB2, 1, 2);
impl_pin!(PB3, 1, 3);
impl_pin!(PB4, 1, 4);
impl_pin!(PB5, 1, 5);
impl_pin!(PB6, 1, 6);
impl_pin!(PB7, 1, 7);
impl_pin!(PB8, 1, 8);
impl_pin!(PB9, 1, 9);
impl_pin!(PB10, 1, 10);
impl_pin!(PB11, 1, 11);
impl_pin!(PB12, 1, 12);
impl_pin!(PB13, 1, 13);
impl_pin!(PB14, 1, 14);
impl_pin!(PB15, 1, 15);
impl_pin!(PC0, 2, 0);
impl_pin!(PC1, 2, 1);
impl_pin!(PC2, 2, 2);
impl_pin!(PC3, 2, 3);
impl_pin!(PC4, 2, 4);
impl_pin!(PC5, 2, 5);
impl_pin!(PC6, 2, 6);
impl_pin!(PC7, 2, 7);
impl_pin!(PC8, 2, 8);
impl_pin!(PC9, 2, 9);
impl_pin!(PC10, 2, 10);
impl_pin!(PC11, 2, 11);
impl_pin!(PC12, 2, 12);
impl_pin!(PC13, 2, 13);
impl_pin!(PC14, 2, 14);
impl_pin!(PC15, 2, 15);
