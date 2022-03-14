#![macro_use]

use futures::future::join;

use super::*;
use crate::dma::{slice_ptr_parts, slice_ptr_parts_mut, Transfer};

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub(super) async fn write_dma_u8(&mut self, write: *const [u8]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
    {
        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
        }

        // TODO: This is unnecessary in some versions because
        // clearing SPE automatically clears the fifos
        flush_rx_fifo(T::regs());

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        unsafe { self.txdma.start_write(tx_request, write, tx_dst) }
        let tx_f = Transfer::new(&mut self.txdma);

        unsafe {
            set_txdmaen(T::regs(), true);
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
            });
        }

        tx_f.await;

        finish_dma(T::regs());

        Ok(())
    }

    pub(super) async fn read_dma_u8(&mut self, read: *mut [u8]) -> Result<(), Error>
    where
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            set_rxdmaen(T::regs(), true);
        }

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
            set_txdmaen(T::regs(), true);
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
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
        Tx: TxDma<T>,
        Rx: RxDma<T>,
    {
        let (_, rx_len) = slice_ptr_parts(read);
        let (_, tx_len) = slice_ptr_parts(write);
        assert_eq!(rx_len, tx_len);

        self.set_word_size(WordSize::EightBit);
        unsafe {
            T::regs().cr1().modify(|w| {
                w.set_spe(false);
            });
            set_rxdmaen(T::regs(), true);
        }

        // TODO: This is unnecessary in some versions because
        // clearing SPE automatically clears the fifos
        flush_rx_fifo(T::regs());

        let rx_request = self.rxdma.request();
        let rx_src = T::regs().rx_ptr();
        unsafe { self.rxdma.start_read(rx_request, rx_src, read) };
        let rx_f = Transfer::new(&mut self.rxdma);

        let tx_request = self.txdma.request();
        let tx_dst = T::regs().tx_ptr();
        unsafe { self.txdma.start_write(tx_request, write, tx_dst) }
        let tx_f = Transfer::new(&mut self.txdma);

        unsafe {
            set_txdmaen(T::regs(), true);
            T::regs().cr1().modify(|w| {
                w.set_spe(true);
            });
            T::regs().cr1().modify(|w| {
                w.set_cstart(true);
            });
        }

        join(tx_f, rx_f).await;

        finish_dma(T::regs());

        Ok(())
    }
}
