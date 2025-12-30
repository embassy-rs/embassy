#![macro_use]

use embassy_hal_internal::{PeripheralType, impl_peripheral};

use crate::Peri;
use crate::pac::common::{RW, Reg};
use crate::pac::iocon::vals::{PioDigimode, PioMode};
use crate::pac::{GPIO, IOCON, SYSCON, iocon};

pub(crate) fn init() {
    // Enable clocks for GPIO, PINT, and IOCON
    SYSCON.ahbclkctrl0().modify(|w| {
        w.set_gpio0(true);
        w.set_gpio1(true);
        w.set_mux(true);
        w.set_iocon(true);
    });
    info!("GPIO initialized");
}

/// The GPIO pin level for pins set on "Digital" mode.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Level {
    /// Logical low. Corresponds to 0V.
    Low,
    /// Logical high. Corresponds to VDD.
    High,
}

/// Pull setting for a GPIO input set on "Digital" mode.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    /// No pull.
    None,
    /// Internal pull-up resistor.
    Up,
    /// Internal pull-down resistor.
    Down,
}

/// The LPC55 boards have two GPIO banks, each with 32 pins. This enum represents the two banks.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Bank {
    Gpio0 = 0,
    Gpio1 = 1,
}

/// GPIO output driver. Internally, this is a specialized [Flex] pin.
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [initial output](Level).
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_output();
        let mut result = Self { pin };

        match initial_output {
            Level::High => result.set_high(),
            Level::Low => result.set_low(),
        };

        result
    }

    pub fn set_high(&mut self) {
        GPIO.set(self.pin.pin_bank() as usize)
            .write(|w| w.set_setp(self.pin.bit()));
    }

    pub fn set_low(&mut self) {
        GPIO.clr(self.pin.pin_bank() as usize)
            .write(|w| w.set_clrp(self.pin.bit()));
    }

    pub fn toggle(&mut self) {
        GPIO.not(self.pin.pin_bank() as usize)
            .write(|w| w.set_notp(self.pin.bit()));
    }

    /// Get the current output level of the pin. Note that the value returned by this function is
    /// the voltage level reported by the pin, not the value set by the output driver.
    pub fn level(&self) -> Level {
        let bits = GPIO.pin(self.pin.pin_bank() as usize).read().port();
        if bits & self.pin.bit() != 0 {
            Level::High
        } else {
            Level::Low
        }
    }
}

/// GPIO input driver. Internally, this is a specialized [Flex] pin.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Pull].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        let mut result = Self { pin };
        result.set_pull(pull);

        result
    }

    /// Set the pull configuration for the pin. To disable the pull, use [Pull::None].
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.set_pull(pull);
    }

    /// Get the current input level of the pin.
    pub fn read(&self) -> Level {
        let bits = GPIO.pin(self.pin.pin_bank() as usize).read().port();
        if bits & self.pin.bit() != 0 {
            Level::High
        } else {
            Level::Low
        }
    }
}

/// A flexible GPIO (digital mode) pin whose mode is not yet determined. Under the hood, this is a
/// reference to a type-erased pin called ["AnyPin"](AnyPin).
pub struct Flex<'d> {
    pub(crate) pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// Note: you cannot assume that the pin will be in Digital mode after this call.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self { pin: pin.into() }
    }

    /// Get the bank of this pin. See also [Bank].
    ///
    /// # Example
    ///
    /// ```
    /// use embassy_nxp::gpio::{Bank, Flex};
    ///
    /// let p = embassy_nxp::init(Default::default());
    /// let pin = Flex::new(p.PIO1_15);
    ///
    /// assert_eq!(pin.pin_bank(), Bank::Bank1);
    /// ```
    pub fn pin_bank(&self) -> Bank {
        self.pin.pin_bank()
    }

    /// Get the number of this pin within its bank. See also [Bank].
    ///
    /// # Example
    ///
    /// ```
    /// use embassy_nxp::gpio::Flex;
    ///
    /// let p = embassy_nxp::init(Default::default());
    /// let pin = Flex::new(p.PIO1_15);
    ///
    /// assert_eq!(pin.pin_number(), 15 as u8);
    /// ```
    pub fn pin_number(&self) -> u8 {
        self.pin.pin_number()
    }

    /// Get the bit mask for this pin. Useful for setting or clearing bits in a register. Note:
    /// PIOx_0 is bit 0, PIOx_1 is bit 1, etc.
    ///
    /// # Example
    ///
    /// ```
    /// use embassy_nxp::gpio::Flex;
    ///
    /// let p = embassy_nxp::init(Default::default());
    /// let pin = Flex::new(p.PIO1_3);
    ///
    /// assert_eq!(pin.bit(), 0b0000_1000);
    /// ```
    pub fn bit(&self) -> u32 {
        1 << self.pin.pin_number()
    }

    /// Set the pull configuration for the pin. To disable the pull, use [Pull::None].
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.pio().modify(|w| match pull {
            Pull::None => w.set_mode(PioMode::INACTIVE),
            Pull::Up => w.set_mode(PioMode::PULL_UP),
            Pull::Down => w.set_mode(PioMode::PULL_DOWN),
        });
    }

    /// Set the pin to digital mode. This is required for using a pin as a GPIO pin. The default
    /// setting for pins is (usually) non-digital.
    fn set_as_digital(&mut self) {
        self.pin.pio().modify(|w| {
            w.set_digimode(PioDigimode::DIGITAL);
        });
    }

    /// Set the pin in output mode. This implies setting the pin to digital mode, which this
    /// function handles itself.
    pub fn set_as_output(&mut self) {
        self.set_as_digital();
        GPIO.dirset(self.pin.pin_bank() as usize)
            .write(|w| w.set_dirsetp(self.bit()))
    }

    pub fn set_as_input(&mut self) {
        self.set_as_digital();
        GPIO.dirclr(self.pin.pin_bank() as usize)
            .write(|w| w.set_dirclrp(self.bit()))
    }
}

/// Sealed trait for pins. This trait is sealed and cannot be implemented outside of this crate.
pub(crate) trait SealedPin: Sized {
    fn pin_bank(&self) -> Bank;
    fn pin_number(&self) -> u8;

    #[inline]
    fn pio(&self) -> Reg<iocon::regs::Pio, RW> {
        match self.pin_bank() {
            Bank::Gpio0 => IOCON.pio0(self.pin_number() as usize),
            Bank::Gpio1 => IOCON.pio1(self.pin_number() as usize),
        }
    }
}

/// Interface for a Pin that can be configured by an [Input] or [Output] driver, or converted to an
/// [AnyPin]. By default, this trait is sealed and cannot be implemented outside of the
/// `embassy-nxp` crate due to the [SealedPin] trait.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Returns the pin number within a bank
    #[inline]
    fn pin(&self) -> u8 {
        self.pin_number()
    }

    /// Returns the bank of this pin
    #[inline]
    fn bank(&self) -> Bank {
        self.pin_bank()
    }
}

/// Type-erased GPIO pin.
pub struct AnyPin {
    pub(crate) pin_bank: Bank,
    pub(crate) pin_number: u8,
}

impl AnyPin {
    /// Unsafely create a new type-erased pin.
    ///
    /// # Safety
    ///
    /// You must ensure that you’re only using one instance of this type at a time.
    pub unsafe fn steal(pin_bank: Bank, pin_number: u8) -> Peri<'static, Self> {
        Peri::new_unchecked(Self { pin_bank, pin_number })
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn pin_bank(&self) -> Bank {
        self.pin_bank
    }

    #[inline]
    fn pin_number(&self) -> u8 {
        self.pin_number
    }
}

macro_rules! impl_pin {
    ($name:ident, $bank:ident, $pin_num:expr) => {
        impl crate::gpio::Pin for peripherals::$name {}
        impl crate::gpio::SealedPin for peripherals::$name {
            #[inline]
            fn pin_bank(&self) -> crate::gpio::Bank {
                crate::gpio::Bank::$bank
            }

            #[inline]
            fn pin_number(&self) -> u8 {
                $pin_num
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                use crate::gpio::SealedPin;

                Self {
                    pin_bank: val.pin_bank(),
                    pin_number: val.pin_number(),
                }
            }
        }
    };
}
