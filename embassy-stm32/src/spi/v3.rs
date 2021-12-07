#![macro_use]

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
use futures::future::join3;

use super::*;

impl<'d, T: Instance, Tx, Rx> Spi<'d, T, Tx, Rx> {
    pub(super) async fn write_dma_u8(&mut self, write: &[u8]) -> Result<(), Error>
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

    pub(super) async fn read_dma_u8(&mut self, read: &mut [u8]) -> Result<(), Error>
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
