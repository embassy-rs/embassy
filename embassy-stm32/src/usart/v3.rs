use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;

use crate::pac::usart::{regs, vals};

use super::*;
use core::future::Future;
use futures::TryFutureExt;

use crate::dma_traits::NoDma;

#[allow(dead_code)]
pub struct Uart<'d, T: Instance, TxDma = NoDma, RxDma = NoDma> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
    tx_dma: TxDma,
    rx_dma: RxDma,
}

impl<'d, T: Instance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        inner: impl Unborrow<Target = T>,
        rx: impl Unborrow<Target = impl RxPin<T>>,
        tx: impl Unborrow<Target = impl TxPin<T>>,
        tx_dma: impl Unborrow<Target = TxDma>,
        rx_dma: impl Unborrow<Target = RxDma>,
        config: Config,
    ) -> Self {
        unborrow!(inner, rx, tx, tx_dma, rx_dma);

        // Uncomment once we find all of the H7's UART clocks.
        T::enable();
        let pclk_freq = T::frequency();

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
                w.set_m(0, vals::M0::BIT8);
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
            tx_dma,
            rx_dma,
        }
    }

    async fn write_dma(&mut self, buffer: &[u8]) -> Result<(), Error>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        let ch = &mut self.tx_dma;
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmat(true);
            });
        }
        let r = self.inner.regs();
        let dst = r.tdr().ptr() as *mut u8;
        ch.transfer(buffer, dst).await;
        Ok(())
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
            for b in buffer {
                loop {
                    let sr = r.isr().read();
                    if sr.pe() {
                        r.rdr().read();
                        return Err(Error::Parity);
                    } else if sr.fe() {
                        r.rdr().read();
                        return Err(Error::Framing);
                    } else if sr.ne() {
                        r.rdr().read();
                        return Err(Error::Noise);
                    } else if sr.ore() {
                        r.rdr().read();
                        return Err(Error::Overrun);
                    } else if sr.rxne() {
                        break;
                    }
                }
                *b = r.rdr().read().0 as u8;
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance, RxDma> embedded_hal::blocking::serial::Write<u8>
    for Uart<'d, T, NoDma, RxDma>
{
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            for &b in buffer {
                while !r.isr().read().txe() {}
                r.tdr().write_value(regs::Tdr(b as u32))
            }
        }
        Ok(())
    }
    fn bflush(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            while !r.isr().read().tc() {}
        }
        Ok(())
    }
}

// rustfmt::skip because intellij removes the 'where' claus on the associated type.
#[rustfmt::skip]
impl<'d, T: Instance, TxDma, RxDma> embassy_traits::uart::Write for Uart<'d, T, TxDma, RxDma>
    where TxDma: crate::usart::TxDma<T>
{
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), embassy_traits::uart::Error>>;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.write_dma(buf).map_err(|_| embassy_traits::uart::Error::Other)
    }
}
