//! Sub-GHz dedicated SPI interface driver (`SUBGHZSPI`).

use embassy_time::Timer;
use embedded_hal_1::spi::Operation;

use crate::mode::{Async, Blocking, Mode};
use crate::spi::mode::Master;
use crate::spi::{Instance, RxDma, Spi, TxDma};
use crate::{Peri, pac};

/// Sub-GHz radio SPI interface driver.
///
/// Wraps the internal `SUBGHZSPI` peripheral and hardware slave-select line (`PWR_SUBGHZSPICR.NSS`).
/// Implements `embedded-hal` and `embedded-hal-async` `SpiDevice` traits.
pub struct SubghzSpi<'d, M: Mode> {
    spi: Spi<'d, M, Master>,
}

impl<'d, M: Mode> SubghzSpi<'d, M> {
    /// Control the Sub-GHz radio SPI NSS (slave select) signal.
    ///
    /// - `false`: NSS low (asserted / active).
    /// - `true`: NSS high (deasserted / inactive).
    #[inline(always)]
    pub fn set_nss(level: bool) {
        pac::PWR.subghzspicr().modify(|w| w.set_nss(level));
    }

    /// Read the current state of the Sub-GHz radio SPI NSS signal.
    #[inline(always)]
    pub fn nss() -> bool {
        pac::PWR.subghzspicr().read().nss()
    }
}

impl<'d> SubghzSpi<'d, Async> {
    /// Create a new asynchronous `SubghzSpi` driver using DMA.
    pub fn new<T: Instance, D1: TxDma<T>, D2: RxDma<T>>(
        peri: Peri<'d, T>,
        tx_dma: Peri<'d, D1>,
        rx_dma: Peri<'d, D2>,
        irq: impl crate::interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + crate::interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        Self::set_nss(true);
        Self {
            spi: Spi::new_subghz(peri, tx_dma, rx_dma, irq),
        }
    }

    /// Send a write command or buffer asynchronously over SUBGHZSPI.
    pub async fn write(&mut self, words: &[u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_async::spi::SpiDevice::write(self, words).await
    }

    /// Read data asynchronously from SUBGHZSPI.
    pub async fn read(&mut self, words: &mut [u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_async::spi::SpiDevice::read(self, words).await
    }

    /// Perform a full-duplex asynchronous transfer over SUBGHZSPI.
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_async::spi::SpiDevice::transfer(self, read, write).await
    }

    /// Perform an in-place asynchronous transfer over SUBGHZSPI.
    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_async::spi::SpiDevice::transfer_in_place(self, words).await
    }
}

impl<'d> SubghzSpi<'d, Blocking> {
    /// Create a new blocking `SubghzSpi` driver without DMA.
    pub fn new_blocking<T: Instance>(peri: Peri<'d, T>) -> Self {
        Self::set_nss(true);
        Self {
            spi: Spi::new_subghz_blocking(peri),
        }
    }

    /// Send a write command or buffer blockingly over SUBGHZSPI.
    pub fn blocking_write(&mut self, words: &[u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_1::spi::SpiDevice::write(self, words)
    }

    /// Read data blockingly from SUBGHZSPI.
    pub fn blocking_read(&mut self, words: &mut [u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_1::spi::SpiDevice::read(self, words)
    }

    /// Perform a full-duplex blocking transfer over SUBGHZSPI.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_1::spi::SpiDevice::transfer(self, read, write)
    }

    /// Perform an in-place blocking transfer over SUBGHZSPI.
    pub fn blocking_transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), crate::spi::Error> {
        embedded_hal_1::spi::SpiDevice::transfer_in_place(self, words)
    }
}

impl<'d, M: Mode> embedded_hal_1::spi::ErrorType for SubghzSpi<'d, M> {
    type Error = crate::spi::Error;
}

impl<'d, M: Mode> embedded_hal_1::spi::SpiDevice<u8> for SubghzSpi<'d, M> {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Self::set_nss(false);

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    Operation::Read(buf) => self.spi.blocking_read(buf),
                    Operation::Write(buf) => self.spi.blocking_write(buf),
                    Operation::Transfer(read, write) => self.spi.blocking_transfer(read, write),
                    Operation::TransferInPlace(buf) => self.spi.blocking_transfer_in_place(buf),
                    Operation::DelayNs(ns) => {
                        let res = embedded_hal_1::spi::SpiBus::<u8>::flush(&mut self.spi);
                        embassy_time::block_for(embassy_time::Duration::from_nanos(*ns as u64));
                        res
                    }
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        let flush_res = embedded_hal_1::spi::SpiBus::<u8>::flush(&mut self.spi);
        Self::set_nss(true);

        op_res?;
        flush_res?;
        Ok(())
    }
}

/// Holds `SUBGHZSPI_NSS` low for the lifetime of the guard.
///
/// Releases NSS on drop, so an async transaction whose future is cancelled
/// (for example by a timeout) does not leave the radio selected.
pub(super) struct NssGuard;

impl NssGuard {
    pub(super) fn assert() -> Self {
        pac::PWR.subghzspicr().modify(|w| w.set_nss(false));
        Self
    }
}

impl Drop for NssGuard {
    fn drop(&mut self) {
        pac::PWR.subghzspicr().modify(|w| w.set_nss(true));
    }
}

impl<'d> embedded_hal_async::spi::SpiDevice<u8> for SubghzSpi<'d, Async> {
    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        let nss = NssGuard::assert();

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    embedded_hal_async::spi::Operation::Read(buf) => self.spi.read(buf).await,
                    embedded_hal_async::spi::Operation::Write(buf) => self.spi.write(buf).await,
                    embedded_hal_async::spi::Operation::Transfer(read, write) => self.spi.transfer(read, write).await,
                    embedded_hal_async::spi::Operation::TransferInPlace(buf) => self.spi.transfer_in_place(buf).await,
                    embedded_hal_async::spi::Operation::DelayNs(ns) => {
                        match embedded_hal_async::spi::SpiBus::<u8>::flush(&mut self.spi).await {
                            Err(e) => Err(e),
                            Ok(()) => {
                                Timer::after_nanos(*ns as u64).await;
                                Ok(())
                            }
                        }
                    }
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        let flush_res = embedded_hal_async::spi::SpiBus::<u8>::flush(&mut self.spi).await;
        drop(nss);

        op_res?;
        flush_res?;
        Ok(())
    }
}
