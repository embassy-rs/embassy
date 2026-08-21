//! General-purpose input/output.

use core::convert::Infallible;
use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};

use embedded_hal::digital::{ErrorType, InputPin, OutputPin, PinState, StatefulOutputPin};

const PINB: *mut u8 = 0x23 as *mut u8;
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;
const PINC: *mut u8 = 0x26 as *mut u8;
const DDRC: *mut u8 = 0x27 as *mut u8;
const PORTC: *mut u8 = 0x28 as *mut u8;
const PIND: *mut u8 = 0x29 as *mut u8;
const DDRD: *mut u8 = 0x2a as *mut u8;
const PORTD: *mut u8 = 0x2b as *mut u8;

/// Logical GPIO level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

impl From<Level> for PinState {
    fn from(level: Level) -> Self {
        match level {
            Level::Low => PinState::Low,
            Level::High => PinState::High,
        }
    }
}

/// Input pull resistor configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pull {
    /// High-impedance input without a pull resistor.
    Floating,
    /// Enable the ATmega328P's internal pull-up resistor.
    Up,
}

pub(crate) mod sealed {
    pub trait Pin {
        const PIN: *mut u8;
        const DDR: *mut u8;
        const PORT: *mut u8;
        const MASK: u8;
    }
}

/// A GPIO pin owned by the caller.
pub trait Pin: sealed::Pin {}

macro_rules! pin {
    ($name:ident, $pin:expr, $ddr:expr, $port:expr, $bit:expr) => {
        #[doc = concat!("GPIO pin `", stringify!($name), "`.")]
        pub struct $name {
            _private: (),
        }

        impl $name {
            pub(crate) const fn new() -> Self {
                Self { _private: () }
            }
        }

        impl sealed::Pin for $name {
            const PIN: *mut u8 = $pin;
            const DDR: *mut u8 = $ddr;
            const PORT: *mut u8 = $port;
            const MASK: u8 = 1 << $bit;
        }

        impl Pin for $name {}
    };
}

pin!(PB0, PINB, DDRB, PORTB, 0);
pin!(PB1, PINB, DDRB, PORTB, 1);
pin!(PB2, PINB, DDRB, PORTB, 2);
pin!(PB3, PINB, DDRB, PORTB, 3);
pin!(PB4, PINB, DDRB, PORTB, 4);
pin!(PB5, PINB, DDRB, PORTB, 5);
pin!(PB6, PINB, DDRB, PORTB, 6);
pin!(PB7, PINB, DDRB, PORTB, 7);
pin!(PC0, PINC, DDRC, PORTC, 0);
pin!(PC1, PINC, DDRC, PORTC, 1);
pin!(PC2, PINC, DDRC, PORTC, 2);
pin!(PC3, PINC, DDRC, PORTC, 3);
pin!(PC4, PINC, DDRC, PORTC, 4);
pin!(PC5, PINC, DDRC, PORTC, 5);
pin!(PC6, PINC, DDRC, PORTC, 6);
pin!(PD0, PIND, DDRD, PORTD, 0);
pin!(PD1, PIND, DDRD, PORTD, 1);
pin!(PD2, PIND, DDRD, PORTD, 2);
pin!(PD3, PIND, DDRD, PORTD, 3);
pin!(PD4, PIND, DDRD, PORTD, 4);
pin!(PD5, PIND, DDRD, PORTD, 5);
pin!(PD6, PIND, DDRD, PORTD, 6);
pin!(PD7, PIND, DDRD, PORTD, 7);

/// A GPIO configured as a digital input.
pub struct Input<P: Pin> {
    pin: P,
}

impl<P: Pin> Input<P> {
    /// Configures an owned pin as an input.
    pub fn new(pin: P, pull: Pull) -> Self {
        avr_device::interrupt::free(|_| unsafe {
            modify(P::DDR, P::MASK, false);
            modify(P::PORT, P::MASK, pull == Pull::Up);
        });
        Self { pin }
    }

    /// Returns the owned pin without changing its hardware configuration.
    pub fn free(self) -> P {
        self.pin
    }

    /// Returns `true` when the electrical input level is high.
    pub fn is_high(&self) -> bool {
        unsafe { read_volatile(P::PIN) & P::MASK != 0 }
    }

    /// Returns `true` when the electrical input level is low.
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }
}

impl<P: Pin> ErrorType for Input<P> {
    type Error = Infallible;
}

impl<P: Pin> InputPin for Input<P> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_low(self))
    }
}

/// A GPIO configured as a push-pull digital output.
pub struct Output<P: Pin> {
    pin: P,
    _not_send: PhantomData<*mut ()>,
}

impl<P: Pin> Output<P> {
    /// Configures an owned pin as an output with the requested initial level.
    pub fn new(pin: P, initial: Level) -> Self {
        avr_device::interrupt::free(|_| unsafe {
            // Set the output latch before enabling the driver to avoid a glitch.
            modify(P::PORT, P::MASK, initial == Level::High);
            modify(P::DDR, P::MASK, true);
        });
        Self {
            pin,
            _not_send: PhantomData,
        }
    }

    /// Returns the owned pin without changing its hardware configuration.
    pub fn free(self) -> P {
        self.pin
    }

    /// Drives the output low.
    pub fn set_low(&mut self) {
        set_output_latch::<P>(false);
    }

    /// Drives the output high.
    pub fn set_high(&mut self) {
        set_output_latch::<P>(true);
    }

    /// Returns whether the output latch is set high.
    pub fn is_set_high(&self) -> bool {
        unsafe { read_volatile(P::PORT) & P::MASK != 0 }
    }

    /// Returns whether the output latch is set low.
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// Toggles the output latch.
    pub fn toggle(&mut self) {
        // On classic AVR, writing a one to PINx atomically toggles PORTx.
        unsafe { write_volatile(P::PIN, P::MASK) }
    }
}

impl<P: Pin> ErrorType for Output<P> {
    type Error = Infallible;
}

impl<P: Pin> OutputPin for Output<P> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Output::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Output::set_high(self);
        Ok(())
    }
}

impl<P: Pin> StatefulOutputPin for Output<P> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Output::is_set_low(self))
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Output::toggle(self);
        Ok(())
    }
}

fn set_output_latch<P: Pin>(high: bool) {
    avr_device::interrupt::free(|_| unsafe { modify(P::PORT, P::MASK, high) });
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
