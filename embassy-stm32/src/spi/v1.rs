#![macro_use]

use futures::future::join;

use super::*;
use crate::dma::{slice_ptr_parts, slice_ptr_parts_mut, Transfer};

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub(super) async fn write_dma_u8(&mut self, write: *const [u8]) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
    {
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }
        self.set_word_size(WordSize::EightBit);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        unsafe { self.txdma.start_write(tx_request, write, tx_dst) }
        let tx_f = Transfer::new(&mut self.txdma);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        tx_f.await;

        finish_dma(T::regs());

        Ok(())
    }

    pub(super) async fn read_dma_u8(&mut self, read: *mut [u8]) -> Result<(), Error>
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

        let (_, clock_byte_count) = slice_ptr_parts_mut(read);

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().rx_ptr();
        unsafe { self.rxdma.start_read(rx_request, rx_src, read) };
        let rx_f = Transfer::new(&mut self.rxdma);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        let clock_byte = 0x00u8;
        let tx_f = crate::dma::write_repeated(
            &mut self.txdma,
            tx_request,
            clock_byte,
            clock_byte_count,
            tx_dst,
        );

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join(tx_f, rx_f).await;

        finish_dma(T::regs());

        Ok(())
    }

    pub(super) async fn transfer_dma_u8(
        &mut self,
        read: *mut [u8],
        write: *const [u8],
    ) -> Result<(), Error>
    where
        Tx: TxDmaChannel<T>,
        Rx: RxDmaChannel<T>,
    {
        let (_, rx_len) = slice_ptr_parts(read);
        let (_, tx_len) = slice_ptr_parts(write);
        assert_eq!(rx_len, tx_len);

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
        unsafe { self.rxdma.start_read(rx_request, rx_src, read) };
        let rx_f = Transfer::new(&mut self.rxdma);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        unsafe { self.txdma.start_write(tx_request, write, tx_dst) }
        let tx_f = Transfer::new(&mut self.txdma);

        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_txdmaen(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
        }

        join(tx_f, rx_f).await;

        finish_dma(T::regs());

        Ok(())
    }
}
