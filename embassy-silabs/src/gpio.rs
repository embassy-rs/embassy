//! General-purpose Input/Output (GPIO).

#![macro_use]

use core::convert::Infallible;

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};

use crate::pac::GPIO;
// GPIO routes to a different register-block version per Series 2 config:
// MG24 (config 4) uses gpio_v3, FG25 (config 5) uses gpio_v4, MG26 (config 6)
// uses gpio_v7. The port register newtypes (PortDout / PortModel / PortModeh)
// have identical shapes across all three; alias the module so the code below
// is version-agnostic.
#[cfg(silabs_series_2_config = "4")]
use crate::pac::gpio_v3 as gpio_mod;
#[cfg(silabs_series_2_config = "5")]
use crate::pac::gpio_v4 as gpio_mod;
#[cfg(silabs_series_2_config = "6")]
use crate::pac::gpio_v7 as gpio_mod;

/// Logic level.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low
    Low,
    /// High
    High,
}

impl From<bool> for Level {
    fn from(b: bool) -> Self {
        if b { Level::High } else { Level::Low }
    }
}
impl From<Level> for bool {
    fn from(l: Level) -> Self {
        matches!(l, Level::High)
    }
}

/// Pull setting for an input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull
    None,
    /// Pull up
    Up,
    /// Pull down
    Down,
}

/// Output drive characteristics.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    /// Low
    Low,
    /// Medium
    Medium,
    /// High
    High,
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum Mode {
    Disabled = 0x0,
    Input = 0x1,
    InputPull = 0x2,
    PushPull = 0x4,
}

/// Type-erased GPIO pin reference.
pub struct AnyPin {
    pin_port: u8,
}

impl_peripheral!(AnyPin);

impl AnyPin {
    /// # Safety
    ///
    /// The caller must ensure that no other `AnyPin` referencing the same
    /// pin number is in scope simultaneously.
    #[inline]
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_port })
    }

    #[inline]
    pub(crate) fn pin_port(&self) -> u8 {
        self.pin_port
    }
}

/// GPIO output driver.
pub struct Output<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level] and [Speed] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial: Level, _speed: Speed) -> Self {
        let pin = pin.into();
        match initial {
            Level::High => write_dout_bit(&pin, true),
            Level::Low => write_dout_bit(&pin, false),
        }
        write_mode(&pin, Mode::PushPull);
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        write_dout_bit(&self.pin, true);
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        write_dout_bit(&self.pin, false);
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        write_dout_bit(&self.pin, level.into());
    }

    /// Toggle the output.
    #[inline]
    pub fn toggle(&mut self) {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = (self.pin.pin_port() & 0xF) as u32;
        critical_section::with(|_| {
            let cur = GPIO.p_dout(port).read().0;
            let new = gpio_mod::regs::PortDout(cur ^ (1 << n));
            GPIO.p_dout(port).write_value(new);
        });
    }

    /// Get the current output level.
    #[inline]
    pub fn get_output_level(&self) -> Level {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = (self.pin.pin_port() & 0xF) as u32;
        if GPIO.p_dout(port).read().0 & (1 << n) != 0 {
            Level::High
        } else {
            Level::Low
        }
    }
}

impl<'d> Drop for Output<'d> {
    fn drop(&mut self) {
        write_mode(&self.pin, Mode::Disabled);
    }
}

/// GPIO input driver.
pub struct Input<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Input<'d> {
    /// Create GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let pin = pin.into();
        match pull {
            Pull::None => write_mode(&pin, Mode::Input),
            Pull::Up => {
                write_dout_bit(&pin, true);
                write_mode(&pin, Mode::InputPull);
            }
            Pull::Down => {
                write_dout_bit(&pin, false);
                write_mode(&pin, Mode::InputPull);
            }
        }
        Self { pin }
    }

    /// Get the current pin input level.
    #[inline]
    pub fn get_level(&self) -> Level {
        if self.is_high() { Level::High } else { Level::Low }
    }

    /// Is the input pin high?
    #[inline]
    pub fn is_high(&self) -> bool {
        let port = (self.pin.pin_port() >> 4) as usize;
        let n = (self.pin.pin_port() & 0xF) as u32;
        GPIO.p_din(port).read().0 & (1 << n) != 0
    }

    /// Is the input pin low?
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }
}

impl<'d> Drop for Input<'d> {
    fn drop(&mut self) {
        write_mode(&self.pin, Mode::Disabled);
    }
}

mod sealed {
    pub trait SealedPin {
        fn pin_port(&self) -> u8;
    }
}

/// GPIO pin trait.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + sealed::SealedPin {}

impl sealed::SealedPin for AnyPin {
    #[inline]
    fn pin_port(&self) -> u8 {
        self.pin_port
    }
}
impl Pin for AnyPin {}

macro_rules! impl_pin {
    ($name:ident, $pin_port:expr) => {
        impl sealed::SealedPin for crate::peripherals::$name {
            #[inline]
            fn pin_port(&self) -> u8 {
                $pin_port
            }
        }
        impl Pin for crate::peripherals::$name {}
        impl From<crate::peripherals::$name> for AnyPin {
            #[inline]
            fn from(_: crate::peripherals::$name) -> Self {
                Self { pin_port: $pin_port }
            }
        }
    };
}

// Port A
impl_pin!(PA00, 0);
impl_pin!(PA01, 1);
impl_pin!(PA02, 2);
impl_pin!(PA03, 3);
impl_pin!(PA04, 4);
impl_pin!(PA05, 5);
impl_pin!(PA06, 6);
impl_pin!(PA07, 7);
impl_pin!(PA08, 8);
impl_pin!(PA09, 9);
impl_pin!(PA10, 10);
impl_pin!(PA11, 11);
impl_pin!(PA12, 12);
impl_pin!(PA13, 13);
impl_pin!(PA14, 14);
impl_pin!(PA15, 15);
// Port B
impl_pin!(PB00, 16);
impl_pin!(PB01, 17);
impl_pin!(PB02, 18);
impl_pin!(PB03, 19);
impl_pin!(PB04, 20);
impl_pin!(PB05, 21);
impl_pin!(PB06, 22);
impl_pin!(PB07, 23);
impl_pin!(PB08, 24);
impl_pin!(PB09, 25);
impl_pin!(PB10, 26);
impl_pin!(PB11, 27);
impl_pin!(PB12, 28);
impl_pin!(PB13, 29);
impl_pin!(PB14, 30);
impl_pin!(PB15, 31);
// Port C
impl_pin!(PC00, 32);
impl_pin!(PC01, 33);
impl_pin!(PC02, 34);
impl_pin!(PC03, 35);
impl_pin!(PC04, 36);
impl_pin!(PC05, 37);
impl_pin!(PC06, 38);
impl_pin!(PC07, 39);
impl_pin!(PC08, 40);
impl_pin!(PC09, 41);
impl_pin!(PC10, 42);
impl_pin!(PC11, 43);
impl_pin!(PC12, 44);
impl_pin!(PC13, 45);
impl_pin!(PC14, 46);
impl_pin!(PC15, 47);
// Port D
impl_pin!(PD00, 48);
impl_pin!(PD01, 49);
impl_pin!(PD02, 50);
impl_pin!(PD03, 51);
impl_pin!(PD04, 52);
impl_pin!(PD05, 53);
impl_pin!(PD06, 54);
impl_pin!(PD07, 55);
impl_pin!(PD08, 56);
impl_pin!(PD09, 57);
impl_pin!(PD10, 58);
impl_pin!(PD11, 59);
impl_pin!(PD12, 60);
impl_pin!(PD13, 61);
impl_pin!(PD14, 62);
impl_pin!(PD15, 63);

#[inline]
fn write_mode(pin: &AnyPin, mode: Mode) {
    let port = (pin.pin_port() >> 4) as usize;
    let pin_in_port = (pin.pin_port() & 0xF) as u32;
    let mode_val = mode as u32;

    critical_section::with(|_| {
        if pin_in_port < 8 {
            let shift = pin_in_port * 4;
            let cur = GPIO.p_model(port).read().0;
            let cleared = cur & !(0xF << shift);
            let new = gpio_mod::regs::PortModel(cleared | (mode_val << shift));
            GPIO.p_model(port).write_value(new);
        } else {
            let shift = (pin_in_port - 8) * 4;
            let cur = GPIO.p_modeh(port).read().0;
            let cleared = cur & !(0xF << shift);
            let new = gpio_mod::regs::PortModeh(cleared | (mode_val << shift));
            GPIO.p_modeh(port).write_value(new);
        }
    });
}

#[inline]
fn write_dout_bit(pin: &AnyPin, high: bool) {
    let port = (pin.pin_port() >> 4) as usize;
    let n = (pin.pin_port() & 0xF) as u32;
    critical_section::with(|_| {
        let cur = GPIO.p_dout(port).read().0;
        let new_raw = if high { cur | (1 << n) } else { cur & !(1 << n) };
        let new = gpio_mod::regs::PortDout(new_raw);
        GPIO.p_dout(port).write_value(new);
    });
}

/// Configure `pin` as a push-pull output driven by a peripheral (e.g. EUSART TX).
/// Sets the line idle-high before switching to push-pull so a UART idle
/// level is presented immediately.
#[inline]
pub(crate) fn set_as_alternate_output(pin: &AnyPin) {
    write_dout_bit(pin, true);
    write_mode(pin, Mode::PushPull);
}

/// Configure `pin` as a floating input consumed by a peripheral (e.g. EUSART RX).
#[inline]
pub(crate) fn set_as_alternate_input(pin: &AnyPin) {
    write_mode(pin, Mode::Input);
}

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}
impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Output::set_low(self);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Output::set_high(self);
        Ok(())
    }
}
impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(matches!(Output::get_output_level(self), Level::High))
    }
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(matches!(Output::get_output_level(self), Level::Low))
    }
}
impl<'d> embedded_hal::digital::ErrorType for Input<'d> {
    type Error = Infallible;
}
impl<'d> embedded_hal::digital::InputPin for Input<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_high(self))
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Input::is_low(self))
    }
}
