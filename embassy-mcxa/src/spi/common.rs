//! Common types, traits, and helper functions for SPI drivers.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::periph_helpers::LpspiConfig;
use crate::clocks::{ClockError, Gate};
use crate::dma::DmaRequest;
use crate::{interrupt, pac};

// =============================================================================
// REGISTER BIT CONSTANTS
// =============================================================================
// NOTE: These constants are used for DMA scatter/gather operations where we need
// to build TCR values programmatically for hardware TCD writes. The PAC provides
// typed accessors for normal register operations (see apply_transfer_tcr), but
// DMA TCDs require raw u32 values. These match the bit positions in LPSPI_TCR.

/// All clearable status flags (TEF, REF, DMF, FCF, WCF, TCF)
pub(super) const LPSPI_ALL_STATUS_FLAGS: u32 = 0x3F00;

/// TCR register bit positions (used for DMA TCD raw value construction)
pub(super) const TCR_CONT: u32 = 1 << 21; // Continuous transfer
pub(super) const TCR_CONTC: u32 = 1 << 20; // Continuing command
pub(super) const TCR_RXMSK: u32 = 1 << 19; // Receive data mask
pub(super) const TCR_TXMSK: u32 = 1 << 18; // Transmit data mask
pub(super) const TCR_BYSW: u32 = 1 << 22; // Byte swap
pub(super) const TCR_PCS_MASK: u32 = 0x3 << 24; // Peripheral chip select mask

/// FIFO size for MCXA276 LPSPI (4 words)
pub(super) const LPSPI_FIFO_SIZE: u8 = 4;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Flush TX and RX FIFOs for a given LPSPI register block
#[inline]
pub(super) fn flush_fifos(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.rtf().txfifo_rst().rrf().rxfifo_rst());
}

/// Clear all status flags for a given LPSPI register block
#[inline]
pub(super) fn clear_status_flags(spi: &pac::lpspi0::RegisterBlock) {
    spi.sr().write(|w| unsafe { w.bits(LPSPI_ALL_STATUS_FLAGS) });
}

/// Clear NOSTALL bit in CFGR1 (disables "no stall" mode)
#[inline]
pub(super) fn clear_nostall(spi: &pac::lpspi0::RegisterBlock) {
    spi.cfgr1().modify(|_, w| w.nostall().disable());
}

/// Read TCR with errata workaround (ERR050606)
#[inline]
pub(super) fn read_tcr_with_errata_workaround(spi: &pac::lpspi0::RegisterBlock) -> u32 {
    let mut last = spi.tcr().read().bits();
    loop {
        let _ = spi.sr().read();
        let now = spi.tcr().read().bits();
        if now == last {
            break now;
        }
        last = now;
    }
}

/// Common setup sequence for async SPI transfers
#[inline]
pub(super) fn prepare_for_transfer(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.men().disabled());
    flush_fifos(spi);
    clear_status_flags(spi);
    spi.ier().write(|w| w);
    clear_nostall(spi);
    spi.cr().modify(|_, w| w.men().enabled());
}

/// Common setup sequence for blocking SPI transfers
#[inline]
pub(super) fn prepare_for_blocking_transfer(spi: &pac::lpspi0::RegisterBlock) {
    spi.cr().modify(|_, w| w.men().disabled());
    flush_fifos(spi);
    clear_status_flags(spi);
    clear_nostall(spi);
    spi.cr().modify(|_, w| w.men().enabled());
}

// =============================================================================
// PUBLIC TYPES
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

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        embedded_hal_1::spi::ErrorKind::Other
    }
}

/// Maximum number of iterations for busy-wait loops.
pub(super) const SPIN_LIMIT: u32 = 10_000_000;

#[inline]
pub(super) fn spin_wait_while(mut cond: impl FnMut() -> bool) -> Result<()> {
    for _ in 0..SPIN_LIMIT {
        if !cond() {
            return Ok(());
        }
        spin_loop();
    }
    Err(Error::Timeout)
}

#[inline]
pub(super) fn dma_start_fence() {
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
pub(super) enum SlaveIrqOp {
    Idle = 0,
    Rx = 1,
    Tx = 2,
    Transfer = 3,
}

pub(super) struct SlaveIrqStateInner {
    pub op: SlaveIrqOp,
    pub rx_ptr: *mut u8,
    pub rx_len: usize,
    pub rx_pos: usize,
    pub rx_store_len: usize,
    pub tx_ptr: *const u8,
    pub tx_len: usize,
    pub tx_pos: usize,
    pub tx_source_len: usize,
    pub error: Option<Error>,
}

impl SlaveIrqStateInner {
    pub const fn new() -> Self {
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

pub(super) struct SlaveIrqState {
    inner: UnsafeCell<SlaveIrqStateInner>,
}

unsafe impl Sync for SlaveIrqState {}

impl SlaveIrqState {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(SlaveIrqStateInner::new()),
        }
    }

    #[inline]
    pub fn with<R>(&'static self, f: impl FnOnce(&mut SlaveIrqStateInner) -> R) -> R {
        critical_section::with(|_| unsafe { f(&mut *self.inner.get()) })
    }
}

// =============================================================================
// INTERRUPT HANDLER
// =============================================================================

#[inline]
pub(super) unsafe fn handle_slave_rx_irq<T: Instance>(
    regs: &pac::lpspi0::RegisterBlock,
    st: &mut SlaveIrqStateInner,
) {
    let sr = regs.sr().read();

    if sr.ref_().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::RxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

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
pub(super) unsafe fn handle_slave_tx_irq<T: Instance>(
    regs: &pac::lpspi0::RegisterBlock,
    st: &mut SlaveIrqStateInner,
) {
    let sr = regs.sr().read();

    if sr.tef().bit_is_set() {
        clear_status_flags(regs);
        st.error = Some(Error::TxFifoError);
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
        return;
    }

    while st.tx_pos < st.tx_len && regs.fsr().read().txcount().bits() < LPSPI_FIFO_SIZE {
        let byte = unsafe { *st.tx_ptr.add(st.tx_pos) };
        regs.tdr().write(|w| unsafe { w.bits(byte as u32) });
        st.tx_pos += 1;
    }

    if st.tx_pos >= st.tx_len {
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
pub(super) unsafe fn handle_slave_transfer_irq<T: Instance>(
    regs: &pac::lpspi0::RegisterBlock,
    st: &mut SlaveIrqStateInner,
) {
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

    while st.rx_pos < st.rx_len && regs.fsr().read().rxcount().bits() > 0 {
        let byte = regs.rdr().read().bits() as u8;
        if st.rx_pos < st.rx_store_len {
            unsafe { *st.rx_ptr.add(st.rx_pos) = byte };
        }
        st.rx_pos += 1;
    }

    while st.tx_pos < st.tx_len && regs.fsr().read().txcount().bits() < LPSPI_FIFO_SIZE {
        let byte = if st.tx_pos < st.tx_source_len {
            unsafe { *st.tx_ptr.add(st.tx_pos) }
        } else {
            0
        };
        regs.tdr().write(|w| unsafe { w.bits(byte as u32) });
        st.tx_pos += 1;
    }

    if st.rx_pos >= st.rx_len {
        st.op = SlaveIrqOp::Idle;
        regs.ier().write(|w| w);
        T::wait_cell().wake();
    }
}

/// Interrupt handler for SPI async operations.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        if regs.ier().read().bits() == 0 {
            return;
        }

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

        if regs.ier().read().bits() != 0 {
            regs.ier().write(|w| w);
            T::wait_cell().wake();
        }
    }
}

// =============================================================================
// SEALED TRAIT AND INSTANCE TRAIT
// =============================================================================

pub(super) mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

// Seal only the specific pins that can be used with SPI, not all GPIO pins.
// This ensures type safety - only pins that are actually muxed into SPI functions
// can be used with the SPI driver.
impl sealed::Sealed for crate::peripherals::P1_0 {} // LPSPI0_SDO
impl sealed::Sealed for crate::peripherals::P1_1 {} // LPSPI0_SCK
impl sealed::Sealed for crate::peripherals::P1_2 {} // LPSPI0_SDI
impl sealed::Sealed for crate::peripherals::P1_3 {} // LPSPI0_PCS0
impl sealed::Sealed for crate::peripherals::P3_8 {} // LPSPI1_SOUT
impl sealed::Sealed for crate::peripherals::P3_9 {} // LPSPI1_SIN
impl sealed::Sealed for crate::peripherals::P3_10 {} // LPSPI1_SCK
impl sealed::Sealed for crate::peripherals::P3_11 {} // LPSPI1_PCS0

pub(super) trait SealedInstance {
    fn regs() -> &'static pac::lpspi0::RegisterBlock;
    fn wait_cell() -> &'static WaitCell;
    fn slave_irq_state() -> &'static SlaveIrqState;
}

/// SPI Instance
#[allow(private_bounds)]
pub trait Instance:
    SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpspiConfig>
{
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

// =============================================================================
// CONFIGURATION TYPES
// =============================================================================

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
    /// SPI mode (combines polarity and phase).
    /// Use MODE_0, MODE_1, MODE_2, or MODE_3 from embedded_hal.
    pub mode: embedded_hal_02::spi::Mode,
    /// Bit order
    pub bit_order: BitOrder,
    /// Bits per frame (1-4096, typically 8).
    pub bits_per_frame: u16,
    /// Chip select to use
    pub chip_select: ChipSelect,
    /// SCK divider (0-255). Baud = src_clk / (prescaler.divisor() * (SCKDIV + 2))
    pub sck_div: u8,
    /// Clock prescaler (Div1 through Div128)
    pub prescaler: Prescaler,
}

impl Config {
    /// Create a new SPI configuration with default settings (MODE_0).
    pub fn new() -> Self {
        Self {
            mode: embedded_hal_02::spi::MODE_0,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
            chip_select: ChipSelect::Pcs0,
            sck_div: 0,
            prescaler: Prescaler::Div1,
        }
    }

    /// Set SPI mode (MODE_0, MODE_1, MODE_2, or MODE_3).
    pub fn mode(&mut self, mode: embedded_hal_02::spi::Mode) -> &mut Self {
        self.mode = mode;
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

    /// Set SCK divider. Baud = src_clk / (prescaler.divisor() * (SCKDIV + 2))
    pub fn sck_div(&mut self, div: u8) -> &mut Self {
        self.sck_div = div;
        self
    }

    /// Set clock prescaler (Div1 through Div128)
    pub fn prescaler(&mut self, prescaler: Prescaler) -> &mut Self {
        self.prescaler = prescaler;
        self
    }

    /// Calculate baud rate parameters for a target frequency.
    pub fn for_frequency(&mut self, src_hz: u32, target_hz: u32) -> &mut Self {
        let (prescaler, sck_div) = compute_baud_params(src_hz, target_hz);
        self.prescaler = prescaler;
        self.sck_div = sck_div;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// SPI clock prescaler values.
/// Maps to the PRESCALE field in the CCR register.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Prescaler {
    /// Divide by 1
    #[default]
    Div1 = 0,
    /// Divide by 2
    Div2 = 1,
    /// Divide by 4
    Div4 = 2,
    /// Divide by 8
    Div8 = 3,
    /// Divide by 16
    Div16 = 4,
    /// Divide by 32
    Div32 = 5,
    /// Divide by 64
    Div64 = 6,
    /// Divide by 128
    Div128 = 7,
}

impl Prescaler {
    /// Get the divisor value for this prescaler setting.
    pub const fn divisor(self) -> u32 {
        1 << (self as u8)
    }

    /// Convert from register value to Prescaler.
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x7 {
            0 => Self::Div1,
            1 => Self::Div2,
            2 => Self::Div4,
            3 => Self::Div8,
            4 => Self::Div16,
            5 => Self::Div32,
            6 => Self::Div64,
            _ => Self::Div128,
        }
    }
}

/// Compute prescaler and SCKDIV for the desired baud rate.
/// Returns (prescaler, sckdiv) where:
/// - prescaler is a Prescaler enum value
/// - sckdiv is 0-255
/// Baud = src_hz / (prescaler.divisor() * (SCKDIV + 2))
pub(super) fn compute_baud_params(src_hz: u32, baud_hz: u32) -> (Prescaler, u8) {
    if baud_hz == 0 {
        return (Prescaler::Div1, 0);
    }

    let prescalers = [
        Prescaler::Div1,
        Prescaler::Div2,
        Prescaler::Div4,
        Prescaler::Div8,
        Prescaler::Div16,
        Prescaler::Div32,
        Prescaler::Div64,
        Prescaler::Div128,
    ];

    let (best_prescaler, best_sckdiv, _) = prescalers.iter().fold(
        (Prescaler::Div1, 0u8, u32::MAX),
        |(best_pre, best_div, best_err), &prescaler| {
            let divisor = prescaler.divisor();
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

    (best_prescaler, best_sckdiv)
}

/// SPI slave configuration
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct SlaveConfig {
    /// SPI mode (must match master). Use MODE_0, MODE_1, MODE_2, or MODE_3.
    pub mode: embedded_hal_02::spi::Mode,
    /// Bit order (must match master)
    pub bit_order: BitOrder,
    /// Bits per frame (8 for typical use)
    pub bits_per_frame: u16,
}

impl SlaveConfig {
    /// Create a new slave configuration with defaults (MODE_0)
    pub fn new() -> Self {
        Self {
            mode: embedded_hal_02::spi::MODE_0,
            bit_order: BitOrder::MsbFirst,
            bits_per_frame: 8,
        }
    }

    /// Set SPI mode (MODE_0, MODE_1, MODE_2, or MODE_3)
    pub fn mode(&mut self, mode: embedded_hal_02::spi::Mode) -> &mut Self {
        self.mode = mode;
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
    fn default() -> Self {
        Self::new()
    }
}
