use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::{pac, peripherals};

pub use embedded_hal_02::spi::{Phase, Polarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

#[non_exhaustive]
pub struct Config {
    pub frequency: u32,
    pub phase: Phase,
    pub polarity: Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
        }
    }
}

pub struct Spi<'d, T: Instance> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
}

fn div_roundup(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

fn calc_prescs(freq: u32) -> (u8, u8) {
    let clk_peri = crate::clocks::clk_peri_freq();

    // final SPI frequency: spi_freq = clk_peri / presc / postdiv
    // presc must be in 2..=254, and must be even
    // postdiv must be in 1..=256

    // divide extra by 2, so we get rid of the "presc must be even" requirement
    let ratio = div_roundup(clk_peri, freq * 2);
    if ratio > 127 * 256 {
        panic!("Requested too low SPI frequency");
    }

    let presc = div_roundup(ratio, 256);
    let postdiv = if presc == 1 {
        ratio
    } else {
        div_roundup(ratio, presc)
    };

    ((presc * 2) as u8, (postdiv - 1) as u8)
}

impl<'d, T: Instance> Spi<'d, T> {
    pub fn new(
        inner: impl Unborrow<Target = T> + 'd,
        clk: impl Unborrow<Target = impl ClkPin<T>> + 'd,
        mosi: impl Unborrow<Target = impl MosiPin<T>> + 'd,
        miso: impl Unborrow<Target = impl MisoPin<T>> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(clk, mosi, miso);
        Self::new_inner(
            inner,
            Some(clk.degrade()),
            Some(mosi.degrade()),
            Some(miso.degrade()),
            None,
            config,
        )
    }

    pub fn new_txonly(
        inner: impl Unborrow<Target = T> + 'd,
        clk: impl Unborrow<Target = impl ClkPin<T>> + 'd,
        mosi: impl Unborrow<Target = impl MosiPin<T>> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(clk, mosi);
        Self::new_inner(
            inner,
            Some(clk.degrade()),
            Some(mosi.degrade()),
            None,
            None,
            config,
        )
    }

    pub fn new_rxonly(
        inner: impl Unborrow<Target = T> + 'd,
        clk: impl Unborrow<Target = impl ClkPin<T>> + 'd,
        miso: impl Unborrow<Target = impl MisoPin<T>> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(clk, miso);
        Self::new_inner(
            inner,
            Some(clk.degrade()),
            None,
            Some(miso.degrade()),
            None,
            config,
        )
    }

    fn new_inner(
        inner: impl Unborrow<Target = T> + 'd,
        clk: Option<AnyPin>,
        mosi: Option<AnyPin>,
        miso: Option<AnyPin>,
        cs: Option<AnyPin>,
        config: Config,
    ) -> Self {
        unborrow!(inner);

        unsafe {
            let p = inner.regs();
            let (presc, postdiv) = calc_prescs(config.frequency);

            p.cpsr().write(|w| w.set_cpsdvsr(presc));
            p.cr0().write(|w| {
                w.set_dss(0b0111); // 8bit
                w.set_spo(config.polarity == Polarity::IdleHigh);
                w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
                w.set_scr(postdiv);
            });
            p.cr1().write(|w| {
                w.set_sse(true); // enable
            });

            if let Some(pin) = &clk {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = &mosi {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = &miso {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = &cs {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
        }
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        unsafe {
            let p = self.inner.regs();
            for &b in data {
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(b as _));
                while !p.sr().read().rne() {}
                let _ = p.dr().read();
            }
        }
        self.flush()?;
        Ok(())
    }

    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let p = self.inner.regs();
            for b in data {
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(*b as _));
                while !p.sr().read().rne() {}
                *b = p.dr().read().data() as u8;
            }
        }
        self.flush()?;
        Ok(())
    }

    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let p = self.inner.regs();
            for b in data {
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(0));
                while !p.sr().read().rne() {}
                *b = p.dr().read().data() as u8;
            }
        }
        self.flush()?;
        Ok(())
    }

    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        unsafe {
            let p = self.inner.regs();
            let len = read.len().max(write.len());
            for i in 0..len {
                let wb = write.get(i).copied().unwrap_or(0);
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(wb as _));
                while !p.sr().read().rne() {}
                let rb = p.dr().read().data() as u8;
                if let Some(r) = read.get_mut(i) {
                    *r = rb;
                }
            }
        }
        self.flush()?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        unsafe {
            let p = self.inner.regs();
            while p.sr().read().bsy() {}
        }
        Ok(())
    }

    pub fn set_frequency(&mut self, freq: u32) {
        let (presc, postdiv) = calc_prescs(freq);
        let p = self.inner.regs();
        unsafe {
            // disable
            p.cr1().write(|w| w.set_sse(false));

            // change stuff
            p.cpsr().write(|w| w.set_cpsdvsr(presc));
            p.cr0().modify(|w| {
                w.set_scr(postdiv);
            });

            // enable
            p.cr1().write(|w| w.set_sse(true));
        }
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> pac::spi::Spi;
    }
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> pac::spi::Spi {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(SPI0, Spi0);
impl_instance!(SPI1, Spi1);

pub trait ClkPin<T: Instance>: GpioPin {}
pub trait CsPin<T: Instance>: GpioPin {}
pub trait MosiPin<T: Instance>: GpioPin {}
pub trait MisoPin<T: Instance>: GpioPin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, SPI0, MisoPin);
impl_pin!(PIN_1, SPI0, CsPin);
impl_pin!(PIN_2, SPI0, ClkPin);
impl_pin!(PIN_3, SPI0, MosiPin);
impl_pin!(PIN_4, SPI0, MisoPin);
impl_pin!(PIN_5, SPI0, CsPin);
impl_pin!(PIN_6, SPI0, ClkPin);
impl_pin!(PIN_7, SPI0, MosiPin);
impl_pin!(PIN_8, SPI1, MisoPin);
impl_pin!(PIN_9, SPI1, CsPin);
impl_pin!(PIN_10, SPI1, ClkPin);
impl_pin!(PIN_11, SPI1, MosiPin);
impl_pin!(PIN_12, SPI1, MisoPin);
impl_pin!(PIN_13, SPI1, CsPin);
impl_pin!(PIN_14, SPI1, ClkPin);
impl_pin!(PIN_15, SPI1, MosiPin);
impl_pin!(PIN_16, SPI0, MisoPin);
impl_pin!(PIN_17, SPI0, CsPin);
impl_pin!(PIN_18, SPI0, ClkPin);
impl_pin!(PIN_19, SPI0, MosiPin);

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, T> {
        type Error = Error;
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            self.blocking_transfer_in_place(words)?;
            Ok(words)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, T> {
        type Error = Error;

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::spi::Error for Error {
        fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
            match *self {}
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::ErrorType for Spi<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::SpiBusFlush for Spi<'d, T> {
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::SpiBusRead<u8> for Spi<'d, T> {
        fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(words, &[])
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::SpiBusWrite<u8> for Spi<'d, T> {
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }
    }

    impl<'d, T: Instance> embedded_hal_1::spi::blocking::SpiBus<u8> for Spi<'d, T> {
        fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_transfer(read, write)
        }

        fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_transfer_in_place(words)
        }
    }
}
