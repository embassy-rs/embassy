#![macro_use]

use crate::dma::NoDma;
use crate::gpio::sealed::Pin;
use crate::spi::{Error, Instance, RegsExt, RxDmaChannel, TxDmaChannel, WordSize};
use core::future::Future;
use core::ptr;
use embassy_traits::spi as traits;
pub use embedded_hal::blocking;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use futures::future::join3;

use super::Spi;

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    fn set_word_size(&mut self, word_size: WordSize) {
        if self.current_word_size == word_size {
            return;
        }
        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_spe(false);
                reg.set_dff(word_size.dff())
            });
            T::regs().cr1().modify(|reg| {
                reg.set_spe(true);
            });
        }
        self.current_word_size = word_size;
    }

    #[allow(unused)]
    async fn write_dma_u8(&mut self, write: &[u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
    {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }
        self.set_word_size(WordSize::EightBit);

        let request = self.txdma.request();
        let dst = T::regs().tx_ptr();
        let f = self.txdma.write(request, write, dst);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        f.await;
        Ok(())
    }

    #[allow(unused)]
    async fn read_dma_u8(&mut self, read: &mut [u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_rxdmaen(true);
            });
        }
        self.set_word_size(WordSize::EightBit);

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
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(false);
                reg.set_rxdmaen(false);
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

        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_rxdmaen(true);
            });
        }
        self.set_word_size(WordSize::EightBit);

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().rx_ptr();
        let rx_f = self
            .rxdma
            .read(rx_request, rx_src, &mut read[0..write.len()]);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        let tx_f = self.txdma.write(tx_request, write, tx_dst);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join3(tx_f, rx_f, Self::wait_for_idle()).await;

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(false);
                reg.set_rxdmaen(false);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        Ok(())
    }

    async fn wait_for_idle() {
        unsafe {
            while T::regs().sr().read().bsy() {
                // spin
            }
        }
    }
}

impl<'d, T: Instance, Tx, Rx> Drop for Spi<'d, T, Tx, Rx> {
    fn drop(&mut self) {
        unsafe {
            self.sck.as_ref().map(|x| x.set_as_analog());
            self.mosi.as_ref().map(|x| x.set_as_analog());
            self.miso.as_ref().map(|x| x.set_as_analog());
        }
    }
}

impl<'d, T: Instance> embedded_hal::blocking::spi::Write<u8> for Spi<'d, T, NoDma, NoDma> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.set_word_size(WordSize::EightBit);
        let regs = T::regs();

        for word in words.iter() {
            write_word(regs, *word)?;
            let _: u8 = read_word(regs)?;
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
            write_word(regs, *word)?;
            *word = read_word(regs)?;
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
            write_word(regs, *word)?;
            let _: u8 = read_word(regs)?;
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
            write_word(regs, *word)?;
            *word = read_word(regs)?;
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
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output=Result<(), Self::Error>> + 'a;

    fn read_write<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Self::WriteReadFuture<'a> {
        self.read_write_dma_u8(read, write)
    }
}

trait Word {}

impl Word for u8 {}
impl Word for u16 {}

fn write_word<W: Word>(regs: &'static crate::pac::spi::Spi, word: W) -> Result<(), Error> {
    loop {
        let sr = unsafe { regs.sr().read() };
        if sr.ovr() {
            return Err(Error::Overrun);
        }
        #[cfg(not(spi_f1))]
        if sr.fre() {
            return Err(Error::Framing);
        }
        if sr.modf() {
            return Err(Error::ModeFault);
        }
        if sr.crcerr() {
            return Err(Error::Crc);
        }
        if sr.txe() {
            unsafe {
                ptr::write_volatile(regs.tx_ptr(), word);
            }
            return Ok(());
        }
    }
}

/// Read a single word blocking. Assumes word size have already been set.
fn read_word<W: Word>(regs: &'static crate::pac::spi::Spi) -> Result<W, Error> {
    loop {
        let sr = unsafe { regs.sr().read() };
        if sr.ovr() {
            return Err(Error::Overrun);
        }
        #[cfg(not(spi_f1))]
        if sr.fre() {
            return Err(Error::Framing);
        }
        if sr.modf() {
            return Err(Error::ModeFault);
        }
        if sr.crcerr() {
            return Err(Error::Crc);
        }
        if sr.rxne() {
            unsafe {
                return Ok(ptr::read_volatile(regs.rx_ptr()));
            }
        }
    }
}
