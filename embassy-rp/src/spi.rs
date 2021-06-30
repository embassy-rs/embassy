use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;
use embedded_hal::blocking::spi as eh;
use embedded_hal::spi as ehnb;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{NoPin, OptionalPin};
use crate::{pac, peripherals};

pub use ehnb::{Phase, Polarity};

#[non_exhaustive]
pub struct Config {
    pub frequency: u32,
    pub phase: ehnb::Phase,
    pub polarity: ehnb::Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            phase: ehnb::Phase::CaptureOnFirstTransition,
            polarity: ehnb::Polarity::IdleLow,
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
        cs: impl Unborrow<Target = impl CsPin<T>>,
        config: Config,
    ) -> Self {
        unborrow!(inner, clk, mosi, miso, cs);

        unsafe {
            let p = inner.regs();

            let clk_peri = crate::clocks::clk_peri_freq();
            assert!(config.frequency <= clk_peri);

            // TODO replace these trial-and-error loops with decent calculations.

            // Find smallest prescale value which puts output frequency in range of
            // post-divide. Prescale is an even number from 2 to 254 inclusive.
            let presc = (2u32..=254).step_by(2).find(|&presc| {
                (clk_peri as u64) < (presc as u64 + 2) * 256 * config.frequency as u64
            });

            // if this fails, frequency is too low.
            let presc = unwrap!(presc);

            // Find largest post-divide which makes output <= baudrate. Post-divide is
            // an integer in the range 1 to 256 inclusive.
            // TODO figure what's up with postdiv=1, it is dividing by 0. Iterate down to 2 for now.
            let postdiv = (2u32..=256)
                .rev()
                .find(|&postdiv| clk_peri / (presc * (postdiv - 1)) > config.frequency);
            let postdiv = unwrap!(postdiv);

            p.cpsr().write(|w| w.set_cpsdvsr(presc as _));
            p.cr0().write(|w| {
                w.set_dss(0b0111); // 8bit
                w.set_spo(config.polarity == ehnb::Polarity::IdleHigh);
                w.set_sph(config.phase == ehnb::Phase::CaptureOnSecondTransition);
                w.set_scr((postdiv - 1) as u8);
            });
            p.cr1().write(|w| {
                w.set_sse(true); // enable
            });

            info!("SPI freq: {=u32}", clk_peri / (presc * postdiv));

            if let Some(pin) = clk.pin_mut() {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = mosi.pin_mut() {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = miso.pin_mut() {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
            if let Some(pin) = cs.pin_mut() {
                pin.io().ctrl().write(|w| w.set_funcsel(1));
            }
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

    pub fn transfer(&mut self, data: &mut [u8]) {
        unsafe {
            let p = self.inner.regs();
            for b in data {
                while !p.sr().read().tnf() {}
                p.dr().write(|w| w.set_data(*b as _));
                while !p.sr().read().rne() {}
                *b = p.dr().read().data() as u8;
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
}

impl<'d, T: Instance> eh::Write<u8> for Spi<'d, T> {
    type Error = core::convert::Infallible;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words);
        Ok(())
    }
}

impl<'d, T: Instance> eh::Transfer<u8> for Spi<'d, T> {
    type Error = core::convert::Infallible;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.transfer(words);
        Ok(words)
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

pub trait ClkPin<T: Instance>: sealed::ClkPin<T> + OptionalPin {}
pub trait CsPin<T: Instance>: sealed::CsPin<T> + OptionalPin {}
pub trait MosiPin<T: Instance>: sealed::MosiPin<T> + OptionalPin {}
pub trait MisoPin<T: Instance>: sealed::MisoPin<T> + OptionalPin {}

impl<T: Instance> sealed::ClkPin<T> for NoPin {}
impl<T: Instance> ClkPin<T> for NoPin {}
impl<T: Instance> sealed::CsPin<T> for NoPin {}
impl<T: Instance> CsPin<T> for NoPin {}
impl<T: Instance> sealed::MosiPin<T> for NoPin {}
impl<T: Instance> MosiPin<T> for NoPin {}
impl<T: Instance> sealed::MisoPin<T> for NoPin {}
impl<T: Instance> MisoPin<T> for NoPin {}

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
