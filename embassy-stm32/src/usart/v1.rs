use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;

use crate::pac::usart::{regs, vals};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits8,
    DataBits9,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "0.5 stop bits"]
    STOP0P5,
    #[doc = "2 stop bits"]
    STOP2,
    #[doc = "1.5 stop bits"]
    STOP1P5,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
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
        config: Config,
        //pclk_freq: u32,
    ) -> Self {
        unborrow!(inner, rx, tx);

        let pclk_freq = T::frequency();
        //let pclk_freq = 16_000_000;

        // TODO: enable in RCC

        // TODO: better calculation, including error checking and OVER8 if possible.
        let div = (pclk_freq.0 + (config.baudrate / 2)) / config.baudrate;

        let r = inner.regs();

        unsafe {
            rx.set_as_af(rx.af_num());
            tx.set_as_af(tx.af_num());

            r.brr().write_value(regs::Brr(div));
            r.cr1().write(|w| {
                w.set_ue(true);
                w.set_te(true);
                w.set_re(true);
                w.set_m(vals::M::M8);
                w.set_pce(config.parity != Parity::ParityNone);
                w.set_ps(match config.parity {
                    Parity::ParityOdd => vals::Ps::ODD,
                    Parity::ParityEven => vals::Ps::EVEN,
                    _ => vals::Ps::EVEN,
                });
            });
            r.cr2().write(|_w| {});
            r.cr3().write(|_w| {});
        }

        Self {
            inner,
            phantom: PhantomData,
        }
    }

    #[cfg(dma_v2)]
    pub async fn write_dma(
        &mut self,
        //ch: &mut impl crate::dma::Channel,
        ch: &mut impl TxDma<T>,
        buffer: &[u8],
    ) -> Result<(), Error> {
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmat(true);
            });
        }
        let r = self.inner.regs();
        let dst = r.dr().ptr() as *mut u8;
        ch.transfer(buffer, dst).await;
        Ok(())
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
            for b in buffer {
                loop {
                    let sr = r.sr().read();
                    if sr.pe() {
                        r.dr().read();
                        return Err(Error::Parity);
                    } else if sr.fe() {
                        r.dr().read();
                        return Err(Error::Framing);
                    } else if sr.ne() {
                        r.dr().read();
                        return Err(Error::Noise);
                    } else if sr.ore() {
                        r.dr().read();
                        return Err(Error::Overrun);
                    } else if sr.rxne() {
                        break;
                    }
                }
                *b = r.dr().read().0 as u8;
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::serial::Write<u8> for Uart<'d, T> {
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            for &b in buffer {
                while !r.sr().read().txe() {}
                r.dr().write_value(regs::Dr(b as u32))
            }
        }
        Ok(())
    }
    fn bflush(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            while !r.sr().read().tc() {}
        }
        Ok(())
    }
}
