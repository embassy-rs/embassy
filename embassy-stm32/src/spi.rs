pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use core::marker::PhantomData;
use embassy::interrupt::Interrupt;
use embedded_hal::blocking::spi::Write;
use crate::pac::spi

pub struct Spi<'d, T: Instance> {
    peri: T,
    irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

pub enum Error {

}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u8> for Spim<'d, T> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let mut recv: &mut [u8] = &mut [];

    }
}

mod sealed {
    use super::*;
    use embassy::util::AtomicWaker;

    pub struct State {
        pub end_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static crate::pac::spi::Spi;
        fn state() -> &'static State;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_spi {
    ($inst:ident) => {
        impl crate::spi::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::spi::Spi {
                crate::pac::$inst
            }
        }

        impl crate::spi::Instance for peripherals::$inst {}
    };
}