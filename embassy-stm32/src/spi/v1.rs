#![macro_use]

use crate::dma::NoDma;
use crate::spi::{
    check_error_flags, Error, Instance, RegsExt, RxDmaChannel, TxDmaChannel, WordSize,
};
use core::ptr;
pub use embedded_hal::blocking;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use futures::future::join3;

use super::Spi;

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub(super) async fn write_dma_u8(&mut self, write: &[u8]) -> Result<(), Error>
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

    pub(super) async fn read_dma_u8(&mut self, read: &mut [u8]) -> Result<(), Error>
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

    pub(super) async fn read_write_dma_u8(
        &mut self,
        read: &mut [u8],
        write: &[u8],
    ) -> Result<(), Error>
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

use super::Word;

fn write_word<W: Word>(regs: &'static crate::pac::spi::Spi, word: W) -> Result<(), Error> {
    loop {
        let sr = unsafe { regs.sr().read() };

        check_error_flags(sr)?;

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

        check_error_flags(sr)?;

        if sr.rxne() {
            unsafe {
                return Ok(ptr::read_volatile(regs.rx_ptr()));
            }
        }
    }
}
