//! SPI Slave DMA driver implementation.

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};

use super::common::*;
use super::pins::*;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::clocks::enable_and_reset;
use crate::dma::{Channel as DmaChannelTrait, DmaChannel, EnableInterrupt};
use crate::gpio::AnyPin;
use crate::pac;

/// SPI Slave with DMA support for TX and RX.
pub struct SpiSlaveDma<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    tx_dma: DmaChannel<TxC>,
    rx_dma: DmaChannel<RxC>,
}

impl<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> SpiSlaveDma<'d, T, TxC, RxC> {
    /// Create a new SPI Slave with DMA support.
    pub fn new(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
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
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        let spi = T::regs();

        // Slave initialization sequence
        spi.cr().write(|w| w.men().disabled());
        spi.cfgr1().modify(|_, w| w.master().slave_mode());
        spi.cfgr1().modify(|_, w| unsafe { w.pcspol().bits(0) });
        spi.cfgr1()
            .modify(|_, w| w.outcfg().retain_lastvalue().pincfg().sin_in_sout_out());
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.mode.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.mode.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            }
        });

        spi.cr().write(|w| w.men().enabled());

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
    fn regs() -> &'static pac::lpspi0::RegisterBlock {
        T::regs()
    }

    fn tdr_addr() -> *mut u8 {
        Self::regs().tdr().as_ptr() as *mut u8
    }

    fn rdr_addr() -> *const u8 {
        Self::regs().rdr().as_ptr() as *const u8
    }

    fn enable_tx_dma() {
        Self::regs().der().modify(|_, w| w.tdde().enable());
    }

    fn disable_tx_dma() {
        Self::regs().der().modify(|_, w| w.tdde().disable());
    }

    fn enable_rx_dma() {
        Self::regs().der().modify(|_, w| w.rdde().enable());
    }

    fn disable_rx_dma() {
        Self::regs().der().modify(|_, w| w.rdde().disable());
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

        spi.cr().modify(|_, w| w.men().disabled());
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        clear_nostall(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        spi.tcr()
            .modify(|_, w| w.txmsk().mask().rxmsk().normal().bysw().enabled());

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

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

        spi.cr().modify(|_, w| w.men().disabled());
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        clear_nostall(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        spi.tcr()
            .modify(|_, w| w.txmsk().normal().rxmsk().normal().bysw().enabled());

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

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
            rx_tcd.tcd_saddr().write(|w| w.saddr().bits(rdr_addr));
            rx_tcd.tcd_soff().write(|w| w.soff().bits(0));
            rx_tcd.tcd_attr().write(|w| w.bits(0x0000));
            rx_tcd.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(1));
            rx_tcd.tcd_slast_sda().write(|w| w.slast_sda().bits(0));
            rx_tcd
                .tcd_daddr()
                .write(|w| w.daddr().bits((&raw mut DUMMY_RX_SINK) as *mut u8 as u32));
            rx_tcd.tcd_doff().write(|w| w.doff().bits(0));
            rx_tcd.tcd_citer_elinkno().write(|w| w.citer().bits(data.len() as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.dlast_sga().bits(0));
            rx_tcd.tcd_csr().write(|w| w.bits(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.biter().bits(data.len() as u16));

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

        spi.cr().modify(|_, w| w.men().disabled());
        Self::flush_fifos_internal();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        clear_nostall(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        spi.tcr()
            .modify(|_, w| w.rxmsk().normal().txmsk().normal().bysw().enabled());

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

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
