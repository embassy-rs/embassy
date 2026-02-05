//! SPI Master DMA driver implementation.

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::Peri;

use super::common::*;
use super::master::Spi;
use super::pins::*;
use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::dma::{Channel as DmaChannelTrait, DmaChannel, EnableInterrupt, Tcd};
use crate::gpio::AnyPin;
use crate::pac;
use crate::pac::lpspi::vals::{Contc, Mbf, Pcs, Rxmsk, Txmsk};

/// Static storage for TX DMA scatter/gather TCDs.
#[repr(C, align(32))]
struct SpiDmaTcds {
    /// Main data TCD: transfers N bytes from tx_buf to TDR+3.
    data_tcd: Tcd,
    /// Software TCD for TCR update: clears CONT to de-assert PCS.
    tcr_tcd: Tcd,
    /// TCR value with CONT=0 and CONTC=0.
    tcr_value: u32,
    /// Padding to maintain 32-byte alignment for scatter/gather.
    _padding: [u32; 5],
}

impl SpiDmaTcds {
    const fn new() -> Self {
        Self {
            data_tcd: Tcd {
                saddr: 0,
                soff: 0,
                attr: 0,
                nbytes: 0,
                slast: 0,
                daddr: 0,
                doff: 0,
                citer: 0,
                dlast_sga: 0,
                csr: 0,
                biter: 0,
            },
            tcr_tcd: Tcd {
                saddr: 0,
                soff: 0,
                attr: 0,
                nbytes: 0,
                slast: 0,
                daddr: 0,
                doff: 0,
                citer: 0,
                dlast_sga: 0,
                csr: 0,
                biter: 0,
            },
            tcr_value: 0,
            _padding: [0; 5],
        }
    }
}

/// SPI Master with DMA support for TX and RX.
///
/// The CS pin is optional. When `Some(pin)`, the hardware PCS signal is used for chip select.
/// When `None`, users must manage chip select externally (e.g., via GPIO with
/// `embassy-embedded-hal::shared_bus::SpiDevice`).
pub struct SpiDma<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Option<Peri<'d, AnyPin>>,
    tx_dma: DmaChannel<TxC>,
    rx_dma: DmaChannel<RxC>,
    #[allow(dead_code)]
    config: Config,
    tcds: SpiDmaTcds,
}

impl<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> SpiDma<'d, T, TxC, RxC> {
    /// Create a new SPI Master with DMA support.
    ///
    /// # Arguments
    /// * `cs` - Optional chip select pin. When `Some(pin)`, hardware PCS is used.
    ///   When `None`, users must manage CS externally (e.g., via GPIO).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Option<Peri<'d, impl CsPin<T>>>,
        tx_dma_ch: Peri<'d, TxC>,
        rx_dma_ch: Peri<'d, RxC>,
        config: Config,
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

        Spi::<'_, T, Blocking, HardwareCs>::set_config(&config)?;

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            tx_dma: DmaChannel::new(tx_dma_ch),
            rx_dma: DmaChannel::new(rx_dma_ch),
            config,
            tcds: SpiDmaTcds::new(),
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

    fn disable_tx_dma() {
        Self::regs().der().modify(|w| w.set_tdde(false));
    }

    fn disable_rx_dma() {
        Self::regs().der().modify(|w| w.set_rdde(false));
    }

    fn enable_tx_rx_dma() {
        Self::regs().der().modify(|w| {
            w.set_tdde(true);
            w.set_rdde(true);
        });
    }

    fn disable_tx_rx_dma() {
        Self::regs().der().modify(|w| {
            w.set_tdde(false);
            w.set_rdde(false);
        });
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

    #[inline]
    fn read_tcr_with_errata_workaround_internal() -> u32 {
        read_tcr_with_errata_workaround(Self::regs())
    }

    /// Configure TX DMA with a software TCD to clear CONT via DMA.
    unsafe fn setup_tx_scatter_gather(
        &mut self,
        tx_ptr: *const u8,
        src_increment: bool,
        transfer_len: usize,
        tcr_with_cont: u32,
    ) -> Result<()> {
        unsafe {
            if transfer_len == 0 || transfer_len > 0x7fff {
                return Err(Error::TransferError);
            }

            let spi = Self::regs();
            let tcr_without_cont = tcr_with_cont & !0x0030_0000;
            let tdr_addr = Self::tdr_addr() as u32 + 3;
            let tcr_addr = spi.tcr().as_ptr() as u32;

            let tcds = &raw mut self.tcds;
            (*tcds).tcr_value = tcr_without_cont;

            let tcr_tcd = &mut (*tcds).tcr_tcd;
            tcr_tcd.saddr = core::ptr::addr_of!((*tcds).tcr_value) as u32;
            tcr_tcd.soff = 0;
            tcr_tcd.attr = 0x0202;
            tcr_tcd.nbytes = 4;
            tcr_tcd.slast = 0;
            tcr_tcd.daddr = tcr_addr;
            tcr_tcd.doff = 0;
            tcr_tcd.citer = 1;
            tcr_tcd.dlast_sga = 0;
            // CSR = 0x0008: DREQ=1 (auto-clear ERQ and set DONE when major loop completes)
            // Note: When loaded via scatter/gather (ESG=1 in previous TCD), execution
            // starts automatically - no START bit needed. INTMAJOR is not used here
            // because we rely on RX DMA completion to signal transfer done.
            tcr_tcd.csr = 0x0008;
            tcr_tcd.biter = 1;

            let data_tcd = &mut (*tcds).data_tcd;
            data_tcd.saddr = tx_ptr as u32;
            data_tcd.soff = if src_increment { 1 } else { 0 };
            data_tcd.attr = 0x0000;
            data_tcd.nbytes = 1;
            data_tcd.slast = 0;
            data_tcd.daddr = tdr_addr;
            data_tcd.doff = 0;
            data_tcd.citer = transfer_len as u16;
            data_tcd.dlast_sga = core::ptr::addr_of!((*tcds).tcr_tcd) as u32 as i32;
            data_tcd.csr = 0x0010;
            data_tcd.biter = transfer_len as u16;

            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();
            self.tx_dma.load_tcd(data_tcd);

            cortex_m::asm::dsb();

            Ok(())
        }
    }

    /// Write data using DMA (TX only, discards RX).
    pub async fn write_dma(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();
        let data_len = data.len();

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

        let tcr = Self::read_tcr_with_errata_workaround_internal();
        let tcr_with_cont = (tcr & 0xFF000000) | 0x00600007;
        spi.tcr().write_value(pac::lpspi::regs::Tcr(tcr_with_cont));
        while spi.fsr().read().txcount() > 0 {}

        static mut DUMMY_RX_SINK: u8 = 0;

        unsafe {
            cortex_m::asm::dsb();

            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
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
            rx_tcd.tcd_citer_elinkno().write(|w| w.set_citer(data_len as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
            rx_tcd.tcd_csr().write_value(pac::edma_0_tcd::regs::TcdCsr(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.set_biter(data_len as u16));

            self.rx_dma.set_request_source::<T::RxDmaRequest>();
            self.setup_tx_scatter_gather(data.as_ptr(), true, data_len, tcr_with_cont)?;

            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable RX DMA interrupt - we use RX completion as the transfer-done signal
            // because LPSPI is full-duplex and every TX byte generates an RX byte.
            self.rx_dma.enable_interrupt();

            Self::enable_tx_rx_dma();
        }

        // Wait for RX DMA completion - this signals all bytes have been clocked through.
        // The TX DMA uses scatter/gather and its completion is not reliable for signaling.
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Wait for TX FIFO to drain and frame to complete
        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

        Self::disable_tx_rx_dma();

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

    /// Read data using DMA (RX only, sends dummy bytes on TX).
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

        // Build TCR value using PAC accessors: enable CONT and BYSW, clear other transfer bits
        let mut tcr = pac::lpspi::regs::Tcr(Self::read_tcr_with_errata_workaround_internal());
        tcr.set_cont(true);
        tcr.set_contc(Contc::START);
        tcr.set_bysw(true);
        tcr.set_pcs(Pcs::TX_PCS0);
        tcr.set_rxmsk(Rxmsk::NORMAL);
        tcr.set_txmsk(Txmsk::NORMAL);
        let new_tcr = tcr.0;
        spi.tcr().write_value(tcr);

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;

        static DUMMY_TX: u8 = 0;

        unsafe {
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, data, EnableInterrupt::Yes);

            self.setup_tx_scatter_gather(core::ptr::addr_of!(DUMMY_TX), false, data.len(), new_tcr)?;

            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            self.rx_dma.enable_interrupt();

            Self::enable_tx_rx_dma();
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

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

        Self::disable_tx_rx_dma();

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
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

        // Build TCR value using PAC accessors: enable CONT and BYSW, clear other transfer bits
        let mut tcr = pac::lpspi::regs::Tcr(Self::read_tcr_with_errata_workaround_internal());
        tcr.set_cont(true);
        tcr.set_contc(Contc::START);
        tcr.set_bysw(true);
        tcr.set_pcs(Pcs::TX_PCS0);
        tcr.set_rxmsk(Rxmsk::NORMAL);
        tcr.set_txmsk(Txmsk::NORMAL);
        let tcr_with_cont = tcr.0;
        spi.tcr().write_value(tcr);

        while spi.fsr().read().txcount() > 0 {}

        unsafe {
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, rx_data, EnableInterrupt::Yes);

            self.setup_tx_scatter_gather(tx_data.as_ptr(), true, tx_data.len(), tcr_with_cont)?;

            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            self.rx_dma.enable_interrupt();

            Self::enable_tx_rx_dma();
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

        spin_wait_while(|| spi.fsr().read().txcount() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

        Self::disable_tx_rx_dma();

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
        }

        Ok(())
    }
}
