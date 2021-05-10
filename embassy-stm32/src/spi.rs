#![macro_use]

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use core::marker::PhantomData;
use embassy::interrupt::Interrupt;
use embedded_hal::blocking::spi::{Write, Transfer};
//use crate::pac::spi;

pub struct Spi<'d, T: Instance> {
    peri: T,
    //irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

pub enum Error {
    Framing,
    Crc,
    Overrun,
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u8> for Spi<'d, T> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let regs = T::regs();

        for word in words.iter() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                regs.dr().write(|reg| reg.0 = *word as u32);
            }
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.fre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crcerr() {
                    return Err(Error::Crc);
                }
                if !sr.txe() {
                    // loop waiting for TXE
                }
            }
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u8> for Spi<'d, T> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let regs = T::regs();

        for word in words.iter_mut() {
            while unsafe { !regs.sr().read().txe() } {
                // spin
            }
            unsafe {
                regs.dr().write(|reg| reg.0 = *word as u32);
            }
            while unsafe { !regs.sr().read().rxne() } {
                // spin waiting for inbound to shift in.
            }
            *word = unsafe { regs.dr().read().0 as u8 };
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.fre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crcerr() {
                    return Err(Error::Crc);
                }
                if !sr.txe() {
                    // loop waiting for TXE
                }
            }
        }

        Ok(words)
    }
}


pub(crate) mod sealed {
    use super::*;
    use embassy::util::AtomicWaker;

    //pub struct State {
    //pub end_waker: AtomicWaker,
    //}

    //impl State {
    //pub const fn new() -> Self {
    //Self {
    //end_waker: AtomicWaker::new(),
    //}
    //}
    //}

    pub trait Instance {
        fn regs() -> &'static crate::pac::spi::Spi;
        //fn state() -> &'static State;
    }
}

pub trait Instance: sealed::Instance + 'static {
    //type Interrupt: Interrupt;
}

macro_rules! impl_spi {
    ($inst:ident) => {
        impl crate::spi::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::spi::Spi {
                &crate::pac::$inst
            }
        }

        impl crate::spi::Instance for peripherals::$inst {}
    };
}