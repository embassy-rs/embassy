//! LPSPI Controller Driver.

use core::marker::PhantomData;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::Peri;
pub use embedded_hal_1::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode, Phase, Polarity};
use nxp_pac::lpspi::vals::{Cpha, Cpol, Lsbf, Master, Mbf, Pcspol, Pincfg, Prescale, Rrf, Rtf, Rxmsk, Txmsk};

use super::{Async, Blocking, Info, Instance, MisoPin, Mode as IoMode, MosiPin, SckPin};
use crate::clocks::periph_helpers::{Div4, LpspiClockSel, LpspiConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
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
            w.set_framesz(8);

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

        Self::new_inner(_peri, sck, Some(mosi), Some(miso), config)
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

        Self::new_inner(_peri, sck, Some(mosi), None, config)
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

        Self::new_inner(_peri, sck, None, Some(miso), config)
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

        Self::new_inner(_peri, sck, Some(mosi), Some(miso), config)
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

        Self::new_inner(_peri, sck, Some(mosi), None, config)
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

        Self::new_inner(_peri, sck, None, Some(miso), config)
    }

    /// Read data from Spi async execution until done.
    pub async fn async_read(&mut self, data: &mut [u8]) -> Result<(), IoError> {
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

    /// Write data to Spi async execution until done.
    pub async fn async_write(&mut self, data: &[u8]) -> Result<(), IoError> {
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

    /// Transfer data to SPI async execution until done.
    pub async fn async_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), IoError> {
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

    /// Transfer data in place to SPI async execution until done.
    pub async fn async_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), IoError> {
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

impl<'d> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, Async> {
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
