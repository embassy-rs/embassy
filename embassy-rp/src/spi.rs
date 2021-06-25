use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;
use embedded_hal::blocking::spi as eh;
use gpio::Pin;

use crate::{gpio, pac, peripherals};

#[non_exhaustive]
pub struct Config {
    pub frequency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
        }
    }
}

pub struct Spi<'d, T: Instance> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Spi<'d, T> {
    pub fn new(
        inner: impl Unborrow<Target = T>,
        clk: impl Unborrow<Target = impl ClkPin<T>>,
        mosi: impl Unborrow<Target = impl MosiPin<T>>,
        miso: impl Unborrow<Target = impl MisoPin<T>>,
        config: Config,
    ) -> Self {
        unborrow!(inner, clk, mosi, miso);

        unsafe {
            let p = inner.regs();

            let clk_peri = crate::clocks::clk_peri_freq();
            assert!(config.frequency <= clk_peri);

            // Find smallest prescale value which puts output frequency in range of
            // post-divide. Prescale is an even number from 2 to 254 inclusive.
            let presc = (2u32..=254).step_by(2).find(|&presc| {
                (clk_peri as u64) < (presc as u64 + 2) * 256 * config.frequency as u64
            });

            // if this fails, frequency is too low.
            let presc = unwrap!(presc);

            // Find largest post-divide which makes output <= baudrate. Post-divide is
            // an integer in the range 1 to 256 inclusive.
            let postdiv = (1u32..=256)
                .rev()
                .find(|&postdiv| clk_peri / (presc * (postdiv - 1)) > config.frequency);
            let postdiv = unwrap!(postdiv);

            p.cpsr().write(|w| w.set_cpsdvsr(presc as _));
            p.cr0().write(|w| {
                w.set_dss(0b0111); // 8bit
                w.set_spo(false);
                w.set_sph(false);
                w.set_scr((postdiv - 1) as u8);
            });
            p.cr1().write(|w| {
                w.set_sse(true); // enable
            });

            info!("SPI freq: {=u32}", clk_peri / (presc * postdiv));

            clk.io().ctrl().write(|w| w.set_funcsel(1));
            mosi.io().ctrl().write(|w| w.set_funcsel(1));
            miso.io().ctrl().write(|w| w.set_funcsel(1));
        }
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        unsafe {
            let p = self.inner.regs();
            for &b in data {
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(b as _));
            }
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            let p = self.inner.regs();
            while p.sr().read().bsy() {}
        }
    }

    fn drain_rx(&mut self) {
        unsafe {
            let p = self.inner.regs();
            while !p.sr().read().rne() {
                p.dr().read();
            }
        }
    }
}

impl<'d, T: Instance> eh::Write<u8> for Spi<'d, T> {
    type Error = core::convert::Infallible;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words);
        Ok(())
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> pac::spi::Spi;
    }
    pub trait ClkPin<T: Instance> {}
    pub trait CsPin<T: Instance> {}
    pub trait MosiPin<T: Instance> {}
    pub trait MisoPin<T: Instance> {}
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

pub trait ClkPin<T: Instance>: sealed::ClkPin<T> + Pin {}
pub trait CsPin<T: Instance>: sealed::CsPin<T> + Pin {}
pub trait MosiPin<T: Instance>: sealed::MosiPin<T> + Pin {}
pub trait MisoPin<T: Instance>: sealed::MisoPin<T> + Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
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
