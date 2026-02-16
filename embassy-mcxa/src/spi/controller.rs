//! LPSPI Controller Driver.

use core::marker::PhantomData;
use core::sync::atomic::{Ordering, fence};

use embassy_embedded_hal::SetConfig;
use embassy_futures::join::join;
use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
pub use embedded_hal_1::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode, Phase, Polarity};
use nxp_pac::lpspi::vals::{Cpha, Cpol, Lsbf, Master, Mbf, Outcfg, Pcspol, Pincfg, Prescale, Rrf, Rtf, Rxmsk, Txmsk};

use super::{Async, AsyncMode, Blocking, Dma, Info, Instance, MisoPin, Mode as IoMode, MosiPin, RxDma, SckPin, TxDma};
use crate::clocks::periph_helpers::{Div4, LpspiClockSel, LpspiConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{DMA_MAX_TRANSFER_SIZE, DmaChannel, EnableInterrupt};
use crate::gpio::AnyPin;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

// LPSPI has a 4-word FIFO.
const LPSPI_FIFO_SIZE: u8 = 4;

/// Errors exclusive to HW initialization
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
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
    _phantom: PhantomData<&'d M>,
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
            _phantom: PhantomData,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), SetupError> {
        let (prescaler, div) = compute_baud_params(self._freq, config.frequency);

        self.info.regs().cr().write(|w| {
            w.set_men(false);
            w.set_rst(true);
            w.set_rtf(Rtf::TXFIFO_RST);
            w.set_rrf(Rrf::RXFIFO_RST);
        });

        self.info.regs().cr().modify(|w| w.set_rst(false));

        self.info.regs().cfgr1().write(|w| {
            w.set_master(Master::MASTER_MODE);
            w.set_pincfg(Pincfg::SIN_IN_SOUT_OUT);
            w.set_pcspol(Pcspol::DISCARDED);
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

            w.set_prescale(prescaler);
        });

        self.info.regs().cr().modify(|w| w.set_men(true));

        Ok(())
    }
}

impl<'d, M: IoMode> Spi<'d, M> {
    fn check_status(&mut self) -> Result<(), IoError> {
        let status = self.info.regs().sr().read();

        if status.ref_() {
            // Empty the RX FIFO.
            self.info.regs().cr().modify(|w| w.set_rrf(Rrf::RXFIFO_RST));
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
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::MASK);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        for word in data {
            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {}
            self.check_status()?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        Ok(())
    }

    /// Write data to Spi blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::MASK);
        });

        let fifo_size = LPSPI_FIFO_SIZE;

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() - fifo_size == 0 {}
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*word as u32));
        }

        self.blocking_flush()
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        let len = read.len().max(write.len());

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        let fifo_size = LPSPI_FIFO_SIZE;

        for i in 0..len {
            let wb = write[i];

            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() - fifo_size == 0 {}
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(wb as u32));

            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {}
            self.check_status()?;
            let rb = self.info.regs().rdr().read().data() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }

        self.blocking_flush()
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        let fifo_size = LPSPI_FIFO_SIZE;

        for word in data {
            // Wait until we have at least one byte space in the TxFIFO.
            while self.info.regs().fsr().read().txcount() - fifo_size == 0 {}
            self.check_status()?;
            self.info.regs().tdr().write(|w| w.set_data(*word as u32));

            // Wait until we have data in the RxFIFO.
            while self.info.regs().fsr().read().rxcount() == 0 {}
            self.check_status()?;
            *word = self.info.regs().rdr().read().data() as u8;
        }

        self.blocking_flush()
    }

    /// Block execution until Spi is done.
    pub fn blocking_flush(&mut self) -> Result<(), IoError> {
        while self.info.regs().sr().read().mbf() == Mbf::BUSY {}
        self.check_status()
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create a SPI driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T> + 'd>,
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
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
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
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
        miso: Peri<'d, impl MisoPin<T> + 'd>,
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
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
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
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
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
        miso: Peri<'d, impl MisoPin<T> + 'd>,
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
        mosi: Peri<'d, impl MosiPin<T> + 'd>,
        miso: Peri<'d, impl MisoPin<T> + 'd>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, SetupError> {
        sck.mux();
        mosi.mux();
        miso.mux();

        let sck = sck.into();
        let mosi = mosi.into();
        let miso = miso.into();

        // grab request numbers
        let tx_request_number = tx_dma.request_number();
        let rx_request_number = rx_dma.request_number();

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
                tx_request_number,
                rx_request_number,
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
            self.mode.rx_dma.set_request_source(self.mode.rx_request_number);
            self.mode.tx_dma.set_request_source(self.mode.tx_request_number);

            self.mode
                .tx_dma
                .setup_write_zeros_to_peripheral(data.len(), tx_peri_addr, EnableInterrupt::No);

            self.mode
                .rx_dma
                .setup_read_from_peripheral(rx_peri_addr, data, EnableInterrupt::Yes);

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
            self.mode.rx_dma.waker().register(cx.waker());
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
            self.mode.tx_dma.set_request_source(self.mode.tx_request_number);

            // Configure TCD for memory-to-peripheral transfer
            self.mode
                .tx_dma
                .setup_write_to_peripheral(data, peri_addr, EnableInterrupt::Yes);

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
            self.mode.tx_dma.waker().register(cx.waker());
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
            self.mode.rx_dma.set_request_source(self.mode.rx_request_number);
            self.mode.tx_dma.set_request_source(self.mode.tx_request_number);

            self.mode
                .tx_dma
                .setup_write_to_peripheral(write, tx_peri_addr, EnableInterrupt::Yes);

            self.mode
                .rx_dma
                .setup_read_from_peripheral(rx_peri_addr, read, EnableInterrupt::Yes);

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
        let tx_transfer = async {
            core::future::poll_fn(|cx| {
                self.mode.tx_dma.waker().register(cx.waker());

                if self.mode.tx_dma.is_done() {
                    core::task::Poll::Ready(())
                } else {
                    core::task::Poll::Pending
                }
            })
            .await;

            if read.len() > write.len() {
                let write_bytes_len = read.len() - write.len();

                unsafe {
                    self.mode.tx_dma.disable_request();
                    self.mode.tx_dma.clear_done();
                    self.mode.tx_dma.clear_interrupt();

                    self.mode.tx_dma.setup_write_zeros_to_peripheral(
                        write_bytes_len,
                        tx_peri_addr,
                        EnableInterrupt::Yes,
                    );

                    self.mode.tx_dma.enable_request();
                }

                core::future::poll_fn(|cx| {
                    self.mode.tx_dma.waker().register(cx.waker());

                    if self.mode.tx_dma.is_done() {
                        core::task::Poll::Ready(())
                    } else {
                        core::task::Poll::Pending
                    }
                })
                .await
            }
        };

        let rx_transfer = core::future::poll_fn(|cx| {
            self.mode.rx_dma.waker().register(cx.waker());
            if self.mode.tx_dma.is_done() {
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

        // if write > read we should clear any overflow of the FIFO SPI buffer
        if write.len() > read.len() {
            while self.info.regs().fsr().read().rxcount() == 0 {}
            self.check_status()?;
            let _ = self.info.regs().rdr().read().data() as u8;
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
            self.mode.rx_dma.set_request_source(self.mode.rx_request_number);
            self.mode.tx_dma.set_request_source(self.mode.tx_request_number);

            self.mode
                .tx_dma
                .setup_write_to_peripheral(data, tx_peri_addr, EnableInterrupt::Yes);

            self.mode
                .rx_dma
                .setup_read_from_peripheral(rx_peri_addr, data, EnableInterrupt::Yes);

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
            self.mode.tx_dma.waker().register(cx.waker());
            if self.mode.tx_dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        });

        let rx_transfer = core::future::poll_fn(|cx| {
            self.mode.rx_dma.waker().register(cx.waker());
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
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::MASK);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        for word in data {
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

            // dummy data
            self.info.regs().tdr().write(|w| w.set_data(0));
            *word = self.info.regs().rdr().read().data() as u8;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, data: &[u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::MASK);
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
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        let len = read.len().max(write.len());

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        for i in 0..len {
            let wb = write[i];

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
            self.info.regs().tdr().write(|w| w.set_data(wb as u32));

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
            let rb = self.info.regs().rdr().read().data() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }

        self.async_flush().await
    }

    async fn async_transfer_in_place_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
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
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        self.info.regs().cfgr1().modify(|w| w.set_outcfg(Outcfg::TRISTATED));

        let _on_drop = OnDrop::new(|| {
            self.info.regs().der().modify(|w| w.set_rdde(false));
            self.info.regs().cfgr1().modify(|w| w.set_outcfg(Outcfg::TRISTATED));
        });

        for chunk in data.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            self.read_dma_chunk(chunk).await?;
        }

        Ok(())
    }

    async fn async_write_internal(&mut self, data: &[u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::MASK);
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
        if read.is_empty() && write.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        let on_drop = OnDrop::new(|| {
            self.info.regs().der().modify(|w| w.set_tdde(false));
        });

        for (read_chunk, write_chunk) in read
            .chunks_mut(DMA_MAX_TRANSFER_SIZE)
            .zip(write.chunks(DMA_MAX_TRANSFER_SIZE))
        {
            self.transfer_dma_chunk(read_chunk, write_chunk).await?;
        }

        on_drop.defuse();

        self.async_flush().await
    }

    async fn async_transfer_in_place_internal(&mut self, data: &mut [u8]) -> Result<(), IoError> {
        if data.is_empty() {
            return Ok(());
        }

        self.info.regs().tcr().modify(|w| {
            w.set_txmsk(Txmsk::NORMAL);
            w.set_rxmsk(Rxmsk::NORMAL);
        });

        for chunk in data.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            self.transfer_in_place_dma_chunk(chunk).await?;
        }

        self.async_flush().await
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
        return (Prescale::DIVIDEBY1, 0);
    }

    let prescalers = [
        Prescale::DIVIDEBY1,
        Prescale::DIVIDEBY2,
        Prescale::DIVIDEBY4,
        Prescale::DIVIDEBY8,
        Prescale::DIVIDEBY16,
        Prescale::DIVIDEBY32,
        Prescale::DIVIDEBY64,
        Prescale::DIVIDEBY128,
    ];

    let (prescaler, div, _) = prescalers.iter().fold(
        (Prescale::DIVIDEBY1, 0u8, u32::MAX),
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
            IoError::Other => embedded_hal_1::spi::ErrorKind::Other,
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
