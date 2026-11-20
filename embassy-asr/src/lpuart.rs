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
//! With `time-driver-rtc`, `XO32K` is typically already enabled by
//! [`crate::init`].
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

use embassy_hal_internal::interrupt::Priority;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Channel as DmaChannel, Request, TransferOptions};
use crate::gpio::{AlternateFunction, AnyPin, Flex, Pin};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::lpuart::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::{interrupt, pac, peripherals};

const BAUD_INT_POS: u32 = 10;
const BAUD_FRA_POS: u32 = 6;
const BAUD_INT_MAX: u32 = 0x0fff;

const SR0_START_INVALID: u32 = 1 << 2;
const SR0_PARITY_ERR: u32 = 1 << 3;
const SR0_STOP_ERR: u32 = 1 << 4;
const SR0_RX_OVERFLOW: u32 = 1 << 5;
const SR0_RX_ERRORS: u32 = SR0_START_INVALID | SR0_PARITY_ERR | SR0_STOP_ERR | SR0_RX_OVERFLOW;

const SR1_WRITE_SR0: u32 = 1 << 1;
const SR1_WRITE_CR0: u32 = 1 << 2;
const SR1_RX_NOT_EMPTY: u32 = 1 << 3;
const SR1_TX_EMPTY: u32 = 1 << 4;
const SR1_TX_DONE: u32 = 1 << 5;

const CR1_RX_DONE_INT: u32 = 1 << 1;
const CR1_START_INVALID_INT: u32 = 1 << 2;
const CR1_PARITY_ERR_INT: u32 = 1 << 3;
const CR1_STOP_ERR_INT: u32 = 1 << 4;
const CR1_RX_OVERFLOW_INT: u32 = 1 << 5;
const CR1_RX_NOT_EMPTY_INT: u32 = 1 << 6;
const CR1_TX_EMPTY_INT: u32 = 1 << 7;
const CR1_TX_DONE_INT: u32 = 1 << 8;
const CR1_RX_INTS: u32 = CR1_RX_DONE_INT
    | CR1_START_INVALID_INT
    | CR1_PARITY_ERR_INT
    | CR1_STOP_ERR_INT
    | CR1_RX_OVERFLOW_INT
    | CR1_RX_NOT_EMPTY_INT;
const CR1_TX_INTS: u32 = CR1_TX_EMPTY_INT | CR1_TX_DONE_INT;

const RCO4M_HZ: u32 = 3_600_000;
const LOW_SPEED_HZ: u32 = 32_768;

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
        let cr1 = regs.cr1().read().bits();
        let sr0 = read_sr0(regs);
        let sr1 = regs.sr1().read().bits();

        let tx_pending = (cr1 & CR1_TX_EMPTY_INT != 0 && sr1 & SR1_TX_EMPTY != 0)
            || (cr1 & CR1_TX_DONE_INT != 0 && sr1 & SR1_TX_DONE != 0);
        if tx_pending {
            set_cr1_bits(regs, CR1_TX_INTS, false);
            info.state.tx_waker.wake();
        }

        let rx_pending = (cr1 & CR1_RX_NOT_EMPTY_INT != 0 && sr1 & SR1_RX_NOT_EMPTY != 0)
            || (cr1 & CR1_RX_DONE_INT != 0 && sr0 & (1 << 1) != 0)
            || (cr1 & CR1_START_INVALID_INT != 0 && sr0 & SR0_START_INVALID != 0)
            || (cr1 & CR1_PARITY_ERR_INT != 0 && sr0 & SR0_PARITY_ERR != 0)
            || (cr1 & CR1_STOP_ERR_INT != 0 && sr0 & SR0_STOP_ERR != 0)
            || (cr1 & CR1_RX_OVERFLOW_INT != 0 && sr0 & SR0_RX_OVERFLOW != 0);
        if rx_pending {
            set_cr1_bits(regs, CR1_RX_INTS, false);
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

/// Number of data bits (`lpuart_data_width_t`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 5 data bits.
    DataBits5,
    /// 6 data bits.
    DataBits6,
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
}

impl DataBits {
    const fn cr0_bits(self) -> u32 {
        match self {
            Self::DataBits5 => 0,
            Self::DataBits6 => 1,
            Self::DataBits7 => 2,
            Self::DataBits8 => 3,
        }
    }
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

impl Parity {
    const fn cr0_bits(self) -> u32 {
        match self {
            Self::Even => 0x0,
            Self::Odd => 0x4,
            Self::Stick0 => 0x8,
            Self::Stick1 => 0xc,
            Self::None => 0x1c,
        }
    }
}

/// Number of stop bits (`lpuart_stop_bits_t`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// One stop bit.
    STOP1,
    /// Two stop bits.
    STOP2,
}

impl StopBits {
    const fn cr0_bits(self) -> u32 {
        match self {
            Self::STOP1 => 0x0,
            Self::STOP2 => 0x20,
        }
    }
}

/// Hardware flow control.
///
/// RTS has a documented pin mux ([`pins::RTS_PD11`]). The datasheet does not
/// list an `LPUART_CTS` pad; CTS can still be enabled in CR1 if the board
/// wires a CTS input through an undocumented mux.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl {
    /// No RTS/CTS.
    None,
    /// RTS only (`LPUART_CR0_RTS_ENABLE`).
    Rts,
    /// CTS only (`LPUART_CR1_CTS_ENABLE`).
    Cts,
    /// RTS and CTS.
    RtsCts,
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

impl Wakeup {
    const fn cr0_bits(self) -> u32 {
        match self {
            Self::None => 0,
            Self::LowLevel => 1 << 22,
            Self::StartBit => 1 << 23,
            Self::RxDone => 1 << 24,
        }
    }
}

/// LPUART functional clock source (`rcc_lpuart_clk_source_t`).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSource {
    /// External 32.768 kHz crystal (`RCC_LPUART_CLK_SOURCE_XO32K`).
    #[default]
    Xo32k,
    /// Internal 32.768 kHz RC (`RCC_LPUART_CLK_SOURCE_RCO32K`).
    Rco32k,
    /// Internal RC reported as 3.6 MHz by the vendor (`RCC_LPUART_CLK_SOURCE_RCO4M`).
    Rco4m,
}

impl ClockSource {
    const fn frequency_hz(self) -> u32 {
        match self {
            Self::Xo32k | Self::Rco32k => LOW_SPEED_HZ,
            Self::Rco4m => RCO4M_HZ,
        }
    }

    const fn sel_bits(self) -> u8 {
        match self {
            Self::Xo32k => 0,
            Self::Rco32k => 1,
            Self::Rco4m => 2,
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

fn configure_pin(pin: Peri<'_, impl Pin>, af: AlternateFunction, output: bool) -> Peri<'_, AnyPin> {
    let mut flex = Flex::new(pin);
    if output {
        flex.set_as_output();
    } else {
        flex.set_as_input();
    }
    flex.set_alternate_function(af);
    flex.into_inner()
}

fn wait_cr0_writable(regs: &RegisterBlock) {
    while regs.sr1().read().bits() & (SR1_WRITE_SR0 | SR1_WRITE_CR0) != (SR1_WRITE_SR0 | SR1_WRITE_CR0) {}
}

fn wait_sr0_readable(regs: &RegisterBlock) {
    while regs.sr1().read().bits() & SR1_WRITE_SR0 == 0 {}
}

fn read_sr0(regs: &RegisterBlock) -> u32 {
    wait_sr0_readable(regs);
    regs.sr0().read().bits()
}

fn clear_sr0(regs: &RegisterBlock, mask: u32) {
    wait_cr0_writable(regs);
    unsafe {
        regs.sr0().write_with_zero(|w| w.bits(mask));
    }
    wait_cr0_writable(regs);
}

fn write_cr0(regs: &RegisterBlock, value: u32) {
    wait_cr0_writable(regs);
    unsafe {
        regs.cr0().write_with_zero(|w| w.bits(value));
    }
    wait_cr0_writable(regs);
}

fn modify_cr0(regs: &RegisterBlock, clear: u32, set: u32) {
    wait_cr0_writable(regs);
    let value = (regs.cr0().read().bits() & !clear) | set;
    unsafe {
        regs.cr0().write_with_zero(|w| w.bits(value));
    }
    wait_cr0_writable(regs);
}

fn set_cr1_bits(regs: &RegisterBlock, mask: u32, enable: bool) {
    let value = regs.cr1().read().bits();
    let value = if enable { value | mask } else { value & !mask };
    unsafe {
        regs.cr1().write_with_zero(|w| w.bits(value));
    }
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
        rcc_regs.cr1().modify(|r, w| {
            let bits = (r.bits() & !0xc) | (u32::from(source.sel_bits()) << 2);
            unsafe { w.bits(bits) }
        });
    });
}

fn enable_clock(source: ClockSource) -> Result<(), ConfigError> {
    set_clock_source(source);
    rcc::enable_peripheral(Peripheral::Lpuart).map_err(ConfigError::Rcc)?;
    rcc::reset_peripheral(Peripheral::Lpuart).map_err(ConfigError::Rcc)?;
    Ok(())
}

fn apply_config(regs: &RegisterBlock, config: Config) -> Result<(), ConfigError> {
    let (ibaud, fbaud) = calc_baud(config.clock_source.frequency_hz(), config.baudrate).ok_or(ConfigError::Baudrate)?;

    let cr0 = (ibaud << BAUD_INT_POS)
        | (fbaud << BAUD_FRA_POS)
        | config.stop_bits.cr0_bits()
        | config.parity.cr0_bits()
        | config.data_bits.cr0_bits()
        | config.wakeup.cr0_bits();

    // Baud / frame / wakeup live in CR0; PAC field accessors cover only the
    // wakeup and RX/RTS enables, so the frame fields are written as raw bits.
    write_cr0(regs, cr0);

    let rts = matches!(config.flow_control, FlowControl::Rts | FlowControl::RtsCts);
    let cts = matches!(config.flow_control, FlowControl::Cts | FlowControl::RtsCts);
    if rts {
        modify_cr0(regs, 0, 1 << 26);
    }
    set_cr1_bits(regs, 1 << 12, cts);

    Ok(())
}

fn set_rx_enable(regs: &RegisterBlock, enable: bool) {
    if enable {
        modify_cr0(regs, 0, 1 << 25);
    } else {
        modify_cr0(regs, 1 << 25, 0);
    }
}

fn set_tx_enable(regs: &RegisterBlock, enable: bool) {
    set_cr1_bits(regs, 1 << 9, enable);
}

fn set_dma_req(regs: &RegisterBlock, tx: bool, enable: bool) {
    set_cr1_bits(regs, if tx { 1 << 10 } else { 1 << 11 }, enable);
}

fn shutdown(info: &'static Info) {
    let regs = info.regs();
    set_tx_enable(regs, false);
    set_rx_enable(regs, false);
    set_cr1_bits(regs, CR1_TX_INTS | CR1_RX_INTS | (1 << 10) | (1 << 11), false);
    let _ = rcc::disable_peripheral(info.rcc);
}

fn take_rx_errors(regs: &RegisterBlock) -> Result<(), Error> {
    let sr0 = read_sr0(regs);
    let errors = sr0 & SR0_RX_ERRORS;
    if errors == 0 {
        return Ok(());
    }
    clear_sr0(regs, errors);
    if errors & SR0_RX_OVERFLOW != 0 {
        return Err(Error::Overrun);
    }
    if errors & SR0_PARITY_ERR != 0 {
        return Err(Error::Parity);
    }
    if errors & SR0_STOP_ERR != 0 {
        return Err(Error::Framing);
    }
    Err(Error::StartInvalid)
}

fn rx_not_empty(regs: &RegisterBlock) -> bool {
    regs.sr1().read().bits() & SR1_RX_NOT_EMPTY != 0
}

fn tx_empty(regs: &RegisterBlock) -> bool {
    regs.sr1().read().bits() & SR1_TX_EMPTY != 0
}

fn tx_done(regs: &RegisterBlock) -> bool {
    regs.sr1().read().bits() & SR1_TX_DONE != 0
}

fn clear_tx_done(regs: &RegisterBlock) {
    // Vendor `lpuart_clear_tx_done_status` sets the TX_DONE bit in SR1.
    regs.sr1().modify(|r, w| unsafe { w.bits(r.bits() | SR1_TX_DONE) });
}

fn read_data(regs: &RegisterBlock) -> u8 {
    regs.data().read().bits() as u8
}

fn write_data(regs: &RegisterBlock, byte: u8) {
    unsafe {
        regs.data().write_with_zero(|w| w.bits(u32::from(byte)));
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
    _pin: Option<Peri<'d, AnyPin>>,
    _rts: Option<Peri<'d, AnyPin>>,
    _mode: PhantomData<&'d mut M>,
}

/// Receive half of an LPUART.
pub struct LpuartRx<'d, M: Mode> {
    info: &'static Info,
    _pin: Option<Peri<'d, AnyPin>>,
    _cts: Option<Peri<'d, AnyPin>>,
    _mode: PhantomData<&'d mut M>,
}

impl<'d> Lpuart<'d, Blocking> {
    /// Create a blocking LPUART.
    ///
    /// `rx` / `tx` are `(pin, alternate_function)` pairs. See [`pins`] for the
    /// datasheet AF numbers. Supply `None` for unused directions.
    pub fn new_blocking(
        peri: Peri<'d, peripherals::LPUART>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, erase_pin(rx), erase_pin(tx), None, None, config, false)
    }

    /// Create a blocking LPUART with explicit RTS/CTS pins and AF selectors.
    pub fn new_blocking_with_flow_control(
        peri: Peri<'d, peripherals::LPUART>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            erase_pin(rx),
            erase_pin(tx),
            erase_pin(rts),
            erase_pin(cts),
            config,
            false,
        )
    }
}

impl<'d> Lpuart<'d, Async> {
    /// Create an interrupt-driven asynchronous LPUART.
    ///
    /// `irq` proves the LPUART vector is bound to [`InterruptHandler`].
    pub fn new(
        peri: Peri<'d, peripherals::LPUART>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<interrupt::typelevel::LPUART, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, erase_pin(rx), erase_pin(tx), None, None, config, true)
    }

    /// Create an asynchronous LPUART with explicit RTS/CTS pins and AF selectors.
    pub fn new_with_flow_control(
        peri: Peri<'d, peripherals::LPUART>,
        rx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        tx: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        rts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        cts: Option<(Peri<'d, impl Pin>, AlternateFunction)>,
        _irq: impl Binding<interrupt::typelevel::LPUART, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            erase_pin(rx),
            erase_pin(tx),
            erase_pin(rts),
            erase_pin(cts),
            config,
            true,
        )
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
    fn new_inner(
        _peri: Peri<'d, peripherals::LPUART>,
        rx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        tx: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        rts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        cts: Option<(Peri<'d, AnyPin>, AlternateFunction)>,
        config: Config,
        enable_irq: bool,
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
        apply_config(regs, config)?;
        set_tx_enable(regs, has_tx);
        set_rx_enable(regs, has_rx);

        info.state.refcount.store(2, Ordering::Release);

        if enable_irq {
            interrupt::typelevel::LPUART::unpend();
            interrupt::typelevel::LPUART::set_priority(Priority::P3);
            unsafe {
                interrupt::typelevel::LPUART::enable();
            }
        }

        Ok(Self {
            tx: LpuartTx {
                info,
                _pin: tx_pin,
                _rts: rts_pin,
                _mode: PhantomData,
            },
            rx: LpuartRx {
                info,
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
}

impl<'d, M: Mode> LpuartTx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Blocking write of the entire buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
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
        for &byte in buffer {
            poll_fn(|cx| {
                if tx_empty(regs) {
                    return Poll::Ready(());
                }
                info.state.tx_waker.register(cx.waker());
                set_cr1_bits(regs, CR1_TX_EMPTY_INT, true);
                if tx_empty(regs) {
                    set_cr1_bits(regs, CR1_TX_EMPTY_INT, false);
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
            write_data(regs, byte);
        }
        set_cr1_bits(regs, CR1_TX_EMPTY_INT, false);
        Ok(())
    }

    /// Wait until `TX_DONE` is set, then clear it.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let info = self.info;
        let regs = info.regs();
        if tx_empty(regs) && !tx_done(regs) {
            return Ok(());
        }
        poll_fn(|cx| {
            if tx_done(regs) {
                return Poll::Ready(());
            }
            info.state.tx_waker.register(cx.waker());
            set_cr1_bits(regs, CR1_TX_DONE_INT, true);
            if tx_done(regs) {
                set_cr1_bits(regs, CR1_TX_DONE_INT, false);
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
        set_cr1_bits(regs, CR1_TX_DONE_INT, false);
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
        let result = unsafe {
            channel
                .write(buffer, dest.cast::<u8>(), self.info.dma_tx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        set_dma_req(regs, true, false);
        result
    }
}

impl<'d, M: Mode> LpuartRx<'d, M> {
    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
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
            set_cr1_bits(regs, CR1_RX_INTS, true);
            if let Err(e) = take_rx_errors(regs) {
                set_cr1_bits(regs, CR1_RX_INTS, false);
                return Poll::Ready(Err(e));
            }
            if rx_not_empty(regs) {
                set_cr1_bits(regs, CR1_RX_INTS, false);
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await?;
        set_cr1_bits(regs, CR1_RX_INTS, false);
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
        let result = unsafe {
            channel
                .read(src.cast::<u8>(), buffer, self.info.dma_rx, TransferOptions::default())
                .map_err(Error::Dma)?
                .await
                .map_err(Error::Dma)
        };
        set_dma_req(regs, false, false);
        result
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

impl<'d, M: Mode> embedded_io::Write for LpuartTx<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_io::Write for Lpuart<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl<'d, M: Mode> embedded_io::Read for LpuartRx<'d, M> {
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

impl<'d, M: Mode> embedded_io::Read for Lpuart<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
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

// --- Instance --------------------------------------------------------------

mod sealed {
    pub trait Instance {
        fn info() -> &'static super::Info;
    }
}

/// LPUART instance.
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// NVIC vector for LPUART.
    type Interrupt: Interrupt;
}

impl sealed::Instance for peripherals::LPUART {
    fn info() -> &'static Info {
        info()
    }
}

impl Instance for peripherals::LPUART {
    type Interrupt = interrupt::typelevel::LPUART;
}
