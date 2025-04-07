#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::PeripheralType;

use crate::gpio::{AnyPin, PfType, Pull, SealedPin};
use crate::interrupt::{Interrupt, InterruptExt};
use crate::mode::{Blocking, Mode};
use crate::pac::uart::{vals, Uart as Regs};
use crate::Peri;

/// The clock source for the UART.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSel {
    /// Use the low frequency clock.
    ///
    /// The LFCLK runs at 32.768 kHz.
    LfClk,

    /// Use the middle frequency clock.
    ///
    /// The MCLK runs at 4 MHz.
    MfClk,
    // BusClk,
    // BusClk depends on the timer's power domain.
    // This will be implemented later.
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The order of bits in byte.
pub enum BitOrder {
    /// The most significant bit is first.
    MsbFirst,

    /// The least significant bit is first.
    LsbFirst,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Number of data bits
pub enum DataBits {
    /// 5 Data Bits
    DataBits5,

    /// 6 Data Bits
    DataBits6,

    /// 7 Data Bits
    DataBits7,

    /// 8 Data Bits
    DataBits8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Parity
pub enum Parity {
    /// No parity
    ParityNone,

    /// Even Parity
    ParityEven,

    /// Odd Parity
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Number of stop bits
pub enum StopBits {
    /// One stop bit
    Stop1,

    /// Two stop bits
    Stop2,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Config Error
pub enum ConfigError {
    /// Rx or Tx not enabled
    RxOrTxNotEnabled,

    /// The baud rate could not be configured with the given clocks.
    InvalidBaudRate,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct Config {
    /// UART clock source.
    pub clock_source: ClockSel,

    /// Baud rate
    pub baudrate: u32,

    /// Number of data bits.
    pub data_bits: DataBits,

    /// Number of stop bits.
    pub stop_bits: StopBits,

    /// Parity type.
    pub parity: Parity,

    /// The order of bits in a transmitted/received byte.
    pub msb_order: BitOrder,

    /// If true: the `TX` is internally connected to `RX`.
    pub loop_back_enable: bool,

    // TODO: Pending way to check if uart is extended
    // /// If true: [manchester coding] is used.
    // ///
    // /// [manchester coding]: https://en.wikipedia.org/wiki/Manchester_code
    // pub manchester: bool,

    // TODO: majority voting

    // TODO: fifo level select - need power domain info in metapac

    // TODO: glitch suppression
    /// If true: invert TX pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_tx: bool,

    /// If true: invert RX pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_rx: bool,

    /// If true: invert RTS pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_rts: bool,

    /// If true: invert CTS pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_cts: bool,

    /// Set the pull configuration for the TX pin.
    pub tx_pull: Pull,

    /// Set the pull configuration for the RX pin.
    pub rx_pull: Pull,

    /// Set the pull configuration for the RTS pin.
    pub rts_pull: Pull,

    /// Set the pull configuration for the CTS pin.
    pub cts_pull: Pull,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSel::MfClk,
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::Stop1,
            parity: Parity::ParityNone,
            // hardware default
            msb_order: BitOrder::LsbFirst,
            loop_back_enable: false,
            // manchester: false,
            invert_tx: false,
            invert_rx: false,
            invert_rts: false,
            invert_cts: false,
            tx_pull: Pull::None,
            rx_pull: Pull::None,
            rts_pull: Pull::None,
            cts_pull: Pull::None,
        }
    }
}

/// Bidirectional UART Driver, which acts as a combination of [`UartTx`] and [`UartRx`].
///
/// ### Notes on [`embedded_io::Read`]
///
/// `embedded_io::Read` requires guarantees that the base [`UartRx`] cannot provide.
///
/// See [`UartRx`] for more details, and see [`BufferedUart`] and [`RingBufferedUartRx`]
/// as alternatives that do provide the necessary guarantees for `embedded_io::Read`.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

impl<'d, M: Mode> SetConfig for Uart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.tx.set_config(config)?;
        self.rx.set_config(config)
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    Framing,

    Noise,

    Overrun,

    Parity,

    Break,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Framing => "Framing Error",
            Self::Noise => "Noise Error",
            Self::Overrun => "RX Buffer Overrun",
            Self::Parity => "Parity Check Error",
            Self::Break => "Break Error",
        };

        write!(f, "{}", message)
    }
}

impl core::error::Error for Error {}

/// Rx-only UART Driver.
///
/// Can be obtained from [`Uart::split`], or can be constructed independently,
/// if you do not need the transmitting half of the driver.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    rx: Option<Peri<'d, AnyPin>>,
    rts: Option<Peri<'d, AnyPin>>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> SetConfig for UartRx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Create a new rx-only UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Rx. It saves 1 pin .
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, new_pin!(rx, config.rx_pf()), None, config)
    }

    /// Create a new rx-only UART with a request-to-send pin
    pub fn new_blocking_with_rts<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_pf()),
            new_pin!(rts, config.rts_pf()),
            config,
        )
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        if let Some(ref rx) = self.rx {
            rx.update_pf(config.rx_pf());
        }

        if let Some(ref rts) = self.rts {
            rts.update_pf(config.rts_pf());
        }

        reconfigure(self.info, self.state, config)
    }

    /// Perform a blocking read into `buffer`
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;

        for b in buffer {
            // Wait if nothing has arrived yet.
            while r.stat().read().rxfe() {}

            // Prevent the compiler from reading from buffer too early
            compiler_fence(Ordering::Acquire);
            *b = read_with_error(r)?;
        }

        Ok(())
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(&self.info, self.state.clock.load(Ordering::Relaxed), baudrate)
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        self.rx.as_ref().map(|x| x.set_as_disconnected());
        self.rts.as_ref().map(|x| x.set_as_disconnected());
    }
}

/// Tx-only UART Driver.
///
/// Can be obtained from [`Uart::split`], or can be constructed independently,
/// if you do not need the receiving half of the driver.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    tx: Option<Peri<'d, AnyPin>>,
    cts: Option<Peri<'d, AnyPin>>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> SetConfig for UartTx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        reconfigure(self.info, self.state, config)
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Create a new blocking tx-only UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Tx. It saves 1 pin.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, new_pin!(tx, config.tx_pf()), None, config)
    }

    /// Create a new blocking tx-only UART with a clear-to-send pin
    pub fn new_blocking_with_cts<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(tx, config.tx_pf()),
            new_pin!(cts, config.cts_pf()),
            config,
        )
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        if let Some(ref tx) = self.tx {
            tx.update_pf(config.tx_pf());
        }

        if let Some(ref cts) = self.cts {
            cts.update_pf(config.cts_pf());
        }

        reconfigure(self.info, self.state, config)
    }

    /// Perform a blocking UART write
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;

        for &b in buffer {
            // Wait if there is no space
            while !r.stat().read().txfe() {}

            // Prevent the compiler from writing to buffer too early
            compiler_fence(Ordering::Release);
            r.txdata().write(|w| {
                w.set_data(b);
            });
        }

        Ok(())
    }

    /// Block until transmission complete
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let r = self.info.regs;

        // Wait until TX fifo/buffer is empty
        while r.stat().read().txfe() {}
        Ok(())
    }

    /// Send break character
    pub fn send_break(&self) {
        let r = self.info.regs;

        r.lcrh().modify(|w| {
            w.set_brk(true);
        });
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(&self.info, self.state.clock.load(Ordering::Relaxed), baudrate)
    }
}

impl<'d, M: Mode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        self.tx.as_ref().map(|x| x.set_as_disconnected());
        self.cts.as_ref().map(|x| x.set_as_disconnected());
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new blocking bidirectional UART.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_pf()),
            new_pin!(tx, config.tx_pf()),
            None,
            None,
            config,
        )
    }

    /// Create a new bidirectional UART with request-to-send and clear-to-send pins
    pub fn new_blocking_with_rtscts<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_pf()),
            new_pin!(tx, config.tx_pf()),
            new_pin!(rts, config.rts_pf()),
            new_pin!(cts, config.cts_pf()),
            config,
        )
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    /// Perform a blocking write
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Block until transmission complete
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Perform a blocking read into `buffer`
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Split the Uart into a transmitter and receiver, which is
    /// particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Split the Uart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (&mut UartTx<'d, M>, &mut UartRx<'d, M>) {
        (&mut self.tx, &mut self.rx)
    }

    /// Send break character
    pub fn send_break(&self) {
        self.tx.send_break();
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(&self.tx.info, self.tx.state.clock.load(Ordering::Relaxed), baudrate)
    }
}

/// Peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// UART `TX` pin trait
pub trait TxPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `TX`.
    fn pf_num(&self) -> u8;
}

/// UART `RX` pin trait
pub trait RxPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `RX`.
    fn pf_num(&self) -> u8;
}

/// UART `CTS` pin trait
pub trait CtsPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `CTS`.
    fn pf_num(&self) -> u8;
}

/// UART `RTS` pin trait
pub trait RtsPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `RTS`.
    fn pf_num(&self) -> u8;
}

// ==== IMPL types ====

pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) interrupt: Interrupt,
}

pub(crate) struct State {
    /// The clock rate of the UART. This might be configured.
    pub(crate) clock: AtomicU32,
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self {
            info: T::info(),
            state: T::state(),
            rx,
            rts,
            _phantom: PhantomData,
        };
        this.enable_and_configure(&config)?;

        Ok(this)
    }

    fn enable_and_configure(&mut self, config: &Config) -> Result<(), ConfigError> {
        let info = self.info;

        enable(info.regs);
        configure(info, self.state, config, true, self.rts.is_some(), false, false)?;

        Ok(())
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Option<Peri<'d, AnyPin>>,
        cts: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self {
            info: T::info(),
            state: T::state(),
            tx,
            cts,
            _phantom: PhantomData,
        };
        this.enable_and_configure(&config)?;

        Ok(this)
    }

    fn enable_and_configure(&mut self, config: &Config) -> Result<(), ConfigError> {
        let info = self.info;
        let state = self.state;

        enable(info.regs);
        configure(info, state, config, false, false, true, self.cts.is_some())?;

        Ok(())
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<Peri<'d, AnyPin>>,
        tx: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        cts: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let info = T::info();
        let state = T::state();

        let mut this = Self {
            tx: UartTx {
                info,
                state,
                tx,
                cts,
                _phantom: PhantomData,
            },
            rx: UartRx {
                info,
                state,
                rx,
                rts,
                _phantom: PhantomData,
            },
        };
        this.enable_and_configure(&config)?;

        Ok(this)
    }

    fn enable_and_configure(&mut self, config: &Config) -> Result<(), ConfigError> {
        let info = self.rx.info;
        let state = self.rx.state;

        enable(info.regs);
        configure(
            info,
            state,
            config,
            true,
            self.rx.rts.is_some(),
            true,
            self.tx.cts.is_some(),
        )?;

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        Ok(())
    }
}

impl Config {
    fn tx_pf(&self) -> PfType {
        PfType::output(self.tx_pull, self.invert_tx)
    }

    fn rx_pf(&self) -> PfType {
        PfType::input(self.rx_pull, self.invert_rx)
    }

    fn rts_pf(&self) -> PfType {
        PfType::output(self.rts_pull, self.invert_rts)
    }

    fn cts_pf(&self) -> PfType {
        PfType::input(self.rts_pull, self.invert_rts)
    }
}

fn enable(regs: Regs) {
    let gprcm = regs.gprcm(0);

    gprcm.rstctl().write(|w| {
        w.set_resetstkyclr(true);
        w.set_resetassert(true);
        w.set_key(vals::ResetKey::KEY);
    });

    gprcm.pwren().write(|w| {
        w.set_enable(true);
        w.set_key(vals::PwrenKey::KEY);
    });
}

fn configure(
    info: &Info,
    state: &State,
    config: &Config,
    enable_rx: bool,
    enable_rts: bool,
    enable_tx: bool,
    enable_cts: bool,
) -> Result<(), ConfigError> {
    let r = info.regs;

    if !enable_rx && !enable_tx {
        return Err(ConfigError::RxOrTxNotEnabled);
    }

    // SLAU846B says that clocks should be enabled before disabling the uart.
    r.clksel().write(|w| match config.clock_source {
        ClockSel::LfClk => {
            w.set_lfclk_sel(true);
            w.set_mfclk_sel(false);
            w.set_busclk_sel(false);
        }
        ClockSel::MfClk => {
            w.set_mfclk_sel(true);
            w.set_lfclk_sel(false);
            w.set_busclk_sel(false);
        }
    });

    let clock = match config.clock_source {
        ClockSel::LfClk => 32768,
        ClockSel::MfClk => 4_000_000,
    };

    state.clock.store(clock, Ordering::Relaxed);

    info.regs.ctl0().modify(|w| {
        w.set_lbe(config.loop_back_enable);
        w.set_rxe(enable_rx);
        w.set_txe(enable_tx);
        // RXD_OUT_EN and TXD_OUT_EN?
        w.set_menc(false);
        w.set_mode(vals::Mode::UART);
        w.set_rtsen(enable_rts);
        w.set_ctsen(enable_cts);
        // oversampling is set later
        // TODO: config
        w.set_fen(false);
        // TODO: config
        w.set_majvote(false);
        w.set_msbfirst(matches!(config.msb_order, BitOrder::MsbFirst));
    });

    info.regs.lcrh().modify(|w| {
        let eps = if matches!(config.parity, Parity::ParityEven) {
            vals::Eps::EVEN
        } else {
            vals::Eps::ODD
        };

        let wlen = match config.data_bits {
            DataBits::DataBits5 => vals::Wlen::DATABIT5,
            DataBits::DataBits6 => vals::Wlen::DATABIT6,
            DataBits::DataBits7 => vals::Wlen::DATABIT7,
            DataBits::DataBits8 => vals::Wlen::DATABIT8,
        };

        // Used in LIN mode only
        w.set_brk(false);
        w.set_pen(config.parity != Parity::ParityNone);
        w.set_eps(eps);
        w.set_stp2(matches!(config.stop_bits, StopBits::Stop2));
        w.set_wlen(wlen);
        // appears to only be used in RS-485 mode.
        w.set_sps(false);
        // IDLE pattern?
        w.set_sendidle(false);
        // ignore extdir_setup and extdir_hold, only used in RS-485 mode.
    });

    set_baudrate_inner(info.regs, clock, config.baudrate)?;

    r.ctl0().modify(|w| {
        w.set_enable(true);
    });

    Ok(())
}

fn reconfigure(info: &Info, state: &State, config: &Config) -> Result<(), ConfigError> {
    info.interrupt.disable();
    let r = info.regs;
    let ctl0 = r.ctl0().read();
    configure(info, state, config, ctl0.rxe(), ctl0.rtsen(), ctl0.txe(), ctl0.ctsen())?;

    info.interrupt.unpend();
    unsafe { info.interrupt.enable() };

    Ok(())
}

/// Set the baud rate and clock settings.
///
/// This should be done relatively late during configuration since some clock settings are invalid depending on mode.
fn set_baudrate(info: &Info, clock: u32, baudrate: u32) -> Result<(), ConfigError> {
    let r = info.regs;

    info.interrupt.disable();

    // Programming baud rate requires that the peripheral is disabled
    critical_section::with(|_cs| {
        r.ctl0().modify(|w| {
            w.set_enable(false);
        });
    });

    // Wait for end of transmission per suggestion in SLAU 845 section 18.3.28
    while !r.stat().read().txfe() {}

    set_baudrate_inner(r, clock, baudrate)?;

    critical_section::with(|_cs| {
        r.ctl0().modify(|w| {
            w.set_enable(true);
        });
    });

    info.interrupt.unpend();
    unsafe { info.interrupt.enable() };

    Ok(())
}

fn set_baudrate_inner(regs: Regs, clock: u32, baudrate: u32) -> Result<(), ConfigError> {
    // Quoting SLAU846 section 18.2.3.4:
    // "When IBRD = 0, FBRD is ignored and no data gets transferred by the UART."
    const MIN_IBRD: u16 = 1;

    // FBRD can be 0
    // FBRD is at most a 6-bit number.
    const MAX_FBRD: u8 = 2_u8.pow(6);

    const DIVS: [(u8, vals::Clkdiv); 8] = [
        (1, vals::Clkdiv::DIV_BY_1),
        (2, vals::Clkdiv::DIV_BY_2),
        (3, vals::Clkdiv::DIV_BY_3),
        (4, vals::Clkdiv::DIV_BY_4),
        (5, vals::Clkdiv::DIV_BY_5),
        (6, vals::Clkdiv::DIV_BY_6),
        (7, vals::Clkdiv::DIV_BY_7),
        (8, vals::Clkdiv::DIV_BY_8),
    ];

    // Quoting from SLAU 846 section 18.2.3.4:
    // "Select oversampling by 3 or 8 to achieve higher speed with UARTclk/8 or UARTclk/3. In this case
    //  the receiver tolerance to clock deviation is reduced."
    //
    // "Select oversampling by 16 to increase the tolerance of the receiver to clock deviations. The
    //  maximum speed is limited to UARTclk/16."
    //
    // Based on these requirements, prioritize higher oversampling first to increase tolerance to clock
    // deviation. If no valid BRD value can be found satisifying the highest sample rate, then reduce
    // sample rate until valid parameters are found.
    const OVS: [(u8, vals::Hse); 3] = [(16, vals::Hse::OVS16), (8, vals::Hse::OVS8), (3, vals::Hse::OVS3)];

    // 3x oversampling is not supported with manchester coding, DALI or IrDA.
    let x3_invalid = {
        let ctl0 = regs.ctl0().read();
        let irctl = regs.irctl().read();

        ctl0.menc() || matches!(ctl0.mode(), vals::Mode::DALI) || irctl.iren()
    };
    let mut found = None;

    'outer: for &(oversampling, hse_value) in &OVS {
        if matches!(hse_value, vals::Hse::OVS3) && x3_invalid {
            continue;
        }

        // Verify that the selected oversampling does not require a clock faster than what the hardware
        // is provided.
        let Some(min_clock) = baudrate.checked_mul(oversampling as u32) else {
            trace!(
                "{}x oversampling would cause overflow for clock: {} Hz",
                oversampling,
                clock
            );
            continue;
        };

        if min_clock > clock {
            trace!("{} oversampling is too high for clock: {} Hz", oversampling, clock);
            continue;
        }

        for &(div, div_value) in &DIVS {
            trace!(
                "Trying div: {}, oversampling {} for {} baud",
                div,
                oversampling,
                baudrate
            );

            let Some((ibrd, fbrd)) = calculate_brd(clock, div, baudrate, oversampling) else {
                trace!("Calculating BRD overflowed: trying another divider");
                continue;
            };

            if ibrd < MIN_IBRD || fbrd > MAX_FBRD {
                trace!("BRD was invalid: trying another divider");
                continue;
            }

            found = Some((hse_value, div_value, ibrd, fbrd));
            break 'outer;
        }
    }

    let Some((hse, div, ibrd, fbrd)) = found else {
        return Err(ConfigError::InvalidBaudRate);
    };

    regs.clkdiv().write(|w| {
        w.set_ratio(div);
    });

    regs.ibrd().write(|w| {
        w.set_divint(ibrd);
    });

    regs.fbrd().write(|w| {
        w.set_divfrac(fbrd);
    });

    regs.ctl0().modify(|w| {
        w.set_hse(hse);
    });

    Ok(())
}

/// Calculate the integer and fractional parts of the `BRD` value.
///
/// Returns [`None`] if calculating this results in overflows.
///
/// Values returned are `(ibrd, fbrd)`
fn calculate_brd(clock: u32, div: u8, baud: u32, oversampling: u8) -> Option<(u16, u8)> {
    use fixed::types::U26F6;

    // Calculate BRD according to SLAU 846 section 18.2.3.4.
    //
    // BRD is a 22-bit value with 16 integer bits and 6 fractional bits.
    //
    // uart_clock = clock / div
    // brd = ibrd.fbrd = uart_clock / (oversampling * baud)"
    //
    // It is tempting to rearrange the equation such that there is only a single division in
    // order to reduce error. However this is wrong since the denominator ends up being too
    // small to represent in 6 fraction bits. This means that FBRD would always be 0.
    //
    // Calculations are done in a U16F6 format. However the fixed crate has no such representation.
    // U26F6 is used since it has the same number of fractional bits and we verify at the end that
    // the integer part did not overflow.
    let clock = U26F6::from_num(clock);
    let div = U26F6::from_num(div);
    let oversampling = U26F6::from_num(oversampling);
    let baud = U26F6::from_num(baud);

    let uart_clock = clock.checked_div(div)?;

    // oversampling * baud
    let denom = oversampling.checked_mul(baud)?;
    // uart_clock / (oversampling * baud)
    let brd = uart_clock.checked_div(denom)?;

    // Checked is used to determine overflow in the 10 most singificant bits since the
    // actual representation of BRD is U16F6.
    let ibrd = brd.checked_to_num::<u16>()?;

    // We need to scale FBRD's representation to an integer.
    let fbrd_scale = U26F6::from_num(2_u32.checked_pow(U26F6::FRAC_NBITS)?);

    // It is suggested that 0.5 is added to ensure that any fractional parts round up to the next
    // integer. If it doesn't round up then it'll get discarded which is okay.
    let half = U26F6::from_num(1) / U26F6::from_num(2);
    // fbrd = INT(((FRAC(BRD) * 64) + 0.5))
    let fbrd = brd
        .frac()
        .checked_mul(fbrd_scale)?
        .checked_add(half)?
        .checked_to_num::<u8>()?;

    Some((ibrd, fbrd))
}

fn read_with_error(r: Regs) -> Result<u8, Error> {
    let rx = r.rxdata().read();

    if rx.frmerr() {
        return Err(Error::Framing);
    } else if rx.parerr() {
        return Err(Error::Parity);
    } else if rx.brkerr() {
        return Err(Error::Break);
    } else if rx.ovrerr() {
        return Err(Error::Overrun);
    } else if rx.nerr() {
        return Err(Error::Noise);
    }

    Ok(rx.data())
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

macro_rules! impl_uart_instance {
    ($instance: ident) => {
        impl crate::uart::SealedInstance for crate::peripherals::$instance {
            fn info() -> &'static crate::uart::Info {
                use crate::interrupt::typelevel::Interrupt;
                use crate::uart::Info;

                const INFO: Info = Info {
                    regs: crate::pac::$instance,
                    interrupt: crate::interrupt::typelevel::$instance::IRQ,
                };
                &INFO
            }

            fn state() -> &'static crate::uart::State {
                use crate::interrupt::typelevel::Interrupt;
                use crate::uart::State;

                static STATE: State = State {
                    clock: core::sync::atomic::AtomicU32::new(0),
                };
                &STATE
            }
        }

        impl crate::uart::Instance for crate::peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$instance;
        }
    };
}

macro_rules! impl_uart_tx_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::uart::TxPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

macro_rules! impl_uart_rx_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::uart::RxPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

macro_rules! impl_uart_cts_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::uart::CtsPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

macro_rules! impl_uart_rts_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::uart::RtsPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::calculate_brd;

    /// This is a smoke test based on the example in SLAU 846 section 18.2.3.4.
    #[test]
    fn datasheet() {
        let brd = calculate_brd(40_000_000, 1, 19200, 16);

        assert!(matches!(brd, Some((130, 13))));
    }
}
