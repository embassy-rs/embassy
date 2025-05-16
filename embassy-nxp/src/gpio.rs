use embassy_hal_internal::{impl_peripheral, PeripheralType};

use crate::pac_utils::*;
use crate::{peripherals, Peri};

pub(crate) fn init() {
    // Enable clocks for GPIO, PINT, and IOCON
    syscon_reg()
        .ahbclkctrl0
        .modify(|_, w| w.gpio0().enable().gpio1().enable().mux().enable().iocon().enable());
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
    Bank0 = 0,
    Bank1 = 1,
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
        gpio_reg().set[self.pin.pin_bank() as usize].write(|w| unsafe { w.bits(self.pin.bit()) })
    }

    pub fn set_low(&mut self) {
        gpio_reg().clr[self.pin.pin_bank() as usize].write(|w| unsafe { w.bits(self.pin.bit()) })
    }

    pub fn toggle(&mut self) {
        gpio_reg().not[self.pin.pin_bank() as usize].write(|w| unsafe { w.bits(self.pin.bit()) })
    }

    /// Get the current output level of the pin. Note that the value returned by this function is
    /// the voltage level reported by the pin, not the value set by the output driver.
    pub fn level(&self) -> Level {
        let bits = gpio_reg().pin[self.pin.pin_bank() as usize].read().bits();
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
        match_iocon!(register, iocon_reg(), self.pin.pin_bank(), self.pin.pin_number(), {
            register.modify(|_, w| match pull {
                Pull::None => w.mode().inactive(),
                Pull::Up => w.mode().pull_up(),
                Pull::Down => w.mode().pull_down(),
            });
        });
    }

    /// Get the current input level of the pin.
    pub fn read(&self) -> Level {
        let bits = gpio_reg().pin[self.pin.pin_bank() as usize].read().bits();
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

    /// Set the pin to digital mode. This is required for using a pin as a GPIO pin. The default
    /// setting for pins is (usually) non-digital.
    fn set_as_digital(&mut self) {
        match_iocon!(register, iocon_reg(), self.pin_bank(), self.pin_number(), {
            register.modify(|_, w| w.digimode().digital());
        });
    }

    /// Set the pin in output mode. This implies setting the pin to digital mode, which this
    /// function handles itself.
    pub fn set_as_output(&mut self) {
        self.set_as_digital();
        gpio_reg().dirset[self.pin.pin_bank() as usize].write(|w| unsafe { w.dirsetp().bits(self.bit()) })
    }

    pub fn set_as_input(&mut self) {
        self.set_as_digital();
        gpio_reg().dirclr[self.pin.pin_bank() as usize].write(|w| unsafe { w.dirclrp().bits(self.bit()) })
    }
}

/// Sealed trait for pins. This trait is sealed and cannot be implemented outside of this crate.
pub(crate) trait SealedPin: Sized {
    fn pin_bank(&self) -> Bank;
    fn pin_number(&self) -> u8;
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
    pin_bank: Bank,
    pin_number: u8,
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
    ($name:ident, $bank:expr, $pin_num:expr) => {
        impl Pin for peripherals::$name {}
        impl SealedPin for peripherals::$name {
            #[inline]
            fn pin_bank(&self) -> Bank {
                $bank
            }

            #[inline]
            fn pin_number(&self) -> u8 {
                $pin_num
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                Self {
                    pin_bank: val.pin_bank(),
                    pin_number: val.pin_number(),
                }
            }
        }
    };
}

impl_pin!(PIO0_0, Bank::Bank0, 0);
impl_pin!(PIO0_1, Bank::Bank0, 1);
impl_pin!(PIO0_2, Bank::Bank0, 2);
impl_pin!(PIO0_3, Bank::Bank0, 3);
impl_pin!(PIO0_4, Bank::Bank0, 4);
impl_pin!(PIO0_5, Bank::Bank0, 5);
impl_pin!(PIO0_6, Bank::Bank0, 6);
impl_pin!(PIO0_7, Bank::Bank0, 7);
impl_pin!(PIO0_8, Bank::Bank0, 8);
impl_pin!(PIO0_9, Bank::Bank0, 9);
impl_pin!(PIO0_10, Bank::Bank0, 10);
impl_pin!(PIO0_11, Bank::Bank0, 11);
impl_pin!(PIO0_12, Bank::Bank0, 12);
impl_pin!(PIO0_13, Bank::Bank0, 13);
impl_pin!(PIO0_14, Bank::Bank0, 14);
impl_pin!(PIO0_15, Bank::Bank0, 15);
impl_pin!(PIO0_16, Bank::Bank0, 16);
impl_pin!(PIO0_17, Bank::Bank0, 17);
impl_pin!(PIO0_18, Bank::Bank0, 18);
impl_pin!(PIO0_19, Bank::Bank0, 19);
impl_pin!(PIO0_20, Bank::Bank0, 20);
impl_pin!(PIO0_21, Bank::Bank0, 21);
impl_pin!(PIO0_22, Bank::Bank0, 22);
impl_pin!(PIO0_23, Bank::Bank0, 23);
impl_pin!(PIO0_24, Bank::Bank0, 24);
impl_pin!(PIO0_25, Bank::Bank0, 25);
impl_pin!(PIO0_26, Bank::Bank0, 26);
impl_pin!(PIO0_27, Bank::Bank0, 27);
impl_pin!(PIO0_28, Bank::Bank0, 28);
impl_pin!(PIO0_29, Bank::Bank0, 29);
impl_pin!(PIO0_30, Bank::Bank0, 30);
impl_pin!(PIO0_31, Bank::Bank0, 31);
impl_pin!(PIO1_0, Bank::Bank1, 0);
impl_pin!(PIO1_1, Bank::Bank1, 1);
impl_pin!(PIO1_2, Bank::Bank1, 2);
impl_pin!(PIO1_3, Bank::Bank1, 3);
impl_pin!(PIO1_4, Bank::Bank1, 4);
impl_pin!(PIO1_5, Bank::Bank1, 5);
impl_pin!(PIO1_6, Bank::Bank1, 6);
impl_pin!(PIO1_7, Bank::Bank1, 7);
impl_pin!(PIO1_8, Bank::Bank1, 8);
impl_pin!(PIO1_9, Bank::Bank1, 9);
impl_pin!(PIO1_10, Bank::Bank1, 10);
impl_pin!(PIO1_11, Bank::Bank1, 11);
impl_pin!(PIO1_12, Bank::Bank1, 12);
impl_pin!(PIO1_13, Bank::Bank1, 13);
impl_pin!(PIO1_14, Bank::Bank1, 14);
impl_pin!(PIO1_15, Bank::Bank1, 15);
impl_pin!(PIO1_16, Bank::Bank1, 16);
impl_pin!(PIO1_17, Bank::Bank1, 17);
impl_pin!(PIO1_18, Bank::Bank1, 18);
impl_pin!(PIO1_19, Bank::Bank1, 19);
impl_pin!(PIO1_20, Bank::Bank1, 20);
impl_pin!(PIO1_21, Bank::Bank1, 21);
impl_pin!(PIO1_22, Bank::Bank1, 22);
impl_pin!(PIO1_23, Bank::Bank1, 23);
impl_pin!(PIO1_24, Bank::Bank1, 24);
impl_pin!(PIO1_25, Bank::Bank1, 25);
impl_pin!(PIO1_26, Bank::Bank1, 26);
impl_pin!(PIO1_27, Bank::Bank1, 27);
impl_pin!(PIO1_28, Bank::Bank1, 28);
impl_pin!(PIO1_29, Bank::Bank1, 29);
impl_pin!(PIO1_30, Bank::Bank1, 30);
impl_pin!(PIO1_31, Bank::Bank1, 31);
