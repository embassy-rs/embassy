#![macro_use]

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use core::marker::PhantomData;
use embassy::interrupt::Interrupt;
use embedded_hal::blocking::spi::{Write, Transfer};
use embassy::util::Unborrow;
use embassy_extras::{impl_unborrow, unborrow};
use crate::gpio::{Pin, AnyPin};
use crate::pac::gpio::vals::Afr;
use crate::pac::gpio::Gpio;
//use crate::pac::spi;

pub struct Spi<'d, T: Instance> {
    peri: T,
    sck: AnyPin,
    mosi: AnyPin,
    miso: AnyPin,
    //irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Spi<'d, T> {
    pub fn new(peri: impl Unborrow<Target=T> + 'd,
               sck: impl Unborrow<Target=impl Sck<T>>,
               mosi: impl Unborrow<Target=impl Mosi<T>>,
               miso: impl Unborrow<Target=impl Miso<T>>,
    ) -> Self {
        unborrow!(peri);
        unborrow!(sck, mosi, miso);

        unsafe {
            Self::configure_pin( sck.block(), sck.pin() as usize, sck.af() );
            Self::configure_pin( mosi.block(), mosi.pin() as usize, mosi.af() );
            Self::configure_pin( miso.block(), miso.pin() as usize, miso.af() );
        }

        let sck = sck.degrade();
        let mosi = mosi.degrade();
        let miso = miso.degrade();

        Self {
            peri,
            sck,
            mosi,
            miso,
            phantom: PhantomData,
        }
    }

    unsafe fn configure_pin(block: Gpio, pin: usize, af_num: u8) {
        let (afr, n_af) = if pin < 8 { (0, pin) } else { (1, pin - 8) };
        block.afr(afr).modify(|w| w.set_afr(n_af, Afr(af_num)));
    }
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

    pub trait Sck<T: Instance> : Pin {
        const AF: u8;
        fn af(&self) -> u8 {
            Self::AF
        }
    }

    pub trait Mosi<T: Instance> : Pin {
        const AF: u8;
        fn af(&self) -> u8 {
            Self::AF
        }
    }

    pub trait Miso<T: Instance> : Pin {
        const AF: u8;
        fn af(&self) -> u8 {
            Self::AF
        }
    }
}

pub trait Instance: sealed::Instance + 'static {
    //type Interrupt: Interrupt;
}

pub trait Sck<T:Instance>: sealed::Sck<T> + 'static {

}

pub trait Mosi<T:Instance>: sealed::Mosi<T> + 'static {

}

pub trait Miso<T:Instance>: sealed::Miso<T> + 'static {

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

macro_rules! impl_spi_pin {
    ($inst:ident, $pin_func:ident, $pin:ident, $af:expr) => {
        impl crate::spi::$pin_func<peripherals::$inst> for peripherals::$pin {
        }

        impl crate::spi::sealed::$pin_func<peripherals::$inst> for peripherals::$pin {
            const AF: u8 = $af;
        }
    }
}