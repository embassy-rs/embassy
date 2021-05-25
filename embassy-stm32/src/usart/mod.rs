#![macro_use]

#[cfg_attr(usart_v1, path = "v1.rs")]
#[cfg_attr(usart_v2, path = "v2.rs")]
mod _version;
pub use _version::*;

use crate::gpio::Pin;
use crate::pac::usart::Usart;

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> Usart;
    }
    pub trait RxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait TxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait CtsPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait RtsPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait CkPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
}
pub trait Instance: sealed::Instance {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> {}
pub trait TxPin<T: Instance>: sealed::TxPin<T> {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> {}
pub trait CkPin<T: Instance>: sealed::CkPin<T> {}

macro_rules! impl_usart {
    ($inst:ident) => {
        impl crate::usart::sealed::Instance for peripherals::$inst {
            fn regs(&self) -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }
        impl crate::usart::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_usart_pin {
    ($inst:ident, $func:ident, $pin:ident, $af:expr) => {
        impl crate::usart::sealed::$func<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
        impl crate::usart::$func<peripherals::$inst> for peripherals::$pin {}
    };
}
