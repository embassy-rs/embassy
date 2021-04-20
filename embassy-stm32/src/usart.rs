use embassy::util::Unborrow;
use embassy_extras::unborrow;

use crate::pac::usart_v1::{regs, vals, Usart};
use crate::peripherals;

mod sealed {
    use super::*;
    pub trait Instance {
        fn regs(&self) -> Usart;
    }
}

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
        tx: impl Unborrow<Target = impl TxPin<T>>,
        rx: impl Unborrow<Target = impl RxPin<T>>,
        cts: impl Unborrow<Target = impl CtsPin<T>>,
        rts: impl Unborrow<Target = impl RtsPin<T>>,
        config: Config,
    ) -> Self {
        unborrow!(inner, tx, rx, cts, rts);
    }
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $addr:expr) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> Usart {
                Usart($addr as _)
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(USART1, 0x40011000);
impl_instance!(USART2, 0x40004400);
impl_instance!(USART3, 0x40004800);
impl_instance!(USART6, 0x40011400);
