//! LPSPI driver for MCXA276.
//!
//! This module provides SPI master and slave drivers with both blocking and async
//! (interrupt-driven) modes. The async APIs are interrupt-driven: the ISR services
//! the FIFOs and wakes the awaiting task via a `WaitCell`.
//!
//! # DMA Support
//!
//! - [`SpiDma`] provides DMA-based SPI **master** transfers.
//! - [`SpiSlaveDma`] provides DMA-based SPI **slave** transfers.
//!
//! The master DMA implementation uses scatter/gather DMA to handle PCS (chip select)
//! de-assertion automatically at the end of a burst.
//!
//! ## Transfer Modes
//!
//! LPSPI is electrically full-duplex (every transmitted frame clocks in a received
//! frame), but many “half-duplex” *protocols* are implemented by sequencing phases.
//!
//! - **TX-only**: transmit bytes while discarding the concurrently received bytes
//!   (e.g. [`Spi::write`], [`SpiDma::write_dma`]).
//! - **RX-only**: receive bytes by transmitting dummy bytes (0x00) to generate clocks
//!   (e.g. [`Spi::read`], [`SpiDma::read_dma`]).
//! - **Full-duplex**: transmit and receive at the same time
//!   (e.g. [`Spi::transfer`], [`SpiDma::transfer_dma`]).
//!
//! For “write-then-read” protocols that require chip-select held across both phases,
//! prefer a single full-duplex burst (send the command bytes followed by dummy bytes,
//! then ignore the initial received bytes).
//!
//! ## Key Implementation Details
//!
//! ### Why RX DMA is Always Configured
//!
//! Even for TX-only transfers, RX DMA must be configured to drain the receive FIFO:
//! - LPSPI is full-duplex at the hardware level; every TX generates an RX byte
//! - If RX FIFO fills up and NOSTALL=0, the transfer stalls
//! - RX DMA completion is used as the "transfer done" signal (not TX completion)
//!
//! ### Scatter/Gather for PCS De-assertion
//!
//! The TX DMA uses a two-TCD scatter/gather chain:
//! 1. **Main TCD**: Transfers data bytes with ESG=1, chains to software TCD
//! 2. **Software TCD**: Writes TCR with CONT=0 to de-assert PCS after last byte
//!
//! ### Byte Swap Mode Addressing
//!
//! With BYSW=1 (byte swap enabled) and 8-bit frames:
//! - TX DMA writes to TDR+3 (byte 3 of the 32-bit register)
//! - RX DMA reads from RDR+3 (byte 3 of the 32-bit register)
//!
//! This ensures correct byte ordering on the wire.
//!
//! ### Transfer Completion Detection
//!
//! Proper completion requires waiting for:
//! 1. RX DMA major loop complete (DONE bit set via DREQ)
//! 2. TX FIFO empty (TXCOUNT == 0)
//! 3. Module not busy (MBF == 0)
//!
//! Only then has PCS been de-asserted and the transfer fully completed.
//!
//! # Examples
//!
//! See the MCXA examples:
//! - `examples/mcxa/src/bin/spi_master_blocking.rs`
//! - `examples/mcxa/src/bin/spi_interrupt_master.rs`
//! - `examples/mcxa/src/bin/spi_master_dma.rs`
//! - `examples/mcxa/src/bin/spi_slave_blocking.rs`
//! - `examples/mcxa/src/bin/spi_interrupt_slave.rs`
//! - `examples/mcxa/src/bin/spi_slave_dma.rs`
//! - `examples/mcxa/src/bin/spi_b2b_master.rs` / `spi_b2b_slave.rs`

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::hint::spin_loop;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
pub use embedded_hal_02::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode as SpiMode, Phase, Polarity};
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::periph_helpers::{Div4, LpspiClockSel, LpspiConfig};
use crate::clocks::{ClockError, Gate, PoweredClock, enable_and_reset};
use crate::gpio::{AnyPin, GpioPin, SealedPin};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac};

// =============================================================================
// REGISTER BIT CONSTANTS
// =============================================================================

/// All clearable status flags (TEF, REF, DMF, FCF, WCF, TCF)
const LPSPI_ALL_STATUS_FLAGS: u32 = 0x3F00;

/// TCR register bit masks
const TCR_CONT: u32 = 1 << 21;
const TCR_CONTC: u32 = 1 << 20;
const TCR_RXMSK: u32 = 1 << 19;
const TCR_TXMSK: u32 = 1 << 18;
const TCR_BYSW: u32 = 1 << 22;
const TCR_PCS_MASK: u32 = 0x3 << 24;

/// FIFO size for MCXA276 LPSPI (4 words)
const LPSPI_FIFO_SIZE: u8 = 4;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Flush TX and RX FIFOs for a given LPSPI register block
#[inline]
fn flush_fifos(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.rtf().txfifo_rst().rrf().rxfifo_rst());
}

/// Clear all status flags for a given LPSPI register block
#[inline]
fn clear_status_flags(spi: &pac::lpspi0::RegisterBlock) {
    spi.sr().write(|w| unsafe { w.bits(LPSPI_ALL_STATUS_FLAGS) });
}

/// Clear NOSTALL bit in CFGR1 (disables "no stall" mode, meaning LPSPI will stall
/// when TX FIFO is empty or RX FIFO is full, preventing data loss)
#[inline]
fn clear_nostall(spi: &pac::lpspi0::RegisterBlock) {
    spi.cfgr1().modify(|_, w| w.nostall().disable());
}

/// Read TCR with errata workaround (ERR050606)
/// The TCR register may return stale values, so we read it twice with an SR read in between
/// and loop until we get consistent values.
#[inline]
fn read_tcr_with_errata_workaround(spi: &pac::lpspi0::RegisterBlock) -> u32 {
    let mut last = spi.tcr().read().bits();
    loop {
        // Read SR to force a different register access (errata workaround)
        let _ = spi.sr().read();
        let now = spi.tcr().read().bits();
        if now == last {
            break now;
        }
        last = now;
    }
}

/// Common setup sequence for async SPI transfers: disable module, flush FIFOs, clear status,
/// disable interrupts, clear NOSTALL, and re-enable.
#[inline]
fn prepare_for_transfer(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.men().disabled());
    flush_fifos(spi);
    clear_status_flags(spi);
    spi.ier().write(|w| w); // Disable all interrupts
    clear_nostall(spi);
    spi.cr().modify(|_, w| w.men().enabled());
}

/// Common setup sequence for blocking SPI transfers: disable module, flush FIFOs,
/// clear status, clear NOSTALL, and re-enable. Does not touch IER (no interrupts used).
#[inline]
fn prepare_for_blocking_transfer(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.men().disabled());
    flush_fifos(spi);
    clear_status_flags(spi);
    clear_nostall(spi);
    spi.cr().modify(|_, w| w.men().enabled());
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Shorthand for `Result<T>`.
pub type Result<T> = core::result::Result<T, Error>;

/// Error information type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Transfer error.
    TransferError,
    /// Transmit FIFO error.
    TxFifoError,
    /// Receive FIFO error.
    RxFifoError,
    /// Module busy timeout.
    Timeout,
    /// SPI peripheral is already in use.
    AlreadyInUse,
    /// Other internal errors or unexpected state.
    Other,
}

/// Maximum number of iterations for busy-wait loops.
///
/// This prevents hard hangs if the peripheral gets stuck due to misconfiguration
/// (e.g. missing PCS, stalled clocks, or hardware error).
const SPIN_LIMIT: u32 = 10_000_000;

#[inline]
fn spin_wait_while(mut cond: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SPIN_LIMIT {
        if !cond() {
            return Ok(());
        }
        spin_loop();
    }
    Err(Error::Timeout)
}

#[inline]
fn dma_start_fence() {
    // Ensure all TCD/register writes are committed before enabling DMA requests.
    compiler_fence(Ordering::Release);
    cortex_m::asm::dsb();
}

// =============================================================================
// ASYNC STATE MANAGEMENT
// =============================================================================

/// Transfer state for interrupt-driven operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransferState {
    /// No transfer in progress
    Idle = 0,
    /// Transfer in progress
    InProgress = 1,
    /// Transfer completed successfully
    Complete = 2,
    /// Transfer error occurred
    Error = 3,
}

impl From<u8> for TransferState {
    fn from(val: u8) -> Self {
        match val {
            0 => TransferState::Idle,
            1 => TransferState::InProgress,
            2 => TransferState::Complete,
            3 => TransferState::Error,
            _ => TransferState::Idle,
        }
    }
}

// =============================================================================
// SLAVE IRQ-DRIVEN ASYNC STATE
// =============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
enum SlaveIrqOp {
    Idle = 0,
    Rx = 1,
    Tx = 2,
    Transfer = 3,
}

struct SlaveIrqStateInner {
    op: SlaveIrqOp,

    rx_ptr: *mut u8,
    rx_len: usize,
    rx_pos: usize,
    rx_store_len: usize,

    tx_ptr: *const u8,
    tx_len: usize,
    tx_pos: usize,
    tx_source_len: usize,

    error: Option<Error>,
}

impl SlaveIrqStateInner {
    const fn new() -> Self {
        Self {
            op: SlaveIrqOp::Idle,

            rx_ptr: core::ptr::null_mut(),
            rx_len: 0,
            rx_pos: 0,
            rx_store_len: 0,

            tx_ptr: core::ptr::null(),
            tx_len: 0,
            tx_pos: 0,
            tx_source_len: 0,

            error: None,
        }
    }
}

struct SlaveIrqState {
    inner: UnsafeCell<SlaveIrqStateInner>,
}

unsafe impl Sync for SlaveIrqState {}

impl SlaveIrqState {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(SlaveIrqStateInner::new()),
        }
    }

    #[inline]
    fn with<R>(&'static self, f: impl FnOnce(&mut SlaveIrqStateInner) -> R) -> R {
        critical_section::with(|_| unsafe { f(&mut *self.inner.get()) })
    }
}

/// Interrupt handler for SPI async operations.
///
/// Disables all interrupts and wakes the WaitCell. The async code
/// will check status flags to determine what happened.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

#[inline]
unsafe fn handle_slave_rx_irq<T: Instance>(regs: &pac::lpspi0::RegisterBlock, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.ref_().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::RxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

    // Drain RX FIFO into the user buffer.
    while st.rx_pos < st.rx_len && regs.fsr().read().rxcount().bits() > 0 {
        let byte = regs.rdr().read().bits() as u8;
        unsafe { *st.rx_ptr.add(st.rx_pos) = byte };
        st.rx_pos += 1;
    }

    if st.rx_pos >= st.rx_len {
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
    }
}

#[inline]
unsafe fn handle_slave_tx_irq<T: Instance>(regs: &pac::lpspi0::RegisterBlock, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.tef().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::TxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

    // Refill TX FIFO as long as there is space.
    while st.tx_pos < st.tx_len && regs.fsr().read().txcount().bits() < LPSPI_FIFO_SIZE {
        let byte = unsafe { *st.tx_ptr.add(st.tx_pos) };
        regs.tdr().write(|w| unsafe { w.bits(byte as u32) });
        st.tx_pos += 1;
    }

    // Once all bytes are queued, wait for frame complete + TX FIFO empty.
    if st.tx_pos >= st.tx_len {
        // Reduce interrupts: we only need frame-complete (and error) now.
        regs.ier().write(|w| w.fcie().enable().teie().enable());

        let tx_empty = regs.fsr().read().txcount().bits() == 0;
        let sr = regs.sr().read();
        if tx_empty && sr.fcf().is_completed() {
            regs.sr().write(|w| w.fcf().clear_bit_by_one());
            st.op = SlaveIrqOp::Idle;
            regs.ier().write(|w| w);
            T::wait_cell().wake();
        }
    }
}

#[inline]
unsafe fn handle_slave_transfer_irq<T: Instance>(regs: &pac::lpspi0::RegisterBlock, st: &mut SlaveIrqStateInner) {
    let sr = regs.sr().read();

    if sr.ref_().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::RxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

    if sr.tef().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::TxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

    // Drain RX FIFO into the user buffer (or discard if rx_store_len < rx_len).
    while st.rx_pos < st.rx_len && regs.fsr().read().rxcount().bits() > 0 {
        let byte = regs.rdr().read().bits() as u8;
        if st.rx_pos < st.rx_store_len {
            unsafe { *st.rx_ptr.add(st.rx_pos) = byte };
        }
        st.rx_pos += 1;
    }

    // Keep TX FIFO topped up so we don't underrun while master clocks data.
    while st.tx_pos < st.tx_len && regs.fsr().read().txcount().bits() < LPSPI_FIFO_SIZE {
        let byte = if st.tx_pos < st.tx_source_len {
            unsafe { *st.tx_ptr.add(st.tx_pos) }
        } else {
            0
        };
        regs.tdr().write(|w| unsafe { w.bits(byte as u32) });
        st.tx_pos += 1;
    }

    // Completion: RX-side reached the requested length.
    // This mirrors the SDK behaviour: when both txData and rxData are provided,
    // RX completion is used as the end-of-transfer signal.
    if st.rx_pos >= st.rx_len {
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
    }
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        // Fast-path: ignore if nothing is armed.
        if regs.ier().read().bits() == 0 {
            return;
        }

        // If an async SLAVE op is active, service it in the ISR (drain/fill FIFOs).
        let mut handled_slave = false;
        T::slave_irq_state().with(|st| match st.op {
            SlaveIrqOp::Rx => {
                handled_slave = true;
                unsafe { handle_slave_rx_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Tx => {
                handled_slave = true;
                unsafe { handle_slave_tx_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Transfer => {
                handled_slave = true;
                unsafe { handle_slave_transfer_irq::<T>(regs, st) };
            }
            SlaveIrqOp::Idle => {}
        });

        if handled_slave {
            return;
        }

        // If any interrupts are enabled, disable them all and wake the task.
        // The async code will check status flags to determine what happened.
        if regs.ier().read().bits() != 0 {
            // Disable all interrupts
            regs.ier().write(|w| w);
            T::wait_cell().wake();
        }
    }
}

mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

impl<T: GpioPin> sealed::Sealed for T {}

trait SealedInstance {
    fn regs() -> &'static pac::lpspi0::RegisterBlock;
    fn wait_cell() -> &'static WaitCell;
    fn slave_irq_state() -> &'static SlaveIrqState;
}

use crate::dma::{Channel as DmaChannelTrait, DmaChannel, DmaRequest, EnableInterrupt};

/// SPI Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpspiConfig> {
    /// Interrupt for this SPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance;
    /// Type-safe DMA request source for TX
    type TxDmaRequest: DmaRequest;
    /// Type-safe DMA request source for RX
    type RxDmaRequest: DmaRequest;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPSPI $n>] {
                    fn regs() -> &'static pac::lpspi0::RegisterBlock {
                        unsafe { &*pac::[<Lpspi $n>]::ptr() }
                    }

                    fn wait_cell() -> &'static WaitCell {
                        static WAIT_CELL: WaitCell = WaitCell::new();
                        &WAIT_CELL
                    }

                    fn slave_irq_state() -> &'static SlaveIrqState {
                        static SLAVE_IRQ_STATE: SlaveIrqState = SlaveIrqState::new();
                        &SLAVE_IRQ_STATE
                    }
                }

                impl Instance for crate::peripherals::[<LPSPI $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<LPSPI $n>];
                    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance
                        = crate::clocks::periph_helpers::LpspiInstance::[<Lpspi $n>];
                    type TxDmaRequest = crate::dma::[<Lpspi $n TxRequest>];
                    type RxDmaRequest = crate::dma::[<Lpspi $n RxRequest>];
                }
            }
        )*
    };
}

impl_instance!(0, 1);

/// Bit order
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BitOrder {
    /// Most significant bit first
    #[default]
    MsbFirst,
    /// Least significant bit first
    LsbFirst,
}

/// Chip select pin selection
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChipSelect {
    #[default]
    Pcs0 = 0,
    Pcs1 = 1,
    Pcs2 = 2,
    Pcs3 = 3,
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

/// SPI master configuration
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// Clock polarity
    pub polarity: Polarity,
    /// Clock phase
    pub phase: Phase,
    /// Bit order
    pub bit_order: BitOrder,
    /// Bits per frame (1-4096, typically 8).
    /// Values outside this range will be clamped: 0 becomes 1, >4096 becomes 4096.
    pub bits_per_frame: u16,
    /// Chip select to use
    pub chip_select: ChipSelect,
    /// SCK divider (0-255). Baud = src_clk / (prescaler * (SCKDIV + 2))
    pub sck_div: u8,
    /// Prescaler value (0-7, divide by 1,2,4,8,16,32,64,128)
    pub prescaler: u8,
}

impl Config {
    /// Create a new SPI configuration with default settings.
    /// Uses mode 0 (CPOL=0, CPHA=0), MSB first, 8 bits.
    pub fn new() -> Self {
        Self {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
            chip_select: ChipSelect::Pcs0,
            sck_div: 0,
            prescaler: 0,
        }
    }

    /// Set clock polarity
    pub fn polarity(&mut self, pol: Polarity) -> &mut Self {
        self.polarity = pol;
        self
    }

    /// Set clock phase
    pub fn phase(&mut self, ph: Phase) -> &mut Self {
        self.phase = ph;
        self
    }

    /// Set bit order
    pub fn bit_order(&mut self, order: BitOrder) -> &mut Self {
        self.bit_order = order;
        self
    }

    /// Set bits per frame (valid range: 1-4096, will be clamped)
    pub fn bits_per_frame(&mut self, bits: u16) -> &mut Self {
        self.bits_per_frame = bits;
        self
    }

    /// Set chip select
    pub fn chip_select(&mut self, cs: ChipSelect) -> &mut Self {
        self.chip_select = cs;
        self
    }

    /// Set SCK divider. Baud = src_clk / (prescaler * (SCKDIV + 2))
    pub fn sck_div(&mut self, div: u8) -> &mut Self {
        self.sck_div = div;
        self
    }

    /// Set prescaler (0-7, divide by 1,2,4,8,16,32,64,128)
    pub fn prescaler(&mut self, prescaler: u8) -> &mut Self {
        self.prescaler = prescaler.min(7);
        self
    }

    /// Calculate baud rate parameters for a target frequency.
    /// Returns (prescaler, sck_div) for the closest achievable baud rate.
    pub fn for_frequency(&mut self, src_hz: u32, target_hz: u32) -> &mut Self {
        let (prescaler, sck_div) = compute_baud_params(src_hz, target_hz);
        self.prescaler = prescaler;
        self.sck_div = sck_div;
        self
    }
}

impl Default for Config {
    /// Provide a sensible default matching `Config::new()`.
    ///
    /// This avoids a footgun where a derived `Default` would leave
    /// `bits_per_frame` as 0, resulting in a 1-bit frame configuration
    /// if used directly.
    fn default() -> Self {
        Self::new()
    }
}

/// Compute prescaler and SCKDIV for the desired baud rate.
/// Returns (prescaler_value, sckdiv) where prescaler_value is 0-7 (divide by 1,2,4,8,16,32,64,128)
/// and sckdiv is 0-255.
/// Baud = src_hz / (prescaler * (SCKDIV + 2))
///
/// If baud_hz is 0, returns default values (0, 0) which gives the slowest possible baud rate.
fn compute_baud_params(src_hz: u32, baud_hz: u32) -> (u8, u8) {
    // Guard against division by zero - if baud_hz is 0, return slowest settings
    if baud_hz == 0 {
        return (0, 0);
    }

    let prescalers: [u32; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
    let mut best_prescaler: u8 = 0;
    let mut best_sckdiv: u8 = 0;
    let mut best_error: u32 = u32::MAX;

    for (i, &prescaler) in prescalers.iter().enumerate() {
        // prescaler is always >= 1, and baud_hz > 0 (checked above), so denom > 0
        let denom = prescaler.saturating_mul(baud_hz);
        let sckdiv_calc = (src_hz + denom / 2) / denom;
        if sckdiv_calc < 2 {
            continue;
        }
        let sckdiv = (sckdiv_calc - 2).min(255) as u8;
        let actual_baud = src_hz / (prescaler * (sckdiv as u32 + 2));
        let error = actual_baud.abs_diff(baud_hz);
        if error < best_error {
            best_error = error;
            best_prescaler = i as u8;
            best_sckdiv = sckdiv;
        }
    }
    (best_prescaler, best_sckdiv)
}

/// SPI Master Driver.
pub struct Spi<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
    chip_select: ChipSelect,
}

impl<'d, T: Instance> Spi<'d, T, Blocking> {
    /// Create a new blocking instance of the SPI Master driver.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> Spi<'d, T, Async> {
    /// Create a new async (interrupt-driven) instance of the SPI Master driver.
    ///
    /// This requires binding the interrupt handler using `bind_interrupts!` macro.
    pub fn new_async(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }

    /// Async write (interrupt-driven).
    ///
    /// Sends data to the slave, discarding any received data.
    /// This is a TX-only transfer: any concurrently received bytes are ignored.
    pub async fn write(&mut self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        prepare_for_transfer(spi);

        let is_pcs_continuous = tx.len() > 1;

        // First TCR write: set PCS only (CONT=0, CONTC=0, RXMSK=0, TXMSK=0)
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Second TCR write: set CONT/CONTC when needed, and set RXMSK=1 (mask RX).
        self.apply_transfer_tcr(is_pcs_continuous, true, false);
        Self::wait_tx_fifo_empty()?;

        // Initial FIFO fill: fill as much as possible without waiting.
        // This pre-fill step avoids an unnecessary interrupt/wait cycle for small transfers
        // that fit entirely in the FIFO.
        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        // Continue sending remaining data using interrupt-driven waits
        while tx_idx < tx.len() {
            T::wait_cell()
                .wait_for(|| {
                    // Enable TX interrupt before waiting
                    spi.ier().modify(|_, w| w.tdie().enable());
                    // Check if FIFO has space
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;

            while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
                spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
                tx_idx += 1;
            }
        }

        // Clear CONT/CONTC at end for PCS de-assertion.
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, true, false);
        }

        // Wait for transfer complete
        Self::wait_tx_fifo_empty()?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }

    /// Async read (interrupt-driven).
    ///
    /// Reads data from the slave by sending zeros.
    /// This is an RX-only transfer: dummy bytes (0x00) are transmitted to generate clocks.
    pub async fn read(&mut self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size() as usize;
        prepare_for_transfer(spi);

        let is_pcs_continuous = rx.len() > 1;

        // First TCR write: set PCS only (CONT=0, CONTC=0, RXMSK=0, TXMSK=0)
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Second TCR write: set CONT/CONTC when needed; RXMSK=0 (receive data), TXMSK=0 (send zeros).
        self.apply_transfer_tcr(is_pcs_continuous, false, false);
        Self::wait_tx_fifo_empty()?;

        let mut tx_remaining = rx.len();
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size;

        while tx_remaining > 0 || rx_idx < rx.len() {
            // Send zeros to clock data in (master must send to receive)
            while tx_remaining > 0
                && Self::get_tx_fifo_count() < Self::get_fifo_size()
                && (rx.len() - rx_idx) - tx_remaining < rx_fifo_max_bytes
            {
                spi.tdr().write(|w| unsafe { w.bits(0) });
                tx_remaining -= 1;
            }

            // Read any available RX data
            while Self::get_rx_fifo_count() > 0 && rx_idx < rx.len() {
                rx[rx_idx] = spi.rdr().read().bits() as u8;
                rx_idx += 1;
            }

            if tx_remaining > 0 || rx_idx < rx.len() {
                T::wait_cell()
                    .wait_for(|| {
                        // Enable RX interrupt before waiting
                        spi.ier().modify(|_, w| w.rdie().enable());
                        // Check if RX FIFO has data or TX FIFO has space
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        // Clear CONT/CONTC at end for PCS de-assertion.
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, false, false);
        }

        Ok(())
    }

    /// Async full-duplex transfer (interrupt-driven).
    ///
    /// Simultaneously writes TX data and reads RX data.
    ///
    /// If `tx` and `rx` have different lengths, the transfer length is the
    /// maximum of the two. If `tx` is shorter, zeros are transmitted for the
    /// remaining bytes. If `rx` is shorter, extra received bytes are discarded.
    pub async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        let tx_len = tx.len();
        let rx_len = rx.len();
        let len = tx_len.max(rx_len);
        if len == 0 {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size() as usize;
        prepare_for_transfer(spi);

        let is_pcs_continuous = len > 1;

        // First TCR write: set PCS only (CONT=0, CONTC=0, RXMSK=0, TXMSK=0)
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Second TCR write: set CONT/CONTC when needed.
        // RXMSK=0 (receive data), TXMSK=0 (send data) - FULL DUPLEX
        self.apply_transfer_tcr(is_pcs_continuous, false, false);
        Self::wait_tx_fifo_empty()?;

        let mut tx_idx = 0usize;
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size;

        while tx_idx < len || rx_idx < len {
            // Send TX data while respecting RX FIFO capacity
            while tx_idx < len
                && Self::get_tx_fifo_count() < Self::get_fifo_size()
                && (len - rx_idx) - (len - tx_idx) < rx_fifo_max_bytes
            {
                // Send actual TX data or zero padding if tx is exhausted
                let byte = if tx_idx < tx_len { tx[tx_idx] } else { 0 };
                spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
                tx_idx += 1;
            }

            // Read any available RX data
            while Self::get_rx_fifo_count() > 0 && rx_idx < len {
                let byte = spi.rdr().read().bits() as u8;
                // Store in rx buffer or discard if rx is exhausted
                if rx_idx < rx_len {
                    rx[rx_idx] = byte;
                }
                rx_idx += 1;
            }

            // Wait for more activity if needed
            if tx_idx < len || rx_idx < len {
                T::wait_cell()
                    .wait_for(|| {
                        // Enable RX interrupt before waiting
                        spi.ier().modify(|_, w| w.rdie().enable());
                        // Check if RX FIFO has data or TX FIFO has space
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        // Clear CONT/CONTC at end for PCS de-assertion.
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, false, false);
        }

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> Spi<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        // Configure clocks - use FRO_HF_DIV as clock source
        let clock_config = LpspiConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroHfDiv,
            div: Div4::no_div(),
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        // Configure pins (including PCS for master mode)
        sck.mux();
        mosi.mux();
        miso.mux();
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        // Initialize the SPI peripheral
        Self::set_config(&config)?;

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            _phantom: PhantomData,
            chip_select: config.chip_select,
        })
    }

    fn set_config(config: &Config) -> Result<()> {
        let spi = T::regs();

        // 1) Disable module
        spi.cr().write(|w| w.men().disabled());

        // 2) Reset FIFOs and module
        spi.cr().modify(|_, w| w.rst().reset());
        spi.cr().modify(|_, w| w.rst().not_reset());
        spi.cr().modify(|_, w| w.rtf().txfifo_rst().rrf().rxfifo_rst());

        // 3) Configure as master mode with standard pin config (SIN in, SOUT out)
        // Also set PCS polarity to active low (PCS0 asserted low)
        spi.cfgr1().write(|w| unsafe {
            w.master().master_mode().pincfg().sin_in_sout_out().pcspol().bits(0) // PCS0 active low
        });

        // 4) Set baud rate and timing via CCR
        //
        // CCR fields:
        // - SCKDIV: SCK divider, determines the SCK frequency
        // - DBT: Delay Between Transfers - time between PCS de-assert and next assert
        // - PCSSCK: PCS-to-SCK Delay - time from PCS assert to first SCK edge
        // - SCKPCS: SCK-to-PCS Delay - time from last SCK edge to PCS de-assert
        //
        // Using sck_div for all timing values provides balanced, symmetric timing
        // that works well for most SPI devices. More aggressive timing (smaller
        // delays) could be achieved but may cause issues with slower peripherals.
        spi.ccr().write(|w| unsafe {
            w.sckdiv()
                .bits(config.sck_div)
                .dbt()
                .bits(config.sck_div)
                .pcssck()
                .bits(config.sck_div)
                .sckpcs()
                .bits(config.sck_div)
        });

        // 5) Set FIFO watermarks to 0 (trigger on any data)
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        // 6) Configure TCR (transfer command register)
        // bits_per_frame is already u16, so no cast needed
        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            };
            match config.chip_select {
                ChipSelect::Pcs0 => w.pcs().tx_pcs0(),
                ChipSelect::Pcs1 => w.pcs().tx_pcs1(),
                ChipSelect::Pcs2 => w.pcs().tx_pcs2(),
                ChipSelect::Pcs3 => w.pcs().tx_pcs3(),
            };
            w.prescale().bits(config.prescaler)
        });

        // 7) Enable the module
        spi.cr().write(|w| w.men().enabled());

        Ok(())
    }

    /// Get TX FIFO count
    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount().bits()
    }

    /// Get RX FIFO count
    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount().bits()
    }

    /// Get FIFO size (4 for MCXA276)
    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    /// Wait for TX FIFO to be empty
    #[inline]
    fn wait_tx_fifo_empty() -> Result<()> {
        spin_wait_while(|| Self::get_tx_fifo_count() != 0)
    }

    /// Check if the module is busy.
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::regs().sr().read().mbf().is_busy()
    }

    /// Wait for all transfers to complete.
    pub fn flush(&self) -> Result<()> {
        spin_wait_while(|| self.is_busy())
    }

    /// Apply TCR settings for a transfer.
    #[inline]
    fn apply_transfer_tcr(&self, continuous_pcs: bool, rx_mask: bool, tx_mask: bool) {
        let spi = T::regs();
        spi.tcr().modify(|_, w| {
            let w = if tx_mask { w.txmsk().mask() } else { w.txmsk().normal() };
            let w = if rx_mask { w.rxmsk().mask() } else { w.rxmsk().normal() };
            let w = if continuous_pcs {
                w.contc().continue_().cont().enabled()
            } else {
                w.contc().start().cont().disabled()
            };
            match self.chip_select {
                ChipSelect::Pcs0 => w.pcs().tx_pcs0(),
                ChipSelect::Pcs1 => w.pcs().tx_pcs1(),
                ChipSelect::Pcs2 => w.pcs().tx_pcs2(),
                ChipSelect::Pcs3 => w.pcs().tx_pcs3(),
            }
        });
    }

    /// Full-duplex transfer (blocking).
    ///
    /// Transmits `tx[i]` while simultaneously receiving into `rx[i]` for each frame.
    ///
    /// If `tx` and `rx` have different lengths, the transfer length is the
    /// maximum of the two. If `tx` is shorter, zeros are transmitted for the
    /// remaining bytes. If `rx` is shorter, extra received bytes are discarded.
    pub fn blocking_transfer(&self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        let tx_len = tx.len();
        let rx_len = rx.len();
        let len = tx_len.max(rx_len);
        if len == 0 {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        // Configure TCR for a new transfer
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Set CONT=1, CONTC=1 for continuous PCS (if len > 1)
        let is_pcs_continuous = len > 1;
        if is_pcs_continuous {
            self.apply_transfer_tcr(true, false, false);
            Self::wait_tx_fifo_empty()?;
        }

        // Write all TX data while reading RX to prevent overflow
        let mut tx_remaining = len;
        let mut rx_remaining = len;
        let mut tx_idx = 0usize;
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size as usize;

        while tx_remaining > 0 {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;

            if rx_remaining - tx_remaining < rx_fifo_max_bytes {
                // Send actual TX data or zero padding if tx is exhausted
                let byte = if tx_idx < tx_len { tx[tx_idx] } else { 0 };
                spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
                tx_idx += 1;
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                let byte = spi.rdr().read().bits() as u8;
                // Store in rx buffer or discard if rx is exhausted
                if rx_idx < rx_len {
                    rx[rx_idx] = byte;
                }
                rx_idx += 1;
                rx_remaining -= 1;
            }
        }

        // Clear CONT/CONTC after all data written
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        // Read remaining RX data
        while rx_remaining > 0 {
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            let byte = spi.rdr().read().bits() as u8;
            // Store in rx buffer or discard if rx is exhausted
            if rx_idx < rx_len {
                rx[rx_idx] = byte;
            }
            rx_idx += 1;
            rx_remaining -= 1;
        }

        Ok(())
    }

    /// TX-only transfer (blocking).
    ///
    /// Transmits all bytes in `tx` and discards the concurrently received bytes.
    pub fn blocking_write(&self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        // Configure TCR for a new transfer
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Set CONT=1, CONTC=1 for continuous PCS, RXMSK=1 (ignore RX)
        let is_pcs_continuous = tx.len() > 1;
        self.apply_transfer_tcr(is_pcs_continuous, true, false);
        Self::wait_tx_fifo_empty()?;

        // Write all TX data
        for &byte in tx.iter() {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
        }

        // Clear CONT/CONTC after all data written
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        // Wait for transfer complete
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }

    /// RX-only transfer (blocking).
    ///
    /// Receives into `rx` by transmitting dummy bytes (0x00) to generate clocks.
    pub fn blocking_read(&self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        // Configure TCR for a new transfer
        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        // Set CONT=1, CONTC=1 for continuous PCS
        let is_pcs_continuous = rx.len() > 1;
        if is_pcs_continuous {
            self.apply_transfer_tcr(true, false, false);
            Self::wait_tx_fifo_empty()?;
        }

        // Transmit zeros while reading RX data
        let mut tx_remaining = rx.len();
        let mut rx_remaining = rx.len();
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size as usize;

        while tx_remaining > 0 {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;

            if rx_remaining - tx_remaining < rx_fifo_max_bytes {
                spi.tdr().write(|w| unsafe { w.bits(0) });
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                rx[rx_idx] = spi.rdr().read().bits() as u8;
                rx_idx += 1;
                rx_remaining -= 1;
            }
        }

        // Clear CONT/CONTC after all data written
        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        // Read remaining RX data
        while rx_remaining > 0 {
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            rx[rx_idx] = spi.rdr().read().bits() as u8;
            rx_idx += 1;
            rx_remaining -= 1;
        }

        Ok(())
    }
}

// embedded-hal 1.0 implementations
impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {
            Self::TransferError | Self::TxFifoError | Self::RxFifoError => embedded_hal_1::spi::ErrorKind::Other,
            _ => embedded_hal_1::spi::ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::ErrorType for Spi<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::SpiBus for Spi<'d, T, M> {
    fn read(&mut self, words: &mut [u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> core::result::Result<(), Self::Error> {
        // blocking_transfer now handles differing lengths: it transfers max(tx, rx) bytes,
        // padding TX with zeros if shorter, discarding extra RX if shorter.
        self.blocking_transfer(write, read)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> core::result::Result<(), Self::Error> {
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        for byte in words.iter_mut() {
            let tx_byte = *byte;

            // Wait for TX FIFO space
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            spi.tdr().write(|w| unsafe { w.bits(tx_byte as u32) });

            // Wait for RX data
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            *byte = spi.rdr().read().bits() as u8;
        }
        Ok(())
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        Spi::flush(self)
    }
}

impl<'d, T: Instance, M: Mode> embassy_embedded_hal::SetConfig for Spi<'d, T, M> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<()> {
        Self::set_config(config)
    }
}

// =============================================================================
// SPI Slave Implementation
// =============================================================================

/// SPI slave configuration
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct SlaveConfig {
    /// Clock polarity (must match master)
    pub polarity: Polarity,
    /// Clock phase (must match master)
    pub phase: Phase,
    /// Bit order (must match master)
    pub bit_order: BitOrder,
    /// Bits per frame (8 for typical use)
    pub bits_per_frame: u16,
}

impl SlaveConfig {
    /// Create a new slave configuration with defaults
    pub fn new() -> Self {
        Self {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
        }
    }

    /// Set clock polarity
    pub fn polarity(&mut self, polarity: Polarity) -> &mut Self {
        self.polarity = polarity;
        self
    }

    /// Set clock phase
    pub fn phase(&mut self, phase: Phase) -> &mut Self {
        self.phase = phase;
        self
    }

    /// Set bit order
    pub fn bit_order(&mut self, order: BitOrder) -> &mut Self {
        self.bit_order = order;
        self
    }

    /// Set bits per frame (valid range: 1-4096, will be clamped)
    pub fn bits_per_frame(&mut self, bits: u16) -> &mut Self {
        self.bits_per_frame = bits;
        self
    }
}

impl Default for SlaveConfig {
    /// Provide a sensible default matching `SlaveConfig::new()`.
    ///
    /// This ensures that `bits_per_frame` defaults to 8 instead of 0,
    /// which would otherwise configure the peripheral for 1-bit frames
    /// and break compatibility with common vendor examples.
    fn default() -> Self {
        Self::new()
    }
}

/// SPI Slave Driver.
pub struct SpiSlave<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
}

impl<'d, T: Instance> SpiSlave<'d, T, Blocking> {
    /// Create a new blocking instance of the SPI Slave driver.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: SlaveConfig,
    ) -> Result<Self> {
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> SpiSlave<'d, T, Async> {
    /// Create a new async (interrupt-driven) instance of the SPI Slave driver.
    ///
    /// This requires binding the interrupt handler using `bind_interrupts!` macro.
    pub fn new_async(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: SlaveConfig,
    ) -> Result<Self> {
        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }

    /// Get RX FIFO count
    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount().bits()
    }

    /// Get TX FIFO count
    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount().bits()
    }

    /// FIFO size
    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    /// Read TCR with errata workaround (ERR050606)
    #[inline]
    fn read_tcr_with_errata_workaround() -> u32 {
        read_tcr_with_errata_workaround(T::regs())
    }

    /// Async read from master.
    ///
    /// Interrupt-driven: arms an RX-only operation and lets the ISR drain the RX FIFO into `rx`,
    /// waking the task when `rx` is full or an error occurs.
    pub async fn read(&mut self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();

        // Prepare fresh RX-only slave transfer.
        // CFGR1 is only safely writable with MEN=0.
        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        // Disable interrupts before arming state.
        spi.ier().write(|w| w);

        // RX-only phase: don't drive MISO (TX masked), just receive MOSI.
        spi.tcr().modify(|_, w| w.rxmsk().normal().txmsk().mask());

        // Arm ISR state.
        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Rx;
            st.error = None;
            st.rx_ptr = rx.as_mut_ptr();
            st.rx_len = rx.len();
            st.rx_pos = 0;
        });

        // Enable RX data + RX error interrupts.
        spi.ier().write(|w| w.rdie().enable().reie().enable());

        // Sleep until the ISR completes the transfer or reports an error.
        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        Ok(())
    }

    /// Async write to master.
    ///
    /// Interrupt-driven: pre-fills TX FIFO, then enables `TDIE` so the ISR can keep the FIFO topped up.
    /// Completion is detected using `FCF` (frame complete) plus TX FIFO empty, similar to the MCUXpresso
    /// transactional driver.
    pub async fn write(&mut self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        // Prepare fresh TX-only slave transfer.
        // CFGR1 is only safely writable with MEN=0.
        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        // Use a higher TX watermark so TDF fires before the FIFO is completely empty.
        // This reduces underrun risk when refilling from ISR.
        spi.fcr().write(|w| unsafe { w.txwater().bits(1).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        // Disable interrupts before arming state.
        spi.ier().write(|w| w);

        // TX-only phase: don't store MOSI into RX FIFO.
        spi.tcr().modify(|_, w| w.rxmsk().mask().txmsk().normal());

        // Pre-fill TX FIFO in task context before enabling IRQs.
        let mut prefill = 0usize;
        while prefill < tx.len() && Self::get_tx_fifo_count() < fifo_size {
            spi.tdr().write(|w| unsafe { w.bits(tx[prefill] as u32) });
            prefill += 1;
        }

        // Arm ISR state (start after the prefill).
        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Tx;
            st.error = None;
            st.tx_ptr = tx.as_ptr();
            st.tx_len = tx.len();
            st.tx_pos = prefill;
        });

        // Enable TX data + TX error + frame-complete interrupts.
        spi.ier().write(|w| w.tdie().enable().teie().enable().fcie().enable());

        // Sleep until ISR completes the transfer or reports an error.
        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        // Ensure module is no longer busy.
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }

    /// Async full-duplex transfer (interrupt-driven).
    ///
    /// Sends tx data while simultaneously receiving rx data.
    /// This is essential for SPI slave since the master may send data
    /// while clocking out the slave's response.
    pub async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.is_empty() && rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        let tx_len = tx.len();
        let rx_len = rx.len();
        let total = core::cmp::max(tx_len, rx_len);
        if total == 0 {
            return Ok(());
        }

        // Prepare LPSPI for a fresh full-duplex slave transfer.
        // CFGR1 is only safely writable with MEN=0.
        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        // Watermarks: RX as soon as there's data, TX before FIFO empties.
        spi.fcr().write(|w| unsafe { w.txwater().bits(1).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        // Disable interrupts before arming state.
        spi.ier().write(|w| w);

        // Ensure full duplex: RX and TX unmasked.
        let tcr = Self::read_tcr_with_errata_workaround();
        let new_tcr = tcr & !(TCR_CONT | TCR_CONTC | TCR_RXMSK | TCR_TXMSK | TCR_PCS_MASK);
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });

        // Discard any already-buffered RX bytes.
        while Self::get_rx_fifo_count() > 0 {
            let _ = spi.rdr().read().bits();
        }

        // Pre-fill TX FIFO in task context before enabling IRQs.
        let mut prefill = 0usize;
        while prefill < total && Self::get_tx_fifo_count() < fifo_size {
            let byte = if prefill < tx_len { tx[prefill] } else { 0 };
            spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
            prefill += 1;
        }

        // Arm ISR state.
        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Transfer;
            st.error = None;

            st.rx_ptr = rx.as_mut_ptr();
            st.rx_len = total;
            st.rx_pos = 0;
            st.rx_store_len = rx_len;

            st.tx_ptr = tx.as_ptr();
            st.tx_len = total;
            st.tx_pos = prefill;
            st.tx_source_len = tx_len;
        });

        // Enable RX/TX data interrupts + errors.
        // RX completion is used as the end-of-transfer signal.
        spi.ier()
            .write(|w| w.rdie().enable().reie().enable().tdie().enable().teie().enable());

        // Sleep until ISR completes the transfer or reports an error.
        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        // Wait for transfer fully complete (FIFO empty AND module not busy).
        spin_wait_while(|| Self::get_tx_fifo_count() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> SpiSlave<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: SlaveConfig,
    ) -> Result<Self> {
        // Configure clocks - use FRO_HF_DIV as clock source
        let clock_config = LpspiConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroHfDiv,
            div: Div4::no_div(),
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        // Configure pins (including PCS for slave mode)
        sck.mux();
        mosi.mux();
        miso.mux();
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        let spi = T::regs();

        // Slave initialization sequence:
        // 1) Set slave mode
        // Note: Module should be disabled when changing CFGR1
        spi.cr().write(|w| w.men().disabled());
        spi.cfgr1().modify(|_, w| w.master().slave_mode());

        // 2) Set PCS polarity
        // Active low for PCS0
        spi.cfgr1().modify(|_, w| unsafe { w.pcspol().bits(0) });

        // 3) Configure CFGR1 for slave
        spi.cfgr1()
            .modify(|_, w| w.outcfg().retain_lastvalue().pincfg().sin_in_sout_out());

        // 4) Set FIFO watermarks
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        // 5) Configure TCR
        // bits_per_frame is already u16, so no cast needed
        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            }
        });

        // 6) Enable the module
        spi.cr().write(|w| w.men().enabled());

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            _phantom: PhantomData,
        })
    }

    /// RX-only receive from the master (blocking).
    ///
    /// The SPI slave cannot generate clocks; the master must provide them.
    /// This method blocks until `rx` is filled by frames clocked by the master.
    pub fn blocking_read(&self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();

        // Disable, flush FIFOs, clear status, re-enable
        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        // Configure TCR for receive
        spi.tcr().modify(|_, w| w.rxmsk().normal().txmsk().mask());

        // Wait for RX data from master
        for byte in rx.iter_mut() {
            while spi.fsr().read().rxcount().bits() == 0 {}
            *byte = spi.rdr().read().bits() as u8;
        }

        Ok(())
    }

    /// TX-only transmit to the master (blocking).
    ///
    /// Pre-fills the TX FIFO and then blocks while the master clocks the bytes out.
    pub fn blocking_write(&self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = LPSPI_FIFO_SIZE;

        // Disable, flush FIFOs, clear status, re-enable
        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        // Configure TCR for transmit
        spi.tcr().modify(|_, w| w.rxmsk().mask().txmsk().normal());

        // Pre-fill TX FIFO
        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && spi.fsr().read().txcount().bits() < fifo_size {
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        // Wait for master to clock out data, refill as needed
        while tx_idx < tx.len() {
            while spi.fsr().read().txcount().bits() >= fifo_size {}
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        // Wait for TX FIFO to empty
        while spi.fsr().read().txcount().bits() != 0 {}
        while spi.sr().read().mbf().is_busy() {}

        Ok(())
    }
}

// =============================================================================
// Pin Traits
// =============================================================================

/// SCK pin trait.
pub trait SckPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// MOSI/SDO pin trait.
pub trait MosiPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// MISO/SDI pin trait.
pub trait MisoPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// PCS/CS pin trait.
pub trait CsPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

macro_rules! impl_spi_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        impl $trait<crate::peripherals::$peri> for crate::peripherals::$pin {
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port0::pcr0::Mux::$fn);
                self.set_enable_input_buffer();
            }
        }
    };
}

// LPSPI0 pins on PORT1 (ALT2) - per reference board pin mux
impl_spi_pin!(P1_0, LPSPI0, Mux2, MosiPin); // LPSPI0_SDO (SOUT)
impl_spi_pin!(P1_1, LPSPI0, Mux2, SckPin); // LPSPI0_SCK
impl_spi_pin!(P1_2, LPSPI0, Mux2, MisoPin); // LPSPI0_SDI (SIN)
impl_spi_pin!(P1_3, LPSPI0, Mux2, CsPin); // LPSPI0_PCS0

// LPSPI1 pins on PORT3 (ALT2)
impl_spi_pin!(P3_8, LPSPI1, Mux2, MosiPin); // LPSPI1_SOUT
impl_spi_pin!(P3_9, LPSPI1, Mux2, MisoPin); // LPSPI1_SIN
impl_spi_pin!(P3_10, LPSPI1, Mux2, SckPin); // LPSPI1_SCK
impl_spi_pin!(P3_11, LPSPI1, Mux2, CsPin); // LPSPI1_PCS0

// =============================================================================
// SPI DMA SUPPORT
// =============================================================================
//
// This section implements LPSPI master DMA transfers using a scatter/gather TX
// chain to ensure PCS de-assertion happens reliably at the end of each transfer.
//
// 1. SCATTER/GATHER TX CHAIN
//    TX DMA uses a two-TCD chain:
//    - data_tcd: transfers N bytes, ESG=1, chains to tcr_tcd
//    - tcr_tcd: software TCD that writes TCR with CONT/CONTC cleared
//
// 2. TCD_CSR BIT VALUES (CRITICAL)
//    - DREQ: auto-clears ERQ when the major loop completes (needed for DONE)
//    - ESG: enables scatter/gather chaining
//    - INTMAJOR: interrupt on major loop complete
//
//    Main TX TCD: CSR = 0x0010 (ESG=1)
//    Software TX TCD: CSR = 0x0008 (DREQ=1)
//    RX TCD: CSR = 0x000A (DREQ=1, INTMAJOR=1)
//
// 3. WHY RX DMA COMPLETION IS USED
//    RX completes after TX due to the shift register latency. Using RX
//    completion ensures all bytes have fully shifted and been received.
//
// 4. BYTE SWAP ADDRESSING
//    With BYSW=1 and 8-bit frames, DMA must use TDR+3 and RDR+3 because the
//    8-bit frame is carried in bits [31:24] of the 32-bit FIFO entry.
//
// =============================================================================

use crate::dma::Tcd;

/// Static storage for TX DMA scatter/gather TCDs.
///
/// # Memory Layout (software TCD chain)
///
/// The eDMA engine requires 32-byte alignment for scatter/gather TCDs.
/// When the main data TCD completes, the DMA engine loads the next TCD
/// from the address in DLAST_SGA.
///
/// ## TCD Chain Flow:
/// ```text
/// [data_tcd] --ESG--> [tcr_tcd] --DREQ--> DONE
///     |                   |
///     v                   v
///   8-bit transfers     32-bit write
///   tx_buf -> TDR+3     tcr_value -> TCR
///   CITER = N bytes     CITER = 1
///   CSR = 0x0010        CSR = 0x0008
/// ```
///
/// ## Why This Matters
///
/// Without the software TCD to clear CONT, PCS would remain asserted
/// after the data transfer, preventing proper framing of subsequent
/// transfers.
#[repr(C, align(32))]
struct SpiDmaTcds {
    /// Main data TCD: transfers N bytes from tx_buf to TDR+3.
    ///
    /// Configuration:
    /// - SADDR: source buffer address
    /// - SOFF: 1 (increment source by 1 byte)
    /// - DADDR: LPSPI_TDR + 3 (for BYSW mode)
    /// - DOFF: 0 (fixed peripheral address)
    /// - ATTR: 0x0000 (8-bit src and dst)
    /// - NBYTES: 1 (one byte per minor loop)
    /// - CITER/BITER: transfer length
    /// - DLAST_SGA: address of tcr_tcd
    /// - CSR: 0x0010 (ESG=1, DREQ=0)
    data_tcd: Tcd,

    /// Software TCD for TCR update: clears CONT to de-assert PCS.
    ///
    /// Configuration:
    /// - SADDR: address of tcr_value (TCR with CONT=0)
    /// - SOFF: 0 (no increment)
    /// - DADDR: LPSPI_TCR register address
    /// - DOFF: 0 (fixed)
    /// - ATTR: 0x0202 (32-bit src and dst)
    /// - NBYTES: 4 (single 32-bit write)
    /// - CITER/BITER: 1
    /// - DLAST_SGA: 0 (no further chaining)
    /// - CSR: 0x0008 (DREQ=1, ESG=0 - final TCD)
    tcr_tcd: Tcd,

    /// TCR value with CONT=0 and CONTC=0.
    ///
    /// This is the value the software TCD writes to the TCR register.
    /// Clearing CONT causes PCS to de-assert after the current frame.
    /// BYSW and FRAMESZ are preserved from the original TCR.
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
/// This struct provides DMA-based SPI transfers for high-performance data movement.
/// It uses scatter/gather DMA to automatically de-assert PCS (chip select) at the
/// end of each transfer.
///
/// # Transfer Modes
///
/// - [`write_dma`](Self::write_dma): TX-only transfer (discards RX data)
/// - [`read_dma`](Self::read_dma): RX-only transfer (sends dummy bytes)
/// - [`transfer_dma`](Self::transfer_dma): Full-duplex transfer
///
/// # Example
///
/// ```no_run
/// // Full-duplex DMA transfer
/// let tx_data = [1, 2, 3, 4];
/// let mut rx_data = [0u8; 4];
/// spi_dma.transfer_dma(&tx_data, &mut rx_data).await?;
/// ```
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

    // Per-instance storage for scatter/gather TCDs.
    // This avoids global mutable state and allows multiple SPI DMA instances.
    tcds: SpiDmaTcds,
}

impl<'d, T: Instance, TxC: DmaChannelTrait, RxC: DmaChannelTrait> SpiDma<'d, T, TxC, RxC> {
    /// Create a new SPI Master with DMA support.
    ///
    /// # Arguments
    /// * `peri` - The SPI peripheral instance
    /// * `sck` - SPI clock pin
    /// * `mosi` - Master Out Slave In pin
    /// * `miso` - Master In Slave Out pin
    /// * `cs` - Chip select pin
    /// * `tx_dma_ch` - DMA channel for TX
    /// * `rx_dma_ch` - DMA channel for RX
    /// * `config` - SPI configuration
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
        // Configure clocks - use FRO_HF_DIV as clock source (same as Spi::new_inner)
        let clock_config = LpspiConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroHfDiv,
            div: Div4::no_div(),
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        // Configure pins
        sck.mux();
        mosi.mux();
        miso.mux();
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        // Initialize the SPI peripheral
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

    /// Get the SPI register block
    #[inline]
    fn regs() -> &'static pac::lpspi0::RegisterBlock {
        T::regs()
    }

    /// Get TDR (TX Data Register) address for DMA
    fn tdr_addr() -> *mut u8 {
        let spi = Self::regs();
        spi.tdr().as_ptr() as *mut u8
    }

    /// Get RDR (RX Data Register) address for DMA
    fn rdr_addr() -> *const u8 {
        let spi = Self::regs();
        spi.rdr().as_ptr() as *const u8
    }

    /// Disable TX DMA request in LPSPI (DER register)
    fn disable_tx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.tdde().disable());
    }

    /// Disable RX DMA request in LPSPI (DER register)
    fn disable_rx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.rdde().disable());
    }

    /// Enable both TX and RX DMA requests in LPSPI (DER register) in a single write
    /// This matches the typical reference-driver ordering: enable TX+RX DMA together.
    fn enable_tx_rx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.tdde().enable().rdde().enable());
    }

    /// Disable both TX and RX DMA requests in LPSPI (DER register) in a single write
    fn disable_tx_rx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.tdde().disable().rdde().disable());
    }

    /// Flush TX and RX FIFOs
    fn flush_fifos() {
        flush_fifos(Self::regs());
    }

    /// Clear all status flags
    fn clear_status() {
        clear_status_flags(Self::regs());
    }

    /// Get FIFO size (4 for MCXA276)
    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    /// Read TCR with errata workaround (ERR050606)
    #[inline]
    fn read_tcr_with_errata_workaround() -> u32 {
        read_tcr_with_errata_workaround(Self::regs())
    }

    /// Debug: Returns key register values after DMA is started
    /// Returns (tx_csr, tx_saddr, tx_daddr, tx_citer, tx_biter, tx_nbytes, rx_csr, rx_saddr, rx_daddr, rx_citer, rx_biter, rx_nbytes, tcr, der, tdr_addr, rdr_addr)
    #[allow(dead_code)]
    pub fn debug_dma_state(
        &self,
    ) -> (
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
    ) {
        let spi = Self::regs();
        let tcr = spi.tcr().read().bits();
        let der = spi.der().read().bits();
        let tdr_addr = Self::tdr_addr() as u32;
        let rdr_addr = Self::rdr_addr() as u32;

        // Get DMA channel registers
        let tx_csr = self.tx_dma.ch_csr();
        let tx_saddr = self.tx_dma.saddr();
        let tx_daddr = self.tx_dma.daddr();
        let tx_citer = self.tx_dma.citer();
        let tx_biter = self.tx_dma.biter();
        let tx_nbytes = self.tx_dma.nbytes();

        let rx_csr = self.rx_dma.ch_csr();
        let rx_saddr = self.rx_dma.saddr();
        let rx_daddr = self.rx_dma.daddr();
        let rx_citer = self.rx_dma.citer();
        let rx_biter = self.rx_dma.biter();
        let rx_nbytes = self.rx_dma.nbytes();

        (
            tx_csr, tx_saddr, tx_daddr, tx_citer, tx_biter, tx_nbytes, rx_csr, rx_saddr, rx_daddr, rx_citer, rx_biter,
            rx_nbytes, tcr, der, tdr_addr, rdr_addr,
        )
    }

    /// Debug: Returns extended DMA state including error status and DMAMUX source
    /// Returns (tx_ch_es, rx_ch_es, tx_mux_src, rx_mux_src)
    ///
    /// - tx_ch_es/rx_ch_es: CH_ES register (error status). Non-zero means an error occurred.
    /// - tx_mux_src/rx_mux_src: CH_MUX.SRC field (DMAMUX request source number).
    ///   For LPSPI1: TX should be 18, RX should be 17.
    #[allow(dead_code)]
    pub fn debug_dma_error_state(&self) -> (u32, u32, u8, u8) {
        let tx_ch_es = self.tx_dma.ch_es();
        let rx_ch_es = self.rx_dma.ch_es();
        let tx_mux_src = self.tx_dma.ch_mux_src();
        let rx_mux_src = self.rx_dma.ch_mux_src();

        (tx_ch_es, rx_ch_es, tx_mux_src, rx_mux_src)
    }

    /// Configure TX DMA with a software TCD to clear CONT via DMA.
    ///
    /// This sets up a scatter/gather chain matching common reference-driver behaviour:
    /// - Main TCD (data_tcd): 8-bit transfers from `tx_buf` to TDR+3
    /// - Software TCD (tcr_tcd): single 32-bit write of TCR with CONT/CONTC cleared
    ///
    /// # Safety
    ///
    /// Caller must ensure exclusive access to the TX DMA channel and SPI instance.
    ///
    unsafe fn setup_tx_scatter_gather(
        &mut self,
        tx_ptr: *const u8,
        src_increment: bool,
        transfer_len: usize,
        tcr_with_cont: u32,
    ) -> Result<()> {
        // SAFETY: This function is marked unsafe because it:
        // - Dereferences raw pointers to configure TCDs
        // - Calls unsafe DMA channel methods
        // All operations are valid because:
        // - self.tcds is owned by this struct and properly aligned
        // - The caller has exclusive access to the DMA channel
        unsafe {
            if transfer_len == 0 || transfer_len > 0x7fff {
                return Err(Error::TransferError);
            }

            let spi = Self::regs();

            // =========================================================================
            // STEP 1: Compute TCR value for PCS de-assertion
            // =========================================================================
            //
            // The software TCD will write this value to TCR after all data bytes
            // have been transmitted. Clearing CONT (bit 21) and CONTC (bit 20)
            // causes PCS to de-assert after the current frame completes.
            //
            // Note: later we clear CONT/CONTC for the final write to de-assert PCS.
            let tcr_without_cont = tcr_with_cont & !0x0030_0000;

            // =========================================================================
            // STEP 2: Compute peripheral addresses for BYSW mode
            // =========================================================================
            //
            // With BYSW=1 (byte swap) and 8-bit frames:
            // - TDR+3: DMA writes to byte 3 of the 32-bit TDR register
            // - TCR: Full 32-bit register address for the software TCD
            //
            // Why TDR+3? LPSPI with BYSW=1 expects the data byte in bits [31:24]
            // of the FIFO entry. An 8-bit DMA write to TDR+3 places the byte there.
            let tdr_addr = Self::tdr_addr() as u32 + 3;
            let tcr_addr = spi.tcr().as_ptr() as u32;

            // =========================================================================
            // STEP 3: Configure software TCD for TCR update
            // =========================================================================
            //
            // This TCD runs AFTER the main data TCD completes (via scatter/gather).
            // It performs a single 32-bit write of tcr_without_cont to the TCR register.
            //
            // TCD field breakdown:
            // - SADDR: Address of tcr_value in RAM
            // - SOFF: 0 (no source increment - single value)
            // - ATTR: 0x0202 (32-bit source, 32-bit destination)
            // - NBYTES: 4 (one 32-bit word per minor loop)
            // - DADDR: LPSPI_TCR register address
            // - DOFF: 0 (no destination increment - fixed register)
            // - CITER/BITER: 1 (single major loop iteration)
            // - DLAST_SGA: 0 (no further chaining)
            // - CSR: 0x0008 (DREQ=1 to set DONE flag, ESG=0 - final TCD)
            //
            // CRITICAL: CSR.DREQ=1 is required! Without it, CH_CSR.DONE never gets
            // set and the DMA channel appears to hang.
            let tcds = &raw mut self.tcds;
            (*tcds).tcr_value = tcr_without_cont;

            let tcr_tcd = &mut (*tcds).tcr_tcd;
            tcr_tcd.saddr = core::ptr::addr_of!((*tcds).tcr_value) as u32;
            tcr_tcd.soff = 0;
            tcr_tcd.attr = 0x0202; // SSIZE=2 (32-bit), DSIZE=2 (32-bit)
            tcr_tcd.nbytes = 4;
            tcr_tcd.slast = 0;
            tcr_tcd.daddr = tcr_addr;
            tcr_tcd.doff = 0;
            tcr_tcd.citer = 1;
            tcr_tcd.dlast_sga = 0;
            tcr_tcd.csr = 0x0008; // DREQ=1, ESG=0 (final TCD in chain)
            tcr_tcd.biter = 1;

            // =========================================================================
            // STEP 4: Configure main data TCD
            // =========================================================================
            //
            // This TCD transfers the actual data bytes from tx_buf to TDR+3.
            // After all bytes are transferred, scatter/gather loads tcr_tcd.
            //
            // TCD field breakdown:
            // - SADDR: tx_buf address
            // - SOFF: 1 (increment source by 1 byte after each minor loop)
            // - ATTR: 0x0000 (8-bit source, 8-bit destination)
            // - NBYTES: 1 (one byte per minor loop)
            // - DADDR: TDR+3 (for BYSW mode)
            // - DOFF: 0 (fixed peripheral address)
            // - CITER/BITER: transfer_len (number of bytes = major loop count)
            // - DLAST_SGA: Address of tcr_tcd (scatter/gather target)
            // - CSR: 0x0010 (ESG=1 to enable scatter/gather, DREQ=0)
            //
            // CRITICAL: CSR.ESG=1 enables scatter/gather. When the major loop
            // completes, the DMA engine loads the TCD from DLAST_SGA instead of
            // stopping. This is how we chain to the TCR update.
            let data_tcd = &mut (*tcds).data_tcd;
            data_tcd.saddr = tx_ptr as u32;
            data_tcd.soff = if src_increment { 1 } else { 0 };
            data_tcd.attr = 0x0000; // SSIZE=0 (8-bit), DSIZE=0 (8-bit)
            data_tcd.nbytes = 1;
            data_tcd.slast = 0;
            data_tcd.daddr = tdr_addr;
            data_tcd.doff = 0; // Fixed destination (TDR)
            data_tcd.citer = transfer_len as u16;
            data_tcd.dlast_sga = core::ptr::addr_of!((*tcds).tcr_tcd) as u32 as i32;
            data_tcd.csr = 0x0010; // ESG=1, DREQ=0 (scatter/gather continues)
            data_tcd.biter = transfer_len as u16;

            // =========================================================================
            // STEP 5: Reset DMA channel and load TCD
            // =========================================================================
            //
            // Order matters here:
            // 1. Disable ERQ to stop any pending requests
            // 2. Clear DONE flag from previous transfer
            // 3. Clear any pending interrupt
            // 4. Set DMAMUX request source for LPSPI TX
            // 5. Load the data TCD into hardware registers
            // 6. DSB to ensure all writes are visible to DMA engine
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();

            self.tx_dma.load_tcd(data_tcd);

            // Data Synchronization Barrier: ensures all TCD writes are visible
            // to the DMA engine before we enable requests.
            cortex_m::asm::dsb();

            Ok(())
        }
    }

    /// Write data using DMA (TX only, discards RX).
    ///
    /// This is the TX-only master DMA transfer:
    /// send a buffer of bytes, ignore what comes back on MISO, and use RX DMA only
    /// to drain the receive FIFO and signal completion.
    ///
    /// Behavioural contract:
    /// - All `data.len()` bytes are shifted out on SOUT with PCS asserted for the
    ///   entire transfer.
    /// - The slave sees a contiguous 8-bit stream.
    /// - RX data is discarded into a dummy buffer; RX DMA completion is used as
    ///   the "transfer finished" event.
    ///
    /// Implementation notes:
    /// - Mirrors `LPSPI_MasterTransferPrepareEDMALite` + `LPSPI_MasterTransferEDMALite`
    ///   for the **txData != NULL, rxData == NULL** case.
    /// - Configures:
    ///   - LPSPI1: FRAMESZ=7 (8-bit), BYSW=1, CONT=1, NOSTALL=0, TX/RX watermarks.
    ///   - RX DMA: 8-bit from `RDR+3` to a static dummy buffer, CITER=BITER=data_len,
    ///            CSR=0x000A (DREQ=1, INTMAJOR=1).
    ///   - TX DMA: 8-bit from `data` to `TDR+3` via a main TCD which chains to a
    ///            software TCD that writes TCR with CONT cleared at the end.
    /// - Enables TX ERQ, then RX ERQ, then DER (TDDE+RDDE) in the same order as
    ///   the SDK, then waits for RX DMA completion and finally drains SR/FSR.
    pub async fn write_dma(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        // Pointer to the LPSPI register block we are driving.
        let spi = Self::regs();
        let data_len = data.len();

        // === 1. Prepare for a fresh DMA transfer ================================
        // Disable module, flush FIFOs, clear status, and disable LPSPI DMA requests.
        spi.cr().modify(|_, w| w.men().disabled());
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // === 2. FIFO watermarks and NOSTALL ====================================
        //
        // Use fifoSize-1 for TX and 0 for RX. On MCXA276 fifoSize is 4, so
        // TX watermark=3, RX watermark=0.  This means:
        //   - TX DMA is requested when the TX FIFO has at least one free slot.
        //   - RX DMA is requested as soon as any data appears in the RX FIFO.
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // Clear NOSTALL:
        //   CFGR1.NOSTALL = 0 -> LPSPI stalls when TX FIFO empty or RX FIFO full.
        // This prevents silently dropping data under load.
        clear_nostall(spi);
        // Re-enable the module so that subsequent TCR writes take effect.
        spi.cr().modify(|_, w| w.men().enabled());

        // === 3. Program TCR for 8-bit, byte-swapped, PCS-continuous transfers ===
        //
        // Compute a "clean" TCR value by:
        //   - preserving the upper byte (PCS/CPOL/CPHA/LSBF)
        //   - forcing CONT=1, BYSW=1, FRAMESZ=7.
        //
        // We must apply the ERR050606 workaround when reading TCR because the
        // register can return stale values if accessed repeatedly without an
        // intervening read of SR.
        let tcr = Self::read_tcr_with_errata_workaround();
        let tcr_with_cont = (tcr & 0xFF000000) | 0x00600007;
        spi.tcr().write(|w| unsafe { w.bits(tcr_with_cont) });
        // TCR shares an internal path with the TX FIFO; poll TXCOUNT until the
        // write has actually been accepted before touching DMA.
        while spi.fsr().read().txcount().bits() > 0 {}

        // === 4. Compute LPSPI RX data address used by the RX TCD =================
        //
        // In BYSW mode with 1-byte frames, use RDR+3 so that the single
        // byte we move by DMA ends up in the MSB of the 32-bit receive register.
        // LPSPI then presents that MSB as the 8-bit frame.
        let rdr_addr = Self::rdr_addr() as u32 + 3; // RDR+3 for byte swap

        static mut DUMMY_RX_SINK: u8 = 0;

        unsafe {
            // Make sure the compiler/CPU have pushed all TCD fields to RAM
            // before we let the DMA engine see them.
            cortex_m::asm::dsb();

            // === 5. Configure RX DMA first =====================================
            //
            // Even for TX-only transfers we drain RDR into a dummy sink and use
            // RX completion as the "transfer finished" event.
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();

            // Configure RX TCD:
            // - 8-bit transfers
            // - SADDR = RDR+3 (see above)
            // - DADDR = static dummy buffer (no increment beyond rx_len bytes)
            // - NBYTES = 1 (one byte per minor loop)
            // - CITER/BTIER = rx_len (major loop count)
            // - CSR = 0x000A (DREQ=1, INTMAJOR=1, ESG=0)
            let rx_tcd = self.rx_dma.tcd();
            rx_tcd.tcd_saddr().write(|w| w.saddr().bits(rdr_addr));
            rx_tcd.tcd_soff().write(|w| w.soff().bits(0));
            // SSIZE=0 (8-bit), DSIZE=0 (8-bit), SMOD=0, DMOD=0
            rx_tcd.tcd_attr().write(|w| w.bits(0x0000));
            rx_tcd.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(1));
            rx_tcd.tcd_slast_sda().write(|w| w.slast_sda().bits(0));
            rx_tcd
                .tcd_daddr()
                .write(|w| w.daddr().bits((&raw mut DUMMY_RX_SINK) as *mut u8 as u32));
            rx_tcd.tcd_doff().write(|w| w.doff().bits(0));
            rx_tcd.tcd_citer_elinkno().write(|w| w.citer().bits(data_len as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.dlast_sga().bits(0));
            // CSR configuration: DREQ=1 (bit 3), INTMAJOR=1 (bit 1)
            // - DREQ=1: Clear ERQ and set DONE when major loop completes
            // - INTMAJOR=1: Generate interrupt on major loop completion (wakes our waker)
            rx_tcd.tcd_csr().write(|w| w.bits(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.biter().bits(data_len as u16));

            // Program DMAMUX for this channel to LPSPI1 RX (request source 17).
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // === 6. Configure TX DMA with scatter/gather (main TCD + SW TCD) =======
            //
            // The helper below writes two TCDs into static memory:
            //  - data_tcd:  8-bit, NBYTES=1, SADDR=data, DADDR=TDR+3, ESG=1,
            //               DLAST_SGA -> software TCD.
            //  - tcr_tcd:   32-bit, NBYTES=4, writes TRANSMIT_COMMAND to TCR
            //               (CONT cleared, BYSW/FRAMESZ preserved) once.
            self.setup_tx_scatter_gather(data.as_ptr(), true, data_len, tcr_with_cont)?;

            // === 7. Start DMA and enable LPSPI DMA requests ========================
            //
            // Order is important:
            //   1) Enable TX ERQ
            //   2) Enable RX ERQ
            //   3) Enable LPSPI DER (TDDE/RDDE)
            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable NVIC interrupt for the RX channel so that INTMAJOR will wake
            // the async task waiting in poll_fn below.
            self.rx_dma.enable_interrupt();

            Self::enable_tx_rx_dma();
        }

        // === 8. Wait for RX DMA completion =======================================
        //
        // The RX channel's INTMAJOR interrupt will invoke the DMA ISR, which in
        // turn wakes the waker registered here via rx_dma.waker().
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // === 9. Drain TX FIFO and wait for end of frame ===========================
        //
        // By the time RX DMA has completed all 64 bytes, the TX side has also
        // finished its major loop and run the software TCD that clears CONT.
        // We still wait for:
        //   - TXCOUNT == 0  (no data buffered in the TX FIFO)
        //   - MBF == 0      (no frame currently in progress on the wire)
        // so that when this function returns, PCS has already been de-asserted.
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        // Disable LPSPI-side DMA requests (DER.TDDE/RDDE) in a single write.
        Self::disable_tx_rx_dma();

        unsafe {
            // Finally, fully quiesce the DMA channels themselves: clear ERQ and
            // DONE and drop any pending interrupt flags.
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
    ///
    /// This is a common "read" phase for half-duplex protocols: after a prior
    /// "write" phase ([`write_dma`]), the master clocks `data.len()` dummy bytes
    /// on SOUT while capturing the peer's response into `data`.
    ///
    /// Note: PCS is de-asserted at the end of each call. If your protocol requires PCS held
    /// across a write+read exchange, prefer a single [`transfer_dma`] with padding.
    ///
    /// Behavioural contract:
    /// - Clocks exactly `data.len()` 8-bit frames while PCS remains asserted.
    /// - Uses BYSW + `RDR+3` so each received frame lands in `data[i]` with the
    ///   expected byte ordering for 8-bit frames.
    /// - Returns only after RX DMA has completed and the last frame has finished
    ///   on the wire (TX FIFO empty and MBF=0).
    ///
    /// Implementation notes:
    /// - Uses the same preparation sequence as the other master DMA helpers.
    /// - Configures LPSPI with FRAMESZ=7 (8-bit), BYSW=1, CONT=1,
    ///   CFGR1.NOSTALL=0, TX watermark=`fifoSize-1`, RX watermark=0, and both
    ///   RXMSK/TXMSK cleared.
    /// - Programs RX DMA as an 8-bit peripheral-to-memory transfer from `RDR+3`
    ///   into the caller's `data` buffer with `CSR=0x000A` (DREQ+INTMAJOR).
    /// - Uses TX DMA scatter/gather (via [`setup_tx_scatter_gather`]) to send a
    ///   fixed-address 0x00 dummy byte via `TDR+3` (source increment disabled) and then run a software TCD that
    ///   rewrites TCR with CONT cleared at the end of the transfer.
    /// - Programs RX first, then TX, then enables DER; completion is driven by
    ///   the RX DMA major-loop interrupt and we still wait for TX FIFO empty +
    ///   MBF=0 before return.
    pub async fn read_dma(&mut self, data: &mut [u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        // 1. Disable module before configuration
        spi.cr().modify(|_, w| w.men().disabled());

        // 2. Flush FIFOs, clear status, disable DMA requests
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // 3. Set FIFO watermarks: TX=fifoSize-1, RX=0
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // 4. Clear NOSTALL - transfers stall when TX FIFO empty or RX FIFO full
        clear_nostall(spi);

        // 5. Enable module for TCR configuration to take effect
        spi.cr().modify(|_, w| w.men().enabled());

        // 6. Configure TCR for DMA transfer
        // Use errata workaround (ERR050606) to read TCR reliably
        let tcr = Self::read_tcr_with_errata_workaround();
        // Clear CONT, CONTC, BYSW, PCS, RXMSK, TXMSK; set CONT=1, BYSW=1, PCS=0, RXMSK=0, TXMSK=0
        let new_tcr = (tcr & !(TCR_CONT | TCR_CONTC | TCR_BYSW | TCR_PCS_MASK | TCR_RXMSK | TCR_TXMSK))
            | TCR_CONT  // Keep CS asserted during transfer
            | TCR_BYSW; // Enable byte swap for correct byte ordering
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });

        // 7. Wait for TCR write to take effect - TCR shares the FIFO!
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

        // Fixed-address dummy byte for TX DMA (generates clocks without a size cap).
        static DUMMY_TX: u8 = 0;

        unsafe {
            // Configure RX DMA first, then TX DMA
            // Configure RX DMA channel
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // Setup RX DMA for peripheral-to-memory transfer
            // With byte swap (BYSW=1) and 1-byte frames, read from RDR+3.
            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, data, EnableInterrupt::Yes);

            // Configure TX DMA with scatter/gather to clear CONT via DMA.
            self.setup_tx_scatter_gather(core::ptr::addr_of!(DUMMY_TX), false, data.len(), new_tcr)?;

            // Start TX DMA first, then RX DMA, then enable LPSPI DMA requests.
            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable NVIC interrupts for DMA channels (required for async wakeups)
            self.rx_dma.enable_interrupt();

            // Enable TX and RX DMA requests in LPSPI (single write)
            Self::enable_tx_rx_dma();
        }

        // Wait for RX DMA completion (RX completes after TX)
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Wait for TX FIFO to empty before PCS deassertion (handled via DMA TCR write)
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        // Disable DMA requests in LPSPI (single write)
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
    ///
    /// Transmits `tx_data[i]` on SOUT while sampling MISO into `rx_data[i]` for
    /// each 8-bit frame, with PCS kept asserted for the entire burst.
    ///
    /// Behavioural contract:
    /// - `tx_data.len() == rx_data.len()` or this returns [`Error::TransferError`].
    /// - For non-empty buffers, clocks exactly `tx_data.len()` frames.
    /// - Returns only after RX DMA has completed and the last frame has finished
    ///   on the wire (TX FIFO empty and MBF=0).
    ///
    /// Implementation notes:
    /// - Uses the same configuration and sequencing as the other master DMA
    ///   helpers, but with both TX and RX active.
    /// - Reuses the same "prepare EDMA" sequence as [`write_dma`] and
    ///   [`read_dma`]: disable MEN, flush FIFOs, clear SR, disable DER, set
    ///   watermarks, clear CFGR1.NOSTALL, then re-enable MEN and rewrite TCR for
    ///   8-bit, byte-swapped, PCS-continuous transfers.
    /// - Programs RX DMA as an 8-bit peripheral-to-memory transfer from `RDR+3`
    ///   into `rx_data` with `CSR=0x000A` (DREQ+INTMAJOR).
    /// - Programs TX DMA (via [`setup_tx_scatter_gather`]) as an 8-bit
    ///   memory-to-peripheral transfer from `tx_data` to `TDR+3`, followed by a
    ///   software TCD that writes TCR with CONT cleared once the data major loop
    ///   completes.
    /// - Starts TX ERQ, then RX ERQ, then enables LPSPI DER (TDDE+RDDE);
    ///   completion is signalled by RX DMA major-loop interrupt and a final TX
    ///   FIFO empty + MBF=0 wait.
    ///
    /// For half-duplex protocols (response after a command phase), you can either:
    /// - Call [`write_dma`] then [`read_dma`] (PCS toggles between calls), or
    /// - Use this method with padding (TX = command bytes + dummy bytes) and ignore the
    ///   initial received bytes to keep PCS asserted for the whole exchange.
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

        // 1. Disable module before configuration
        spi.cr().modify(|_, w| w.men().disabled());

        // 2. Flush FIFOs, clear status, disable DMA requests
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // 3. Set FIFO watermarks: TX=fifoSize-1, RX=0
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // 4. Clear NOSTALL - transfers stall when TX FIFO empty or RX FIFO full
        clear_nostall(spi);

        // 5. Enable module for TCR configuration to take effect
        spi.cr().modify(|_, w| w.men().enabled());

        // 6. Configure TCR for DMA transfer
        // Use errata workaround (ERR050606) to read TCR reliably
        let tcr = Self::read_tcr_with_errata_workaround();
        // Clear CONT, CONTC, BYSW, PCS, RXMSK, TXMSK; set CONT=1, BYSW=1, PCS=0, RXMSK=0, TXMSK=0
        let new_tcr = (tcr & !(TCR_CONT | TCR_CONTC | TCR_BYSW | TCR_PCS_MASK | TCR_RXMSK | TCR_TXMSK))
            | TCR_CONT  // Keep CS asserted during transfer
            | TCR_BYSW; // Enable byte swap for correct byte ordering
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });
        let tcr_with_cont = new_tcr;

        // 7. Wait for TCR write to take effect - TCR shares the FIFO!
        while spi.fsr().read().txcount().bits() > 0 {}

        unsafe {
            // Configure RX DMA first, then TX DMA
            // Configure RX DMA channel
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();
            // With byte swap (BYSW=1) and 1-byte frames, read from RDR+3.
            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, rx_data, EnableInterrupt::Yes);

            // Configure TX DMA with scatter/gather to clear CONT via DMA.
            self.setup_tx_scatter_gather(tx_data.as_ptr(), true, tx_data.len(), tcr_with_cont)?;

            // Start TX DMA first, then RX DMA, then enable LPSPI DMA requests.
            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable NVIC interrupts for DMA channels (required for async wakeups)
            self.rx_dma.enable_interrupt();

            // Enable TX and RX DMA requests in LPSPI (single write)
            Self::enable_tx_rx_dma();
        }

        // Wait for RX DMA completion (RX completes after TX)
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Wait for TX FIFO to empty (PCS is deasserted via DMA TCR write)
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        // Disable DMA requests in LPSPI (single write)
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

/// SPI Slave with DMA support for TX and RX.
///
/// Provides DMA-based SPI slave transfers for high-performance data movement.
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
    ///
    /// # Arguments
    /// * `peri` - The SPI peripheral instance
    /// * `sck` - SPI clock pin
    /// * `mosi` - Master Out Slave In pin (slave receives on this)
    /// * `miso` - Master In Slave Out pin (slave transmits on this)
    /// * `cs` - Chip select pin
    /// * `tx_dma_ch` - DMA channel for TX
    /// * `rx_dma_ch` - DMA channel for RX
    /// * `config` - SPI slave configuration
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
        // Configure clocks - use FRO_HF_DIV as clock source
        let clock_config = LpspiConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroHfDiv,
            div: Div4::no_div(),
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        // Configure pins
        sck.mux();
        mosi.mux();
        miso.mux();
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        let spi = T::regs();

        // Slave initialization sequence:
        // 1) Set slave mode
        spi.cr().write(|w| w.men().disabled());
        spi.cfgr1().modify(|_, w| w.master().slave_mode());

        // 2) Set PCS polarity (active low)
        spi.cfgr1().modify(|_, w| unsafe { w.pcspol().bits(0) });

        // 3) Configure CFGR1 for slave
        spi.cfgr1()
            .modify(|_, w| w.outcfg().retain_lastvalue().pincfg().sin_in_sout_out());

        // 4) Set FIFO watermarks
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        // 5) Configure TCR
        // bits_per_frame is already u16, so no cast needed
        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            }
        });

        // 6) Enable the module
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

    /// Get the SPI register block
    #[inline]
    fn regs() -> &'static pac::lpspi0::RegisterBlock {
        T::regs()
    }

    /// Get TDR (TX Data Register) address for DMA
    fn tdr_addr() -> *mut u8 {
        let spi = Self::regs();
        spi.tdr().as_ptr() as *mut u8
    }

    /// Get RDR (RX Data Register) address for DMA
    fn rdr_addr() -> *const u8 {
        let spi = Self::regs();
        spi.rdr().as_ptr() as *const u8
    }

    /// Enable TX DMA request in LPSPI (DER register)
    fn enable_tx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.tdde().enable());
    }

    /// Disable TX DMA request in LPSPI (DER register)
    fn disable_tx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.tdde().disable());
    }

    /// Enable RX DMA request in LPSPI (DER register)
    fn enable_rx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.rdde().enable());
    }

    /// Disable RX DMA request in LPSPI (DER register)
    fn disable_rx_dma() {
        let spi = Self::regs();
        spi.der().modify(|_, w| w.rdde().disable());
    }

    /// Flush TX and RX FIFOs
    fn flush_fifos() {
        flush_fifos(Self::regs());
    }

    /// Clear all status flags
    fn clear_status() {
        clear_status_flags(Self::regs());
    }

    /// Get FIFO size (4 for MCXA276)
    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    /// Read data from master using DMA (RX only).
    ///
    /// Waits for master to send data and receives it via DMA.
    ///
    /// Notes:
    /// - RX is never masked (RX DMA completion signals transfer end)
    /// - TX is masked when not sending data
    /// - NOSTALL is cleared to allow stalling when FIFOs are empty/full
    /// - Byte swap is enabled with address offset for correct byte ordering
    pub async fn read_dma(&mut self, data: &mut [u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        // 1. Disable module before configuration
        spi.cr().modify(|_, w| w.men().disabled());

        // 2. Flush FIFOs, clear status, disable DMA requests
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // 3. Set FIFO watermarks: TX=fifoSize-1, RX=0
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // 4. Clear NOSTALL - transfers stall when TX FIFO empty or RX FIFO full
        clear_nostall(spi);

        // 5. Enable module for TCR configuration to take effect
        spi.cr().modify(|_, w| w.men().enabled());

        // 6. Configure TCR: enable byte swap, mask TX (RX-only), keep RX unmasked.
        // RX must never be masked in slave DMA mode because RX DMA completion is
        // used as the transfer-finished signal.
        spi.tcr().modify(|_, w| {
            w.txmsk()
                .mask() // Mask TX - slave not sending
                .rxmsk()
                .normal() // RX unmasked - always receive
                .bysw()
                .enabled() // Enable byte swap for correct byte ordering
        });

        // 7. Wait for TCR write to take effect - TCR shares the FIFO!
        // When TX is masked (RX-only), wait for TX FIFO to be empty.
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

        unsafe {
            // Configure RX DMA channel
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // Setup DMA for peripheral-to-memory transfer
            // With byte swap (BYSW=1) and 1-byte frames, read from RDR+3.
            let peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(peri_addr, data, EnableInterrupt::Yes);

            // Start DMA transfer first (enable ERQ), then enable LPSPI DMA (DER)
            // This ensures the DMA channel is ready before LPSPI starts generating requests
            dma_start_fence();
            self.rx_dma.enable_request();

            // Enable NVIC interrupt for DMA channel (required for async wakeups)
            self.rx_dma.enable_interrupt();

            // Enable RX DMA request in LPSPI (sets DER.RDDE)
            Self::enable_rx_dma();
        }

        // Wait for DMA completion
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Disable RX DMA request
        Self::disable_rx_dma();

        unsafe {
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
        }

        Ok(())
    }

    /// Write data to master using DMA (TX only).
    ///
    /// Pre-loads TX FIFO via DMA for when master clocks the data out.
    ///
    /// Notes:
    /// - RX is never masked in slave DMA mode (RX DMA completion is used as the
    ///   transfer-finished signal)
    /// - TX DMA sends data, RX DMA receives (discarded) for completion signaling
    /// - Byte swap is enabled with address offset for correct byte ordering
    pub async fn write_dma(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > 0x7fff {
            return Err(Error::TransferError);
        }

        let spi = Self::regs();

        // 1. Disable module before configuration
        spi.cr().modify(|_, w| w.men().disabled());

        // 2. Flush FIFOs, clear status, disable DMA
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // 3. Set FIFO watermarks: TX=fifoSize-1, RX=0
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // 4. Clear NOSTALL - transfers stall when TX FIFO empty or RX FIFO full
        clear_nostall(spi);

        // 5. Enable module for TCR configuration to take effect
        spi.cr().modify(|_, w| w.men().enabled());

        // 6. Configure TCR: enable byte swap, keep RX unmasked.
        // RX must never be masked in slave DMA mode because RX DMA completion is
        // used as the transfer-finished signal.
        spi.tcr().modify(|_, w| {
            w.txmsk()
                .normal() // TX unmasked - send data
                .rxmsk()
                .normal() // RX unmasked - REQUIRED for slave DMA (used for completion)
                .bysw()
                .enabled() // Enable byte swap for correct byte ordering
        });

        // 7. Wait for TCR write to take effect - TCR shares the FIFO!
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

        // Fixed-address sink byte to drain RX FIFO and signal completion.
        static mut DUMMY_RX_SINK: u8 = 0;

        unsafe {
            // Configure TX DMA channel
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();

            // Setup TX DMA for memory-to-peripheral transfer
            // With byte swap (BYSW=1) and 1-byte frames, write to TDR+3.
            let tx_peri_addr = (Self::tdr_addr() as usize + 3) as *mut u8;
            self.tx_dma
                .setup_write_to_peripheral(data, tx_peri_addr, EnableInterrupt::No);

            // Configure RX DMA channel (for completion signaling, data discarded)
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();

            // Setup RX DMA for peripheral-to-memory transfer (fixed-address sink).
            // With byte swap (BYSW=1) and 1-byte frames, read from RDR+3.
            let rdr_addr = (Self::rdr_addr() as usize + 3) as u32;
            let rx_tcd = self.rx_dma.tcd();
            rx_tcd.tcd_saddr().write(|w| w.saddr().bits(rdr_addr));
            rx_tcd.tcd_soff().write(|w| w.soff().bits(0));
            // SSIZE=0 (8-bit), DSIZE=0 (8-bit), SMOD=0, DMOD=0
            rx_tcd.tcd_attr().write(|w| w.bits(0x0000));
            rx_tcd.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(1));
            rx_tcd.tcd_slast_sda().write(|w| w.slast_sda().bits(0));
            rx_tcd
                .tcd_daddr()
                .write(|w| w.daddr().bits((&raw mut DUMMY_RX_SINK) as *mut u8 as u32));
            rx_tcd.tcd_doff().write(|w| w.doff().bits(0));
            rx_tcd.tcd_citer_elinkno().write(|w| w.citer().bits(data.len() as u16));
            rx_tcd.tcd_dlast_sga().write(|w| w.dlast_sga().bits(0));
            // CSR configuration: DREQ=1 (bit 3), INTMAJOR=1 (bit 1)
            rx_tcd.tcd_csr().write(|w| w.bits(0x000A));
            rx_tcd.tcd_biter_elinkno().write(|w| w.biter().bits(data.len() as u16));

            // Start DMA transfers first (enable ERQ), then enable LPSPI DMA (DER)
            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable NVIC interrupt for DMA channel (required for async wakeups)
            self.rx_dma.enable_interrupt();

            // Enable TX and RX DMA requests in LPSPI
            Self::enable_tx_dma();
            Self::enable_rx_dma();
        }

        // Wait for RX DMA completion (signals master has clocked all bytes)
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Disable DMA requests
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
    ///
    /// Transmits `tx_data` while receiving into `rx_data`.
    /// Both buffers must have the same length.
    ///
    /// Notes:
    /// - Both TX and RX are unmasked
    /// - Byte swap is enabled with address offsets for correct byte ordering
    /// - RX DMA completion signals the end of transfer
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

        // 1. Disable module before configuration
        spi.cr().modify(|_, w| w.men().disabled());

        // 2. Flush FIFOs, clear status, disable DMA
        Self::flush_fifos();
        Self::clear_status();
        Self::disable_tx_dma();
        Self::disable_rx_dma();

        // 3. Set FIFO watermarks: TX=fifoSize-1, RX=0
        let fifo_size = Self::get_fifo_size();
        let tx_watermark = if fifo_size >= 1 { fifo_size - 1 } else { 0 };
        spi.fcr()
            .write(|w| unsafe { w.txwater().bits(tx_watermark as u8).rxwater().bits(0) });

        // 4. Clear NOSTALL
        clear_nostall(spi);

        // 5. Enable module for TCR configuration to take effect
        spi.cr().modify(|_, w| w.men().enabled());

        // 6. Configure TCR for full duplex with byte swap
        spi.tcr()
            .modify(|_, w| w.rxmsk().normal().txmsk().normal().bysw().enabled());

        // 7. Wait for TCR write to take effect - TCR shares the FIFO!
        spin_wait_while(|| spi.fsr().read().txcount().bits() > 0)?;

        unsafe {
            // Configure TX DMA channel
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source::<T::TxDmaRequest>();
            // With byte swap, write to TDR+3 for 1-byte transfers
            let tx_peri_addr = (Self::tdr_addr() as usize + 3) as *mut u8;
            self.tx_dma
                .setup_write_to_peripheral(tx_data, tx_peri_addr, EnableInterrupt::No);

            // Configure RX DMA channel
            self.rx_dma.disable_request();
            self.rx_dma.clear_done();
            self.rx_dma.clear_interrupt();
            self.rx_dma.set_request_source::<T::RxDmaRequest>();
            // With byte swap, read from RDR+3 for 1-byte transfers
            let rx_peri_addr = (Self::rdr_addr() as usize + 3) as *const u8;
            self.rx_dma
                .setup_read_from_peripheral(rx_peri_addr, rx_data, EnableInterrupt::Yes);

            // Start DMA transfers first (enable ERQ), then enable LPSPI DMA (DER)
            dma_start_fence();
            self.tx_dma.enable_request();
            self.rx_dma.enable_request();

            // Enable NVIC interrupt for DMA channel (required for async wakeups)
            self.rx_dma.enable_interrupt();

            // Enable DMA requests in LPSPI (sets DER.TDDE and DER.RDDE)
            Self::enable_tx_dma();
            Self::enable_rx_dma();
        }

        // Wait for RX DMA completion
        poll_fn(|cx| {
            self.rx_dma.waker().register(cx.waker());
            if self.rx_dma.is_done() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // Disable DMA requests
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
