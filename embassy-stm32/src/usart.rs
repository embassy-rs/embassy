use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;

use crate::gpio::{NoPin, Pin};
use crate::pac::usart_v1::{regs, vals, Usart};
use crate::peripherals;

#[non_exhaustive]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: 8,
            stop_bits: 1,
        }
    }
}

pub struct Uart<'d, T: Instance> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Uart<'d, T> {
    pub fn new(
        inner: impl Unborrow<Target = T>,
        rx: impl Unborrow<Target = impl RxPin<T>>,
        tx: impl Unborrow<Target = impl TxPin<T>>,
        cts: impl Unborrow<Target = impl CtsPin<T>>,
        rts: impl Unborrow<Target = impl RtsPin<T>>,
        config: Config,
    ) -> Self {
        unborrow!(inner, rx, tx, cts, rts);

        Self {
            inner,
            phantom: PhantomData,
        }
    }
}

pub(crate) mod sealed {
    use crate::gpio::{OptionalPin, Pin};

    use super::*;
    pub trait Instance {
        fn regs(&self) -> Usart;
    }
    pub trait RxPin<T: Instance>: OptionalPin {
        const AF_NUM: u8;
    }
    pub trait TxPin<T: Instance>: OptionalPin {
        const AF_NUM: u8;
    }
    pub trait CtsPin<T: Instance>: OptionalPin {
        const AF_NUM: u8;
    }
    pub trait RtsPin<T: Instance>: OptionalPin {
        const AF_NUM: u8;
    }
    pub trait CkPin<T: Instance>: OptionalPin {
        const AF_NUM: u8;
    }
}
pub trait Instance: sealed::Instance {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> {}
pub trait TxPin<T: Instance>: sealed::TxPin<T> {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> {}
pub trait CkPin<T: Instance>: sealed::CkPin<T> {}

impl<T: Instance> sealed::RxPin<T> for NoPin {
    const AF_NUM: u8 = 0;
}
impl<T: Instance> RxPin<T> for NoPin {}
impl<T: Instance> sealed::TxPin<T> for NoPin {
    const AF_NUM: u8 = 0;
}
impl<T: Instance> TxPin<T> for NoPin {}
impl<T: Instance> sealed::CtsPin<T> for NoPin {
    const AF_NUM: u8 = 0;
}
impl<T: Instance> CtsPin<T> for NoPin {}
impl<T: Instance> sealed::RtsPin<T> for NoPin {
    const AF_NUM: u8 = 0;
}
impl<T: Instance> RtsPin<T> for NoPin {}
impl<T: Instance> sealed::CkPin<T> for NoPin {
    const AF_NUM: u8 = 0;
}
impl<T: Instance> CkPin<T> for NoPin {}

macro_rules! impl_usart {
    ($inst:ident, $addr:expr) => {
        impl crate::usart::sealed::Instance for peripherals::$inst {
            fn regs(&self) -> crate::pac::usart_v1::Usart {
                crate::pac::usart_v1::Usart($addr as _)
            }
        }
        impl crate::usart::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_usart_pin {
    ($inst:ident, $func:ident, $pin:ident, $num:expr) => {
        impl crate::usart::sealed::$func<peripherals::$inst> for peripherals::$pin {
            const AF_NUM: u8 = $num;
        }
        impl crate::usart::$func<peripherals::$inst> for peripherals::$pin {}
    };
}
