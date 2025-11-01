//! GPIO driver built around a type-erased `Flex` pin, similar to other Embassy HALs.
//! The exported `Output`/`Input` drivers own a `Flex` so they no longer depend on the
//! concrete pin type.

use core::marker::PhantomData;

use crate::{pac, pins as pin_config};

/// Logical level for GPIO pins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Level {
    Low,
    High,
}

pub type Gpio = crate::peripherals::GPIO;

/// Type-erased representation of a GPIO pin.
#[derive(Copy, Clone)]
pub struct AnyPin {
    port: u8,
    pin: u8,
    gpio: *const pac::gpio0::RegisterBlock,
}

impl AnyPin {
    /// Create an `AnyPin` from raw components.
    pub fn new(port: u8, pin: u8, gpio: *const pac::gpio0::RegisterBlock) -> Self {
        Self { port, pin, gpio }
    }

    #[inline(always)]
    fn mask(&self) -> u32 {
        1u32 << self.pin
    }

    #[inline(always)]
    fn gpio(&self) -> &'static pac::gpio0::RegisterBlock {
        unsafe { &*self.gpio }
    }

    #[inline(always)]
    pub fn port_index(&self) -> u8 {
        self.port
    }

    #[inline(always)]
    pub fn pin_index(&self) -> u8 {
        self.pin
    }
}

/// Type-level trait implemented by concrete pin ZSTs.
pub trait PinId {
    fn port_index() -> u8;
    fn pin_index() -> u8;
    fn gpio_ptr() -> *const pac::gpio0::RegisterBlock;

    fn set_mux_gpio() {
        unsafe { pin_config::set_pin_mux_gpio(Self::port_index(), Self::pin_index()) }
    }

    fn degrade() -> AnyPin {
        AnyPin::new(Self::port_index(), Self::pin_index(), Self::gpio_ptr())
    }
}

pub mod pins {
    use super::{pac, AnyPin, PinId};

    macro_rules! define_pin {
        ($Name:ident, $port:literal, $pin:literal, $GpioBlk:ident) => {
            pub struct $Name;
            impl super::PinId for $Name {
                #[inline(always)]
                fn port_index() -> u8 {
                    $port
                }
                #[inline(always)]
                fn pin_index() -> u8 {
                    $pin
                }
                #[inline(always)]
                fn gpio_ptr() -> *const pac::gpio0::RegisterBlock {
                    pac::$GpioBlk::ptr()
                }
            }

            impl $Name {
                /// Convenience helper to obtain a type-erased handle to this pin.
                pub fn degrade() -> AnyPin {
                    <Self as PinId>::degrade()
                }

                pub fn set_mux_gpio() {
                    <Self as PinId>::set_mux_gpio()
                }
            }
        };
    }

    // Extend this list as more pins are needed.
    define_pin!(PIO3_18, 3, 18, Gpio3);
}

/// A flexible pin that can be configured as input or output.
pub struct Flex<'d> {
    pin: AnyPin,
    _marker: PhantomData<&'d mut ()>,
}

impl<'d> Flex<'d> {
    pub fn new(pin: AnyPin) -> Self {
        Self {
            pin,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn gpio(&self) -> &'static pac::gpio0::RegisterBlock {
        self.pin.gpio()
    }

    #[inline(always)]
    fn mask(&self) -> u32 {
        self.pin.mask()
    }

    pub fn set_as_input(&mut self) {
        let mask = self.mask();
        let gpio = self.gpio();
        gpio.pddr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
    }

    pub fn set_as_output(&mut self) {
        let mask = self.mask();
        let gpio = self.gpio();
        gpio.pddr()
            .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
    }

    pub fn set_high(&mut self) {
        self.gpio().psor().write(|w| unsafe { w.bits(self.mask()) });
    }

    pub fn set_low(&mut self) {
        self.gpio().pcor().write(|w| unsafe { w.bits(self.mask()) });
    }

    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::High => self.set_high(),
            Level::Low => self.set_low(),
        }
    }

    pub fn toggle(&mut self) {
        self.gpio().ptor().write(|w| unsafe { w.bits(self.mask()) });
    }

    pub fn is_high(&self) -> bool {
        (self.gpio().pdir().read().bits() & self.mask()) != 0
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
    }
}

/// GPIO output driver that owns a `Flex` pin.
pub struct Output<'d> {
    flex: Flex<'d>,
}

impl<'d> Output<'d> {
    pub fn new(pin: AnyPin, initial: Level) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_level(initial);
        flex.set_as_output();
        Self { flex }
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.flex.set_high();
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.flex.set_low();
    }

    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.flex.set_level(level);
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.flex.toggle();
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.flex.is_high()
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Expose the inner `Flex` if callers need to reconfigure the pin.
    pub fn into_flex(self) -> Flex<'d> {
        self.flex
    }
}

/// GPIO input driver that owns a `Flex` pin.
pub struct Input<'d> {
    flex: Flex<'d>,
}

impl<'d> Input<'d> {
    pub fn new(pin: AnyPin) -> Self {
        let mut flex = Flex::new(pin);
        flex.set_as_input();
        Self { flex }
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.flex.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.flex.is_low()
    }

    pub fn into_flex(self) -> Flex<'d> {
        self.flex
    }
}
