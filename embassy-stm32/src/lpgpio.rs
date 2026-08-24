//! LPGPIO — low-power GPIO port (Stop 2/3 capable on supported U5 lines).

use embassy_hal_internal::PeripheralType;

use crate::pac::lpgpio::Lpgpio as Regs;
use crate::{Peri, rcc};

/// Number of pins on LPGPIO1.
pub const PIN_COUNT: u8 = 16;

/// Pin mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    /// Input.
    Input,
    /// Push-pull output.
    Output,
}

/// LPGPIO driver.
pub struct Lpgpio<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Lpgpio<'d, T> {
    /// Create a new LPGPIO instance.
    pub fn new(peri: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        Self { _peri: peri }
    }

    /// Set pin mode (`false` = input, `true` = output in hardware).
    pub fn set_mode(&mut self, pin: u8, mode: Mode) {
        assert!(pin < PIN_COUNT);
        let mask = 1u32 << pin;
        T::regs().moder().modify(|w| match mode {
            Mode::Input => w.0 &= !mask,
            Mode::Output => w.0 |= mask,
        });
    }

    /// Drive an output pin high or low. Pin must be configured as output.
    pub fn set_level(&mut self, pin: u8, high: bool) {
        assert!(pin < PIN_COUNT);
        if high {
            T::regs().bsrr().write(|w| w.0 = 1 << pin);
        } else {
            T::regs().brr().write(|w| w.0 = 1 << pin);
        }
    }

    /// Read the input level of a pin.
    pub fn get_level(&self, pin: u8) -> bool {
        assert!(pin < PIN_COUNT);
        (T::regs().idr().read().0 & (1 << pin)) != 0
    }

    /// Toggle an output pin.
    pub fn toggle(&mut self, pin: u8) {
        self.set_level(pin, !self.get_level(pin));
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// LPGPIO instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

foreach_peripheral!(
    (lpgpio, $inst:ident) => {
        impl crate::lpgpio::SealedInstance for crate::peripherals::$inst {
            fn regs() -> Regs {
                crate::pac::$inst
            }
        }
        impl crate::lpgpio::Instance for crate::peripherals::$inst {}
    };
);
