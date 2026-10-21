//! LPSPI Controller Driver.

use core::marker::PhantomData;
use core::sync::atomic::{Ordering, fence};

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
pub use embedded_hal_1::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode, Phase, Polarity};
use nxp_pac::lpspi::{Cpha, Cpol, Lsbf, Master, Mbf, Outcfg, Pcspol, Pincfg, Prescale, Rrf, Rtf, Rxmsk, Txmsk};

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, Mode as IoMode, SckPin, SdiPin, SdoPin};
use crate::clocks::periph_helpers::{Div4, LpspiClockSel, LpspiConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, TransferOptions};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

// LPSPI has a 4-word FIFO.
const LPSPI_FIFO_SIZE: u8 = 4;

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Resume the controller before changing its configuration.
    Suspended,
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Other internal errors or unexpected state.
    Other,
}

/// I/O Errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IoError {
    /// The controller is explicitly suspended.
    Suspended,
    /// A transfer, DMA request, or unread FIFO data prevents suspension.
    Busy,
    /// Receive error.
    ///
    /// Indicates FIFO overflow condition has happened.
    ReceiveError,
    /// Transmit error.
    ///
    /// Indicated FIFO underrun condition has happened.
    TransmitError,
    /// Other internal errors or unexpected state.
    Other,
}

fn wait_for_module_enable(enabled: impl FnMut() -> bool, expected: bool) -> Result<(), IoError> {
    crate::clocks::wait_for_enable_state(enabled, expected)
        .then_some(())
        .ok_or(IoError::Other)
}

impl From<crate::dma::InvalidParameters> for IoError {
    fn from(_value: crate::dma::InvalidParameters) -> Self {
        IoError::Other
    }
}

/// SPI interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();
        let r = T::info().regs().ier().read().0;
        if r != 0 {
            T::info().regs().ier().write(|w| w.0 = 0);
            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

/// SPI target clock configuration
#[derive(Clone)]
#[non_exhaustive]
pub struct ClockConfig {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// LPSPI clock source
    pub source: LpspiClockSel,
    /// LPSPI pre-divider
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroLfDiv,
            div: const { Div4::no_div() },
        }
    }
}

/// SPI bit order
#[derive(Default)]
pub enum BitOrder {
    /// Most-significant bit first
    #[default]
    MsbFirst,
    /// Least-significant bit first
    LsbFirst,
}

/// SPI controller configuration
#[non_exhaustive]
pub struct Config {
    /// Frequency in Hertz.
    pub frequency: u32,
    /// SPI operating mode.
    pub mode: Mode,
    /// Bit order
    pub bit_order: BitOrder,
    /// Clock configuration
    pub clock_config: ClockConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            mode: MODE_0,
            bit_order: Default::default(),
            clock_config: Default::default(),
        }
    }
}

/// Spi driver.
pub struct Spi<'d, M: IoMode> {
    info: &'static Info,
    _sck: Peri<'d, AnyPin>,
    _miso: Option<Peri<'d, AnyPin>>,
    _mosi: Option<Peri<'d, AnyPin>>,
    _freq: u32,
    mode: M,
    _wg: Option<WakeGuard>,
    power: PoweredClock,
    suspended: bool,
    _phantom: PhantomData<&'d mut M>,
}

impl<'d, M: IoMode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        _sck: Peri<'d, AnyPin>,
        _mosi: Option<Peri<'d, AnyPin>>,
        _miso: Option<Peri<'d, AnyPin>>,
        config: Config,
        mode: M,
    ) -> Result<Self, SetupError> {
        let ClockConfig { power, source, div } = config.clock_config;

        // Enable clocks
        let conf = LpspiConfig {
            power,
            source,
            div,
            instance: T::CLOCK_INSTANCE,
        };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(SetupError::ClockSetup)? };

        let mut inst = Self {
            info: T::info(),
            _sck,
            _mosi,
            _miso,
            mode,
            _freq: parts.freq,
            _wg: parts.wake_guard,
            power,
            suspended: false,
            _phantom: PhantomData,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), SetupError> {
        if self.suspended {
            return Err(SetupError::Suspended);
        }
        let (prescaler, div) = compute_baud_params(self._freq, config.frequency);

        self.info.regs().cr().write(|w| {
            w.set_men(false);
            w.set_rst(true);
            w.set_rtf(Rtf::TxfifoRst);
            w.set_rrf(Rrf::RxfifoRst);
        });

        self.info.regs().cr().modify(|w| w.set_rst(false));

        self.info.regs().cfgr1().write(|w| {
            w.set_master(Master::MasterMode);
            w.set_pincfg(Pincfg::SinInSoutOut);
            w.set_pcspol(Pcspol::Discarded);
        });

        self.info.regs().ccr().write(|w| {
            w.set_sckdiv(div);
            w.set_dbt(div);
            w.set_pcssck(div);
            w.set_sckpcs(div);
        });

        self.info.regs().fcr().write(|w| {
            w.set_txwater(0);
            w.set_rxwater(0);
        });

        self.info.regs().tcr().write(|w| {
            // Assuming byte transfers
            w.set_framesz(7);

            w.set_cpol(match config.mode.polarity {
                Polarity::IdleLow => Cpol::InactiveLow,
                Polarity::IdleHigh => Cpol::InactiveHigh,
            });

            w.set_cpha(match config.mode.phase {
                Phase::CaptureOnFirstTransition => Cpha::Captured,
                Phase::CaptureOnSecondTransition => Cpha::Changed,
            });

            w.set_lsbf(match config.bit_order {
                BitOrder::MsbFirst => Lsbf::MsbFirst,
                BitOrder::LsbFirst => Lsbf::LsbFirst,
            });

            w.set_prescale(prescaler);
        });

        self.info.regs().cr().modify(|w| w.set_men(true));

        Ok(())
    }
}

impl<'d, M: IoMode> Spi<'d, M> {
    /// Stop an idle controller and release its Deep Sleep requirement.
    ///
    /// Cancel outstanding I/O and deassert any externally managed chip select
    /// first. Pending transfers, FIFO data, or armed DMA requests return
    /// [`IoError::Busy`] without changing hardware or its guard.
    /// Once a disable is requested, I/O stays inhibited even if acknowledgement
    /// fails. The guard remains held in that case; explicitly resume to recover.
    pub fn suspend(&mut self) -> Result<(), IoError> {
        critical_section::with(|_| {
            let fifo = self.info.regs().fsr().read();
            if self.info.regs().sr().read().mbf() == Mbf::Busy
                || fifo.txcount() != 0
                || fifo.rxcount() != 0
                || self.info.regs().der().read().0 != 0
                || self.mode.dma_is_armed_or_active()
            {
                return Err(IoError::Busy);
            }
            self.suspended = true;
            self.info.regs().ier().write(|w| w.0 = 0);
            self.mode.stop_dma();
            self.info.regs().cr().modify(|w| w.set_men(false));
            wait_for_module_enable(|| self.info.regs().cr().read().men(), false)?;
            self._wg = None;
            Ok(())
        })
    }

    /// Restore the requirement before enabling and acknowledging the controller.
    ///
    /// An unexpectedly disabled but otherwise idle configured controller can
    /// also be resumed. Queued data or a lost master configuration is rejected.
    pub fn resume(&mut self) -> Result<(), IoError> {
        if self.suspended || !self.info.regs().cr().read().men() {
            critical_section::with(|_| {
                let was_suspended = self.suspended;
                self.suspended = true;
                self._wg = WakeGuard::for_power(&self.power);
                if self.info.regs().cfgr1().read().master() != Master::MasterMode {
                    return Err(IoError::Other);
                }
                if !was_suspended {
                    let fifo = self.info.regs().fsr().read();
                    if fifo.txcount() != 0 || fifo.rxcount() != 0 || self.mode.dma_is_armed_or_active() {
                        return Err(IoError::Busy);
                    }
                }
                self.info.regs().cr().modify(|w| w.set_men(true));
                wait_for_module_enable(|| self.info.regs().cr().read().men(), true)?;
                self.suspended = false;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Whether I/O is inhibited until an explicit resume.
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn ensure_active(&self) -> Result<(), IoError> {
        if self.suspended {
            Err(IoError::Suspended)
        } else if !self.info.regs().cr().read().men() {
            Err(IoError::Other)
        } else {
            Ok(())
        }
    }

    fn check_status(&mut self) -> Result<(), IoError> {
        self.ensure_active()?;
        let status = self.info.regs().sr().read();

        if status.ref_() {
            // Empty the RX FIFO.
            self.info.regs().cr().modify(|w| w.set_rrf(Rrf::RxfifoRst));
            self.info.regs().sr().write(|w| w.set_ref_(true));
            Err(IoError::ReceiveError)
        } else if status.tef() {
            self.info.regs().sr().write(|w| w.set_tef(true));
            Err(IoError::TransmitError)
        } else {
            Ok(())
        }
    }

    /// Read data from Spi blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        // Queue zero-valued transmit frames through TDR to generate the receive clocks.
        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for word in data {
            while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                self.check_status()?;
            }
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(0));

            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {
                self.check_status()?;
            }
            self.check_status()?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        Ok(())
    }

    /// Write data to Spi blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Mask);
        });

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                self.check_status()?;
            }
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*word as u32));
        }

        self.blocking_flush()
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        let common = read.len().min(write.len());
        let (read_common, read_tail) = read.split_at_mut(common);
        let (write_common, write_tail) = write.split_at(common);

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for (rb, wb) in read_common.iter_mut().zip(write_common.iter()) {
            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                self.check_status()?;
            }
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*wb as u32));

            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {
                self.check_status()?;
            }
            self.check_status()?;
            *rb = self.info.regs().rdr().read().data() as u8;
        }

        if !read_tail.is_empty() {
            for rb in read_tail {
                while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                    self.check_status()?;
                }
                self.check_status()?;
                self.info.regs().tdr().write(|w| w.set_data(0));

                while self.info.regs().fsr().read().rxcount() == 0 {
                    self.check_status()?;
                }
                self.check_status()?;
                *rb = self.info.regs().rdr().read().data() as u8;
            }
        }

        if !write_tail.is_empty() {
            while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                self.check_status()?;
            }
            self.check_status()?;
            self.info.regs().tcr().modify(|w| w.set_rxmsk(Rxmsk::Mask));

            for wb in write_tail {
                while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                    self.check_status()?;
                }
                self.check_status()?;
                self.info.regs().tdr().write(|w| w.set_data(*wb as u32));
            }
        }

        self.blocking_flush()
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() >= LPSPI_FIFO_SIZE {
                self.check_status()?;
            }
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*word as u32));

            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {
                self.check_status()?;
            }
            self.check_status()?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        self.blocking_flush()
    }

    /// Block execution until Spi is done.
    pub fn blocking_flush(&mut self) -> Result<(), IoError> {
        self.ensure_active()?;
        while self.info.regs().sr().read().mbf() == Mbf::Busy {
            self.check_status()?;
        }
        self.check_status()
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create a SPI driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl SdiPin<T> + 'd>,
        miso: Peri<'d, impl SdoPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();
        miso.mux();

        let sck = sck.into();
        let mosi = mosi.into();
        let miso = miso.into();

        Self::new_inner(_peri, sck, Some(mosi), Some(miso), config, Blocking)
    }

    /// Create a TX-only SPI driver in blocking mode.
    pub fn new_blocking_txonly<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl SdiPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();

        let sck = sck.into();
        let mosi = mosi.into();

        Self::new_inner(_peri, sck, Some(mosi), None, config, Blocking)
    }

    /// Create an RX-only SPI driver in blocking mode.
    pub fn new_blocking_rxonly<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        miso: Peri<'d, impl SdoPin<T> + 'd>,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        miso.mux();

        let sck = sck.into();
        let miso = miso.into();

        Self::new_inner(_peri, sck, None, Some(miso), config, Blocking)
    }
}

impl<'d> Spi<'d, Async> {
    /// Create a SPI driver in async mode.
    pub fn new_async<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl SdiPin<T> + 'd>,
        miso: Peri<'d, impl SdoPin<T> + 'd>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();
        miso.mux();

        let sck = sck.into();
        let mosi = mosi.into();
        let miso = miso.into();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_peri, sck, Some(mosi), Some(miso), config, Async)
    }

    /// Create a TX-only SPI driver in async mode.
    pub fn new_async_txonly<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl SdiPin<T> + 'd>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();

        let sck = sck.into();
        let mosi = mosi.into();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_peri, sck, Some(mosi), None, config, Async)
    }

    /// Create an RX-only SPI driver in async mode.
    pub fn new_async_rxonly<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        miso: Peri<'d, impl SdoPin<T> + 'd>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        miso.mux();

        let sck = sck.into();
        let miso = miso.into();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_peri, sck, None, Some(miso), config, Async)
    }
}

impl<'d> Spi<'d, Dma<'d>> {
    /// Create a SPI driver in async mode.
    pub fn new_async_with_dma<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl SdiPin<T> + 'd>,
        miso: Peri<'d, impl SdoPin<T> + 'd>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();
        miso.mux();

        let sck = sck.into();
        let mosi = mosi.into();
        let miso = miso.into();

        let tx_dma = DmaChannel::new(tx_dma);
        let rx_dma = DmaChannel::new(rx_dma);

        // enable this channel's interrupt
        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(
            _peri,
            sck,
            Some(mosi),
            Some(miso),
            config,
            Dma {
                tx_dma,
                rx_dma,
                tx_request: T::TX_DMA_REQUEST,
                rx_request: T::RX_DMA_REQUEST,
            },
        )
    }

    async fn read_dma_chunk(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        let rx_peri_addr = self.info.regs().rdr().as_ptr() as *mut u8;
        let tx_peri_addr = self.info.regs().tdr().as_ptr() as *mut u8;

        unsafe {
            // Clean up channel state
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type
            self.mode.rx_dma.set_request_source(self.mode.rx_request);
            self.mode.tx_dma.set_request_source(self.mode.tx_request);

            self.mode.tx_dma.setup_write_zeros_to_peripheral(
                data.len(),
                tx_peri_addr,
                TransferOptions::NO_INTERRUPTS,
            )?;

            self.mode.rx_dma.setup_read_from_peripheral(
                rx_peri_addr,
                data,
                false,
                TransferOptions::COMPLETE_INTERRUPT,
            )?;

            // Enable SPI DMA request
            self.info.regs().der().modify(|w| {
                w.set_rdde(true);
                w.set_tdde(true);
            });

            // Enable DMA channel request
            self.mode.rx_dma.enable_request();
            self.mode.tx_dma.enable_request();
        }

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            let _ = self.mode.rx_dma.wait_cell().poll_wait(cx);
            if self.mode.rx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Cleanup
        self.info.regs().der().modify(|w| {
            w.set_rdde(false);
            w.set_tdde(false);
        });
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
        }

        // Ensure all writes by DMA are visible to the CPU
        // TODO: ensure this is done internal to the DMA methods so individual drivers
        // don't need to handle this?
        fence(Ordering::Acquire);

        Ok(())
    }

    async fn write_dma_chunk(&mut self, data: &[u8]) -> Result<(), IoError> {
        let peri_addr = self.info.regs().tdr().as_ptr() as *mut u8;

        unsafe {
            // Clean up channel state
            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            self.mode.tx_dma.set_request_source(self.mode.tx_request);

            // Configure TCD for memory-to-peripheral transfer
            self.mode
                .tx_dma
                .setup_write_to_peripheral(data, peri_addr, false, TransferOptions::COMPLETE_INTERRUPT)?;

            // Ensure all writes by CPU are visible to the DMA
            // TODO: ensure this is done internal to the DMA methods so individual drivers
            // don't need to handle this?
            fence(Ordering::Release);

            // Enable SPI TX DMA request
            self.info.regs().der().modify(|w| w.set_tdde(true));

            // Enable DMA channel request
            self.mode.tx_dma.enable_request();
        }

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            let _ = self.mode.tx_dma.wait_cell().poll_wait(cx);
            if self.mode.tx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Cleanup
        self.info.regs().der().modify(|w| w.set_tdde(false));
        unsafe {
            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
        }

        Ok(())
    }

    async fn transfer_dma_chunk(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
        let rx_peri_addr = self.info.regs().rdr().as_ptr() as *mut u8;
        let tx_peri_addr = self.info.regs().tdr().as_ptr() as *mut u8;

        unsafe {
            // Clean up channel state
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type
            self.mode.rx_dma.set_request_source(self.mode.rx_request);
            self.mode.tx_dma.set_request_source(self.mode.tx_request);

            self.mode.tx_dma.setup_write_to_peripheral(
                write,
                tx_peri_addr,
                false,
                TransferOptions::COMPLETE_INTERRUPT,
            )?;

            self.mode.rx_dma.setup_read_from_peripheral(
                rx_peri_addr,
                read,
                false,
                TransferOptions::COMPLETE_INTERRUPT,
            )?;

            // Ensure all writes by CPU are visible to the DMA
            // TODO: ensure this is done internal to the DMA methods so individual drivers
            // don't need to handle this?
            fence(Ordering::Release);

            // Enable SPI DMA request
            self.info.regs().der().modify(|w| {
                w.set_rdde(true);
                w.set_tdde(true);
            });

            // Enable DMA channel request
            self.mode.rx_dma.enable_request();
            self.mode.tx_dma.enable_request();
        }

        // Wait for completion asynchronously
        let tx_transfer = core::future::poll_fn(|cx| {
            let _ = self.mode.tx_dma.wait_cell().poll_wait(cx);
            if self.mode.tx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        });

        let rx_transfer = core::future::poll_fn(|cx| {
            let _ = self.mode.rx_dma.wait_cell().poll_wait(cx);
            if self.mode.rx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        });

        join(tx_transfer, rx_transfer).await;

        // Cleanup
        self.info.regs().der().modify(|w| {
            w.set_rdde(false);
            w.set_tdde(false);
        });
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
        }

        // Ensure all writes by DMA are visible to the CPU
        // TODO: ensure this is done internal to the DMA methods so individual drivers
        // don't need to handle this?
        fence(Ordering::Acquire);

        Ok(())
    }

    async fn transfer_in_place_dma_chunk(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        let rx_peri_addr = self.info.regs().rdr().as_ptr() as *mut u8;
        let tx_peri_addr = self.info.regs().tdr().as_ptr() as *mut u8;

        unsafe {
            // Clean up channel state
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();
            self.mode.rx_dma.clear_interrupt();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
            self.mode.tx_dma.clear_interrupt();

            // Set DMA request source from instance type
            self.mode.rx_dma.set_request_source(self.mode.rx_request);
            self.mode.tx_dma.set_request_source(self.mode.tx_request);

            self.mode.tx_dma.setup_write_to_peripheral(
                data,
                tx_peri_addr,
                false,
                TransferOptions::COMPLETE_INTERRUPT,
            )?;
            self.mode.rx_dma.setup_read_from_peripheral(
                rx_peri_addr,
                data,
                false,
                TransferOptions::COMPLETE_INTERRUPT,
            )?;

            // Ensure all writes by CPU are visible to the DMA
            // TODO: ensure this is done internal to the DMA methods so individual drivers
            // don't need to handle this?
            fence(Ordering::Release);

            // Enable SPI DMA request
            self.info.regs().der().modify(|w| {
                w.set_rdde(true);
                w.set_tdde(true);
            });

            // Enable DMA channel request
            self.mode.rx_dma.enable_request();
            self.mode.tx_dma.enable_request();
        }

        // Wait for completion asynchronously
        let tx_transfer = core::future::poll_fn(|cx| {
            let _ = self.mode.tx_dma.wait_cell().poll_wait(cx);
            if self.mode.tx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        });

        let rx_transfer = core::future::poll_fn(|cx| {
            let _ = self.mode.rx_dma.wait_cell().poll_wait(cx);
            if self.mode.rx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        });

        join(tx_transfer, rx_transfer).await;

        // Cleanup
        self.info.regs().der().modify(|w| {
            w.set_rdde(false);
            w.set_tdde(false);
        });
        unsafe {
            self.mode.rx_dma.disable_request();
            self.mode.rx_dma.clear_done();

            self.mode.tx_dma.disable_request();
            self.mode.tx_dma.clear_done();
        }

        // Ensure all writes by DMA are visible to the CPU
        // TODO: ensure this is done internal to the DMA methods so individual drivers
        // don't need to handle this?
        fence(Ordering::Acquire);

        Ok(())
    }
}

trait AsyncEngine {
    async fn async_read_internal(&mut self, data: &mut [u8]) -> Result<(), IoError>;
    async fn async_write_internal(&mut self, data: &[u8]) -> Result<(), IoError>;
    async fn async_transfer_internal(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError>;
    async fn async_transfer_in_place_internal(&mut self, data: &mut [u8]) -> Result<(), IoError>;
}

#[allow(private_bounds)]
impl<'d, M: AsyncMode> Spi<'d, M>
where
    Self: AsyncEngine,
{
    /// Read data from Spi async execution until done.
    pub fn async_read(&mut self, data: &mut [u8]) -> impl Future<Output = Result<(), IoError>> {
        <Self as AsyncEngine>::async_read_internal(self, data)
    }

    /// Write data to Spi async execution until done.
    pub fn async_write(&mut self, data: &[u8]) -> impl Future<Output = Result<(), IoError>> {
        <Self as AsyncEngine>::async_write_internal(self, data)
    }

    /// Transfer data to SPI async execution until done.
    pub fn async_transfer(&mut self, read: &mut [u8], write: &[u8]) -> impl Future<Output = Result<(), IoError>> {
        <Self as AsyncEngine>::async_transfer_internal(self, read, write)
    }

    /// Transfer data in place to SPI async execution until done.
    pub fn async_transfer_in_place(&mut self, data: &mut [u8]) -> impl Future<Output = Result<(), IoError>> {
        <Self as AsyncEngine>::async_transfer_in_place_internal(self, data)
    }

    /// Async flush.
    pub async fn async_flush(&mut self) -> Result<(), IoError> {
        self.ensure_active()?;
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().ier().write(|w| w.set_tcie(true));
                self.info.regs().sr().read().tcf()
            })
            .await
            .map_err(|_| IoError::Other)
    }
}

impl<'d> AsyncEngine for Spi<'d, Async> {
    async fn async_read_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        // Queue zero-valued transmit frames through TDR to generate the receive clocks.
        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for word in data {
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_tdie(true);
                        w.set_teie(true);
                    });
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(0));

            // Wait until we have data in the RxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_rdie(true);
                        w.set_reie(true);
                    });
                    self.info.regs().fsr().read().rxcount() > 0 || self.info.regs().sr().read().ref_()
                })
                .await
                .map_err(|_| IoError::Other)?;

            self.check_status()?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, data: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Mask);
        });

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_tdie(true);
                        w.set_teie(true);
                    });
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                })
                .await
                .map_err(|_| IoError::Other)?;

            self.check_status()?;

            self.info.regs().tdr().write(|w| w.set_data(*word as u32));
        }

        self.async_flush().await
    }

    async fn async_transfer_internal(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        let common = read.len().min(write.len());
        let (read_common, read_tail) = read.split_at_mut(common);
        let (write_common, write_tail) = write.split_at(common);

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for (rb, wb) in read_common.iter_mut().zip(write_common.iter()) {
            // Wait until we have at least one byte space in the TxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_tdie(true);
                        w.set_teie(true);
                    });
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*wb as u32));

            // Wait until we have data in the RxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_rdie(true);
                        w.set_reie(true);
                    });
                    self.info.regs().fsr().read().rxcount() > 0 || self.info.regs().sr().read().ref_()
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.check_status()?;
            *rb = self.info.regs().rdr().read().data() as u8;
        }

        if !read_tail.is_empty() {
            for rb in read_tail {
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        self.info.regs().ier().modify(|w| {
                            w.set_tdie(true);
                            w.set_teie(true);
                        });
                        self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                    })
                    .await
                    .map_err(|_| IoError::Other)?;
                self.check_status()?;
                self.info.regs().tdr().write(|w| w.set_data(0));

                self.info
                    .wait_cell()
                    .wait_for(|| {
                        self.info.regs().ier().modify(|w| {
                            w.set_rdie(true);
                            w.set_reie(true);
                        });
                        self.info.regs().fsr().read().rxcount() > 0 || self.info.regs().sr().read().ref_()
                    })
                    .await
                    .map_err(|_| IoError::Other)?;
                self.check_status()?;
                *rb = self.info.regs().rdr().read().data() as u8;
            }
        }

        if !write_tail.is_empty() {
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_tdie(true);
                        w.set_teie(true);
                    });
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.check_status()?;
            self.info.regs().tcr().modify(|w| w.set_rxmsk(Rxmsk::Mask));

            for wb in write_tail {
                self.info
                    .wait_cell()
                    .wait_for(|| {
                        self.info.regs().ier().modify(|w| {
                            w.set_tdie(true);
                            w.set_teie(true);
                        });
                        self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                    })
                    .await
                    .map_err(|_| IoError::Other)?;
                self.check_status()?;
                self.info.regs().tdr().write(|w| w.set_data(*wb as u32));
            }
        }

        self.async_flush().await
    }

    async fn async_transfer_in_place_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| w.set_tdie(true));
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.info.regs().tdr().write(|w| w.set_data(*word as u32));

            // Wait until we have data in the RxFIFO.
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| w.set_rdie(true));
                    self.info.regs().fsr().read().rxcount() > 0
                })
                .await
                .map_err(|_| IoError::Other)?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        self.async_flush().await
    }
}

impl<'d> AsyncEngine for Spi<'d, Dma<'d>> {
    async fn async_read_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        self.info.regs().cfgr1().modify(|w| w.set_outcfg(Outcfg::Tristated));

        let _on_drop = OnDrop::new(|| {
            self.info.regs().der().modify(|w| w.set_rdde(false));
            self.info.regs().cfgr1().modify(|w| w.set_outcfg(Outcfg::Tristated));
        });

        for chunk in data.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            self.read_dma_chunk(chunk).await?;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, data: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Mask);
        });

        let on_drop = OnDrop::new(|| {
            self.info.regs().der().modify(|w| w.set_tdde(false));
        });

        for chunk in data.chunks(DMA_MAX_TRANSFER_SIZE) {
            self.write_dma_chunk(chunk).await?;
        }

        on_drop.defuse();

        self.async_flush().await
    }

    async fn async_transfer_internal(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        let common = read.len().min(write.len());
        let (read_common, read_tail) = read.split_at_mut(common);
        let (write_common, write_tail) = write.split_at(common);

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        let on_drop = OnDrop::new(|| {
            self.info.regs().der().modify(|w| w.set_tdde(false));
        });

        for (read_chunk, write_chunk) in read_common
            .chunks_mut(DMA_MAX_TRANSFER_SIZE)
            .zip(write_common.chunks(DMA_MAX_TRANSFER_SIZE))
        {
            self.transfer_dma_chunk(read_chunk, write_chunk).await?;
        }

        if !read_tail.is_empty() {
            for read_chunk in read_tail.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
                self.read_dma_chunk(read_chunk).await?;
            }
        }

        if !write_tail.is_empty() {
            self.info
                .wait_cell()
                .wait_for(|| {
                    self.info.regs().ier().modify(|w| {
                        w.set_tdie(true);
                        w.set_teie(true);
                    });
                    self.info.regs().fsr().read().txcount() < LPSPI_FIFO_SIZE || self.info.regs().sr().read().tef()
                })
                .await
                .map_err(|_| IoError::Other)?;
            self.check_status()?;
            self.info.regs().tcr().modify(|w| w.set_rxmsk(Rxmsk::Mask));

            for write_chunk in write_tail.chunks(DMA_MAX_TRANSFER_SIZE) {
                self.write_dma_chunk(write_chunk).await?;
            }
        }

        on_drop.defuse();

        self.async_flush().await
    }

    async fn async_transfer_in_place_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        self.ensure_active()?;
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::Normal);
            w.set_rxmsk(Rxmsk::Normal);
        });

        for chunk in data.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            self.transfer_in_place_dma_chunk(chunk).await?;
        }

        self.async_flush().await
    }
}

impl<M: IoMode> Drop for Spi<'_, M> {
    fn drop(&mut self) {
        self.info.regs().ier().write(|w| w.0 = 0);
        self.info.regs().der().write(|w| w.0 = 0);
        self.mode.stop_dma();
        self.info.regs().cr().modify(|w| {
            w.set_men(false);
            w.set_rst(true);
        });
        self._sck.set_as_disabled();
        if let Some(pin) = &self._miso {
            pin.set_as_disabled();
        }
        if let Some(pin) = &self._mosi {
            pin.set_as_disabled();
        }
    }
}

/// Compute prescaler and SCKDIV for the desired baud rate.
/// Returns (prescaler, sckdiv) where:
/// - prescaler is a Prescaler enum value
/// - sckdiv is 0-255
///
/// Baud = src_hz / (prescaler.divisor() * (SCKDIV + 2))
pub(super) fn compute_baud_params(src_hz: u32, baud_hz: u32) -> (Prescale, u8) {
    if baud_hz == 0 {
        return (Prescale::DivideBy1, 0);
    }

    let prescalers = [
        Prescale::DivideBy1,
        Prescale::DivideBy2,
        Prescale::DivideBy4,
        Prescale::DivideBy8,
        Prescale::DivideBy16,
        Prescale::DivideBy32,
        Prescale::DivideBy64,
        Prescale::DivideBy128,
    ];

    let (prescaler, div, _) = prescalers.iter().fold(
        (Prescale::DivideBy1, 0u8, u32::MAX),
        |(best_pre, best_div, best_err), &prescaler| {
            let divisor: u32 = 1 << (prescaler as u8);
            let denom = divisor.saturating_mul(baud_hz);
            let sckdiv_calc = (src_hz + denom / 2) / denom;
            if sckdiv_calc < 2 {
                return (best_pre, best_div, best_err);
            }
            let sckdiv = (sckdiv_calc - 2).min(255) as u8;
            let actual_baud = src_hz / (divisor * (sckdiv as u32 + 2));
            let error = actual_baud.abs_diff(baud_hz);
            if error < best_err {
                (prescaler, sckdiv, error)
            } else {
                (best_pre, best_div, best_err)
            }
        },
    );

    (prescaler, div)
}

impl<'d, M: IoMode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, M> {
    type Error = IoError;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, M: IoMode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, M> {
    type Error = IoError;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

impl embedded_hal_1::spi::Error for IoError {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {
            IoError::Other | IoError::Suspended | IoError::Busy => embedded_hal_1::spi::ErrorKind::Other,
            IoError::ReceiveError => embedded_hal_1::spi::ErrorKind::Overrun,
            IoError::TransmitError => embedded_hal_1::spi::ErrorKind::Other,
        }
    }
}

impl<'d, M: IoMode> embedded_hal_1::spi::ErrorType for Spi<'d, M> {
    type Error = IoError;
}

impl<'d, M: IoMode> embedded_hal_1::spi::SpiBus<u8> for Spi<'d, M> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl<'d, M: AsyncMode> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, M>
where
    Spi<'d, M>: AsyncEngine,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.async_flush().await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.async_write(words).await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.async_read(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.async_transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.async_transfer_in_place(words).await
    }
}

impl<'d, M: IoMode> SetConfig for Spi<'d, M> {
    type Config = Config;
    type ConfigError = SetupError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use embassy_futures::block_on;
    use std::boxed::Box;

    fn test_spi<'d, M: IoMode + 'd>(mode: M, power: PoweredClock) -> Spi<'d, M> {
        let memory = Box::leak(Box::new([0u32; 256]));
        // SAFETY: this uniquely allocated aligned block covers the SPI registers.
        let regs = unsafe { crate::pac::lpspi::Lpspi::from_ptr(memory.as_mut_ptr().cast()) };
        let info = Box::leak(Box::new(Info {
            regs,
            wait_cell: maitake_sync::WaitCell::new(),
        }));
        info.regs().cr().modify(|w| w.set_men(true));
        info.regs().cfgr1().modify(|w| w.set_master(Master::MasterMode));
        Spi {
            info,
            _sck: crate::test_support::pin(),
            _miso: Some(crate::test_support::pin()),
            _mosi: Some(crate::test_support::pin()),
            _freq: 12_000_000,
            mode,
            _wg: WakeGuard::for_power(&power),
            power,
            suspended: false,
            _phantom: PhantomData,
        }
    }

    #[test]
    fn spi_suspension_rejects_io_until_explicit_resume() {
        critical_section::with(|cs| {
            let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
            spi.suspend().unwrap();
            assert!(spi.is_suspended());
            assert!(!spi.info.regs().cr().read().men());
            assert!(!crate::clocks::active_wake_guards(&cs));
            assert_eq!(spi.blocking_write(&[]), Err(IoError::Suspended));
            assert_eq!(spi.blocking_read(&mut []), Err(IoError::Suspended));
            assert_eq!(spi.blocking_transfer(&mut [], &[]), Err(IoError::Suspended));
            assert_eq!(spi.blocking_transfer_in_place(&mut []), Err(IoError::Suspended));
            assert_eq!(spi.blocking_flush(), Err(IoError::Suspended));
            assert_eq!(spi.set_configuration(&Config::default()), Err(SetupError::Suspended));
            spi.suspend().unwrap();
            spi.resume().unwrap();
            spi.resume().unwrap();
            assert!(spi.info.regs().cr().read().men());
            assert!(crate::clocks::active_wake_guards(&cs));
            let info = spi.info;
            drop(spi);
            assert!(info.regs().cr().read().rst());
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn module_enable_acknowledgement_waits_for_delayed_readback() {
        let mut reads = 0;
        let result = wait_for_module_enable(
            || {
                reads += 1;
                reads > 3
            },
            true,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(reads, 4);
    }

    #[test]
    fn module_enable_acknowledgement_is_bounded() {
        let mut reads = 0;
        let result = wait_for_module_enable(
            || {
                reads += 1;
                true
            },
            false,
        );
        assert_eq!(result, Err(IoError::Other));
        assert_eq!(reads, 1024);
    }

    #[test]
    fn resume_repairs_an_idle_disabled_module_even_if_software_was_active() {
        critical_section::with(|cs| {
            let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
            spi.info.regs().cr().modify(|w| w.set_men(false));
            assert!(!spi.is_suspended());
            spi.resume().unwrap();
            assert!(spi.info.regs().cr().read().men());
            assert!(!spi.is_suspended());
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(spi);
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn disabled_module_with_queued_data_is_not_replayed_by_resume() {
        critical_section::with(|cs| {
            let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
            spi.info.regs().cr().modify(|w| w.set_men(false));
            // SAFETY: the read-only hardware register is backed by writable fixture RAM.
            unsafe { spi.info.regs().fsr().as_ptr().write_volatile(crate::pac::lpspi::Fsr(2)) };
            assert_eq!(spi.resume(), Err(IoError::Busy));
            assert!(spi.is_suspended());
            assert!(!spi.info.regs().cr().read().men());
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn resume_rejects_lost_configuration_without_releasing_the_guard() {
        critical_section::with(|cs| {
            let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
            spi.suspend().unwrap();
            spi.info.regs().cfgr1().write(|w| w.0 = 0);
            assert_eq!(spi.resume(), Err(IoError::Other));
            assert!(spi.is_suspended());
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn disabled_hardware_returns_error_before_the_jedec_fifo_wait() {
        let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
        spi.info.regs().cr().modify(|w| w.set_men(false));
        let mut jedec = [0x9f, 0, 0, 0];
        assert_eq!(spi.blocking_transfer_in_place(&mut jedec), Err(IoError::Other));
        assert_eq!(spi.check_status(), Err(IoError::Other));
        assert_eq!(spi.info.regs().fsr().read().txcount(), 0);
    }

    #[test]
    fn spi_busy_fifo_or_dma_rejection_retains_guard() {
        critical_section::with(|cs| {
            let mut spi = test_spi(Blocking, PoweredClock::NormalEnabledDeepSleepDisabled);
            spi.info.regs().sr().modify(|w| w.set_mbf(Mbf::Busy));
            assert_eq!(spi.suspend(), Err(IoError::Busy));
            spi.info.regs().sr().write(|w| w.0 = 0);
            let mut fifo = spi.info.regs().fsr().read();
            fifo.set_txcount(1);
            // SAFETY: the read-only hardware register is backed by writable test RAM.
            unsafe { spi.info.regs().fsr().as_ptr().write_volatile(fifo) };
            assert_eq!(spi.suspend(), Err(IoError::Busy));
            // SAFETY: this register points into the isolated fixture allocation.
            unsafe { spi.info.regs().fsr().as_ptr().write_volatile(crate::pac::lpspi::Fsr(0)) };
            spi.info.regs().der().write(|w| w.set_rdde(true));
            assert_eq!(spi.suspend(), Err(IoError::Busy));
            assert!(!spi.is_suspended());
            assert!(spi.info.regs().cr().read().men());
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn suspended_async_spi_cannot_rearm_hardware() {
        let mut spi = test_spi(Async, PoweredClock::AlwaysEnabled);
        spi.suspend().unwrap();
        assert_eq!(block_on(spi.async_read(&mut [])), Err(IoError::Suspended));
        assert_eq!(block_on(spi.async_write(&[])), Err(IoError::Suspended));
        assert_eq!(block_on(spi.async_transfer(&mut [], &[])), Err(IoError::Suspended));
        assert_eq!(block_on(spi.async_transfer_in_place(&mut [])), Err(IoError::Suspended));
        assert_eq!(block_on(spi.async_flush()), Err(IoError::Suspended));
        assert_eq!(spi.info.regs().ier().read().0, 0);
        spi.resume().unwrap();
        assert!(spi._wg.is_none());
    }

    #[test]
    fn armed_dma_rejects_suspend_even_when_peripheral_requests_are_disabled() {
        struct ArmedMode<'a>(&'a core::cell::Cell<bool>);
        impl super::super::sealed::Sealed for ArmedMode<'_> {
            fn stop_dma(&self) {
                self.0.set(true);
            }

            fn dma_is_armed_or_active(&self) -> bool {
                true
            }
        }
        impl super::super::Mode for ArmedMode<'_> {}

        critical_section::with(|cs| {
            let stopped = core::cell::Cell::new(false);
            let mut spi = test_spi(ArmedMode(&stopped), PoweredClock::NormalEnabledDeepSleepDisabled);
            assert_eq!(spi.suspend(), Err(IoError::Busy));
            assert!(!stopped.get());
            assert!(!spi.is_suspended());
            assert!(spi.info.regs().cr().read().men());
            assert!(crate::clocks::active_wake_guards(&cs));
        });
    }
}
