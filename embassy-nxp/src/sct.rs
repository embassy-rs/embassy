#![macro_use]

use embassy_hal_internal::PeripheralType;
use nxp_pac::iocon::vals::PioFunc;

use crate::gpio;

/// SCT instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

pub(crate) trait SealedInstance {}

/// An SCT output.
#[allow(private_bounds)]
pub trait Output<T: Instance>: SealedOutput + PeripheralType {}

pub(crate) trait SealedOutput {
    /// Output number.
    fn number(&self) -> usize;
}

/// An SCT output capable pin.
pub trait OutputPin<T: Instance, O: Output<T>>: gpio::Pin {
    fn pin_func(&self) -> PioFunc;
}

macro_rules! impl_sct_instance {
    ($instance: ident) => {
        impl crate::sct::SealedInstance for crate::peripherals::$instance {}
        impl crate::sct::Instance for crate::peripherals::$instance {}
    };
}

macro_rules! impl_sct_output_instance {
    ($instance: ident, $name: ident, $num: expr) => {
        impl crate::sct::SealedOutput for crate::peripherals::$name {
            fn number(&self) -> usize {
                $num as usize
            }
        }
        impl crate::sct::Output<crate::peripherals::$instance> for crate::peripherals::$name {}
    };
}

macro_rules! impl_sct_output_pin {
    ($instance: ident, $output_instance: ident, $pin: ident, $alt: ident) => {
        impl crate::sct::OutputPin<crate::peripherals::$instance, crate::peripherals::$output_instance>
            for crate::peripherals::$pin
        {
            fn pin_func(&self) -> crate::pac::iocon::vals::PioFunc {
                crate::pac::iocon::vals::PioFunc::$alt
            }
        }
    };
}
