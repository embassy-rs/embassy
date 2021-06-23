#![macro_use]
use core::convert::Infallible;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_extras::{unborrow, unsafe_impl_unborrow};
use embedded_hal::digital::v2::{toggleable, InputPin, OutputPin, StatefulOutputPin};

use crate::pac;
use crate::pac::gpio::{self, vals};
use crate::peripherals;

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    None,
    Up,
    Down,
}

/// Pull setting for an input.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    LowSpeed,
    MediumSpeed,
    #[cfg(not(syscfg_f0))]
    HighSpeed,
    VeryHighSpeed,
}

impl From<Speed> for vals::Ospeedr {
    fn from(speed: Speed) -> Self {
        use Speed::*;

        match speed {
            LowSpeed => vals::Ospeedr::LOWSPEED,
            MediumSpeed => vals::Ospeedr::MEDIUMSPEED,
            #[cfg(not(syscfg_f0))]
            HighSpeed => vals::Ospeedr::HIGHSPEED,
            VeryHighSpeed => vals::Ospeedr::VERYHIGHSPEED,
        }
    }
}

/// GPIO input driver.
pub struct Input<'d, T: Pin> {
    pub(crate) pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Input<'d, T> {
    pub fn new(pin: impl Unborrow<Target = T> + 'd, pull: Pull) -> Self {
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
    pub fn new(
        pin: impl Unborrow<Target = T> + 'd,
        initial_output: Level,
        speed: Speed,
        open_drain: bool,
    ) -> Self {
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
            if open_drain {
                r.otyper().modify(|w| w.set_ot(n, vals::Ot::OPENDRAIN));
            }
            pin.set_speed(speed);
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
            r.otyper().modify(|w| w.set_ot(n, vals::Ot::PUSHPULL));
            self.pin.set_speed(Speed::LowSpeed);
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

impl<'d, T: Pin> InputPin for Output<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_set_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_set_low()
    }
}

impl<'d, T: Pin> toggleable::Default for Output<'d, T> {}

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
            pac::GPIO(self._port() as _)
        }

        /// Set the output as high.
        #[inline]
        fn set_high(&self) {
            unsafe {
                let n = self._pin() as _;
                self.block().bsrr().write(|w| w.set_bs(n, true));
            }
        }

        /// Set the output as low.
        #[inline]
        fn set_low(&self) {
            unsafe {
                let n = self._pin() as _;
                self.block().bsrr().write(|w| w.set_br(n, true));
            }
        }

        unsafe fn set_as_af(&self, af_num: u8) {
            let pin = self._pin() as usize;
            let block = self.block();
            block
                .moder()
                .modify(|w| w.set_moder(pin, vals::Moder::ALTERNATE));
            block
                .afr(pin / 8)
                .modify(|w| w.set_afr(pin % 8, vals::Afr(af_num)));
        }

        unsafe fn set_as_analog(&self) {
            let pin = self._pin() as usize;
            let block = self.block();
            block
                .moder()
                .modify(|w| w.set_moder(pin, vals::Moder::ANALOG));
        }

        unsafe fn set_speed(&self, speed: Speed) {
            let pin = self._pin() as usize;
            self.block()
                .ospeedr()
                .modify(|w| w.set_ospeedr(pin, speed.into()));
        }
    }

    pub trait OptionalPin {}
}

pub trait Pin: sealed::Pin + Sized {
    type ExtiChannel: crate::exti::Channel;

    /// Number of the pin within the port (0..31)
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Port of the pin
    #[inline]
    fn port(&self) -> u8 {
        self._port()
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

    #[inline]
    fn _port(&self) -> u8 {
        self.pin_port / 16
    }

    #[inline]
    pub fn block(&self) -> gpio::Gpio {
        pac::GPIO(self._port() as _)
    }
}

unsafe_impl_unborrow!(AnyPin);
impl Pin for AnyPin {
    type ExtiChannel = crate::exti::AnyChannel;
}
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
unsafe_impl_unborrow!(NoPin);
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

crate::pac::pins!(
    ($pin_name:ident, $port_name:ident, $port_num:expr, $pin_num:expr, $exti_ch:ident) => {
        impl Pin for peripherals::$pin_name {
            type ExtiChannel = peripherals::$exti_ch;
        }
        impl sealed::Pin for peripherals::$pin_name {
            #[inline]
            fn pin_port(&self) -> u8 {
                $port_num * 16 + $pin_num
            }
        }
    };
);
