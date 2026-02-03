//! SPI Master DMA driver implementation.

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::Peri;

use super::common::*;
use super::master::Spi;
use super::pins::*;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::clocks::enable_and_reset;
use crate::dma::{Channel as DmaChannelTrait, DmaChannel, EnableInterrupt, Tcd};
use crate::gpio::AnyPin;
use crate::pac;

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
pub struct SpiDma<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    tx_dma: DmaChannel<TxC>,
    rx_dma: DmaChannel<RxC>,
    #[allow(dead_code)]
    config: Config,
    tcds: SpiDmaTcds,
}

impl<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> SpiDma<'d, T, TxC, RxC> {
    /// Create a new SPI Master with DMA support.
    pub fn new(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
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
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        Spi::<'_, T, Blocking>::set_config(&config)?;

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
    fn regs() -> &'static pac::lpspi0::RegisterBlock {
        T::regs()
    }

    fn tdr_addr() -> *mut u8 {
        Self::regs().tdr().as_ptr() as *mut u8
    }

    fn rdr_addr() -> *const u8 {
        Self::regs().rdr().as_ptr() as *const u8
    }

    fn disable_tx_dma() {
        Self::regs().der().modify(|_, w| w.tdde().disable());
    }

    fn disable_rx_dma() {
        Self::regs().der().modify(|_, w| w.rdde().disable());
    }

    fn enable_tx_rx_dma() {
        Self::regs().der().modify(|_, w| w.tdde().enable().rdde().enable());
    }

    fn disable_tx_rx_dma() {
        Self::regs().der().modify(|_, w| w.tdde().disable().rdde().disable());
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

        let tcr = Self::read_tcr_with_errata_workaround_internal();
        let tcr_with_cont = (tcr & 0xFF000000) | 0x00600007;
        spi.tcr().write(|w| unsafe { w.bits(tcr_with_cont) });
        while spi.fsr().read().txcount().bits() > 0 {}

        let rdr_addr = Self::rdr_addr() as u32 + 3;

        static mut DUMMY_RX_SINK: u8 = 0;

        unsafe {
            cortex_m::asm::dsb();

            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();

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
            rx_tcd.tcd_citer_elinkno().write(|w| w.citer().bits(data_len as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.dlast_sga().bits(0));
            rx_tcd.tcd_csr().write(|w| w.bits(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.biter().bits(data_len as u16));

            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            self.setup_tx_scatter_gather(data.as_ptr(), true, data_len, tcr_with_cont)?;

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

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

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

        let tcr = Self::read_tcr_with_errata_workaround_internal();
        let new_tcr =
            (tcr & !(TCR_CONT | TCR_CONTC | TCR_BYSW | TCR_PCS_MASK | TCR_RXMSK | TCR_TXMSK)) | TCR_CONT | TCR_BYSW;
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

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

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

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

        let tcr = Self::read_tcr_with_errata_workaround_internal();
        let new_tcr =
            (tcr & !(TCR_CONT | TCR_CONTC | TCR_BYSW | TCR_PCS_MASK | TCR_RXMSK | TCR_TXMSK)) | TCR_CONT | TCR_BYSW;
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });
        let tcr_with_cont = new_tcr;

        while spi.fsr().read().txcount().bits() > 0 {}

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

        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

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
