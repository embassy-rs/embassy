//! SPI Slave DMA driver implementation.

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};

use super::common::*;
use super::pins::*;
use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::dma::{Channel as DmaChannelTrait, DmaChannel, EnableInterrupt};
use crate::gpio::AnyPin;
use crate::pac;
use crate::pac::lpspi::vals::{Cpha, Cpol, Lsbf, Master, Outcfg, Pcspol, Pincfg, Rxmsk, Txmsk};

/// SPI Slave with DMA support for TX and RX.
///
/// The CS pin is optional. When `Some(pin)`, the hardware PCS signal is used for chip select.
/// When `None`, users must manage chip select externally. Note: For most slave use cases,
/// a CS pin is required to know when the slave is being addressed.
pub struct SpiSlaveDma<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Option<Peri<'d, AnyPin>>,
    tx_dma: DmaChannel<TxC>,
    rx_dma: DmaChannel<RxC>,
}

impl<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> SpiSlaveDma<'d, T, TxC, RxC> {
    /// Create a new SPI Slave with DMA support.
    ///
    /// # Arguments
    /// * `cs` - Optional chip select pin. When `Some(pin)`, hardware PCS is used.
    ///   When `None`, users must manage CS externally. Note: Most slave applications
    ///   require a CS signal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Option<Peri<'d, impl CsPin<T>>>,
        tx_dma_ch: Peri<'d, TxC>,
        rx_dma_ch: Peri<'d, RxC>,
        config: SlaveConfig,
    ) -> Result<Self> {
        let clock_config = LpspiConfig {
            power: config.clock_power,
            source: config.clock_source,
            div: config.clock_div,
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        sck.mux();
        mosi.mux();
        miso.mux();
        if let Some(ref pin) = cs {
            pin.mux();
        }

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.map(|p| p.into());

        let spi = T::regs();

        // Slave initialization sequence
        spi.cr().write(|w| w.set_men(false));
        spi.cfgr1().modify(|w| w.set_master(Master::SLAVE_MODE));
        spi.cfgr1().modify(|w| w.set_pcspol(Pcspol::from_bits(0)));
        spi.cfgr1().modify(|w| {
            w.set_outcfg(Outcfg::RETAIN_LASTVALUE);
            w.set_pincfg(Pincfg::SIN_IN_SOUT_OUT);
        });
        spi.fcr().write(|w| {
            w.set_txwater(0);
            w.set_rxwater(0);
        });

        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| {
            w.set_framesz(framesz);
            w.set_cpol(match config.mode.polarity {
                Polarity::IdleLow => Cpol::INACTIVE_LOW,
                Polarity::IdleHigh => Cpol::INACTIVE_HIGH,
            });
            w.set_cpha(match config.mode.phase {
                Phase::CaptureOnFirstTransition => Cpha::CAPTURED,
                Phase::CaptureOnSecondTransition => Cpha::CHANGED,
            });
            w.set_lsbf(match config.bit_order {
                BitOrder::MsbFirst => Lsbf::MSB_FIRST,
                BitOrder::LsbFirst => Lsbf::LSB_FIRST,
            });
        });

        spi.cr().write(|w| w.set_men(true));

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            tx_dma: DmaChannel::new(tx_dma_ch),
            rx_dma: DmaChannel::new(rx_dma_ch),
        })
    }

    #[inline]
    fn regs() -> pac::lpspi::Lpspi {
        T::regs()
    }

    fn tdr_addr() -> *mut u8 {
        Self::regs().tdr().as_ptr() as *mut u8
    }

    fn rdr_addr() -> *const u8 {
        Self::regs().rdr().as_ptr() as *const u8
    }

    fn enable_tx_dma() {
        Self::regs().der().modify(|w| w.set_tdde(true));
    }

    fn disable_tx_dma() {
        Self::regs().der().modify(|w| w.set_tdde(false));
    }

    fn enable_rx_dma() {
        Self::regs().der().modify(|w| w.set_rdde(true));
    }

    fn disable_rx_dma() {
        Self::regs().der().modify(|w| w.set_rdde(false));
    }

    fn flush_fifos_internal() {
        flush_fifos(Self::regs());
    }

    fn clear_status() {
        clear_status_flags(Self::regs());
    }

    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    /// Read data from master using DMA (RX only).
    pub async fn read_dma(&mut self, data: &mut [u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        spi.cr().modify(|w| w.set_men(false));
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = fifo_size.saturating_sub(1);
        spi.fcr().write(|w| {
            w.set_txwater(tx_watermark);
            w.set_rxwater(0);
        });

        clear_nostall(spi);
        spi.cr().modify(|w| w.set_men(true));

        spi.tcr().modify(|w| {
            w.set_txmsk(Txmsk::MASK);
            w.set_rxmsk(Rxmsk::NORMAL);
            w.set_bysw(true);
        });

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;

        unsafe {
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            let peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(peri_addr, data, EnableInterrupt::Yes);

            dma_start_fence();
            self.rx_dma.enable_request();

            self.rx_dma.enable_interrupt();

            Self::enable_rx_dma();
        }

        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        Self::disable_rx_dma();

        unsafe {
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
        }

        Ok(())
    }

    /// Write data to master using DMA (TX only).
    pub async fn write_dma(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        spi.cr().modify(|w| w.set_men(false));
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = fifo_size.saturating_sub(1);
        spi.fcr().write(|w| {
            w.set_txwater(tx_watermark);
            w.set_rxwater(0);
        });

        clear_nostall(spi);
        spi.cr().modify(|w| w.set_men(true));

        spi.tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
            w.set_bysw(true);
        });

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;

        static mut DUMMY_RX_SINK: u8 = 0;

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();

            let tx_peri_addr = (Self::tdr_addr() as usize + 3) as *mut u8;
            self.tx_dma
                .setup_write_to_peripheral(data, tx_peri_addr, EnableInterrupt::No);

            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            let rdr_addr = (Self::rdr_addr() as usize + 3) as u32;
            let rx_tcd = self.rx_dma.tcd();
            rx_tcd.tcd_saddr().write(|w| w.set_saddr(rdr_addr));
            rx_tcd.tcd_soff().write(|w| w.set_soff(0));
            rx_tcd.tcd_attr().write_value(pac::edma_0_tcd::regs::TcdAttr(0x0000));
            rx_tcd.tcd_nbytes_mloffno().write(|w| w.set_nbytes(1));
            rx_tcd.tcd_slast_sda().write(|w| w.set_slast_sda(0));
            rx_tcd
                .tcd_daddr()
                .write(|w| w.set_daddr((&raw mut DUMMY_RX_SINK) as u32));
            rx_tcd.tcd_doff().write(|w| w.set_doff(0));
            rx_tcd.tcd_citer_elinkno().write(|w| w.set_citer(data.len() as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
            rx_tcd.tcd_csr().write_value(pac::edma_0_tcd::regs::TcdCsr(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.set_biter(data.len() as u16));

            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            self.rx_dma.enable_interrupt();

            Self::enable_tx_dma();
            Self::enable_rx_dma();
        }

        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        Self::disable_tx_dma();
        Self::disable_rx_dma();

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
        }

        Ok(())
    }

    /// Full-duplex transfer using DMA (TX and RX simultaneously).
    pub async fn transfer_dma(&mut self, tx_data: &[u8], rx_data: &mut [u8]) -> Result<()> {
        if tx_data.len() != rx_data.len() {
            return Err(Error::TransferError);
        }
        if tx_data.is_empty() {
            return Ok(());
        }

        if tx_data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        spi.cr().modify(|w| w.set_men(false));
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = fifo_size.saturating_sub(1);
        spi.fcr().write(|w| {
            w.set_txwater(tx_watermark);
            w.set_rxwater(0);
        });

        clear_nostall(spi);
        spi.cr().modify(|w| w.set_men(true));

        spi.tcr().modify(|w| {
            w.set_rxmsk(Rxmsk::NORMAL);
            w.set_txmsk(Txmsk::NORMAL);
            w.set_bysw(true);
        });

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();

            let tx_peri_addr = (Self::tdr_addr() as usize + 3) as *mut u8;
            self.tx_dma
                .setup_write_to_peripheral(tx_data, tx_peri_addr, EnableInterrupt::No);

            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, rx_data, EnableInterrupt::Yes);

            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            self.rx_dma.enable_interrupt();

            Self::enable_tx_dma();
            Self::enable_rx_dma();
        }

        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        Self::disable_tx_dma();
        Self::disable_rx_dma();

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
        }

        Ok(())
    }
}
