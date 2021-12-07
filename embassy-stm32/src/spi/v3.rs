#![macro_use]

use crate::dma::NoDma;
use crate::spi::{Error, Instance, RegsExt, RxDmaChannel, TxDmaChannel, WordSize};
use core::future::Future;
use core::ptr;
use embassy_traits::spi as traits;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use futures::future::join3;

use super::Spi;

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    #[allow(unused)]
    async fn write_dma_u8(&mut self, write: &[u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
    {
        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        let request = self.txdma.request();
        let dst = T::regs().tx_ptr();
        let f = self.txdma.write(request, write, dst);

        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
            });
        }

        f.await;
        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_txdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        Ok(())
    }

    #[allow(unused)]
    async fn read_dma_u8(&mut self, read: &mut [u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cfg1().modify(|reg| {
                reg.set_rxdmaen(true);
            });
        }

        let clock_byte_count = read.len();

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().rx_ptr();
        let rx_f = self.rxdma.read(rx_request, rx_src, read);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        let clock_byte = 0x00;
        let tx_f = self
            .txdma
            .write_x(tx_request, &clock_byte, clock_byte_count, tx_dst);

        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;
        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_rxdmaen(false);
                reg.set_txdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }
        Ok(())
    }

    #[allow(unused)]
    async fn read_write_dma_u8(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        assert!(read.len() >= write.len());

        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cfg1().modify(|reg| {
                reg.set_rxdmaen(true);
            });

            // Flush the read buffer to avoid errornous data from being read
            while T::regs().sr().read().rxp() {
                let _ = T::regs().rxdr().read();
            }
        }

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().rx_ptr();
        let rx_f = self
            .rxdma
            .read(rx_request, rx_src, &mut read[0..write.len()]);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        let tx_f = self.txdma.write(tx_request, write, tx_dst);

        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;
        unsafe {
            T::regs().cfg1().modify(|reg| {
                reg.set_rxdmaen(false);
                reg.set_txdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }
        Ok(())
    }

    async fn wait_for_idle() {
        unsafe {
            while !T::regs().sr().read().txc() {
                // spin
            }
            while T::regs().sr().read().rxplvl().0 > 0 {
                // spin
            }
        }
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u8> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter() {
            while unsafe { !regs.sr().read().txp() } {
                // spin
            }
            unsafe {
                ptr::write_volatile(regs.tx_ptr(), *word);
                regs.cr1().modify(|reg| reg.set_cstart(true));
            }
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.tifre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crce() {
                    return Err(Error::Crc);
                }
                if !sr.txp() {
                    // loop waiting for TXE
                    continue;
                }
                break;
            }
            unsafe {
                // discard read to prevent pverrun.
                let _: u8 = ptr::read_volatile(T::regs().rx_ptr());
            }
        }

        while unsafe { !regs.sr().read().txc() } {
            // spin
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u8> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            unsafe {
                regs.cr1().modify(|reg| {
                    reg.set_ssi(false);
                });
            }
            while unsafe { !regs.sr().read().txp() } {
                // spin
            }
            unsafe {
                ptr::write_volatile(T::regs().tx_ptr(), *word);
                regs.cr1().modify(|reg| reg.set_cstart(true));
            }
            loop {
                let sr = unsafe { regs.sr().read() };

                if sr.rxp() {
                    break;
                }
                if sr.tifre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crce() {
                    return Err(Error::Crc);
                }
            }
            unsafe {
                *word = ptr::read_volatile(T::regs().rx_ptr());
            }
            let sr = unsafe { regs.sr().read() };
            if sr.tifre() {
                return Err(Error::Framing);
            }
            if sr.ovr() {
                return Err(Error::Overrun);
            }
            if sr.crce() {
                return Err(Error::Crc);
            }
        }

        Ok(words)
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u16> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn write(&mut self, words: &[u16]) -> Result<(), Self::Error> {
        self.set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter() {
            while unsafe { !regs.sr().read().txp() } {
                // spin
            }
            unsafe {
                let txdr = regs.txdr().ptr() as *mut u16;
                ptr::write_volatile(txdr, *word);
                regs.cr1().modify(|reg| reg.set_cstart(true));
            }
            loop {
                let sr = unsafe { regs.sr().read() };
                if sr.tifre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crce() {
                    return Err(Error::Crc);
                }
                if !sr.txp() {
                    // loop waiting for TXE
                    continue;
                }
                break;
            }

            unsafe {
                let rxdr = regs.rxdr().ptr() as *const u8;
                // discard read to prevent pverrun.
                let _ = ptr::read_volatile(rxdr);
            }
        }

        while unsafe { !regs.sr().read().txc() } {
            // spin
        }

        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Transfer<u16> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u16]) -> Result<&'w [u16], Self::Error> {
        self.set_word_size(WordSize::SixteenBit);
        let regs = T::regs();

        for word in words.iter_mut() {
            while unsafe { !regs.sr().read().txp() } {
                // spin
            }
            unsafe {
                let txdr = regs.txdr().ptr() as *mut u16;
                ptr::write_volatile(txdr, *word);
                regs.cr1().modify(|reg| reg.set_cstart(true));
            }

            loop {
                let sr = unsafe { regs.sr().read() };

                if sr.rxp() {
                    break;
                }
                if sr.tifre() {
                    return Err(Error::Framing);
                }
                if sr.ovr() {
                    return Err(Error::Overrun);
                }
                if sr.crce() {
                    return Err(Error::Crc);
                }
            }

            unsafe {
                let rxdr = regs.rxdr().ptr() as *const u16;
                *word = ptr::read_volatile(rxdr);
            }
            let sr = unsafe { regs.sr().read() };
            if sr.tifre() {
                return Err(Error::Framing);
            }
            if sr.ovr() {
                return Err(Error::Overrun);
            }
            if sr.crce() {
                return Err(Error::Crc);
            }
        }

        Ok(words)
    }
}

impl<'d, T: Instance, Tx, Rx> traits::Spi<u8> for Spi<'d, T, Tx, Rx> {
    type Error = super::Error;
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx> traits::Write<u8> for Spi<'d, T, Tx, Rx> {
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, data: &'a [u8]) -> Self::WriteFuture<'a> {
        self.write_dma_u8(data)
    }
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx: RxDmaChannel<T>> traits::Read<u8>
    for Spi<'d, T, Tx, Rx>
{
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.read_dma_u8(data)
    }
}

impl<'d, T: Instance, Tx: TxDmaChannel<T>, Rx: RxDmaChannel<T>> traits::FullDuplex<u8>
    for Spi<'d, T, Tx, Rx>
{
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read_write<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        self.read_write_dma_u8(read, write)
    }
}
