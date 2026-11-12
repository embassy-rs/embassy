//! Low-power UART (LPUART) driver for the ASR6601.
//!
//! Register programming follows the vendor `tremo_lpuart` driver. Pin mux
//! numbers are supplied explicitly by the caller as `(pin, AlternateFunction)`
//! pairs; see [`pins`] for the datasheet mappings used by the SDK examples.
//!
//! # Clocking
//!
//! LPUART is clocked from the always-on domain (`XO32K`, `RCO32K`, or
//! `RCO4M`). Select the source with [`Config::clock_source`]. The selected
//! oscillator must already be running; this driver does not power oscillators.
//! When the system clock uses `XO32K`, or when `time-driver-lptim0` is active,
//! `XO32K` is typically already enabled by [`crate::init`].
//!
//! # Wakeup (STOP)
//!
//! The SDK exposes three mutually exclusive CR0 wakeup enables (see
//! [`Wakeup`]). The `lpuart_wakeup_stop` example enables
//! [`Wakeup::LowLevel`] and then enters STOP3; the LoRaWAN AT path uses
//! [`Wakeup::RxDone`] with the RX-done interrupt. Wakeup only affects the
//! CR0 enable bits — entering STOP is done through the power driver.
//!
//! # Unsafe contracts
//!
//! - [`InterruptHandler::on_interrupt`] may be called only from the vector
//!   bound through [`crate::bind_interrupts!`] for [`InterruptHandler`].
//! - DMA helpers require exclusive channel ownership and a buffer that stays
//!   valid and DMA-accessible until the call returns.
//! - Constructing a driver steals the PAC LPUART block. The owned [`Peri`]
//!   token is the exclusivity proof; do not also use `pac::Lpuart::steal()`
//!   from application code while the driver is alive.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::interrupt::Priority;
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Channel as DmaChannel, Request, TransferOptions};
use crate::gpio::{AlternateFunction, AnyPin, Flex, Pin};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::lpuart::RegisterBlock;
use crate::rcc::{self, Peripheral, RCO4M_HZ, RCO32K_HZ, XO32K_HZ};
pub use crate::serial::{DataBits, FlowControl, StopBits};
use crate::{interrupt, pac, peripherals};

// Width of the 12-bit BAUD_RATE_INT field.
const BAUD_INT_MAX: u32 = 0x0fff;

static STATE: State = State::new();

struct State {
    tx_waker: AtomicWaker,
    rx_waker: AtomicWaker,
    /// Number of live [`LpuartTx`] / [`LpuartRx`] halves.
    refcount: AtomicU8,
}

impl State {
    const fn new() -> Self {
        Self {
            tx_waker: AtomicWaker::new(),
            rx_waker: AtomicWaker::new(),
            refcount: AtomicU8::new(0),
        }
    }
}

struct Info {
    base: usize,
    rcc: Peripheral,
    dma_tx: Request,
    dma_rx: Request,
    state: &'static State,
}

impl Info {
    fn regs(&self) -> &'static RegisterBlock {
        unsafe { &*(self.base as *const RegisterBlock) }
    }
}

/// LPUART interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl Handler<interrupt::typelevel::LPUART> for InterruptHandler {
    unsafe fn on_interrupt() {
        let info = info();
        let regs = info.regs();
        let cr1 = regs.cr1().read();
        let sr0 = read_sr0(regs);
        let sr1 = regs.sr1().read();

        let tx_pending = (cr1.tx_empty_int().bit_is_set() && sr1.tx_empty_state().bit_is_set())
            || (cr1.tx_done_int().bit_is_set() && sr1.tx_done_state().bit_is_set());
        if tx_pending {
            set_tx_interrupts(regs, false);
            info.state.tx_waker.wake();
        }

        let rx_pending = (cr1.rx_not_empty_int().bit_is_set() && sr1.rx_not_empty_state().bit_is_set())
            || (cr1.rx_done_int().bit_is_set() && sr0.rx_done_state().bit_is_set())
            || (cr1.start_invalid_int().bit_is_set() && sr0.start_invalid_state().bit_is_set())
            || (cr1.parity_err_int().bit_is_set() && sr0.parity_err_state().bit_is_set())
            || (cr1.stop_err_int().bit_is_set() && sr0.stop_err_state().bit_is_set())
            || (cr1.rx_overflow_int().bit_is_set() && sr0.rx_overflow_state().bit_is_set());
        if rx_pending {
            set_rx_interrupts(regs, false);
            info.state.rx_waker.wake();
        }
    }
}

/// Documented datasheet / SDK LPUART pin mux values.
///
/// Pass these with the matching pad to constructors, for example
/// `(p.PC15, pins::TX_PC15)`.
pub mod pins {
    use crate::gpio::AlternateFunction;

    /// `GPIO47` / `PC15` AF2 — `LPUART_TX` (SDK `lpuart_txrx` example).
    pub const TX_PC15: AlternateFunction = AlternateFunction::Function2;

    /// `GPIO60` / `PD12` AF2 — `LPUART_RX` (SDK examples).
    pub const RX_PD12: AlternateFunction = AlternateFunction::Function2;

    /// `GPIO62` / `PD14` AF2 — `LPUART_RX`.
    pub const RX_PD14: AlternateFunction = AlternateFunction::Function2;

    /// `GPIO58` / `PD10` AF5 — `LPUART_RX`.
    pub const RX_PD10: AlternateFunction = AlternateFunction::Function5;

    /// `GPIO59` / `PD11` AF5 — `LPUART_RTS`.
    ///
    /// The datasheet does not list an `LPUART_CTS` mux entry.
    pub const RTS_PD11: AlternateFunction = AlternateFunction::Function5;
}

/// Parity selection (`lpuart_parity_t`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// Even parity (`LPUART_PARITY_EVEN`).
    Even,
    /// Odd parity (`LPUART_PARITY_ODD`).
    Odd,
    /// Sticky 0 parity (`LPUART_PARITY_STICK_0`).
    Stick0,
    /// Sticky 1 parity (`LPUART_PARITY_STICK_1`).
    Stick1,
    /// No parity (`LPUART_PARITY_NONE`).
    None,
}

/// CR0 wakeup source (`lpuart_init_t` wakeup flags).
///
/// The vendor `lpuart_init` treats these as mutually exclusive (`if` /
/// `else if`). At most one bit is programmed into CR0.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Wakeup {
    /// No wakeup enable bit.
    #[default]
    None,
    /// `LPUART_CR0_LOW_LEVEL_WAKEUP` — wake on RX low level.
    ///
    /// Used by `projects/*/examples/lpuart/lpuart_wakeup_stop` before
    /// `pwr_deepsleep_wfi(PWR_LP_MODE_STOP3)`.
    LowLevel,
    /// `LPUART_CR0_START_WAKEUP` — wake on a start bit.
    StartBit,
    /// `LPUART_CR0_RX_DONE_WAKEUP` — wake when a character reception completes.
    ///
    /// Used by `projects/*/examples/lorawan/lorawan_at` together with the
    /// RX-done interrupt (`LPUART_CR1_RX_DONE_INT`).
    RxDone,
}

/// LPUART functional clock source (`rcc_lpuart_clk_source_t`).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSource {
    /// External 32.768 kHz crystal (`RCC_LPUART_CLK_SOURCE_XO32K`).
    #[default]
    Xo32k,
    /// Internal 32 kHz RC (`RCC_LPUART_CLK_SOURCE_RCO32K`).
    Rco32k,
    /// Internal RC reported as 3.6 MHz by the vendor (`RCC_LPUART_CLK_SOURCE_RCO4M`).
    Rco4m,
}

impl ClockSource {
    const fn frequency_hz(self) -> u32 {
        match self {
            Self::Xo32k => XO32K_HZ,
            Self::Rco32k => RCO32K_HZ,
            Self::Rco4m => RCO4M_HZ,
        }
    }
}

/// Baud rates listed as macros in `tremo_lpuart.h`.
///
/// Other rates are accepted when they fit the baud divider; these constants
/// match the SDK naming.
pub mod baudrate {
    /// 110 baud.
    pub const B110: u32 = 110;
    /// 300 baud.
    pub const B300: u32 = 300;
    /// 600 baud.
    pub const B600: u32 = 600;
    /// 1200 baud.
    pub const B1200: u32 = 1200;
    /// 2400 baud.
    pub const B2400: u32 = 2400;
    /// 4800 baud.
    pub const B4800: u32 = 4800;
    /// 9600 baud (default in SDK examples).
    pub const B9600: u32 = 9600;
}

/// LPUART configuration.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate in bits per second.
    pub baudrate: u32,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Parity.
    pub parity: Parity,
    /// Number of stop bits.
    pub stop_bits: StopBits,
    /// Hardware flow control.
    pub flow_control: FlowControl,
    /// STOP / deep-sleep wakeup source programmed into CR0.
    pub wakeup: Wakeup,
    /// Functional clock source used for baud generation.
    pub clock_source: ClockSource,
}

impl Config {
    /// Create a 9600 8N1 configuration matching the SDK examples.
    pub const fn new() -> Self {
        Self {
            baudrate: baudrate::B9600,
            data_bits: DataBits::DataBits8,
            parity: Parity::None,
            stop_bits: StopBits::STOP1,
            flow_control: FlowControl::None,
            wakeup: Wakeup::None,
            clock_source: ClockSource::Xo32k,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Baud rate cannot be represented for the selected LPUART clock.
    Baudrate,
    /// Neither an RX nor a TX pin was supplied.
    NoRxOrTx,
    /// Flow-control pins required by [`Config::flow_control`] were missing.
    MissingFlowControlPin,
    /// Enabling, resetting, or selecting the LPUART clock failed.
    Rcc(rcc::Error),
}

/// LPUART transfer / framing error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// RX overflow (`LPUART_SR0_RX_OVERFLOW_STATE`).
    Overrun,
    /// Stop-bit error (`LPUART_SR0_STOP_ERR_STATE`).
    Framing,
    /// Parity error (`LPUART_SR0_PARITY_ERR_STATE`).
    Parity,
    /// Invalid start condition (`LPUART_SR0_START_INVALID_STATE`).
    StartInvalid,
    /// DMA helper failed.
    Dma(dma::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => write!(f, "LPUART overrun"),
            Self::Framing => write!(f, "LPUART framing error"),
            Self::Parity => write!(f, "LPUART parity error"),
            Self::StartInvalid => write!(f, "LPUART start invalid"),
            Self::Dma(_) => write!(f, "LPUART DMA error"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Self::Overrun => embedded_io::ErrorKind::OutOfMemory,
            Self::Framing | Self::Parity | Self::StartInvalid => embedded_io::ErrorKind::InvalidData,
            Self::Dma(_) => embedded_io::ErrorKind::Other,
        }
    }
}

fn info() -> &'static Info {
    static INFO: Info = Info {
        base: 0x4000_5000,
        rcc: Peripheral::Lpuart,
        dma_tx: Request::LpuartTx,
        dma_rx: Request::LpuartRx,
        state: &STATE,
    };
    &INFO
}

fn erase_pin<'d>(
    pin: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
) -> Option<(Peri<'d, AnyPin>, AlternateFunction)> {
    pin.map(|(p, af)| (p.into(), af))
}

fn configure_pin(pin: Peri<'_, impl Pin>, af: AlternateFunction, output: bool) -> Flex<'_> {
    let mut flex = Flex::new(pin);
    if output {
        flex.set_as_output();
    } else {
        flex.set_as_input();
    }
    flex.set_alternate_function(af);
    flex
}

fn wait_cr0_writable(regs: &RegisterBlock) {
    let ready = || {
        let sr1 = regs.sr1().read();
        sr1.write_sr0_state().bit_is_set() && sr1.write_cr0_state().bit_is_set()
    };
    while !ready() {}
}

fn wait_sr0_readable(regs: &RegisterBlock) {
    while !regs.sr1().read().write_sr0_state().bit_is_set() {}
}

fn read_sr0(regs: &RegisterBlock) -> pac::lpuart::sr0::R {
    wait_sr0_readable(regs);
    regs.sr0().read()
}

/// Clear latched SR0 receive-error flags (write-1-to-clear per flag).
fn clear_sr0_errors(regs: &RegisterBlock, sr0: &pac::lpuart::sr0::R) {
    wait_cr0_writable(regs);
    let start_invalid = sr0.start_invalid_state().bit_is_set();
    let parity_err = sr0.parity_err_state().bit_is_set();
    let stop_err = sr0.stop_err_state().bit_is_set();
    let rx_overflow = sr0.rx_overflow_state().bit_is_set();
    unsafe {
        regs.sr0().write_with_zero(|w| {
            w.start_invalid_state().bit(start_invalid);
            w.parity_err_state().bit(parity_err);
            w.stop_err_state().bit(stop_err);
            w.rx_overflow_state().bit(rx_overflow)
        });
    }
    wait_cr0_writable(regs);
}

/// Enable or disable the receive interrupt group (RX-done, invalid start,
/// parity/stop errors, overflow, RX-not-empty).
fn set_rx_interrupts(regs: &RegisterBlock, enable: bool) {
    regs.cr1().modify(|_, w| {
        w.rx_done_int().bit(enable);
        w.start_invalid_int().bit(enable);
        w.parity_err_int().bit(enable);
        w.stop_err_int().bit(enable);
        w.rx_overflow_int().bit(enable);
        w.rx_not_empty_int().bit(enable)
    });
}

/// Enable or disable the transmit interrupt group (TX-empty, TX-done).
fn set_tx_interrupts(regs: &RegisterBlock, enable: bool) {
    regs.cr1().modify(|_, w| {
        w.tx_empty_int().bit(enable);
        w.tx_done_int().bit(enable)
    });
}

/// Enable or disable the TX-empty interrupt alone.
fn set_tx_empty_interrupt(regs: &RegisterBlock, enable: bool) {
    regs.cr1().modify(|_, w| w.tx_empty_int().bit(enable));
}

/// Enable or disable the TX-done interrupt alone.
fn set_tx_done_interrupt(regs: &RegisterBlock, enable: bool) {
    regs.cr1().modify(|_, w| w.tx_done_int().bit(enable));
}

fn calc_baud(freq: u32, baud: u32) -> Option<(u32, u32)> {
    if baud == 0 || freq < baud {
        return None;
    }
    let ibaud = freq / baud;
    if ibaud > BAUD_INT_MAX {
        return None;
    }
    // Vendor: fbaud = ((freq % baud) * 16 + baud / 2) / baud
    let fbaud = ((freq % baud) * 16 + baud / 2) / baud;
    Some((ibaud, fbaud & 0xf))
}

fn set_clock_source(source: ClockSource) {
    // Mirror `rcc_set_lpuart_clk_source`: only the functional CGR0 gate must be
    // off while the selector is written (AON stays untouched).
    critical_section::with(|_| {
        let rcc_regs = unsafe { pac::Rcc::steal() };
        if rcc_regs.sr1().read().lpuart_clk_en_sync().bit_is_set() {
            rcc_regs.cgr0().modify(|_, w| w.lpuart_clk_en().clear_bit());
        }
    });
    while unsafe { pac::Rcc::steal() }
        .sr1()
        .read()
        .lpuart_clk_en_sync()
        .bit_is_set()
    {}

    critical_section::with(|_| {
        let rcc_regs = unsafe { pac::Rcc::steal() };
        rcc_regs.cr1().modify(|_, w| {
            w.lpuart_clk_sel().variant(match source {
                ClockSource::Xo32k => pac::rcc::cr1::LpuartClkSel::Xo32k,
                ClockSource::Rco32k => pac::rcc::cr1::LpuartClkSel::Rco32k,
                ClockSource::Rco4m => pac::rcc::cr1::LpuartClkSel::Rco4m,
            })
        });
    });
}

fn enable_clock(source: ClockSource) -> Result<(), ConfigError> {
    set_clock_source(source);
    rcc::enable_peripheral(Peripheral::Lpuart).map_err(ConfigError::Rcc)?;
    rcc::reset_peripheral(Peripheral::Lpuart).map_err(ConfigError::Rcc)?;
    Ok(())
}

fn apply_config(regs: &RegisterBlock, config: &Config, has_rx: bool, has_tx: bool) -> Result<(), ConfigError> {
    let (ibaud, fbaud) = calc_baud(config.clock_source.frequency_hz(), config.baudrate).ok_or(ConfigError::Baudrate)?;

    let data_len = match config.data_bits {
        DataBits::DataBits5 => 0,
        DataBits::DataBits6 => 1,
        DataBits::DataBits7 => 2,
        DataBits::DataBits8 => 3,
    };
    let parity = match config.parity {
        Parity::Even => pac::lpuart::cr0::LpuartParityCfg::Even,
        Parity::Odd => pac::lpuart::cr0::LpuartParityCfg::Odd,
        Parity::Stick0 => pac::lpuart::cr0::LpuartParityCfg::Bit0,
        Parity::Stick1 => pac::lpuart::cr0::LpuartParityCfg::Bit1,
        Parity::None => pac::lpuart::cr0::LpuartParityCfg::None,
    };
    let stop = match config.stop_bits {
        StopBits::STOP1 => pac::lpuart::cr0::LpuartStopLen::Value1,
        StopBits::STOP2 => pac::lpuart::cr0::LpuartStopLen::Value2,
    };

    // CR0 lives behind an asynchronous clock-domain handshake (SR1
    // WRITE_CR0/WRITE_SR0): a write issued while the previous one is still
    // being synchronised is silently dropped, so wait before and after every
    // CR0 access (vendor `lpuart_init` / `lpuart_config_rx` do the same).
    //
    // Full-register write like the vendor init: frame fields, baud dividers
    // and wakeup enables; RX/RTS enables stay cleared until the direction
    // setup below.
    wait_cr0_writable(regs);
    unsafe {
        regs.cr0().write_with_zero(|w| {
            w.lpuart_baud_rate_int().bits(ibaud as u16);
            w.lpuart_baud_rate_fra().bits(fbaud as u8);
            w.lpuart_data_len().bits(data_len);
            w.lpuart_parity_cfg().variant(parity);
            w.lpuart_stop_len().variant(stop);
            w.low_level_wakeup().bit(matches!(config.wakeup, Wakeup::LowLevel));
            w.start_wakeup().bit(matches!(config.wakeup, Wakeup::StartBit));
            w.rx_done_wakeup().bit(matches!(config.wakeup, Wakeup::RxDone))
        });
    }
    wait_cr0_writable(regs);

    let rts = matches!(config.flow_control, FlowControl::Rts | FlowControl::RtsCts);
    let cts = matches!(config.flow_control, FlowControl::Cts | FlowControl::RtsCts);
    if rts {
        wait_cr0_writable(regs);
        regs.cr0().modify(|_, w| w.rts_enable().set_bit());
        wait_cr0_writable(regs);
    }
    regs.cr1().modify(|_, w| w.cts_enable().bit(cts));

    // The full CR0 write above clears RX_ENABLE. Restore the directions that
    // were actually configured on this driver.
    set_tx_enable(regs, has_tx);
    set_rx_enable(regs, has_rx);

    Ok(())
}

/// Functional clock currently selected in `RCC.CR1`, for baud re-computation.
fn current_clock_source() -> ClockSource {
    let sel = unsafe { pac::Rcc::steal() }.cr1().read().lpuart_clk_sel();
    if sel.is_rco4m() {
        ClockSource::Rco4m
    } else if sel.is_rco32k() {
        ClockSource::Rco32k
    } else {
        ClockSource::Xo32k
    }
}

fn set_rx_enable(regs: &RegisterBlock, enable: bool) {
    wait_cr0_writable(regs);
    regs.cr0().modify(|_, w| w.rx_enable().bit(enable));
    wait_cr0_writable(regs);
}

fn set_tx_enable(regs: &RegisterBlock, enable: bool) {
    regs.cr1().modify(|_, w| w.tx_enable().bit(enable));
}

fn set_dma_req(regs: &RegisterBlock, tx: bool, enable: bool) {
    regs.cr1().modify(|_, w| {
        if tx {
            w.tx_dma().bit(enable)
        } else {
            w.rx_dma().bit(enable)
        }
    });
}

fn shutdown(info: &'static Info) {
    let regs = info.regs();
    set_tx_enable(regs, false);
    set_rx_enable(regs, false);
    set_tx_interrupts(regs, false);
    set_rx_interrupts(regs, false);
    set_dma_req(regs, true, false);
    set_dma_req(regs, false, false);
    let _ = rcc::disable_peripheral(info.rcc);
}

fn take_rx_errors(regs: &RegisterBlock) -> Result<(), Error> {
    let sr0 = read_sr0(regs);
    let start_invalid = sr0.start_invalid_state().bit_is_set();
    let parity_err = sr0.parity_err_state().bit_is_set();
    let stop_err = sr0.stop_err_state().bit_is_set();
    let rx_overflow = sr0.rx_overflow_state().bit_is_set();
    if !(start_invalid || parity_err || stop_err || rx_overflow) {
        return Ok(());
    }
    clear_sr0_errors(regs, &sr0);
    if rx_overflow {
        return Err(Error::Overrun);
    }
    if parity_err {
        return Err(Error::Parity);
    }
    if stop_err {
        return Err(Error::Framing);
    }
    Err(Error::StartInvalid)
}

fn rx_not_empty(regs: &RegisterBlock) -> bool {
    regs.sr1().read().rx_not_empty_state().bit_is_set()
}

fn tx_empty(regs: &RegisterBlock) -> bool {
    regs.sr1().read().tx_empty_state().bit_is_set()
}

fn tx_done(regs: &RegisterBlock) -> bool {
    regs.sr1().read().tx_done_state().bit_is_set()
}

fn clear_tx_done(regs: &RegisterBlock) {
    // Vendor `lpuart_clear_tx_done_status` sets the TX_DONE bit in SR1.
    unsafe {
        regs.sr1().write_with_zero(|w| w.tx_done_state().set_bit());
    }
}

fn read_data(regs: &RegisterBlock) -> u8 {
    regs.data().read().lpuart_data().bits()
}

fn write_data(regs: &RegisterBlock, byte: u8) {
    unsafe {
        regs.data().write_with_zero(|w| w.lpuart_data().bits(byte));
    }
}

/// Bidirectional LPUART driver.
pub struct Lpuart<'d, M: Mode> {
    tx: LpuartTx<'d, M>,
    rx: LpuartRx<'d, M>,
}

/// Transmit half of an LPUART.
pub struct LpuartTx<'d, M: Mode> {
    info: &'static Info,
    has_tx: bool,
    has_rx: bool,
    // `Flex` (not bare `Peri`) so dropping the half deconfigures the pins
    // back to analog via `Flex::drop`.
    _pin: Option<Flex<'d>>,
    _rts: Option<Flex<'d>>,
    _mode: PhantomData<&'d mut M>,
}

/// Receive half of an LPUART.
pub struct LpuartRx<'d, M: Mode> {
    info: &'static Info,
    has_tx: bool,
    has_rx: bool,
    _pin: Option<Flex<'d>>,
    _cts: Option<Flex<'d>>,
    _mode: PhantomData<&'d mut M>,
}

impl<'d> Lpuart<'d, Blocking> {
    /// Create a blocking LPUART.
    ///
    /// `tx` / `rx` are `(pin, alternate_function)` pairs. See [`pins`] for the
    /// datasheet AF numbers. Supply `None` for unused directions.
    pub fn new_blocking(
        peri: Peri<'d, peripherals::LPUART>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, erase_pin(tx), erase_pin(rx), None, None, config)
    }

    /// Create a blocking LPUART with explicit RTS/CTS pins and AF selectors.
    pub fn new_blocking_with_rtscts(
        peri: Peri<'d, peripherals::LPUART>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            erase_pin(tx),
            erase_pin(rx),
            erase_pin(rts),
            erase_pin(cts),
            config,
        )
    }
}

impl<'d> Lpuart<'d, Async> {
    /// Create an interrupt-driven asynchronous LPUART.
    ///
    /// `irq` proves the LPUART vector is bound to [`InterruptHandler`].
    pub fn new(
        peri: Peri<'d, peripherals::LPUART>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<interrupt::typelevel::LPUART, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let this = Self::new_inner(peri, erase_pin(tx), erase_pin(rx), None, None, config)?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Create an asynchronous LPUART with explicit RTS/CTS pins and AF selectors.
    pub fn new_with_rtscts(
        peri: Peri<'d, peripherals::LPUART>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<interrupt::typelevel::LPUART, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let this = Self::new_inner(
            peri,
            erase_pin(tx),
            erase_pin(rx),
            erase_pin(rts),
            erase_pin(cts),
            config,
        )?;
        this.enable_interrupts();
        Ok(this)
    }

    /// Write `buffer` using interrupt-driven TX.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Read into `buffer` until it is full using interrupt-driven RX.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Wait until the last byte has left the transmitter (`TX_DONE`).
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Write `buffer` with a DMA channel (memory-to-peripheral).
    pub async fn write_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write_dma(channel, buffer).await
    }

    /// Read `buffer.len()` bytes with a DMA channel (peripheral-to-memory).
    pub async fn read_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read_dma(channel, buffer).await
    }
}

impl<'d, M: Mode> Lpuart<'d, M> {
    /// Arm the NVIC vector for interrupt-driven use.
    ///
    /// Called by the async constructors only; blocking instances never
    /// enable the vector.
    fn enable_interrupts(&self) {
        interrupt::typelevel::LPUART::unpend();
        interrupt::typelevel::LPUART::set_priority(Priority::P3);
        unsafe {
            interrupt::typelevel::LPUART::enable();
        }
    }

    fn new_inner(
        _peri: Peri<'d, peripherals::LPUART>,
        tx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        cts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        match config.flow_control {
            FlowControl::Rts if rts.is_none() => return Err(ConfigError::MissingFlowControlPin),
            FlowControl::Cts if cts.is_none() => return Err(ConfigError::MissingFlowControlPin),
            FlowControl::RtsCts if rts.is_none() || cts.is_none() => {
                return Err(ConfigError::MissingFlowControlPin);
            }
            _ => {}
        }

        let has_rx = rx.is_some();
        let has_tx = tx.is_some();
        if !has_rx && !has_tx {
            return Err(ConfigError::NoRxOrTx);
        }

        let info = info();
        enable_clock(config.clock_source)?;

        let rx_pin = rx.map(|(p, af)| configure_pin(p, af, false));
        let tx_pin = tx.map(|(p, af)| configure_pin(p, af, true));
        let rts_pin = rts.map(|(p, af)| configure_pin(p, af, true));
        let cts_pin = cts.map(|(p, af)| configure_pin(p, af, false));

        let regs = info.regs();
        apply_config(regs, &config, has_rx, has_tx)?;

        info.state.refcount.store(2, Ordering::Release);

        Ok(Self {
            tx: LpuartTx {
                info,
                has_tx,
                has_rx,
                _pin: tx_pin,
                _rts: rts_pin,
                _mode: PhantomData,
            },
            rx: LpuartRx {
                info,
                has_tx,
                has_rx,
                _pin: rx_pin,
                _cts: cts_pin,
                _mode: PhantomData,
            },
        })
    }

    /// Split into independent TX and RX drivers.
    pub fn split(self) -> (LpuartTx<'d, M>, LpuartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Reborrow as independent TX and RX drivers.
    ///
    /// Deliberately `&mut` borrows rather than the guideline's owned-halves
    /// form: the two halves share one peripheral closed over by a shutdown
    /// refcount (see the `Drop` impls below), so owned `split_ref` halves
    /// would run `Drop` on scope exit and shut the peripheral down while the
    /// driver is still alive.
    pub fn split_ref(&mut self) -> (&mut LpuartTx<'d, M>, &mut LpuartRx<'d, M>) {
        (&mut self.tx, &mut self.rx)
    }

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Blocking read that fills the entire buffer.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Blocking flush of the transmitter.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Whether the transmitter is currently busy (last byte not yet sent).
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    /// Blocking DMA write.
    pub fn blocking_write_dma(&mut self, channel: &mut DmaChannel<'_, Blocking>, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write_dma(channel, buffer)
    }

    /// Blocking DMA read of `buffer.len()` bytes.
    pub fn blocking_read_dma(
        &mut self,
        channel: &mut DmaChannel<'_, Blocking>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.rx.blocking_read_dma(channel, buffer)
    }

    /// Reconfigure the LPUART with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        let info = self.tx.info;
        enable_clock(config.clock_source)?;
        let regs = info.regs();
        apply_config(regs, config, self.tx.has_rx, self.tx.has_tx)
    }

    /// Change the baud rate, keeping all other settings.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)
    }
}

impl<'d, M: Mode> LpuartTx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
        clear_tx_done(regs);
        for &byte in buffer {
            while !tx_empty(regs) {}
            write_data(regs, byte);
        }
        Ok(())
    }

    /// Blocking flush: wait for `TX_DONE`, then clear it (SDK pattern).
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let regs = self.regs();
        // Idle with a cleared TX_DONE means nothing is in flight.
        if tx_empty(regs) && !tx_done(regs) {
            return Ok(());
        }
        while !tx_done(regs) {}
        clear_tx_done(regs);
        Ok(())
    }

    /// Whether the transmitter is currently busy (last byte not yet sent).
    pub fn busy(&self) -> bool {
        !tx_done(self.regs())
    }

    /// Change the baud rate, keeping all other settings.
    ///
    /// Rewrites only the CR0 baud-divider fields behind the write handshake;
    /// the active clock selector in `RCC.CR1` decides the base frequency.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        let regs = self.regs();
        let freq = current_clock_source().frequency_hz();
        let (ibaud, fbaud) = calc_baud(freq, baudrate).ok_or(ConfigError::Baudrate)?;
        wait_cr0_writable(regs);
        unsafe {
            regs.cr0().modify(|_, w| {
                w.lpuart_baud_rate_int().bits(ibaud as u16);
                w.lpuart_baud_rate_fra().bits(fbaud as u8);
                w
            });
        }
        wait_cr0_writable(regs);
        Ok(())
    }

    /// Reconfigure with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        enable_clock(config.clock_source)?;
        apply_config(self.regs(), config, self.has_rx, self.has_tx)
    }

    // No `send_break`: the LPUART register block exposes no break-transmit
    // bit in the PAC (unlike UART `LCR_H.BRK`), so there is nothing to drive.

    /// Blocking DMA write.
    pub fn blocking_write_dma(&mut self, channel: &mut DmaChannel<'_, Blocking>, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let dest = regs.data().as_ptr();
        set_dma_req(regs, true, true);
        let result = unsafe {
            channel
                .start_write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .blocking_wait()
                .map_err(Error::Dma)
        };
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d> LpuartTx<'d, Async> {
    /// Interrupt-driven write of the entire buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        clear_tx_done(regs);
        // If dropped mid-byte the TX interrupt would stay armed with no
        // waiter; disarm it.
        let irq_guard = OnDrop::new(|| set_tx_empty_interrupt(regs, false));
        for &byte in buffer {
            poll_fn(|cx| {
                if tx_empty(regs) {
                    return Poll::Ready(());
                }
                info.state.tx_waker.register(cx.waker());
                set_tx_empty_interrupt(regs, true);
                if tx_empty(regs) {
                    set_tx_empty_interrupt(regs, false);
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
            write_data(regs, byte);
        }
        irq_guard.defuse();
        set_tx_empty_interrupt(regs, false);
        Ok(())
    }

    /// Wait until `TX_DONE` is set, then clear it.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        if tx_empty(regs) && !tx_done(regs) {
            return Ok(());
        }
        let irq_guard = OnDrop::new(|| set_tx_done_interrupt(regs, false));
        poll_fn(|cx| {
            if tx_done(regs) {
                return Poll::Ready(());
            }
            info.state.tx_waker.register(cx.waker());
            set_tx_done_interrupt(regs, true);
            if tx_done(regs) {
                set_tx_done_interrupt(regs, false);
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
        irq_guard.defuse();
        set_tx_done_interrupt(regs, false);
        clear_tx_done(regs);
        Ok(())
    }

    /// DMA write using an async DMA channel.
    pub async fn write_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let dest = regs.data().as_ptr();
        set_dma_req(regs, true, true);
        compiler_fence(Ordering::SeqCst);
        // If dropped mid-transfer the DMA request would stay armed with no
        // waiter; disarm it. The DMA `Transfer` future aborts its own channel.
        let dma_guard = OnDrop::new(|| set_dma_req(regs, true, false));
        let result = unsafe {
            channel
                .write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        dma_guard.defuse();
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d, M: Mode> LpuartRx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Reconfigure with new settings.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        enable_clock(config.clock_source)?;
        apply_config(self.regs(), config, self.has_rx, self.has_tx)
    }

    /// Blocking read that fills the entire buffer.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let regs = self.regs();
        for slot in buffer.iter_mut() {
            while !rx_not_empty(regs) {
                take_rx_errors(regs)?;
            }
            take_rx_errors(regs)?;
            *slot = read_data(regs);
        }
        Ok(())
    }

    /// Blocking DMA read of `buffer.len()` bytes.
    pub fn blocking_read_dma(
        &mut self,
        channel: &mut DmaChannel<'_, Blocking>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let src = regs.data().as_ptr();
        set_dma_req(regs, false, true);
        let result = unsafe {
            channel
                .start_read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .blocking_wait()
                .map_err(Error::Dma)
        };
        set_dma_req(regs, false, false);
        result
    }

    fn nb_read_byte(&mut self) -> Result<Option<u8>, Error> {
        let regs = self.regs();
        take_rx_errors(regs)?;
        if !rx_not_empty(regs) {
            Ok(None)
        } else {
            Ok(Some(read_data(regs)))
        }
    }
}

impl<'d> LpuartRx<'d, Async> {
    async fn wait_rx_ready(&mut self) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        poll_fn(|cx| {
            if let Err(e) = take_rx_errors(regs) {
                return Poll::Ready(Err(e));
            }
            if rx_not_empty(regs) {
                return Poll::Ready(Ok(()));
            }
            info.state.rx_waker.register(cx.waker());
            set_rx_interrupts(regs, true);
            if let Err(e) = take_rx_errors(regs) {
                set_rx_interrupts(regs, false);
                return Poll::Ready(Err(e));
            }
            if rx_not_empty(regs) {
                set_rx_interrupts(regs, false);
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await?;
        set_rx_interrupts(regs, false);
        Ok(())
    }

    /// Interrupt-driven read that fills the entire buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            self.wait_rx_ready().await?;
            take_rx_errors(self.regs())?;
            *slot = read_data(self.regs());
        }
        Ok(())
    }

    /// DMA read of `buffer.len()` bytes.
    pub async fn read_dma(&mut self, channel: &mut DmaChannel<'_, Async>, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        let regs = self.regs();
        let src = regs.data().as_ptr();
        set_dma_req(regs, false, true);
        compiler_fence(Ordering::SeqCst);
        // If dropped mid-transfer the DMA request would stay armed with no
        // waiter; disarm it. The DMA `Transfer` future aborts its own channel.
        let dma_guard = OnDrop::new(|| set_dma_req(regs, false, false));
        let result = unsafe {
            channel
                .read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        dma_guard.defuse();
        set_dma_req(regs, false, false);
        result
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for Lpuart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d, M: Mode> Drop for LpuartTx<'d, M> {
    fn drop(&mut self) {
        if self.info.state.refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
            shutdown(self.info);
        }
    }
}

impl<'d, M: Mode> Drop for LpuartRx<'d, M> {
    fn drop(&mut self) {
        if self.info.state.refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
            shutdown(self.info);
        }
    }
}

// --- embedded-io -----------------------------------------------------------

impl<'d, M: Mode> embedded_io::ErrorType for Lpuart<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for LpuartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for LpuartRx<'d, M> {
    type Error = Error;
}

impl<'d> embedded_io::Write for LpuartTx<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::Write for Lpuart<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for LpuartTx<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(tx_empty(self.regs()))
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for Lpuart<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(tx_empty(self.tx.regs()))
    }
}

impl<'d> embedded_io::Read for LpuartRx<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        while !rx_not_empty(self.regs()) {
            take_rx_errors(self.regs())?;
        }
        let mut n = 0;
        while n < buf.len() {
            match self.nb_read_byte()? {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        Ok(n)
    }
}

impl<'d> embedded_io::Read for Lpuart<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for LpuartRx<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(rx_not_empty(self.regs()))
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for Lpuart<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(rx_not_empty(self.rx.regs()))
    }
}

impl<'d> embedded_io_async::Write for LpuartTx<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        LpuartTx::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        LpuartTx::flush(self).await
    }
}

impl<'d> embedded_io_async::Write for Lpuart<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Lpuart::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Lpuart::flush(self).await
    }
}

impl<'d> embedded_io_async::Read for LpuartRx<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.wait_rx_ready().await?;
        let mut n = 0;
        while n < buf.len() {
            match self.nb_read_byte()? {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        Ok(n)
    }
}

impl<'d> embedded_io_async::Read for Lpuart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(&mut self.rx, buf).await
    }
}

// --- embedded-hal 0.2 blocking traits ----------------------------------------

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for Lpuart<'d, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for LpuartTx<'d, Blocking> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}
