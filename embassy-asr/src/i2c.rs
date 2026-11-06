//! I2C0–I2C2 driver for the ASR6601 (Marvell-style TWSI).
//!
//! Register programming follows the vendor `tremo_i2c` driver and the I2C
//! examples under `projects/*/examples/i2c`. All modes use the non-FIFO byte
//! path (`DBR` + `CR.TRANS_BYTE`), which is the SDK default. Async master
//! waits per byte on `IDBR_EMPTY` / `DBR_FULL` interrupts and sleeps (WFE)
//! between them; blocking master and both slave modes poll the same flags
//! with a bounded poll budget (`BLOCKING_POLL_LIMIT`) so a wedged bus fails
//! with [`Error::Timeout`] instead of spinning forever.
//!
//! The FIFO engine (`WFIFO`/`RFIFO` + `CR.TRANS_BEGIN` + `SR.TRANS_DONE`) is
//! deliberately not used, although it would cut IRQ wakes to ~one per
//! transaction. Plausible chip quirk, observed on hardware: in FIFO mode a
//! slave NACK does not reliably set `BED`, and the engine can then halt with
//! entries stranded in the WFIFO, `UNIT_BUSY` latched and `TRANS_DONE` never
//! coming — a state no flag combination resolves except a full bus recovery.
//! The byte path reports the same event promptly via `BED`/`ACK_STATUS` and
//! leaves no engine state behind, so subsequent transactions start clean by
//! construction. DMA is not used either (RM §15.8/15.12): it only replaces
//! the FIFO feed mechanism while the NACK handling stays identical, so it
//! cannot help here.
//!
//! Alternate-function numbers come from datasheet Table 4-3 (Fun=3 for every
//! documented I2C remap). Constructors either take typed pins that encode that
//! AF, or accept an explicit [`AlternateFunction`] per pin.
//!
//! # Unsafe contracts
//!
//! - [`InterruptHandler::on_interrupt`] may be called only from the vector bound
//!   through [`crate::bind_interrupts!`] for the matching I2C instance.
//! - Constructing a driver steals the PAC register block for that I2C. The
//!   owned [`Peri`] token is the exclusivity proof; do not also use
//!   `pac::I2c*::steal()` from application code while the driver is alive.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::interrupt::Priority;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal::i2c::Operation as EhOperation;
pub use embedded_hal::i2c::Operation;

use crate::gpio::{AlternateFunction, Flex, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::i2c0::RegisterBlock;
use crate::rcc::{self, Peripheral};
use crate::{Peri, PeripheralType, interrupt, peripherals};

/// Interrupt enables used by the driver, mirroring `CR` bits 18–27.
///
/// Passed by value so each wait arms exactly what it needs; the ISR clears
/// all of them. Positions live in the PAC field accessors, not here.
#[derive(Clone, Copy, Default)]
struct IrqEnables {
    /// Arbitration-loss detection.
    arb_loss: bool,
    /// `IDBR` transmit-empty (byte done).
    tx_empty: bool,
    /// `DBR` receive-full (byte ready).
    rx_full: bool,
    /// Bus error.
    bus_error: bool,
    /// Slave address match.
    slave_addr: bool,
    /// Slave stop detected.
    slave_stop: bool,
    /// Master stop detection interrupt.
    master_stop: bool,
    /// FIFO transaction done.
    trans_done: bool,
}

/// Bounded poll budget for the blocking master waits below.
///
/// A byte at 100 kHz takes ~90 us (~2k HCLK cycles); this budget is ~1000x
/// that, so healthy transfers always complete while a wedged bus (SCL held
/// low, no error flags) fails fast with [`Error::Timeout`] instead of
/// spinning forever inside an executor task.
const BLOCKING_POLL_LIMIT: u32 = 1_000_000;

/// I2C transfer errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// I2C driver error.
///
/// Deliberately a single type rather than an `Error`/`ConfigError` split:
/// `AddressMode` is validated per transfer (the address is a per-operation
/// parameter, not construction config) and `Timeout` spans both init
/// handshakes and transfer waits, so a split would duplicate those variants
/// in both enums for no checking benefit.
#[non_exhaustive]
pub enum Error {
    /// ACK was not received for an address or data byte.
    Nack,
    /// Arbitration was lost.
    Arbitration,
    /// Protocol / bus error flag.
    Bus,
    /// Timed out waiting for the unit to leave reset / busy.
    Timeout,
    /// 10-bit addressing is not supported by the vendor programming model.
    AddressMode,
    /// RCC clocks have not been published by [`crate::init`].
    ClocksNotInitialized,
    /// Enabling or resetting the I2C clock failed.
    Rcc(rcc::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Nack => f.write_str("I2C NACK"),
            Self::Arbitration => f.write_str("I2C arbitration lost"),
            Self::Bus => f.write_str("I2C bus error"),
            Self::Timeout => f.write_str("I2C timeout"),
            Self::AddressMode => f.write_str("I2C 10-bit addressing unsupported"),
            Self::ClocksNotInitialized => f.write_str("RCC clocks are not initialized"),
            Self::Rcc(_) => f.write_str("I2C RCC error"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Self::Nack => embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Unknown),
            Self::Arbitration => embedded_hal::i2c::ErrorKind::ArbitrationLoss,
            Self::Bus => embedded_hal::i2c::ErrorKind::Bus,
            _ => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

/// Bus speed programmed through `CR.BUS_MODE` and the LCR/WCR dividers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frequency {
    /// Standard mode, 100 kHz.
    Standard,
    /// Fast mode, 400 kHz.
    Fast,
}

impl Frequency {
    const fn hertz(self) -> u32 {
        match self {
            Self::Standard => 100_000,
            Self::Fast => 400_000,
        }
    }
}

/// I2C master configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target bus frequency.
    pub frequency: Frequency,
    /// Optional internal pull on SCL.
    ///
    /// External pull-ups are recommended; leave this as [`Pull::None`] when
    /// board pull-ups are present.
    pub scl_pull: Pull,
    /// Optional internal pull on SDA.
    pub sda_pull: Pull,
}

impl Config {
    /// Standard-mode master with no internal pulls.
    pub const fn new() -> Self {
        Self {
            frequency: Frequency::Standard,
            scl_pull: Pull::None,
            sda_pull: Pull::None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction observed after a slave address match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlaveOp {
    /// Master is writing; the slave should receive.
    Read,
    /// Master is reading; the slave should transmit.
    Write,
}

#[derive(Clone, Copy)]
enum PclkSel {
    Pclk0,
    Pclk1,
}

struct State {
    waker: AtomicWaker,
    /// True while the async waits have the CR event enables armed.
    ///
    /// Kept so a poll that finds the transfer still pending does not rewrite
    /// `CR` while a byte is in flight: busy-polling callers that spin the
    /// future with interrupts disabled would otherwise perform a CR
    /// read-modify-write millions of times per second, racing the engine's own
    /// clearing of the auto-cleared `TB`/`START` control bits. The ISR clears
    /// this together with the enables, so a spurious wake re-arms.
    armed: AtomicBool,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            armed: AtomicBool::new(false),
        }
    }
}

static STATE_I2C0: State = State::new();
static STATE_I2C1: State = State::new();
static STATE_I2C2: State = State::new();

struct Info {
    regs: *const RegisterBlock,
    peripheral: Peripheral,
    pclk: PclkSel,
    state: &'static State,
}

// Register blocks are Sync; the pointer is unique to the owned I2C instance.
unsafe impl Sync for Info {}

impl Info {
    fn regs(&self) -> &'static RegisterBlock {
        unsafe { &*self.regs }
    }
}

/// Interrupt handler for one I2C instance.
///
/// Masks the sticky event enables that woke the vector and wakes any async
/// waiter. Status bits are left for the waiter to observe and clear.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();

        // Mask level/event enables; the waiter re-enables as needed.
        //
        // This read-modify-write must never interleave with the driver's CR
        // writes (they run in critical sections with IRQs disabled), otherwise
        // a stale write here could cancel a START/ABORT the driver just
        // issued and wedge the unit.
        regs.cr().modify(|_, w| {
            w.arb_loss_det_intr_en().clear_bit();
            w.idbr_empty_intr_en().clear_bit();
            w.dbr_full_intr_en().clear_bit();
            w.bus_error_intr_en().clear_bit();
            w.slave_addr_det_intr_en().clear_bit();
            w.slave_stop_det_intr_en().clear_bit();
            w.master_stop_det_intr_en().clear_bit();
            w.trans_done_intr_en().clear_bit()
        });

        // Error flags are NOT cleared here: the waiter (async or blocking
        // poll loop) has to observe them through `check_errors`. A latched
        // flag left behind by a cancelled waiter is reported (and cleared)
        // by the next transaction's first status check.

        // The enables this ISR just cleared must be re-armed by the waiter on
        // its next poll; clearing the flag couples the two.
        info.state.armed.store(false, Ordering::Release);
        info.state.waker.wake();
    }
}

/// I2C master driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    scl: Flex<'d>,
    sda: Flex<'d>,
    scl_af: AlternateFunction,
    sda_af: AlternateFunction,
    scl_pull: Pull,
    sda_pull: Pull,
    frequency: Frequency,
    _mode: PhantomData<&'d mut M>,
}

/// I2C slave driver.
///
/// Matches the vendor slave example: wait for address detect, then exchange
/// bytes with ACK/NAK control until stop.
pub struct I2cSlave<'d, M: Mode> {
    info: &'static Info,
    _scl: Flex<'d>,
    _sda: Flex<'d>,
    _mode: PhantomData<&'d mut M>,
}

fn configure_i2c_pin<'d>(pin: Peri<'d, impl GpioPin>, af: AlternateFunction, pull: Pull) -> Flex<'d> {
    let mut flex = Flex::new(pin);
    // The TWSI pad is open-drain when muxed; direction follows the vendor
    // examples which only program the IOMUX.
    flex.set_as_input();
    flex.set_pull(pull);
    flex.set_alternate_function(af);
    flex
}

fn pclk_hz(sel: PclkSel) -> Result<u32, Error> {
    let clocks = rcc::clocks().ok_or(Error::ClocksNotInitialized)?;
    Ok(match sel {
        PclkSel::Pclk0 => clocks.pclk0_hz,
        PclkSel::Pclk1 => clocks.pclk1_hz,
    })
}

fn unit_reset(regs: &RegisterBlock) -> Result<(), Error> {
    let mut timeout = 1_000u32;
    while regs.sr().read().unit_busy().bit_is_set() {
        timeout = timeout.saturating_sub(1);
        if timeout == 0 {
            return Err(Error::Timeout);
        }
    }

    // Vendor: CR = UNIT_RESET only, pulse reset, clear SR, clear reset bit.
    // The SR step writes the snapshot back verbatim (clearing every latched
    // flag at once, exactly like the vendor `i2cx->SR = i2cx->SR`); keep it
    // bit-identical rather than clearing flags one by one.
    unsafe {
        regs.cr().write_with_zero(|w| w.unit_reset().set_bit());
        let sr = regs.sr().read().bits();
        regs.sr().write_with_zero(|w| w.bits(sr));
        regs.cr().write_with_zero(|w| w.bits(0));
    }
    Ok(())
}

fn program_timing(regs: &RegisterBlock, pclk: u32, frequency: Frequency) {
    let slv = (pclk / Frequency::Standard.hertz()).saturating_sub(8) / 2;
    let flv = (pclk / Frequency::Fast.hertz()).saturating_sub(8) / 2;
    let flv = flv.saturating_sub(1);
    let wcr = flv / 3;

    unsafe {
        regs.lcr().write_with_zero(|w| {
            // 9-bit phase-decrementer loads; the masks match the PAC fields.
            w.slv().bits((slv & 0x1ff) as u16);
            w.flv().bits((flv & 0x1ff) as u16)
        });
        // 5-bit setup/hold counter; in-range for every supported PCLK.
        regs.wcr().write_with_zero(|w| w.count().bits(wcr as u8));
    }

    regs.cr().modify(|_, w| match frequency {
        Frequency::Standard => w.bus_mode().standard(),
        Frequency::Fast => w.bus_mode().fast(),
    });
}

fn enable_unit(regs: &RegisterBlock, enable: bool) {
    regs.cr().modify(|_, w| {
        w.twsi_unit_en().bit(enable);
        w.scl_en().bit(enable)
    });
}

/// Clear one latched `SR` event flag (write-1-to-clear). One helper per flag
/// so a wrong-position clear is a compile error, not a silent mis-clear.
fn clear_arb_loss(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.arb_loss_det().clear_bit_by_one());
    }
}

/// Clear the latched bus-error flag.
fn clear_bus_error(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.bus_error().clear_bit_by_one());
    }
}

/// Clear the latched transmit-empty flag.
fn clear_tx_empty(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.idbr_empty().clear_bit_by_one());
    }
}

/// Clear the latched receive-full flag.
fn clear_rx_full(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.dbr_full().clear_bit_by_one());
    }
}

/// Clear the latched slave-address-detect flag.
fn clear_slave_addr(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.slave_addr_det().set_bit());
    }
}

/// Clear the latched slave-stop-detect flag.
fn clear_slave_stop(regs: &RegisterBlock) {
    unsafe {
        regs.sr().write_with_zero(|w| w.slave_stop_det().set_bit());
    }
}

fn check_errors(regs: &RegisterBlock) -> Result<(), Error> {
    let sr = regs.sr().read();
    if sr.arb_loss_det().bit_is_set() {
        clear_arb_loss(regs);
        return Err(Error::Arbitration);
    }
    if sr.bus_error().bit_is_set() {
        clear_bus_error(regs);
        return Err(Error::Bus);
    }
    Ok(())
}

fn nack_seen(regs: &RegisterBlock) -> bool {
    // ACK_STATUS = 1 means NAK was received for the previous byte.
    regs.sr().read().ack_status().bit_is_set()
}

/// Run a CR control sequence atomically with respect to the ISR, whose own
/// CR write would otherwise interleave with (and clobber) these read-modify-
/// writes, e.g. cancelling a START that was just issued.
fn critical<R>(f: impl FnOnce() -> R) -> R {
    critical_section::with(|_| f())
}

/// Busy-wait used by [`I2c::bus_recover`]. The recovery clock period is not
/// critical; 50k system cycles is comfortably above the ~10 µs half-period
/// of 100 kHz I2C at any plausible CPU frequency.
fn recover_delay() {
    cortex_m::asm::delay(50_000);
}

fn master_abort(regs: &RegisterBlock) {
    critical(|| {
        regs.cr().modify(|_, w| {
            w.start().clear_bit();
            w.master_abort().set_bit()
        });
    });
}

/// Bus-free time between transactions: spin until the unit reports the bus
/// free after a STOP, so a directly following START cannot collide with the
/// STOP on the wire (I2C `tBUF`: 4.7 µs standard, 1.3 µs fast).
///
/// The driver returns from a transfer the moment the last byte completes;
/// without this wait a tight loop (or LTO-tightened caller code) issues the
/// next START within a microsecond, the slave misses it and NACKs the
/// address (or flags a bus error). Some slaves wedge on such back-to-back
/// traffic while spaced traffic is clean.
///
/// Bounded and infallible by design: on a wedged bus (slave holding SDA)
/// `BUS_BUSY` never clears, so give up after the budget and let the caller
/// observe the resulting NACK/timeout and run `bus_recover`. Synchronous
/// spin (no timer await) so it is safe in every context, including the
/// interrupt-disabled RFAL busy-poll. Typical cost is a few iterations.
fn wait_bus_free(regs: &RegisterBlock) {
    for _ in 0..10_000 {
        if regs.sr().read().bus_busy().bit_is_clear() {
            break;
        }
        core::hint::spin_loop();
    }
    // Slave-processing floor on top of the bus-free time above: the unit
    // reports the bus free the moment the STOP completes on the wire, but
    // the slave may still be digesting the transaction (register write
    // propagation, state-machine settle). Hitting it again within a few
    // microseconds NACKs the address or wedges it; a few hundred
    // microseconds of spacing makes back-to-back traffic reliable at any
    // caller code speed (including LTO-tightened release builds).
    // Synchronous spin so it is safe with interrupts disabled.
    cortex_m::asm::delay(5_000);
}

/// Issue a STOP and wait until the bus is free for the next transaction.
///
/// Also clears any latched arbitration-loss / bus-error flags. Those are
/// per-transaction state, but they can latch after the last status check of
/// a transfer — in the flag-check window, or as a side effect of the STOP
/// itself when the slave still holds SDA — and would otherwise be reported
/// as a spurious error by the *next* transaction's first `check_errors`.
/// A genuine mid-transfer error is always reported by the transfer itself
/// before this runs; anything latched at STOP time did not corrupt data.
fn master_stop(regs: &RegisterBlock) {
    master_abort(regs);
    wait_bus_free(regs);
    clear_arb_loss(regs);
    clear_bus_error(regs);
}

fn master_send_start(regs: &RegisterBlock, addr7: u8, read: bool) {
    let data = (addr7 << 1) | u8::from(read);
    critical(|| {
        regs.cr().modify(|_, w| w.master_abort().clear_bit());
        unsafe {
            regs.dbr().write_with_zero(|w| w.data_buffer().bits(data));
        }
        regs.cr().modify(|_, w| {
            w.stop().clear_bit();
            w.start().set_bit();
            w.trans_byte().set_bit()
        });
    });
}

fn master_send_byte(regs: &RegisterBlock, data: u8) {
    critical(|| {
        unsafe {
            regs.dbr().write_with_zero(|w| w.data_buffer().bits(data));
        }
        regs.cr().modify(|_, w| {
            w.start().clear_bit();
            w.trans_byte().set_bit()
        });
    });
}

fn set_receive_mode(regs: &RegisterBlock, ack: bool) {
    // CR.ACKNAK = 1 requests NAK.
    critical(|| {
        regs.cr().modify(|_, w| {
            w.acknak().bit(!ack);
            w.start().clear_bit();
            w.trans_byte().set_bit()
        });
    });
}

fn read_dbr(regs: &RegisterBlock) -> u8 {
    regs.dbr().read().data_buffer().bits()
}

fn enable_clock(info: &'static Info) -> Result<(), Error> {
    rcc::enable_peripheral(info.peripheral).map_err(Error::Rcc)?;
    rcc::reset_peripheral(info.peripheral).map_err(Error::Rcc)?;
    Ok(())
}

fn init_master(info: &'static Info, config: Config) -> Result<(), Error> {
    let regs = info.regs();
    let pclk = pclk_hz(info.pclk)?;
    unit_reset(regs)?;
    // FIFO stays disabled (SDK default).
    regs.cr().modify(|_, w| w.fifo_en().clear_bit());
    program_timing(regs, pclk, config.frequency);
    enable_unit(regs, true);
    Ok(())
}

fn init_slave(info: &'static Info, address: u8) -> Result<(), Error> {
    if address > 0x7f {
        return Err(Error::AddressMode);
    }
    let regs = info.regs();
    let pclk = pclk_hz(info.pclk)?;
    unit_reset(regs)?;
    regs.cr().modify(|_, w| w.fifo_en().clear_bit());
    // Dividers are still required by the hardware; use standard timing.
    program_timing(regs, pclk, Frequency::Standard);
    unsafe {
        regs.sar().write_with_zero(|w| w.slave_address().bits(address));
    }
    enable_unit(regs, true);
    Ok(())
}

fn shutdown(info: &'static Info) {
    let regs = info.regs();
    enable_unit(regs, false);
    let _ = rcc::disable_peripheral(info.peripheral);
}

fn set_irq_enables(regs: &RegisterBlock, en: IrqEnables) {
    critical(|| {
        regs.cr().modify(|_, w| {
            w.arb_loss_det_intr_en().bit(en.arb_loss);
            w.idbr_empty_intr_en().bit(en.tx_empty);
            w.dbr_full_intr_en().bit(en.rx_full);
            w.bus_error_intr_en().bit(en.bus_error);
            w.slave_addr_det_intr_en().bit(en.slave_addr);
            w.slave_stop_det_intr_en().bit(en.slave_stop);
            w.master_stop_det_intr_en().bit(en.master_stop);
            w.trans_done_intr_en().bit(en.trans_done)
        });
    });
}

impl<'d> I2c<'d, Blocking> {
    /// Create a blocking master using typed pins (datasheet Fun=3 remaps).
    pub fn new_blocking<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_blocking_af(peri, scl, Scl::AF, sda, Sda::AF, config)
    }

    /// Create a blocking master with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_blocking_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, config)
    }
}

impl<'d> I2c<'d, Async> {
    /// Create an async master using typed pins.
    ///
    /// `irq` proves the I2C vector is bound to [`InterruptHandler<T>`].
    pub fn new<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_af(peri, scl, Scl::AF, sda, Sda::AF, irq, config)
    }

    /// Create an async master with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let this = Self::new_inner(peri, scl, scl_af, sda, sda_af, config)?;
        this.enable_interrupts::<T>();
        Ok(this)
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    /// Arm the NVIC vector for interrupt-driven use.
    ///
    /// Called by the async constructors only; blocking instances never
    /// enable the vector.
    fn enable_interrupts<T: Instance>(&self) {
        T::Interrupt::unpend();
        T::Interrupt::set_priority(Priority::P3);
        unsafe {
            T::Interrupt::enable();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        config: Config,
    ) -> Result<Self, Error> {
        let info = T::info();
        enable_clock(info)?;

        let scl = configure_i2c_pin(scl, scl_af, config.scl_pull);
        let sda = configure_i2c_pin(sda, sda_af, config.sda_pull);

        init_master(info, config)?;

        Ok(Self {
            info,
            scl,
            sda,
            scl_af,
            sda_af,
            scl_pull: config.scl_pull,
            sda_pull: config.sda_pull,
            frequency: config.frequency,
            _mode: PhantomData,
        })
    }

    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    fn finish_error(&self, err: Error) -> Error {
        master_stop(self.regs());
        err
    }

    fn blocking_wait_tx_empty(&self) -> Result<(), Error> {
        let regs = self.regs();
        for _ in 0..BLOCKING_POLL_LIMIT {
            let sr = regs.sr().read();
            // RM 15.13.2: for a master transmitter `SR.BUS_ERROR` (BED) is
            // set when a sent byte was not ACKed, i.e. it is this unit's
            // NACK indication, not a bus fault. Clear it and report a NACK
            // so callers can tell a missing (or still booting) slave from a
            // wedged bus.
            if sr.bus_error().bit_is_set() {
                clear_bus_error(regs);
                return Err(self.finish_error(Error::Nack));
            }
            if sr.arb_loss_det().bit_is_set() {
                clear_arb_loss(regs);
                return Err(self.finish_error(Error::Arbitration));
            }
            if sr.idbr_empty().bit_is_set() {
                clear_tx_empty(regs);
                if nack_seen(regs) {
                    return Err(self.finish_error(Error::Nack));
                }
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(self.finish_error(Error::Timeout))
    }

    fn blocking_wait_rx_full(&self) -> Result<(), Error> {
        let regs = self.regs();
        for _ in 0..BLOCKING_POLL_LIMIT {
            check_errors(regs)?;
            if regs.sr().read().dbr_full().bit_is_set() {
                clear_rx_full(regs);
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(self.finish_error(Error::Timeout))
    }

    fn blocking_write_ops(&mut self, addr: u8, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        let regs = self.regs();
        master_send_start(regs, addr, false);
        clear_tx_empty(regs);
        self.blocking_wait_tx_empty()?;

        for &byte in bytes {
            master_send_byte(regs, byte);
            clear_tx_empty(regs);
            self.blocking_wait_tx_empty()?;
        }

        if send_stop {
            master_stop(regs);
        }
        Ok(())
    }

    fn blocking_read_ops(&mut self, addr: u8, buffer: &mut [u8], restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            if send_stop {
                master_stop(self.regs());
            }
            return Ok(());
        }

        let regs = self.regs();
        if !restart {
            // Fresh transaction: address phase.
        }
        master_send_start(regs, addr, true);
        clear_tx_empty(regs);
        self.blocking_wait_tx_empty()?;

        let last = buffer.len() - 1;
        for (i, slot) in buffer.iter_mut().enumerate() {
            let ack = i != last;
            set_receive_mode(regs, ack);
            self.blocking_wait_rx_full()?;
            *slot = read_dbr(regs);
        }

        if send_stop {
            master_stop(regs);
        }
        Ok(())
    }

    /// Blocking write of `bytes` to `address` (7-bit).
    pub fn blocking_write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_write_ops(address, bytes, true)
    }

    /// Blocking read of `buffer.len()` bytes from `address` (7-bit).
    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_read_ops(address, buffer, false, true)
    }

    /// Blocking write then repeated-start read.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.blocking_write_ops(address, write, false)?;
        self.blocking_read_ops(address, read, true, true)
    }

    /// Blocking embedded-hal transaction.
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let last = operations.len() - 1;
        for (i, op) in operations.iter_mut().enumerate() {
            let stop = i == last;
            match op {
                EhOperation::Write(bytes) => self.blocking_write_ops(address, bytes, stop)?,
                EhOperation::Read(buffer) => {
                    let restart = i != 0;
                    self.blocking_read_ops(address, buffer, restart, stop)?;
                }
            }
        }
        Ok(())
    }
}

impl<'d> I2c<'d, Async> {
    async fn wait_on<F>(&mut self, mut ready: F, irq: IrqEnables) -> Result<(), Error>
    where
        F: FnMut(&RegisterBlock) -> Poll<Result<(), Error>>,
    {
        let regs = self.regs();
        let state = self.info.state;
        // Error enables ride along with every wait.
        let on = IrqEnables {
            arb_loss: true,
            bus_error: true,
            ..irq
        };
        let off = IrqEnables::default();

        // If dropped mid-wait (executor cancellation, a `select` loser, or
        // an outer watchdog giving up on this transfer) the unit would be left
        // mid-transfer with START pending and no STOP: a slave holding SDA for
        // its next bit then wedges the bus for every later transaction. Abort
        // (STOP) first so the bus is released, then disarm the enables so the
        // ISR stops firing with nothing to wake.
        let irq_guard = OnDrop::new(|| {
            master_stop(regs);
            set_irq_enables(regs, off);
            state.armed.store(false, Ordering::Release);
        });
        let result = poll_fn(|cx| {
            state.waker.register(cx.waker());
            match ready(regs) {
                Poll::Ready(result) => {
                    // Disarm everything that could have been armed by a
                    // previous wait (including the error enables), so no
                    // latched flag can re-trigger the ISR later.
                    set_irq_enables(regs, off);
                    state.armed.store(false, Ordering::Release);
                    Poll::Ready(result)
                }
                Poll::Pending => {
                    // Arm the enables once per wait, not once per poll: while
                    // they stay armed, re-polling must not touch CR. Under the
                    // RFAL busy-poll this runs with interrupts disabled and
                    // would otherwise rewrite CR millions of times per second
                    // while a byte is in flight, racing the engine's own
                    // clearing of the auto-cleared TB/START bits.
                    if !state.armed.load(Ordering::Acquire) {
                        state.armed.store(true, Ordering::Release);
                        set_irq_enables(regs, on);
                        // Re-check after arming to close the race with an
                        // event that latched before the enables were set.
                        return match ready(regs) {
                            Poll::Ready(result) => {
                                set_irq_enables(regs, off);
                                state.armed.store(false, Ordering::Release);
                                Poll::Ready(result)
                            }
                            Poll::Pending => Poll::Pending,
                        };
                    }
                    Poll::Pending
                }
            }
        })
        .await;
        irq_guard.defuse();
        result
    }

    async fn wait_tx_empty(&mut self) -> Result<(), Error> {
        self.wait_on(
            |regs| {
                let sr = regs.sr().read();
                // BED (`BUS_ERROR`) is the master-transmit no-ACK indication
                // (RM 15.13.2); report it as a NACK, not a bus error, for the
                // same reason as `blocking_wait_tx_empty`.
                if sr.bus_error().bit_is_set() {
                    clear_bus_error(regs);
                    return Poll::Ready(Err(Error::Nack));
                }
                if sr.arb_loss_det().bit_is_set() {
                    clear_arb_loss(regs);
                    return Poll::Ready(Err(Error::Arbitration));
                }
                if sr.idbr_empty().bit_is_set() {
                    clear_tx_empty(regs);
                    if nack_seen(regs) {
                        Poll::Ready(Err(Error::Nack))
                    } else {
                        Poll::Ready(Ok(()))
                    }
                } else {
                    Poll::Pending
                }
            },
            IrqEnables {
                tx_empty: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| self.finish_error(e))
    }

    async fn wait_rx_full(&mut self) -> Result<(), Error> {
        self.wait_on(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().dbr_full().bit_is_set() {
                    clear_rx_full(regs);
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            IrqEnables {
                rx_full: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| self.finish_error(e))
    }

    /// Attempt to recover a jammed bus.
    ///
    /// This is synchronous bit-banging (GPIO remux + `asm::delay` clocks, up
    /// to milliseconds) on an async method: recovery is inherently a
    /// blocking bus procedure with no interrupt to wait on, and it runs
    /// rarely (only after a wedged transfer). Do not call it in a hot path.
    ///
    /// A slave stuck mid-byte holds SDA Low, so the TWSI unit can never
    /// generate a START and every transfer hangs. If SDA is observed Low the
    /// SCL/SDA pads are temporarily remuxed to GPIO, up to nine SCL clocks
    /// are bit-banged so the stuck slave can finish its byte, and a STOP
    /// condition is generated. The TWSI unit is aborted, reset, and its pin
    /// driving released before the remux (a plain CR write does not stop an
    /// in-flight transfer; the unit keeps driving the pads and the GPIO
    /// remux has no effect on the bus). The alternate muxing is restored in
    /// all cases.
    ///
    /// Returns `true` if the bus was idle or SDA was released.
    pub async fn bus_recover(&mut self) -> bool {
        let regs = self.regs();

        // Abort any in-flight transfer first (vendor `i2c_master_send_stop`).
        // Best effort: a wedged unit may ignore it, which is why a hard reset
        // follows below. No bus-free wait here: the RCC reset below disrupts
        // the unit anyway, and the bit-bang delays already space the bus.
        master_abort(regs);

        // A stuck unit (SR.UNIT_BUSY latched High, a byte never shifting out of
        // the IDBR) will not respond to the soft CR UNIT_RESET: `unit_reset`
        // waits for UNIT_BUSY to clear first and so skips the reset pulse
        // entirely. Gate the peripheral clock and apply a hard RCC reset
        // instead — that returns the unit to its default state (UNIT_BUSY
        // cleared, SCL_EN and TWSI_UNIT_EN off, SDA/SCL pads released) no matter
        // what state it is wedged in. Clock-gating alone does not release the
        // pads, so the reset is what makes the GPIO bit-bang below reach the bus.
        let _ = rcc::disable_peripheral(self.info.peripheral);
        let _ = rcc::reset_peripheral(self.info.peripheral);

        // Remux the pads to GPIO (AF0) so the GPIO block — not the I2C TWSI
        // peripheral — owns the pads. On this chip the pad mux follows the
        // alternate-function (IOMUX) register, so merely switching the pin to
        // input/output mode leaves it routed to I2C and the bit-bang below
        // never reaches the bus. Then configure as pull-up inputs and inspect
        // SDA.
        self.scl.set_alternate_function(AlternateFunction::Function0);
        self.sda.set_alternate_function(AlternateFunction::Function0);
        self.scl.set_as_input();
        self.scl.set_pull(Pull::Up);
        self.sda.set_as_input();
        self.sda.set_pull(Pull::Up);
        recover_delay();

        let mut released = self.sda.is_high();

        if !released {
            // Bit-bang clocks so a stuck slave can finish its byte(s) and
            // release SDA. Nine clocks cover one byte; budget two bytes plus
            // margin because an abandoned multi-byte transaction (timeout
            // drop, retried timeout) can leave the slave more than one byte
            // deep — SDA reading high mid-way may just be a 1 bit, not idle.
            // Keep clocking while SDA is low, then flush a full extra byte
            // once it reads high before generating the STOP below (a STOP
            // sent mid-byte may be ignored, leaving the slave confused).
            self.scl.set_as_output();
            self.scl.set_high();
            recover_delay();
            let mut clocks = 0u8;
            let mut high_streak = 0u8;
            while clocks < 27 {
                self.scl.set_low();
                recover_delay();
                self.scl.set_high();
                recover_delay();
                clocks += 1;
                if self.sda.is_high() {
                    high_streak += 1;
                    if high_streak >= 9 {
                        break;
                    }
                } else {
                    high_streak = 0;
                }
            }
            released = high_streak >= 9;
            if released {
                // STOP condition: SDA Low -> High while SCL is High.
                self.sda.set_as_output();
                self.sda.set_low();
                recover_delay();
                self.sda.set_high();
                recover_delay();
            }
        }

        // Restore the I2C alternate muxing.
        self.sda.set_as_input();
        self.sda.set_pull(self.sda_pull);
        self.scl.set_as_input();
        self.scl.set_pull(self.scl_pull);
        self.scl.set_alternate_function(self.scl_af);
        self.sda.set_alternate_function(self.sda_af);

        // Re-power the I2C block, then re-initialize the TWSI unit (reset,
        // timing, enable).
        let _ = rcc::enable_peripheral(self.info.peripheral);
        let config = Config {
            frequency: self.frequency,
            scl_pull: Pull::None,
            sda_pull: Pull::None,
        };
        let _ = init_master(self.info, config);

        released
    }

    async fn write_ops(&mut self, addr: u8, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        let regs = self.regs();
        master_send_start(regs, addr, false);
        clear_tx_empty(regs);
        self.wait_tx_empty().await?;

        for &byte in bytes {
            master_send_byte(self.regs(), byte);
            clear_tx_empty(self.regs());
            self.wait_tx_empty().await?;
        }

        if send_stop {
            master_stop(self.regs());
        }
        Ok(())
    }

    async fn read_ops(&mut self, addr: u8, buffer: &mut [u8], _restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            if send_stop {
                master_stop(self.regs());
            }
            return Ok(());
        }

        let regs = self.regs();
        master_send_start(regs, addr, true);
        clear_tx_empty(regs);
        self.wait_tx_empty().await?;

        let last = buffer.len() - 1;
        for (i, slot) in buffer.iter_mut().enumerate() {
            let ack = i != last;
            set_receive_mode(self.regs(), ack);
            self.wait_rx_full().await?;
            *slot = read_dbr(self.regs());
        }

        if send_stop {
            master_stop(self.regs());
        }
        Ok(())
    }

    /// Async write of `bytes` to `address` (7-bit).
    pub async fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.write_ops(address, bytes, true).await
    }

    /// Async read of `buffer.len()` bytes from `address` (7-bit).
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.read_ops(address, buffer, false, true).await
    }

    /// Async write then repeated-start read.
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        self.write_ops(address, write, false).await?;
        self.read_ops(address, read, true, true).await
    }

    /// Async embedded-hal transaction.
    pub async fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Error> {
        if address > 0x7f {
            return Err(Error::AddressMode);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let last = operations.len() - 1;
        for (i, op) in operations.iter_mut().enumerate() {
            let stop = i == last;
            match op {
                EhOperation::Write(bytes) => self.write_ops(address, bytes, stop).await?,
                EhOperation::Read(buffer) => {
                    let restart = i != 0;
                    self.read_ops(address, buffer, restart, stop).await?;
                }
            }
        }
        Ok(())
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for I2c<'d, M> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        // I2c is always master; I2cSlave is a separate type. Reprogram the
        // timing as at construction; only call this with the bus idle.
        init_master(self.info, *config)?;
        self.scl.set_pull(config.scl_pull);
        self.sda.set_pull(config.sda_pull);
        self.scl_pull = config.scl_pull;
        self.sda_pull = config.sda_pull;
        self.frequency = config.frequency;
        Ok(())
    }
}

impl<'d, M: Mode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        shutdown(self.info);
    }
}

impl<'d> I2cSlave<'d, Blocking> {
    /// Create a blocking slave using typed pins.
    pub fn new_blocking<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
    ) -> Result<Self, Error> {
        Self::new_blocking_af(peri, scl, Scl::AF, sda, Sda::AF, address, scl_pull, sda_pull)
    }

    /// Create a blocking slave with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_blocking_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, scl_af, sda, sda_af, address, scl_pull, sda_pull)
    }
}

impl<'d> I2cSlave<'d, Async> {
    /// Create an async slave using typed pins.
    pub fn new<T: Instance, Scl: SclPin<T>, Sda: SdaPin<T>>(
        peri: Peri<'d, T>,
        scl: Peri<'d, Scl>,
        sda: Peri<'d, Sda>,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
        irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_af(peri, scl, Scl::AF, sda, Sda::AF, address, scl_pull, sda_pull, irq)
    }

    /// Create an async slave with explicit alternate-function numbers.
    #[allow(clippy::too_many_arguments)]
    pub fn new_af<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        let this = Self::new_inner(peri, scl, scl_af, sda, sda_af, address, scl_pull, sda_pull)?;
        this.enable_interrupts::<T>();
        Ok(this)
    }
}

impl<'d, M: Mode> I2cSlave<'d, M> {
    /// Arm the NVIC vector for interrupt-driven use.
    ///
    /// Called by the async constructors only; blocking instances never
    /// enable the vector.
    fn enable_interrupts<T: Instance>(&self) {
        T::Interrupt::unpend();
        T::Interrupt::set_priority(Priority::P3);
        unsafe {
            T::Interrupt::enable();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl GpioPin + 'd>,
        scl_af: AlternateFunction,
        sda: Peri<'d, impl GpioPin + 'd>,
        sda_af: AlternateFunction,
        address: u8,
        scl_pull: Pull,
        sda_pull: Pull,
    ) -> Result<Self, Error> {
        let info = T::info();
        enable_clock(info)?;

        let scl = configure_i2c_pin(scl, scl_af, scl_pull);
        let sda = configure_i2c_pin(sda, sda_af, sda_pull);

        init_slave(info, address)?;

        Ok(Self {
            info,
            _scl: scl,
            _sda: sda,
            _mode: PhantomData,
        })
    }

    fn regs(&self) -> &'static RegisterBlock {
        self.info.regs()
    }

    /// Block until the slave address is detected; return the transfer direction.
    ///
    /// Deliberately unbounded: a slave's job is to wait for a master, and a
    /// quiet bus is idle, not wedged. Pair with `embassy_time::with_timeout`
    /// if the application needs a deadline.
    pub fn blocking_listen(&mut self) -> Result<SlaveOp, Error> {
        let regs = self.regs();
        loop {
            check_errors(regs)?;
            if regs.sr().read().slave_addr_det().bit_is_set() {
                clear_slave_addr(regs);
                // RW_MODE mirrors the R/W bit: 0 = master write / slave read.
                let op = if regs.sr().read().rw_mode().bit_is_set() {
                    SlaveOp::Write
                } else {
                    SlaveOp::Read
                };
                return Ok(op);
            }
        }
    }

    /// Receive `buffer.len()` bytes as slave (master write).
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let regs = self.regs();
        for slot in buffer.iter_mut() {
            set_receive_mode(regs, true);
            loop {
                check_errors(regs)?;
                if regs.sr().read().dbr_full().bit_is_set() {
                    clear_rx_full(regs);
                    break;
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_slave_stop(regs);
                    return Ok(());
                }
            }
            *slot = read_dbr(regs);
        }
        Ok(())
    }

    /// Transmit `bytes` as slave (master read).
    pub fn blocking_write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let regs = self.regs();
        for &byte in bytes {
            master_send_byte(regs, byte);
            clear_tx_empty(regs);
            loop {
                check_errors(regs)?;
                if regs.sr().read().idbr_empty().bit_is_set() {
                    clear_tx_empty(regs);
                    break;
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_slave_stop(regs);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Wait for a STOP condition after a slave transfer.
    pub fn blocking_wait_stop(&mut self) -> Result<(), Error> {
        let regs = self.regs();
        set_receive_mode(regs, true);
        loop {
            check_errors(regs)?;
            if regs.sr().read().slave_stop_det().bit_is_set() {
                clear_slave_stop(regs);
                return Ok(());
            }
        }
    }
}

impl<'d> I2cSlave<'d, Async> {
    async fn wait_mask<F>(&mut self, mut ready: F, irq: IrqEnables) -> Result<(), Error>
    where
        F: FnMut(&RegisterBlock) -> Poll<Result<(), Error>>,
    {
        let regs = self.regs();
        let state = self.info.state;
        // Error enables ride along with every wait.
        let on = IrqEnables {
            arb_loss: true,
            bus_error: true,
            ..irq
        };
        let off = IrqEnables::default();

        // If dropped mid-wait the armed enables would outlive the waiter and
        // the ISR would keep firing with nothing to wake; disarm them.
        let irq_guard = OnDrop::new(|| {
            set_irq_enables(regs, off);
            state.armed.store(false, Ordering::Release);
        });
        let result = poll_fn(|cx| {
            state.waker.register(cx.waker());
            match ready(regs) {
                Poll::Ready(result) => {
                    set_irq_enables(regs, off);
                    state.armed.store(false, Ordering::Release);
                    Poll::Ready(result)
                }
                Poll::Pending => {
                    // Arm once per wait, not once per poll; see
                    // [`I2c::wait_on`] and `State::armed`.
                    if !state.armed.load(Ordering::Acquire) {
                        state.armed.store(true, Ordering::Release);
                        set_irq_enables(regs, on);
                        return match ready(regs) {
                            Poll::Ready(result) => {
                                set_irq_enables(regs, off);
                                state.armed.store(false, Ordering::Release);
                                Poll::Ready(result)
                            }
                            Poll::Pending => Poll::Pending,
                        };
                    }
                    Poll::Pending
                }
            }
        })
        .await;
        irq_guard.defuse();
        result
    }

    /// Wait until the slave address is detected; return the transfer direction.
    ///
    /// Deliberately without timeout (see [`I2cSlave::blocking_listen`]).
    ///
    /// Between consecutive transactions, consume the trailing STOP with
    /// [`I2cSlave::wait_stop`] (as the vendor slave example does): a latched,
    /// unconsumed STOP can keep the slave engine from matching the next
    /// address on single-board loopback, wedging both sides.
    pub async fn listen(&mut self) -> Result<SlaveOp, Error> {
        let mut op = SlaveOp::Read;
        self.wait_mask(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().slave_addr_det().bit_is_set() {
                    clear_slave_addr(regs);
                    op = if regs.sr().read().rw_mode().bit_is_set() {
                        SlaveOp::Write
                    } else {
                        SlaveOp::Read
                    };
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            IrqEnables {
                slave_addr: true,
                ..Default::default()
            },
        )
        .await?;
        Ok(op)
    }

    /// Receive `buffer.len()` bytes as slave.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            set_receive_mode(self.regs(), true);
            let mut done = false;
            self.wait_mask(
                |regs| {
                    if let Err(e) = check_errors(regs) {
                        return Poll::Ready(Err(e));
                    }
                    if regs.sr().read().dbr_full().bit_is_set() {
                        clear_rx_full(regs);
                        done = true;
                        Poll::Ready(Ok(()))
                    } else if regs.sr().read().slave_stop_det().bit_is_set() {
                        clear_slave_stop(regs);
                        done = false;
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                },
                IrqEnables {
                    rx_full: true,
                    slave_stop: true,
                    ..Default::default()
                },
            )
            .await?;
            if !done {
                return Ok(());
            }
            *slot = read_dbr(self.regs());
        }
        Ok(())
    }

    /// Transmit `bytes` as slave.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for &byte in bytes {
            master_send_byte(self.regs(), byte);
            clear_tx_empty(self.regs());
            let mut early_stop = false;
            self.wait_mask(
                |regs| {
                    if let Err(e) = check_errors(regs) {
                        return Poll::Ready(Err(e));
                    }
                    if regs.sr().read().idbr_empty().bit_is_set() {
                        clear_tx_empty(regs);
                        Poll::Ready(Ok(()))
                    } else if regs.sr().read().slave_stop_det().bit_is_set() {
                        clear_slave_stop(regs);
                        early_stop = true;
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                },
                IrqEnables {
                    tx_empty: true,
                    slave_stop: true,
                    ..Default::default()
                },
            )
            .await?;
            if early_stop {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Wait for a STOP condition after a slave transfer.
    pub async fn wait_stop(&mut self) -> Result<(), Error> {
        set_receive_mode(self.regs(), true);
        self.wait_mask(
            |regs| {
                if let Err(e) = check_errors(regs) {
                    return Poll::Ready(Err(e));
                }
                if regs.sr().read().slave_stop_det().bit_is_set() {
                    clear_slave_stop(regs);
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            },
            IrqEnables {
                slave_stop: true,
                ..Default::default()
            },
        )
        .await
    }
}

impl<'d, M: Mode> Drop for I2cSlave<'d, M> {
    fn drop(&mut self) {
        shutdown(self.info);
    }
}

// --- embedded-hal ----------------------------------------------------------

impl<'d, M: Mode> embedded_hal::i2c::ErrorType for I2c<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for I2c<'d, M> {
    fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Self::Error> {
        self.blocking_transaction(address, operations)
    }
}

impl<'d> embedded_hal_async::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for I2c<'d, Async> {
    async fn transaction(&mut self, address: u8, operations: &mut [EhOperation<'_>]) -> Result<(), Self::Error> {
        I2c::transaction(self, address, operations).await
    }
}

// --- embedded-hal 0.2 blocking traits ----------------------------------------

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, M> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, M> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, M> {
    type Error = Error;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        use embedded_hal_02::blocking::i2c::Operation;
        for op in operations.iter_mut() {
            match op {
                Operation::Read(buffer) => self.blocking_read(address, buffer)?,
                Operation::Write(bytes) => self.blocking_write(address, bytes)?,
            }
        }
        Ok(())
    }
}

// --- Instance / pins -------------------------------------------------------

trait SealedInstance {
    fn info() -> &'static Info;
}

/// I2C instance (I2C0–I2C2).
#[allow(private_bounds, private_interfaces)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// NVIC vector for this I2C.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_i2c {
    ($peri:ident, $interrupt:ident, $base:expr, $rcc:ident, $pclk:ident, $state:ident) => {
        #[allow(private_interfaces)]
        impl SealedInstance for peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: $base as *const RegisterBlock,
                    peripheral: Peripheral::$rcc,
                    pclk: PclkSel::$pclk,
                    state: &$state,
                };
                &INFO
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_i2c!(I2C0, I2C0, 0x4000_7000, I2c0, Pclk0, STATE_I2C0);
impl_i2c!(I2C1, I2C1, 0x4001_4000, I2c1, Pclk1, STATE_I2C1);
impl_i2c!(I2C2, I2C2, 0x4001_5000, I2c2, Pclk1, STATE_I2C2);

/// SCL pin for an I2C instance.
pub trait SclPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

/// SDA pin for an I2C instance.
pub trait SdaPin<T: Instance>: GpioPin {
    /// Datasheet alternate-function number for this pin/signal.
    const AF: AlternateFunction;
}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $trait:ident, $af:ident) => {
        impl $trait<peripherals::$instance> for peripherals::$pin {
            const AF: AlternateFunction = AlternateFunction::$af;
        }
    };
}

// Datasheet Table 4-3: every documented I2C remap uses Fun=3.
impl_pin!(PA2, I2C0, SclPin, Function3);
impl_pin!(PA3, I2C0, SdaPin, Function3);
impl_pin!(PA14, I2C0, SclPin, Function3);
impl_pin!(PA15, I2C0, SdaPin, Function3);
impl_pin!(PB10, I2C0, SclPin, Function3);
impl_pin!(PB11, I2C0, SdaPin, Function3);

impl_pin!(PA6, I2C1, SclPin, Function3);
impl_pin!(PA7, I2C1, SdaPin, Function3);
impl_pin!(PB14, I2C1, SclPin, Function3);
impl_pin!(PB15, I2C1, SdaPin, Function3);
impl_pin!(PC10, I2C1, SclPin, Function3);

impl_pin!(PA10, I2C2, SclPin, Function3);
impl_pin!(PA11, I2C2, SdaPin, Function3);
impl_pin!(PB7, I2C2, SdaPin, Function3);
impl_pin!(PC2, I2C2, SclPin, Function3);
impl_pin!(PC3, I2C2, SdaPin, Function3);
impl_pin!(PC15, I2C2, SdaPin, Function3);
