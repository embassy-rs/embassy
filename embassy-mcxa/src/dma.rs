//! DMA driver for MCXA276.
//!
//! This module provides a typed channel abstraction over the EDMA_0_TCD0 array
//! and helpers for configuring the channel MUX. The driver supports both
//! low-level TCD configuration and higher-level async transfer APIs.
//!
//! # Architecture
//!
//! The MCXA276 has 8 DMA channels (0-7), each with its own interrupt vector.
//! Each channel has a Transfer Control Descriptor (TCD) that defines the
//! transfer parameters.
//!
//! # Choosing the Right API
//!
//! This module provides several API levels to match different use cases:
//!
//! ## High-Level Async API (Recommended for Most Users)
//!
//! Use the async methods when you want simple, safe DMA transfers:
//!
//! | Method | Description |
//! |--------|-------------|
//! | [`DmaChannel::mem_to_mem()`] | Memory-to-memory copy |
//! | [`DmaChannel::memset()`] | Fill memory with a pattern |
//! | [`DmaChannel::write()`] | Memory-to-peripheral (TX) |
//! | [`DmaChannel::read()`] | Peripheral-to-memory (RX) |
//!
//! These return a [`Transfer`] future that can be `.await`ed:
//!
//! ```no_run
//! # use embassy_mcxa::dma::{DmaChannel, TransferOptions};
//! # let dma_ch = DmaChannel::new(p.DMA_CH0);
//! # let src = [0u32; 4];
//! # let mut dst = [0u32; 4];
//! // Simple memory-to-memory transfer
//! unsafe {
//!     dma_ch.mem_to_mem(&src, &mut dst, TransferOptions::default()).await;
//! }
//! ```
//!
//! ## Setup Methods (For Peripheral Drivers)
//!
//! Use setup methods when you need manual lifecycle control:
//!
//! | Method | Description |
//! |--------|-------------|
//! | [`DmaChannel::setup_write()`] | Configure TX without starting |
//! | [`DmaChannel::setup_read()`] | Configure RX without starting |
//!
//! These configure the TCD but don't start the transfer. You control:
//! 1. When to call [`DmaChannel::enable_request()`]
//! 2. How to detect completion (polling or interrupts)
//! 3. When to clean up with [`DmaChannel::clear_done()`]
//!
//! ## Circular/Ring Buffer API (For Continuous Reception)
//!
//! Use [`DmaChannel::setup_circular_read()`] for continuous data reception:
//!
//! ```no_run
//! # use embassy_mcxa::dma::DmaChannel;
//! # let dma_ch = DmaChannel::new(p.DMA_CH0);
//! # let uart_rx_addr = 0x4000_0000 as *const u8;
//! static mut RX_BUF: [u8; 64] = [0; 64];
//!
//! let ring_buf = unsafe {
//!     dma_ch.setup_circular_read(uart_rx_addr, &mut RX_BUF)
//! };
//!
//! // Read data as it arrives
//! let mut buf = [0u8; 16];
//! let n = ring_buf.read(&mut buf).await.unwrap();
//! ```
//!
//! ## Scatter-Gather Builder (For Chained Transfers)
//!
//! Use [`ScatterGatherBuilder`] for complex multi-segment transfers:
//!
//! ```no_run
//! # use embassy_mcxa::dma::{DmaChannel, ScatterGatherBuilder};
//! # let dma_ch = DmaChannel::new(p.DMA_CH0);
//! let mut builder = ScatterGatherBuilder::<u32>::new();
//! builder.add_transfer(&src1, &mut dst1);
//! builder.add_transfer(&src2, &mut dst2);
//!
//! let transfer = unsafe { builder.build(&dma_ch).unwrap() };
//! transfer.await;
//! ```
//!
//! ## Direct TCD Access (For Advanced Use Cases)
//!
//! For full control, use the channel's `tcd()` method to access TCD registers directly.
//! See the `dma_*` examples for patterns.
//!
//! # Example
//!
//! ```no_run
//! use embassy_mcxa::dma::{DmaChannel, TransferOptions, Direction};
//!
//! let dma_ch = DmaChannel::new(p.DMA_CH0);
//! // Configure and trigger a transfer...
//! ```

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering, fence};
use core::task::{Context, Poll};

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::Gate;
use crate::pac;
use crate::pac::Interrupt;
use crate::peripherals::DMA0;

/// Initialize DMA controller (clock enabled, reset released, controller configured).
///
/// This function is intended to be called ONCE during HAL initialization (`hal::init()`).
///
/// The function enables the DMA0 clock, releases reset, and configures the controller
/// for normal operation with round-robin arbitration.
pub(crate) fn init() {
    unsafe {
        // Enable DMA0 clock and release reset
        DMA0::enable_clock();
        DMA0::release_reset();

        // Configure DMA controller
        let dma = &(*pac::Dma0::ptr());
        dma.mp_csr().modify(|_, w| {
            w.edbg()
                .enable()
                .erca()
                .enable()
                .halt()
                .normal_operation()
                .gclc()
                .available()
                .gmrc()
                .available()
        });
    }
}

// ============================================================================
// Phase 1: Foundation Types (Embassy-aligned)
// ============================================================================

/// DMA transfer direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Transfer from memory to memory.
    MemoryToMemory,
    /// Transfer from memory to a peripheral register.
    MemoryToPeripheral,
    /// Transfer from a peripheral register to memory.
    PeripheralToMemory,
}

/// DMA transfer priority.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Low priority (channel priority 7).
    Low,
    /// Medium priority (channel priority 4).
    Medium,
    /// High priority (channel priority 1).
    #[default]
    High,
    /// Highest priority (channel priority 0).
    Highest,
}

impl Priority {
    /// Convert to hardware priority value (0 = highest, 7 = lowest).
    pub fn to_hw_priority(self) -> u8 {
        match self {
            Priority::Low => 7,
            Priority::Medium => 4,
            Priority::High => 1,
            Priority::Highest => 0,
        }
    }
}

/// DMA transfer data width.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WordSize {
    /// 8-bit (1 byte) transfers.
    OneByte,
    /// 16-bit (2 byte) transfers.
    TwoBytes,
    /// 32-bit (4 byte) transfers.
    #[default]
    FourBytes,
}

impl WordSize {
    /// Size in bytes.
    pub const fn bytes(self) -> usize {
        match self {
            WordSize::OneByte => 1,
            WordSize::TwoBytes => 2,
            WordSize::FourBytes => 4,
        }
    }

    /// Convert to hardware SSIZE/DSIZE field value.
    pub const fn to_hw_size(self) -> u8 {
        match self {
            WordSize::OneByte => 0,
            WordSize::TwoBytes => 1,
            WordSize::FourBytes => 2,
        }
    }

    /// Create from byte width (1, 2, or 4).
    pub const fn from_bytes(bytes: u8) -> Option<Self> {
        match bytes {
            1 => Some(WordSize::OneByte),
            2 => Some(WordSize::TwoBytes),
            4 => Some(WordSize::FourBytes),
            _ => None,
        }
    }
}

/// Trait for types that can be transferred via DMA.
///
/// This provides compile-time type safety for DMA transfers.
pub trait Word: Copy + 'static {
    /// The word size for this type.
    fn size() -> WordSize;
}

impl Word for u8 {
    fn size() -> WordSize {
        WordSize::OneByte
    }
}

impl Word for u16 {
    fn size() -> WordSize {
        WordSize::TwoBytes
    }
}

impl Word for u32 {
    fn size() -> WordSize {
        WordSize::FourBytes
    }
}

/// DMA transfer options.
///
/// This struct configures various aspects of a DMA transfer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Transfer priority.
    pub priority: Priority,
    /// Enable circular (continuous) mode.
    ///
    /// When enabled, the transfer repeats automatically after completing.
    pub circular: bool,
    /// Enable interrupt on half transfer complete.
    pub half_transfer_interrupt: bool,
    /// Enable interrupt on transfer complete.
    pub complete_transfer_interrupt: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            priority: Priority::High,
            circular: false,
            half_transfer_interrupt: false,
            complete_transfer_interrupt: true,
        }
    }
}

/// DMA error types.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The DMA controller reported a bus error.
    BusError,
    /// The transfer was aborted.
    Aborted,
    /// Configuration error (e.g., invalid parameters).
    Configuration,
    /// Buffer overrun (for ring buffers).
    Overrun,
}

/// Whether to enable the major loop completion interrupt.
///
/// This enum provides better readability than a boolean parameter
/// for functions that configure DMA interrupt behavior.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnableInterrupt {
    /// Enable the interrupt on major loop completion.
    Yes,
    /// Do not enable the interrupt.
    No,
}

// ============================================================================
// DMA Constants
// ============================================================================

/// Maximum bytes per DMA transfer (eDMA4 CITER/BITER are 15-bit fields).
///
/// This is a hardware limitation of the eDMA4 controller. Transfers larger
/// than this must be split into multiple DMA operations.
pub const DMA_MAX_TRANSFER_SIZE: usize = 0x7FFF;

// ============================================================================
// DMA Request Source Types (Type-Safe API)
// ============================================================================

/// Trait for type-safe DMA request sources.
///
/// Each peripheral that can trigger DMA requests implements this trait
/// with marker types that encode the correct request source number at
/// compile time. This prevents using the wrong request source for a
/// peripheral.
///
/// # Example
///
/// ```ignore
/// // The LPUART2 RX request source is automatically derived from the type:
/// channel.set_request_source::<Lpuart2RxRequest>();
/// ```
///
/// This trait is sealed and cannot be implemented outside this crate.
#[allow(private_bounds)]
pub trait DmaRequest: sealed::SealedDmaRequest {
    /// The hardware request source number for the DMA mux.
    const REQUEST_NUMBER: u8;
}

/// Macro to define a DMA request type.
///
/// Creates a zero-sized marker type that implements `DmaRequest` with
/// the specified request number.
macro_rules! define_dma_request {
    ($(#[$meta:meta])* $name:ident = $num:expr) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone)]
        pub struct $name;

        impl sealed::SealedDmaRequest for $name {}

        impl DmaRequest for $name {
            const REQUEST_NUMBER: u8 = $num;
        }
    };
}

// LPUART DMA request sources (from MCXA276 reference manual Table 4-8)
define_dma_request!(
    /// DMA request source for LPUART0 RX.
    Lpuart0RxRequest = 21
);
define_dma_request!(
    /// DMA request source for LPUART0 TX.
    Lpuart0TxRequest = 22
);
define_dma_request!(
    /// DMA request source for LPUART1 RX.
    Lpuart1RxRequest = 23
);
define_dma_request!(
    /// DMA request source for LPUART1 TX.
    Lpuart1TxRequest = 24
);
define_dma_request!(
    /// DMA request source for LPUART2 RX.
    Lpuart2RxRequest = 25
);
define_dma_request!(
    /// DMA request source for LPUART2 TX.
    Lpuart2TxRequest = 26
);
define_dma_request!(
    /// DMA request source for LPUART3 RX.
    Lpuart3RxRequest = 27
);
define_dma_request!(
    /// DMA request source for LPUART3 TX.
    Lpuart3TxRequest = 28
);
define_dma_request!(
    /// DMA request source for LPUART4 RX.
    Lpuart4RxRequest = 29
);
define_dma_request!(
    /// DMA request source for LPUART4 TX.
    Lpuart4TxRequest = 30
);
define_dma_request!(
    /// DMA request source for LPUART5 RX.
    Lpuart5RxRequest = 31
);
define_dma_request!(
    /// DMA request source for LPUART5 TX.
    Lpuart5TxRequest = 32
);

// ============================================================================
// Channel Trait (Sealed Pattern)
// ============================================================================

mod sealed {
    use crate::pac::Interrupt;

    /// Sealed trait for DMA channels.
    pub trait SealedChannel {
        /// Zero-based channel index into the TCD array.
        fn index(&self) -> usize;
        /// Interrupt vector for this channel.
        fn interrupt(&self) -> Interrupt;
    }

    /// Sealed trait for DMA request sources.
    pub trait SealedDmaRequest {}
}

/// Marker trait implemented by HAL peripheral tokens that map to a DMA0
/// channel backed by one EDMA_0_TCD0 TCD slot.
///
/// This trait is sealed and cannot be implemented outside this crate.
#[allow(private_bounds)]
pub trait Channel: sealed::SealedChannel + PeripheralType + Into<AnyChannel> + 'static {
    /// Zero-based channel index into the TCD array.
    const INDEX: usize;
    /// Interrupt vector for this channel.
    const INTERRUPT: Interrupt;
}

/// Type-erased DMA channel.
///
/// This allows storing DMA channels in a uniform way regardless of their
/// concrete type, useful for async transfer futures and runtime channel selection.
#[derive(Debug, Clone, Copy)]
pub struct AnyChannel {
    index: usize,
    interrupt: Interrupt,
}

impl AnyChannel {
    /// Get the channel index.
    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Get the channel interrupt.
    #[inline]
    pub const fn interrupt(&self) -> Interrupt {
        self.interrupt
    }

    /// Get a reference to the TCD register block for this channel.
    ///
    /// This steals the eDMA pointer internally since MCXA276 has only one eDMA instance.
    #[inline]
    fn tcd(&self) -> &'static pac::edma_0_tcd0::Tcd {
        // Safety: MCXA276 has a single eDMA instance, and we're only accessing
        // the TCD for this specific channel
        let edma = unsafe { &*pac::Edma0Tcd0::ptr() };
        edma.tcd(self.index)
    }

    /// Check if the channel's DONE flag is set.
    pub fn is_done(&self) -> bool {
        self.tcd().ch_csr().read().done().bit_is_set()
    }

    /// Get the waker for this channel.
    pub fn waker(&self) -> &'static AtomicWaker {
        &STATES[self.index].waker
    }
}

impl sealed::SealedChannel for AnyChannel {
    fn index(&self) -> usize {
        self.index
    }

    fn interrupt(&self) -> Interrupt {
        self.interrupt
    }
}

/// Macro to implement Channel trait for a peripheral.
macro_rules! impl_channel {
    ($peri:ident, $index:expr, $irq:ident) => {
        impl sealed::SealedChannel for crate::peripherals::$peri {
            fn index(&self) -> usize {
                $index
            }

            fn interrupt(&self) -> Interrupt {
                Interrupt::$irq
            }
        }

        impl Channel for crate::peripherals::$peri {
            const INDEX: usize = $index;
            const INTERRUPT: Interrupt = Interrupt::$irq;
        }

        impl From<crate::peripherals::$peri> for AnyChannel {
            fn from(_: crate::peripherals::$peri) -> Self {
                AnyChannel {
                    index: $index,
                    interrupt: Interrupt::$irq,
                }
            }
        }
    };
}

impl_channel!(DMA_CH0, 0, DMA_CH0);
impl_channel!(DMA_CH1, 1, DMA_CH1);
impl_channel!(DMA_CH2, 2, DMA_CH2);
impl_channel!(DMA_CH3, 3, DMA_CH3);
impl_channel!(DMA_CH4, 4, DMA_CH4);
impl_channel!(DMA_CH5, 5, DMA_CH5);
impl_channel!(DMA_CH6, 6, DMA_CH6);
impl_channel!(DMA_CH7, 7, DMA_CH7);

/// Strongly-typed handle to a DMA0 channel.
///
/// The lifetime of this value is tied to the unique peripheral token
/// supplied by `embassy_hal_internal::peripherals!`, so safe code cannot
/// create two `DmaChannel` instances for the same hardware channel.
pub struct DmaChannel<C: Channel> {
    _ch: core::marker::PhantomData<C>,
}

// ============================================================================
// DMA Transfer Methods - API Overview
// ============================================================================
//
// The DMA API provides two categories of methods for configuring transfers:
//
// ## 1. Async Methods (Return `Transfer` Future)
//
// These methods return a [`Transfer`] Future that must be `.await`ed:
//
// - [`write()`](DmaChannel::write) - Memory-to-peripheral using default eDMA TCD block
// - [`read()`](DmaChannel::read) - Peripheral-to-memory using default eDMA TCD block
// - [`write_to_peripheral()`](DmaChannel::write_to_peripheral) - Memory-to-peripheral with custom eDMA TCD block
// - [`read_from_peripheral()`](DmaChannel::read_from_peripheral) - Peripheral-to-memory with custom eDMA TCD block
// - [`mem_to_mem()`](DmaChannel::mem_to_mem) - Memory-to-memory using default eDMA TCD block
//
// The `Transfer` manages the DMA lifecycle automatically:
// - Enables channel request
// - Waits for completion via async/await
// - Cleans up on completion
//
// **Important:** `Transfer::Drop` aborts the transfer if dropped before completion.
// This means you MUST `.await` the Transfer or it will be aborted when it goes out of scope.
//
// **Use case:** When you want to use async/await and let the Transfer handle lifecycle management.
//
// ## 2. Setup Methods (Configure TCD Only)
//
// These methods configure the TCD but do NOT return a `Transfer`:
//
// - [`setup_write()`](DmaChannel::setup_write) - Memory-to-peripheral using default eDMA TCD block
// - [`setup_read()`](DmaChannel::setup_read) - Peripheral-to-memory using default eDMA TCD block
// - [`setup_write_to_peripheral()`](DmaChannel::setup_write_to_peripheral) - Memory-to-peripheral with custom eDMA TCD block
// - [`setup_read_from_peripheral()`](DmaChannel::setup_read_from_peripheral) - Peripheral-to-memory with custom eDMA TCD block
//
// The caller is responsible for the complete DMA lifecycle:
// 1. Call [`enable_request()`](DmaChannel::enable_request) to start the transfer
// 2. Poll [`is_done()`](DmaChannel::is_done) or use interrupts to detect completion
// 3. Call [`disable_request()`](DmaChannel::disable_request), [`clear_done()`](DmaChannel::clear_done),
//    [`clear_interrupt()`](DmaChannel::clear_interrupt) for cleanup
//
// **Use case:** Peripheral drivers (like LPUART) that need fine-grained control over
// DMA setup before starting a `Transfer`.
//
// ============================================================================

impl<C: Channel> DmaChannel<C> {
    /// Wrap a DMA channel token (takes ownership of the Peri wrapper).
    ///
    /// Note: DMA is initialized during `hal::init()` via `dma::init()`.
    #[inline]
    pub fn new(_ch: embassy_hal_internal::Peri<'_, C>) -> Self {
        unsafe {
            cortex_m::peripheral::NVIC::unmask(C::INTERRUPT);
        }
        Self {
            _ch: core::marker::PhantomData,
        }
    }

    /// Channel index in the EDMA_0_TCD0 array.
    #[inline]
    pub const fn index(&self) -> usize {
        C::INDEX
    }

    /// Convert this typed channel into a type-erased `AnyChannel`.
    #[inline]
    pub fn into_any(self) -> AnyChannel {
        AnyChannel {
            index: C::INDEX,
            interrupt: C::INTERRUPT,
        }
    }

    /// Get a reference to the type-erased channel info.
    #[inline]
    pub fn as_any(&self) -> AnyChannel {
        AnyChannel {
            index: C::INDEX,
            interrupt: C::INTERRUPT,
        }
    }

    /// Return a reference to the underlying TCD register block.
    ///
    /// This steals the eDMA pointer internally since MCXA276 has only one eDMA instance.
    ///
    /// # Note
    ///
    /// This is exposed for advanced use cases that need direct TCD access.
    /// For most use cases, prefer the higher-level transfer methods.
    #[inline]
    pub fn tcd(&self) -> &'static pac::edma_0_tcd0::Tcd {
        // Safety: MCXA276 has a single eDMA instance
        let edma = unsafe { &*pac::Edma0Tcd0::ptr() };
        edma.tcd(C::INDEX)
    }

    fn clear_tcd(t: &'static pac::edma_0_tcd0::Tcd) {
        // Full TCD reset following NXP SDK pattern (EDMA_TcdResetExt).
        // Reset ALL TCD registers to 0 to clear any stale configuration from
        // previous transfers. This is critical when reusing a channel.
        t.tcd_saddr().write(|w| unsafe { w.saddr().bits(0) });
        t.tcd_soff().write(|w| unsafe { w.soff().bits(0) });
        t.tcd_attr().write(|w| unsafe { w.bits(0) });
        t.tcd_nbytes_mloffno().write(|w| unsafe { w.nbytes().bits(0) });
        t.tcd_slast_sda().write(|w| unsafe { w.slast_sda().bits(0) });
        t.tcd_daddr().write(|w| unsafe { w.daddr().bits(0) });
        t.tcd_doff().write(|w| unsafe { w.doff().bits(0) });
        t.tcd_citer_elinkno().write(|w| unsafe { w.bits(0) });
        t.tcd_dlast_sga().write(|w| unsafe { w.dlast_sga().bits(0) });
        t.tcd_csr().write(|w| unsafe { w.bits(0) }); // Clear CSR completely
        t.tcd_biter_elinkno().write(|w| unsafe { w.bits(0) });
    }

    #[inline]
    fn set_major_loop_ct_elinkno(t: &'static pac::edma_0_tcd0::Tcd, count: u16) {
        t.tcd_biter_elinkno().write(|w| unsafe { w.biter().bits(count) });
        t.tcd_citer_elinkno().write(|w| unsafe { w.citer().bits(count) });
    }

    #[inline]
    fn set_minor_loop_ct_no_offsets(t: &'static pac::edma_0_tcd0::Tcd, count: u32) {
        t.tcd_nbytes_mloffno().write(|w| unsafe { w.nbytes().bits(count) });
    }

    #[inline]
    fn set_no_final_adjustments(t: &'static pac::edma_0_tcd0::Tcd) {
        // No source/dest adjustment after major loop
        t.tcd_slast_sda().write(|w| unsafe { w.slast_sda().bits(0) });
        t.tcd_dlast_sga().write(|w| unsafe { w.dlast_sga().bits(0) });
    }

    #[inline]
    fn set_source_ptr<T>(t: &'static pac::edma_0_tcd0::Tcd, p: *const T) {
        t.tcd_saddr().write(|w| unsafe { w.saddr().bits(p as u32) });
    }

    #[inline]
    fn set_source_increment(t: &'static pac::edma_0_tcd0::Tcd, sz: WordSize) {
        t.tcd_soff().write(|w| unsafe { w.soff().bits(sz.bytes() as u16) });
    }

    #[inline]
    fn set_source_fixed(t: &'static pac::edma_0_tcd0::Tcd) {
        t.tcd_soff().write(|w| unsafe { w.soff().bits(0) });
    }

    #[inline]
    fn set_dest_ptr<T>(t: &'static pac::edma_0_tcd0::Tcd, p: *mut T) {
        t.tcd_daddr().write(|w| unsafe { w.daddr().bits(p as u32) });
    }

    #[inline]
    fn set_dest_increment(t: &'static pac::edma_0_tcd0::Tcd, sz: WordSize) {
        t.tcd_doff().write(|w| unsafe { w.doff().bits(sz.bytes() as u16) });
    }

    #[inline]
    fn set_dest_fixed(t: &'static pac::edma_0_tcd0::Tcd) {
        t.tcd_doff().write(|w| unsafe { w.doff().bits(0) });
    }

    #[inline]
    fn set_even_transfer_size(t: &'static pac::edma_0_tcd0::Tcd, sz: WordSize) {
        let hw_size = sz.to_hw_size();
        t.tcd_attr()
            .write(|w| unsafe { w.ssize().bits(hw_size).dsize().bits(hw_size) });
    }

    #[inline]
    fn reset_channel_state(t: &'static pac::edma_0_tcd0::Tcd) {
        // CSR: Resets to all zeroes (disabled), "done" is cleared by writing 1
        t.ch_csr().write(|w| w.done().clear_bit_by_one());
        // ES: Resets to all zeroes (disabled), "err" is cleared by writing 1
        t.ch_es().write(|w| w.err().clear_bit_by_one());
        // INT: Resets to all zeroes (disabled), "int" is cleared by writing 1
        t.ch_int().write(|w| w.int().clear_bit_by_one());
    }

    /// Start an async transfer.
    ///
    /// The channel must already be configured. This enables the channel
    /// request and returns a `Transfer` future that resolves when the
    /// DMA transfer completes.
    ///
    /// # Safety
    ///
    /// The caller must ensure the DMA channel has been properly configured
    /// and that source/destination buffers remain valid for the duration
    /// of the transfer.
    pub unsafe fn start_transfer(&self) -> Transfer<'_> {
        // Clear any previous DONE/INT flags
        let t = self.tcd();
        t.ch_csr().modify(|_, w| w.done().clear_bit_by_one());
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // Enable the channel request
        t.ch_csr().modify(|_, w| w.erq().enable());

        Transfer::new(self.as_any())
    }

    // ========================================================================
    // Type-Safe Transfer Methods (Embassy-style API)
    // ========================================================================

    /// Perform a memory-to-memory DMA transfer (simplified API).
    ///
    /// This is a type-safe wrapper that uses the `Word` trait to determine
    /// the correct transfer width automatically. Uses the global eDMA TCD
    /// register accessor internally.
    ///
    /// # Arguments
    ///
    /// * `src` - Source buffer
    /// * `dst` - Destination buffer (must be at least as large as src)
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// The source and destination buffers must remain valid for the
    /// duration of the transfer.
    pub fn mem_to_mem<W: Word>(
        &self,
        src: &[W],
        dst: &mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'_>, Error> {
        let mut invalid = false;
        invalid |= src.is_empty();
        invalid |= src.len() > dst.len();
        invalid |= src.len() > 0x7fff;
        if invalid {
            return Err(Error::Configuration);
        }

        let size = W::size();
        let byte_count = (src.len() * size.bytes()) as u32;

        let t = self.tcd();

        // Reset channel state - clear DONE, disable requests, clear errors
        Self::reset_channel_state(t);

        // Memory barrier to ensure channel state is fully reset before touching TCD
        cortex_m::asm::dsb();

        Self::clear_tcd(t);

        // Memory barrier after TCD reset
        cortex_m::asm::dsb();

        // Note: Priority is managed by round-robin arbitration (set in init())
        // Per-channel priority can be configured via ch_pri() if needed

        // Now configure the new transfer

        // Source address and increment
        Self::set_source_ptr(t, src.as_ptr());
        Self::set_source_increment(t, size);

        // Destination address and increment
        Self::set_dest_ptr(t, dst.as_mut_ptr());
        Self::set_dest_increment(t, size);

        // Transfer attributes (size)
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer all bytes in one minor loop
        Self::set_minor_loop_ct_no_offsets(t, byte_count);

        // No source/dest adjustment after major loop
        Self::set_no_final_adjustments(t);

        // Major loop count = 1 (single major loop)
        // Write BITER first, then CITER (CITER must match BITER at start)
        Self::set_major_loop_ct_elinkno(t, 1);

        // Memory barrier before setting START
        cortex_m::asm::dsb();

        // Control/status: interrupt on major complete, start
        // Write this last after all other TCD registers are configured
        let int_major = options.complete_transfer_interrupt;
        t.tcd_csr().write(|w| {
            w.intmajor()
                .bit(int_major)
                .inthalf()
                .bit(options.half_transfer_interrupt)
                .dreq()
                .set_bit() // Auto-disable request after major loop
                .start()
                .set_bit() // Start the channel
        });

        Ok(Transfer::new(self.as_any()))
    }

    /// Fill a memory buffer with a pattern value (memset).
    ///
    /// This performs a DMA transfer where the source address remains fixed
    /// (pattern value) while the destination address increments through the buffer.
    /// It's useful for quickly filling large memory regions with a constant value.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Reference to the pattern value (will be read repeatedly)
    /// * `dst` - Destination buffer to fill
    /// * `options` - Transfer configuration options
    ///
    /// # Example
    ///
    /// ```no_run
    /// use embassy_mcxa::dma::{DmaChannel, TransferOptions};
    ///
    /// let dma_ch = DmaChannel::new(p.DMA_CH0);
    /// let pattern: u32 = 0xDEADBEEF;
    /// let mut buffer = [0u32; 256];
    ///
    /// unsafe {
    ///     dma_ch.memset(&pattern, &mut buffer, TransferOptions::default()).await;
    /// }
    /// // buffer is now filled with 0xDEADBEEF
    /// ```
    ///
    pub fn memset<W: Word>(&self, pattern: &W, dst: &mut [W], options: TransferOptions) -> Transfer<'_> {
        assert!(!dst.is_empty());
        assert!(dst.len() <= 0x7fff);

        let size = W::size();
        let byte_size = size.bytes();
        // Total bytes to transfer - all in one minor loop for software-triggered transfers
        let total_bytes = (dst.len() * byte_size) as u32;

        let t = self.tcd();

        // Reset channel state - clear DONE, disable requests, clear errors
        Self::reset_channel_state(t);

        // Memory barrier to ensure channel state is fully reset before touching TCD
        cortex_m::asm::dsb();

        Self::clear_tcd(t);

        // Memory barrier after TCD reset
        cortex_m::asm::dsb();

        // Now configure the new transfer
        //
        // For software-triggered memset, we use a SINGLE minor loop that transfers
        // all bytes at once. The source address stays fixed (SOFF=0) while the
        // destination increments (DOFF=byte_size). The eDMA will read from the
        // same source address for each destination word.
        //
        // This is necessary because the START bit only triggers ONE minor loop
        // iteration. Using CITER>1 with software trigger would require multiple
        // START triggers.

        // Source: pattern address, fixed (soff=0)
        Self::set_source_ptr(t, pattern);
        Self::set_source_fixed(t);

        // Destination: memory buffer, incrementing by word size
        Self::set_dest_ptr(t, dst.as_mut_ptr());
        Self::set_dest_increment(t, size);

        // Transfer attributes - source and dest are same word size
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer ALL bytes in one minor loop (like mem_to_mem)
        // This allows the entire transfer to complete with a single START trigger
        Self::set_minor_loop_ct_no_offsets(t, total_bytes);

        // No address adjustment after major loop
        Self::set_no_final_adjustments(t);

        // Major loop count = 1 (single major loop, all data in minor loop)
        // Write BITER first, then CITER (CITER must match BITER at start)
        Self::set_major_loop_ct_elinkno(t, 1);

        // Memory barrier before setting START
        cortex_m::asm::dsb();

        // Control/status: interrupt on major complete, start immediately
        // Write this last after all other TCD registers are configured
        let int_major = options.complete_transfer_interrupt;
        t.tcd_csr().write(|w| {
            w.intmajor()
                .bit(int_major)
                .inthalf()
                .bit(options.half_transfer_interrupt)
                .dreq()
                .set_bit() // Auto-disable request after major loop
                .start()
                .set_bit() // Start the channel
        });

        Transfer::new(self.as_any())
    }

    /// Write data from memory to a peripheral register.
    ///
    /// The destination address remains fixed (peripheral register) while
    /// the source address increments through the buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - Source buffer to write from
    /// * `peri_addr` - Peripheral register address
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub unsafe fn write<W: Word>(&self, buf: &[W], peri_addr: *mut W, options: TransferOptions) -> Transfer<'_> {
        unsafe { self.write_to_peripheral(buf, peri_addr, options) }
    }

    /// Configure a memory-to-peripheral DMA transfer without starting it.
    ///
    /// This is a convenience wrapper around [`setup_write_to_peripheral()`](Self::setup_write_to_peripheral)
    /// that uses the default eDMA TCD register block.
    ///
    /// This method configures the TCD but does NOT return a `Transfer`. The caller
    /// is responsible for the complete DMA lifecycle:
    /// 1. Call [`enable_request()`](Self::enable_request) to start the transfer
    /// 2. Poll [`is_done()`](Self::is_done) or use interrupts to detect completion
    /// 3. Call [`disable_request()`](Self::disable_request), [`clear_done()`](Self::clear_done),
    ///    [`clear_interrupt()`](Self::clear_interrupt) for cleanup
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embassy_mcxa::dma::DmaChannel;
    /// # let dma_ch = DmaChannel::new(p.DMA_CH0);
    /// # let uart_tx_addr = 0x4000_0000 as *mut u8;
    /// let data = [0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    ///
    /// unsafe {
    ///     // Configure the transfer
    ///     dma_ch.setup_write(&data, uart_tx_addr, EnableInterrupt::Yes);
    ///
    ///     // Start when peripheral is ready
    ///     dma_ch.enable_request();
    ///
    ///     // Wait for completion (or use interrupt)
    ///     while !dma_ch.is_done() {}
    ///
    ///     // Clean up
    ///     dma_ch.clear_done();
    ///     dma_ch.clear_interrupt();
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `buf` - Source buffer to write from
    /// * `peri_addr` - Peripheral register address
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub unsafe fn setup_write<W: Word>(&self, buf: &[W], peri_addr: *mut W, enable_interrupt: EnableInterrupt) {
        unsafe { self.setup_write_to_peripheral(buf, peri_addr, enable_interrupt) }
    }

    /// Write data from memory to a peripheral register.
    ///
    /// The destination address remains fixed (peripheral register) while
    /// the source address increments through the buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - Source buffer to write from
    /// * `peri_addr` - Peripheral register address
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub unsafe fn write_to_peripheral<W: Word>(
        &self,
        buf: &[W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'_> {
        assert!(!buf.is_empty());
        assert!(buf.len() <= 0x7fff);

        let size = W::size();
        let byte_size = size.bytes();

        let t = self.tcd();

        // Reset channel state
        Self::reset_channel_state(t);

        // Addresses
        Self::set_source_ptr(t, buf.as_ptr());
        Self::set_dest_ptr(t, peri_addr);

        // Offsets: Source increments, Dest fixed
        Self::set_source_increment(t, size);
        Self::set_dest_fixed(t);

        // Attributes: set size and explicitly disable modulo
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer one word per request (match old: only set nbytes)
        Self::set_minor_loop_ct_no_offsets(t, byte_size as u32);

        // No final adjustments
        Self::set_no_final_adjustments(t);

        // Major loop count = number of words
        let count = buf.len() as u16;
        Self::set_major_loop_ct_elinkno(t, count);

        // CSR: interrupt on major loop complete and auto-clear ERQ
        t.tcd_csr().write(|w| {
            let w = if options.complete_transfer_interrupt {
                w.intmajor().enable()
            } else {
                w.intmajor().disable()
            };
            w.inthalf()
                .disable()
                .dreq()
                .erq_field_clear() // Disable request when done
                .esg()
                .normal_format()
                .majorelink()
                .disable()
                .eeop()
                .disable()
                .esda()
                .disable()
                .bwc()
                .no_stall()
        });

        // Ensure all TCD writes have completed before DMA engine reads them
        cortex_m::asm::dsb();

        Transfer::new(self.as_any())
    }

    /// Read data from a peripheral register to memory.
    ///
    /// The source address remains fixed (peripheral register) while
    /// the destination address increments through the buffer.
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address
    /// * `buf` - Destination buffer to read into
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for reads.
    pub unsafe fn read<W: Word>(&self, peri_addr: *const W, buf: &mut [W], options: TransferOptions) -> Transfer<'_> {
        unsafe { self.read_from_peripheral(peri_addr, buf, options) }
    }

    /// Configure a peripheral-to-memory DMA transfer without starting it.
    ///
    /// This is a convenience wrapper around [`setup_read_from_peripheral()`](Self::setup_read_from_peripheral)
    /// that uses the default eDMA TCD register block.
    ///
    /// This method configures the TCD but does NOT return a `Transfer`. The caller
    /// is responsible for the complete DMA lifecycle:
    /// 1. Call [`enable_request()`](Self::enable_request) to start the transfer
    /// 2. Poll [`is_done()`](Self::is_done) or use interrupts to detect completion
    /// 3. Call [`disable_request()`](Self::disable_request), [`clear_done()`](Self::clear_done),
    ///    [`clear_interrupt()`](Self::clear_interrupt) for cleanup
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embassy_mcxa::dma::DmaChannel;
    /// # let dma_ch = DmaChannel::new(p.DMA_CH0);
    /// # let uart_rx_addr = 0x4000_0000 as *const u8;
    /// let mut buf = [0u8; 32];
    ///
    /// unsafe {
    ///     // Configure the transfer
    ///     dma_ch.setup_read(uart_rx_addr, &mut buf, EnableInterrupt::Yes);
    ///
    ///     // Start when peripheral is ready
    ///     dma_ch.enable_request();
    ///
    ///     // Wait for completion (or use interrupt)
    ///     while !dma_ch.is_done() {}
    ///
    ///     // Clean up
    ///     dma_ch.clear_done();
    ///     dma_ch.clear_interrupt();
    /// }
    /// // buf now contains received data
    /// ```
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address
    /// * `buf` - Destination buffer to read into
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for reads.
    pub unsafe fn setup_read<W: Word>(&self, peri_addr: *const W, buf: &mut [W], enable_interrupt: EnableInterrupt) {
        unsafe { self.setup_read_from_peripheral(peri_addr, buf, enable_interrupt) }
    }

    /// Read data from a peripheral register to memory.
    ///
    /// The source address remains fixed (peripheral register) while
    /// the destination address increments through the buffer.
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address
    /// * `buf` - Destination buffer to read into
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for reads.
    pub unsafe fn read_from_peripheral<W: Word>(
        &self,
        peri_addr: *const W,
        buf: &mut [W],
        options: TransferOptions,
    ) -> Transfer<'_> {
        assert!(!buf.is_empty());
        assert!(buf.len() <= 0x7fff);

        let size = W::size();
        let byte_size = size.bytes();

        let t = self.tcd();

        // Reset channel control/error/interrupt state
        Self::reset_channel_state(t);

        // Source: peripheral register, fixed
        Self::set_source_ptr(t, peri_addr);
        Self::set_source_fixed(t);

        // Destination: memory buffer, incrementing
        Self::set_dest_ptr(t, buf.as_mut_ptr());
        Self::set_dest_increment(t, size);

        // Transfer attributes: set size and explicitly disable modulo
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer one word per request, no offsets
        Self::set_minor_loop_ct_no_offsets(t, byte_size as u32);

        // Major loop count = number of words
        let count = buf.len() as u16;
        Self::set_major_loop_ct_elinkno(t, count);

        // No address adjustment after major loop
        Self::set_no_final_adjustments(t);

        // Control/status: interrupt on major complete, auto-clear ERQ when done
        t.tcd_csr().write(|w| {
            let w = if options.complete_transfer_interrupt {
                w.intmajor().enable()
            } else {
                w.intmajor().disable()
            };
            let w = if options.half_transfer_interrupt {
                w.inthalf().enable()
            } else {
                w.inthalf().disable()
            };
            w.dreq()
                .erq_field_clear() // Disable request when done (important for peripheral DMA)
                .esg()
                .normal_format()
                .majorelink()
                .disable()
                .eeop()
                .disable()
                .esda()
                .disable()
                .bwc()
                .no_stall()
        });

        // Ensure all TCD writes have completed before DMA engine reads them
        cortex_m::asm::dsb();

        Transfer::new(self.as_any())
    }

    /// Configure a memory-to-peripheral DMA transfer without starting it.
    ///
    /// This configures the TCD for a memory-to-peripheral transfer but does NOT
    /// return a Transfer object. The caller is responsible for:
    /// 1. Enabling the peripheral's DMA request
    /// 2. Calling `enable_request()` to start the transfer
    /// 3. Polling `is_done()` or using interrupts to detect completion
    /// 4. Calling `disable_request()`, `clear_done()`, `clear_interrupt()` for cleanup
    ///
    /// Use this when you need manual control over the DMA lifecycle (e.g., in
    /// peripheral drivers that have their own completion polling).
    ///
    /// # Arguments
    ///
    /// * `buf` - Source buffer to write from
    /// * `peri_addr` - Peripheral register address
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub unsafe fn setup_write_to_peripheral<W: Word>(
        &self,
        buf: &[W],
        peri_addr: *mut W,
        enable_interrupt: EnableInterrupt,
    ) {
        assert!(!buf.is_empty());
        assert!(buf.len() <= 0x7fff);

        let size = W::size();
        let byte_size = size.bytes();

        let t = self.tcd();

        // Reset channel state
        Self::reset_channel_state(t);

        // Addresses
        Self::set_source_ptr(t, buf.as_ptr());
        Self::set_dest_ptr(t, peri_addr);

        // Offsets: Source increments, Dest fixed
        Self::set_source_increment(t, size);
        Self::set_dest_fixed(t);

        // Attributes: set size and explicitly disable modulo
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer one word per request
        Self::set_minor_loop_ct_no_offsets(t, byte_size as u32);

        // No final adjustments
        Self::set_no_final_adjustments(t);

        // Major loop count = number of words
        let count = buf.len() as u16;
        Self::set_major_loop_ct_elinkno(t, count);

        // CSR: optional interrupt on major loop complete and auto-clear ERQ
        t.tcd_csr().write(|w| {
            let w = match enable_interrupt {
                EnableInterrupt::Yes => w.intmajor().enable(),
                EnableInterrupt::No => w.intmajor().disable(),
            };
            w.inthalf()
                .disable()
                .dreq()
                .erq_field_clear()
                .esg()
                .normal_format()
                .majorelink()
                .disable()
                .eeop()
                .disable()
                .esda()
                .disable()
                .bwc()
                .no_stall()
        });

        // Ensure all TCD writes have completed before DMA engine reads them
        cortex_m::asm::dsb();
    }

    /// Configure a peripheral-to-memory DMA transfer without starting it.
    ///
    /// This configures the TCD for a peripheral-to-memory transfer but does NOT
    /// return a Transfer object. The caller is responsible for:
    /// 1. Enabling the peripheral's DMA request
    /// 2. Calling `enable_request()` to start the transfer
    /// 3. Polling `is_done()` or using interrupts to detect completion
    /// 4. Calling `disable_request()`, `clear_done()`, `clear_interrupt()` for cleanup
    ///
    /// Use this when you need manual control over the DMA lifecycle (e.g., in
    /// peripheral drivers that have their own completion polling).
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address
    /// * `buf` - Destination buffer to read into
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for reads.
    pub unsafe fn setup_read_from_peripheral<W: Word>(
        &self,
        peri_addr: *const W,
        buf: &mut [W],
        enable_interrupt: EnableInterrupt,
    ) {
        assert!(!buf.is_empty());
        assert!(buf.len() <= 0x7fff);

        let size = W::size();
        let byte_size = size.bytes();

        let t = self.tcd();

        // Reset channel control/error/interrupt state
        Self::reset_channel_state(t);

        // Source: peripheral register, fixed
        Self::set_source_ptr(t, peri_addr);
        Self::set_source_fixed(t);

        // Destination: memory buffer, incrementing
        Self::set_dest_ptr(t, buf.as_mut_ptr());
        Self::set_dest_increment(t, size);

        // Attributes: set size and explicitly disable modulo
        Self::set_even_transfer_size(t, size);

        // Minor loop: transfer one word per request
        Self::set_minor_loop_ct_no_offsets(t, byte_size as u32);

        // No final adjustments
        Self::set_no_final_adjustments(t);

        // Major loop count = number of words
        let count = buf.len() as u16;
        Self::set_major_loop_ct_elinkno(t, count);

        // CSR: optional interrupt on major loop complete and auto-clear ERQ
        t.tcd_csr().write(|w| {
            let w = match enable_interrupt {
                EnableInterrupt::Yes => w.intmajor().enable(),
                EnableInterrupt::No => w.intmajor().disable(),
            };
            w.inthalf()
                .disable()
                .dreq()
                .erq_field_clear()
                .esg()
                .normal_format()
                .majorelink()
                .disable()
                .eeop()
                .disable()
                .esda()
                .disable()
                .bwc()
                .no_stall()
        });

        // Ensure all TCD writes have completed before DMA engine reads them
        cortex_m::asm::dsb();
    }

    /// Configure the integrated channel MUX to use the given typed
    /// DMA request source (e.g., [`Lpuart2TxRequest`] or [`Lpuart2RxRequest`]).
    ///
    /// This is the type-safe version that uses marker types to ensure
    /// compile-time verification of request source validity.
    ///
    /// # Safety
    ///
    /// The channel must be properly configured before enabling requests.
    /// The caller must ensure the DMA request source matches the peripheral
    /// that will drive this channel.
    ///
    /// # Note
    ///
    /// The NXP SDK requires a two-step write sequence: first clear
    /// the mux to 0, then set the actual source. This is a hardware
    /// requirement on eDMA4 for the mux to properly latch.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use embassy_mcxa::dma::{DmaChannel, Lpuart2RxRequest};
    ///
    /// // Type-safe: compiler verifies this is a valid DMA request type
    /// unsafe {
    ///     channel.set_request_source::<Lpuart2RxRequest>();
    /// }
    /// ```
    #[inline]
    pub unsafe fn set_request_source<R: DmaRequest>(&self) {
        unsafe {
            // Two-step write per NXP SDK: clear to 0, then set actual source.
            self.tcd().ch_mux().write(|w| w.src().bits(0));
            cortex_m::asm::dsb(); // Ensure the clear completes before setting new source
            self.tcd().ch_mux().write(|w| w.src().bits(R::REQUEST_NUMBER));
        }
    }

    /// Enable hardware requests for this channel (ERQ=1).
    ///
    /// # Safety
    ///
    /// The channel must be properly configured before enabling requests.
    pub unsafe fn enable_request(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|_, w| w.erq().enable());
    }

    /// Disable hardware requests for this channel (ERQ=0).
    ///
    /// # Safety
    ///
    /// Disabling requests on an active transfer may leave the transfer incomplete.
    pub unsafe fn disable_request(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|_, w| w.erq().disable());
    }

    /// Return true if the channel's DONE flag is set.
    pub fn is_done(&self) -> bool {
        let t = self.tcd();
        t.ch_csr().read().done().bit_is_set()
    }

    /// Clear the DONE flag for this channel.
    ///
    /// Uses modify to preserve other bits (especially ERQ) unlike write
    /// which would clear ERQ and halt an active transfer.
    ///
    /// # Safety
    ///
    /// Clearing DONE while a transfer is in progress may cause undefined behavior.
    pub unsafe fn clear_done(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|_, w| w.done().clear_bit_by_one());
    }

    /// Clear the channel interrupt flag (CH_INT.INT).
    ///
    /// # Safety
    ///
    /// Must be called from the correct interrupt context or with interrupts disabled.
    pub unsafe fn clear_interrupt(&self) {
        let t = self.tcd();
        t.ch_int().write(|w| w.int().clear_bit_by_one());
    }

    /// Trigger a software start for this channel.
    ///
    /// # Safety
    ///
    /// The channel must be properly configured with a valid TCD before triggering.
    pub unsafe fn trigger_start(&self) {
        let t = self.tcd();
        t.tcd_csr().modify(|_, w| w.start().channel_started());
    }

    /// Get the waker for this channel
    pub fn waker(&self) -> &'static AtomicWaker {
        &STATES[C::INDEX].waker
    }

    /// Enable the interrupt for this channel in the NVIC.
    pub fn enable_interrupt(&self) {
        unsafe {
            cortex_m::peripheral::NVIC::unmask(C::INTERRUPT);
        }
    }

    /// Enable Major Loop Linking.
    ///
    /// When the major loop completes, the hardware will trigger a service request
    /// on `link_ch`.
    ///
    /// # Arguments
    ///
    /// * `link_ch` - Target channel index (0-7) to link to
    ///
    /// # Safety
    ///
    /// The channel must be properly configured before setting up linking.
    pub unsafe fn set_major_link(&self, link_ch: usize) {
        unsafe {
            let t = self.tcd();
            t.tcd_csr()
                .modify(|_, w| w.majorelink().enable().majorlinkch().bits(link_ch as u8));
        }
    }

    /// Disable Major Loop Linking.
    ///
    /// Removes any major loop channel linking previously configured.
    ///
    /// # Safety
    ///
    /// The caller must ensure this doesn't disrupt an active transfer that
    /// depends on the linking.
    pub unsafe fn clear_major_link(&self) {
        let t = self.tcd();
        t.tcd_csr().modify(|_, w| w.majorelink().disable());
    }

    /// Enable Minor Loop Linking.
    ///
    /// After each minor loop, the hardware will trigger a service request
    /// on `link_ch`.
    ///
    /// # Arguments
    ///
    /// * `link_ch` - Target channel index (0-7) to link to
    ///
    /// # Note
    ///
    /// This rewrites CITER and BITER registers to the ELINKYES format.
    /// It preserves the current loop count.
    ///
    /// # Safety
    ///
    /// The channel must be properly configured before setting up linking.
    pub unsafe fn set_minor_link(&self, link_ch: usize) {
        unsafe {
            let t = self.tcd();

            // Read current CITER (assuming ELINKNO format initially)
            let current_citer = t.tcd_citer_elinkno().read().citer().bits();
            let current_biter = t.tcd_biter_elinkno().read().biter().bits();

            // Write back using ELINKYES format
            t.tcd_citer_elinkyes().write(|w| {
                w.citer()
                    .bits(current_citer)
                    .elink()
                    .enable()
                    .linkch()
                    .bits(link_ch as u8)
            });

            t.tcd_biter_elinkyes().write(|w| {
                w.biter()
                    .bits(current_biter)
                    .elink()
                    .enable()
                    .linkch()
                    .bits(link_ch as u8)
            });
        }
    }

    /// Disable Minor Loop Linking.
    ///
    /// Removes any minor loop channel linking previously configured.
    /// This rewrites CITER and BITER registers to the ELINKNO format,
    /// preserving the current loop count.
    ///
    /// # Safety
    ///
    /// The caller must ensure this doesn't disrupt an active transfer that
    /// depends on the linking.
    pub unsafe fn clear_minor_link(&self) {
        unsafe {
            let t = self.tcd();

            // Read current CITER (could be in either format, but we only need the count)
            // Note: In ELINKYES format, citer is 9 bits; in ELINKNO, it's 15 bits.
            // We read from ELINKNO which will give us the combined value.
            let current_citer = t.tcd_citer_elinkno().read().citer().bits();
            let current_biter = t.tcd_biter_elinkno().read().biter().bits();

            // Write back using ELINKNO format (disabling link)
            t.tcd_citer_elinkno()
                .write(|w| w.citer().bits(current_citer).elink().disable());

            t.tcd_biter_elinkno()
                .write(|w| w.biter().bits(current_biter).elink().disable());
        }
    }

    /// Load a TCD from memory into the hardware channel registers.
    ///
    /// This is useful for scatter/gather and ping-pong transfers where
    /// TCDs are prepared in RAM and then loaded into the hardware.
    ///
    /// # Safety
    ///
    /// - The TCD must be properly initialized.
    /// - The caller must ensure no concurrent access to the same channel.
    pub unsafe fn load_tcd(&self, tcd: &Tcd) {
        unsafe {
            let t = self.tcd();
            t.tcd_saddr().write(|w| w.saddr().bits(tcd.saddr));
            t.tcd_soff().write(|w| w.soff().bits(tcd.soff as u16));
            t.tcd_attr().write(|w| w.bits(tcd.attr));
            t.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(tcd.nbytes));
            t.tcd_slast_sda().write(|w| w.slast_sda().bits(tcd.slast as u32));
            t.tcd_daddr().write(|w| w.daddr().bits(tcd.daddr));
            t.tcd_doff().write(|w| w.doff().bits(tcd.doff as u16));
            t.tcd_citer_elinkno().write(|w| w.citer().bits(tcd.citer));
            t.tcd_dlast_sga().write(|w| w.dlast_sga().bits(tcd.dlast_sga as u32));
            t.tcd_csr().write(|w| w.bits(tcd.csr));
            t.tcd_biter_elinkno().write(|w| w.biter().bits(tcd.biter));
        }
    }
}

/// In-memory representation of a Transfer Control Descriptor (TCD).
///
/// This matches the hardware layout (32 bytes).
#[repr(C, align(32))]
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Tcd {
    pub saddr: u32,
    pub soff: i16,
    pub attr: u16,
    pub nbytes: u32,
    pub slast: i32,
    pub daddr: u32,
    pub doff: i16,
    pub citer: u16,
    pub dlast_sga: i32,
    pub csr: u16,
    pub biter: u16,
}

struct State {
    /// Waker for transfer complete interrupt
    waker: AtomicWaker,
    /// Waker for half-transfer interrupt
    half_waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            half_waker: AtomicWaker::new(),
        }
    }
}

static STATES: [State; 8] = [
    State::new(),
    State::new(),
    State::new(),
    State::new(),
    State::new(),
    State::new(),
    State::new(),
    State::new(),
];

pub(crate) fn waker(idx: usize) -> &'static AtomicWaker {
    &STATES[idx].waker
}

pub(crate) fn half_waker(idx: usize) -> &'static AtomicWaker {
    &STATES[idx].half_waker
}

// ============================================================================
// Async Transfer Future
// ============================================================================

/// An in-progress DMA transfer.
///
/// This type implements `Future` and can be `.await`ed to wait for the
/// transfer to complete. Dropping the transfer will abort it.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: AnyChannel,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> Transfer<'a> {
    /// Create a new transfer for the given channel.
    ///
    /// The caller must have already configured and started the DMA channel.
    pub(crate) fn new(channel: AnyChannel) -> Self {
        Self {
            channel,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Check if the transfer is still running.
    pub fn is_running(&self) -> bool {
        !self.channel.is_done()
    }

    /// Get the remaining transfer count.
    pub fn remaining(&self) -> u16 {
        let t = self.channel.tcd();
        t.tcd_citer_elinkno().read().citer().bits()
    }

    /// Block until the transfer completes.
    pub fn blocking_wait(self) {
        while self.is_running() {
            core::hint::spin_loop();
        }

        // Ensure all DMA writes are visible
        fence(Ordering::SeqCst);

        // Don't run drop (which would abort)
        core::mem::forget(self);
    }

    /// Wait for the half-transfer interrupt asynchronously.
    ///
    /// This is useful for double-buffering scenarios where you want to process
    /// the first half of the buffer while the second half is being filled.
    ///
    /// Returns `true` if the half-transfer occurred, `false` if the transfer
    /// completed before the half-transfer interrupt.
    ///
    /// # Note
    ///
    /// The transfer must be configured with `TransferOptions::half_transfer_interrupt = true`
    /// for this method to work correctly.
    pub async fn wait_half(&mut self) -> Result<bool, TransferErrorRaw> {
        use core::future::poll_fn;

        poll_fn(|cx| {
            let state = &STATES[self.channel.index];

            // Register the half-transfer waker
            state.half_waker.register(cx.waker());

            // Check if there's an error
            let t = self.channel.tcd();
            let es = t.ch_es().read();
            if es.err().is_error() {
                // Currently, all error fields are in the lowest 8 bits, as-casting truncates
                let errs = es.bits() as u8;
                return Poll::Ready(Err(TransferErrorRaw(errs)));
            }

            // Check if we're past the half-way point
            let biter = t.tcd_biter_elinkno().read().biter().bits();
            let citer = t.tcd_citer_elinkno().read().citer().bits();
            let half_point = biter / 2;

            if self.channel.is_done() {
                // Transfer completed before half-transfer
                Poll::Ready(Ok(false))
            } else if citer <= half_point {
                // We're past the half-way point
                fence(Ordering::SeqCst);
                Poll::Ready(Ok(true))
            } else {
                Poll::Pending
            }
        })
        .await
    }

    /// Abort the transfer.
    fn abort(&mut self) {
        let t = self.channel.tcd();

        // Disable channel requests
        t.ch_csr().modify(|_, w| w.erq().disable());

        // Clear any pending interrupt
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // Clear DONE flag
        t.ch_csr().modify(|_, w| w.done().clear_bit_by_one());

        fence(Ordering::SeqCst);
    }
}

/// Raw transfer error bits. Can be queried or all errors can be iterated over
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct TransferErrorRaw(u8);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct TransferErrorRawIter(u8);

impl TransferErrorRaw {
    const MAP: &[(u8, TransferError)] = &[
        (1 << 0, TransferError::DestinationBus),
        (1 << 1, TransferError::SourceBus),
        (1 << 2, TransferError::ScatterGatherConfiguration),
        (1 << 3, TransferError::NbytesCiterConfiguration),
        (1 << 4, TransferError::DestinationOffset),
        (1 << 5, TransferError::DestinationAddress),
        (1 << 6, TransferError::SourceOffset),
        (1 << 7, TransferError::SourceAddress),
    ];

    /// Convert to an iterator of contained errors
    pub fn err_iter(self) -> TransferErrorRawIter {
        TransferErrorRawIter(self.0)
    }

    /// Destination Bus Error
    #[inline]
    pub fn has_destination_bus_err(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// Source Bus Error
    #[inline]
    pub fn has_source_bus_err(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// Indicates that `TCDn_DLAST_SGA` is not on a 32-byte boundary. This field is
    /// checked at the beginning of a scatter/gather operation after major loop completion
    /// if `TCDn_CSR[ESG]` is enabled.
    #[inline]
    pub fn has_scatter_gather_configuration_err(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// This error indicates that one of the following has occurred:
    ///
    /// * `TCDn_NBYTES` is not a multiple of `TCDn_ATTR[SSIZE]` and `TCDn_ATTR[DSIZE]`
    /// * `TCDn_CITER[CITER]` is equal to zero
    /// * `TCDn_CITER[ELINK]` is not equal to `TCDn_BITER[ELINK]`
    #[inline]
    pub fn has_nbytes_citer_configuration_err(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    /// `TCDn_DOFF` is inconsistent with `TCDn_ATTR[DSIZE]`.
    #[inline]
    pub fn has_destination_offset_err(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// `TCDn_DADDR` is inconsistent with `TCDn_ATTR[DSIZE]`.
    #[inline]
    pub fn has_destination_address_err(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// `TCDn_SOFF` is inconsistent with `TCDn_ATTR[SSIZE]`.
    #[inline]
    pub fn has_source_offset_err(&self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    /// `TCDn_SADDR` is inconsistent with `TCDn_ATTR[SSIZE]`
    #[inline]
    pub fn has_source_address_err(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }
}

impl Iterator for TransferErrorRawIter {
    type Item = TransferError;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        for (mask, var) in TransferErrorRaw::MAP {
            // If the bit is set...
            if self.0 & mask != 0 {
                // clear the bit
                self.0 &= !mask;
                // and return the answer
                return Some(*var);
            }
        }

        // Shouldn't happen, but oh well.
        None
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferError {
    /// `TCDn_SADDR` is inconsistent with `TCDn_ATTR[SSIZE]`
    SourceAddress,
    /// `TCDn_SOFF` is inconsistent with `TCDn_ATTR[SSIZE]`.
    SourceOffset,
    /// `TCDn_DADDR` is inconsistent with `TCDn_ATTR[DSIZE]`.
    DestinationAddress,
    /// `TCDn_DOFF` is inconsistent with `TCDn_ATTR[DSIZE]`.
    DestinationOffset,
    /// This error indicates that one of the following has occurred:
    ///
    /// * `TCDn_NBYTES` is not a multiple of `TCDn_ATTR[SSIZE]` and `TCDn_ATTR[DSIZE]`
    /// * `TCDn_CITER[CITER]` is equal to zero
    /// * `TCDn_CITER[ELINK]` is not equal to `TCDn_BITER[ELINK]`
    NbytesCiterConfiguration,
    /// Indicates that `TCDn_DLAST_SGA` is not on a 32-byte boundary. This field is
    /// checked at the beginning of a scatter/gather operation after major loop completion
    /// if `TCDn_CSR[ESG]` is enabled.
    ScatterGatherConfiguration,
    /// Source Bus Error
    SourceBus,
    /// Destination Bus Error
    DestinationBus,
}

impl<'a> Unpin for Transfer<'a> {}

impl<'a> Future for Transfer<'a> {
    type Output = Result<(), TransferErrorRaw>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = &STATES[self.channel.index];

        // Register waker first
        state.waker.register(cx.waker());

        let done = self.channel.is_done();

        if done {
            // Ensure all DMA writes are visible before returning
            fence(Ordering::SeqCst);

            let es = self.channel.tcd().ch_es().read();
            if es.err().is_error() {
                // Currently, all error fields are in the lowest 8 bits, as-casting truncates
                let errs = es.bits() as u8;
                Poll::Ready(Err(TransferErrorRaw(errs)))
            } else {
                Poll::Ready(Ok(()))
            }
        } else {
            Poll::Pending
        }
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        // Only abort if the transfer is still running
        // If already complete, no need to abort
        if self.is_running() {
            self.abort();

            // Wait for abort to complete
            while self.is_running() {
                core::hint::spin_loop();
            }
        }

        fence(Ordering::SeqCst);
    }
}

// ============================================================================
// Ring Buffer for Circular DMA
// ============================================================================

/// A ring buffer for continuous DMA reception.
///
/// This structure manages a circular DMA transfer, allowing continuous
/// reception of data without losing bytes between reads. It uses both
/// half-transfer and complete-transfer interrupts to track available data.
///
/// # Example
///
/// ```no_run
/// use embassy_mcxa::dma::{DmaChannel, RingBuffer, TransferOptions};
///
/// static mut RX_BUF: [u8; 64] = [0; 64];
///
/// let dma_ch = DmaChannel::new(p.DMA_CH0);
/// let ring_buf = unsafe {
///     dma_ch.setup_circular_read(
///         uart_rx_addr,
///         &mut RX_BUF,
///     )
/// };
///
/// // Read data as it arrives
/// let mut buf = [0u8; 16];
/// let n = ring_buf.read(&mut buf).await?;
/// ```
pub struct RingBuffer<'a, W: Word> {
    channel: AnyChannel,
    /// Buffer pointer. We use NonNull instead of &mut because DMA acts like
    /// a separate thread writing to this buffer, and &mut claims exclusive
    /// access which the compiler could optimize incorrectly.
    buf: NonNull<[W]>,
    /// Buffer length cached for convenience
    buf_len: usize,
    /// Read position in the buffer (consumer side)
    read_pos: AtomicUsize,
    /// Phantom data to tie the lifetime to the original buffer
    _lt: PhantomData<&'a mut [W]>,
}

impl<'a, W: Word> RingBuffer<'a, W> {
    /// Create a new ring buffer for the given channel and buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The DMA channel has been configured for circular transfer
    /// - The buffer remains valid for the lifetime of the ring buffer
    /// - Only one RingBuffer exists per DMA channel at a time
    pub(crate) unsafe fn new(channel: AnyChannel, buf: &'a mut [W]) -> Self {
        let buf_len = buf.len();
        Self {
            channel,
            buf: NonNull::from(buf),
            buf_len,
            read_pos: AtomicUsize::new(0),
            _lt: PhantomData,
        }
    }

    /// Get a slice reference to the buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that DMA is not actively writing to the
    /// portion of the buffer being accessed, or that the access is
    /// appropriately synchronized.
    #[inline]
    unsafe fn buf_slice(&self) -> &[W] {
        unsafe { self.buf.as_ref() }
    }

    /// Get the current DMA write position in the buffer.
    ///
    /// This reads the current destination address from the DMA controller
    /// and calculates the buffer offset.
    fn dma_write_pos(&self) -> usize {
        let t = self.channel.tcd();
        let daddr = t.tcd_daddr().read().daddr().bits() as usize;
        let buf_start = self.buf.as_ptr() as *const W as usize;

        // Calculate offset from buffer start
        let offset = daddr.wrapping_sub(buf_start) / core::mem::size_of::<W>();

        // Ensure we're within bounds (DMA wraps around)
        offset % self.buf_len
    }

    /// Returns the number of bytes available to read.
    pub fn available(&self) -> usize {
        let write_pos = self.dma_write_pos();
        let read_pos = self.read_pos.load(Ordering::Acquire);

        if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            self.buf_len - read_pos + write_pos
        }
    }

    /// Check if the buffer has overrun (data was lost).
    ///
    /// This happens when DMA writes faster than the application reads.
    pub fn is_overrun(&self) -> bool {
        // In a true overrun, the DMA would have wrapped around and caught up
        // to our read position. We can detect this by checking if available()
        // equals the full buffer size (minus 1 to distinguish from empty).
        self.available() >= self.buf_len - 1
    }

    /// Read data from the ring buffer into the provided slice.
    ///
    /// Returns the number of elements read, which may be less than
    /// `dst.len()` if not enough data is available.
    ///
    /// This method does not block; use `read_async()` for async waiting.
    pub fn read_immediate(&self, dst: &mut [W]) -> usize {
        let write_pos = self.dma_write_pos();
        let read_pos = self.read_pos.load(Ordering::Acquire);

        // Calculate available bytes
        let available = if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            self.buf_len - read_pos + write_pos
        };

        let to_read = dst.len().min(available);
        if to_read == 0 {
            return 0;
        }

        // Safety: We only read from portions of the buffer that DMA has
        // already written to (between read_pos and write_pos).
        let buf = unsafe { self.buf_slice() };

        // Read data, handling wrap-around
        let first_chunk = (self.buf_len - read_pos).min(to_read);
        dst[..first_chunk].copy_from_slice(&buf[read_pos..read_pos + first_chunk]);

        if to_read > first_chunk {
            let second_chunk = to_read - first_chunk;
            dst[first_chunk..to_read].copy_from_slice(&buf[..second_chunk]);
        }

        // Update read position
        let new_read_pos = (read_pos + to_read) % self.buf_len;
        self.read_pos.store(new_read_pos, Ordering::Release);

        to_read
    }

    /// Read data from the ring buffer asynchronously.
    ///
    /// This waits until at least one byte is available, then reads as much
    /// as possible into the destination buffer.
    ///
    /// Returns the number of elements read.
    pub async fn read(&self, dst: &mut [W]) -> Result<usize, Error> {
        use core::future::poll_fn;

        if dst.is_empty() {
            return Ok(0);
        }

        poll_fn(|cx| {
            // Check for overrun
            if self.is_overrun() {
                return Poll::Ready(Err(Error::Overrun));
            }

            // Try to read immediately
            let n = self.read_immediate(dst);
            if n > 0 {
                return Poll::Ready(Ok(n));
            }

            // Register wakers for both half and complete interrupts
            let state = &STATES[self.channel.index()];
            state.waker.register(cx.waker());
            state.half_waker.register(cx.waker());

            // Check again after registering waker (avoid race)
            let n = self.read_immediate(dst);
            if n > 0 {
                return Poll::Ready(Ok(n));
            }

            Poll::Pending
        })
        .await
    }

    /// Clear the ring buffer, discarding all unread data.
    pub fn clear(&self) {
        let write_pos = self.dma_write_pos();
        self.read_pos.store(write_pos, Ordering::Release);
    }

    /// Stop the DMA transfer and consume the ring buffer.
    ///
    /// Returns any remaining unread data count.
    pub fn stop(mut self) -> usize {
        let res = self.teardown();
        drop(self);
        res
    }

    /// Stop the DMA transfer. Intended to be called by `stop()` or `Drop`.
    fn teardown(&mut self) -> usize {
        let available = self.available();

        // Disable the channel
        let t = self.channel.tcd();
        t.ch_csr().modify(|_, w| w.erq().disable());

        // Clear flags
        t.ch_int().write(|w| w.int().clear_bit_by_one());
        t.ch_csr().modify(|_, w| w.done().clear_bit_by_one());

        fence(Ordering::SeqCst);

        available
    }
}

impl<'a, W: Word> Drop for RingBuffer<'a, W> {
    fn drop(&mut self) {
        self.teardown();
    }
}

impl<C: Channel> DmaChannel<C> {
    /// Set up a circular DMA transfer for continuous peripheral-to-memory reception.
    ///
    /// This configures the DMA channel for circular operation with both half-transfer
    /// and complete-transfer interrupts enabled. The transfer runs continuously until
    /// stopped via [`RingBuffer::stop()`].
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address to read from
    /// * `buf` - Destination buffer (should be power-of-2 size for best efficiency)
    ///
    /// # Returns
    ///
    /// A [`RingBuffer`] that can be used to read received data.
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the lifetime of the returned RingBuffer.
    /// - The peripheral address must be valid for reads.
    /// - The peripheral's DMA request must be configured to trigger this channel.
    pub unsafe fn setup_circular_read<'a, W: Word>(&self, peri_addr: *const W, buf: &'a mut [W]) -> RingBuffer<'a, W> {
        unsafe {
            assert!(!buf.is_empty());
            assert!(buf.len() <= 0x7fff);
            // For circular mode, buffer size should ideally be power of 2
            // but we don't enforce it

            let size = W::size();
            let byte_size = size.bytes();

            let t = self.tcd();

            // Reset channel state
            Self::reset_channel_state(t);

            // Source: peripheral register, fixed
            Self::set_source_ptr(t, peri_addr);
            Self::set_source_fixed(t);

            // Destination: memory buffer, incrementing
            Self::set_dest_ptr(t, buf.as_mut_ptr());
            Self::set_dest_increment(t, size);

            // Transfer attributes
            Self::set_even_transfer_size(t, size);

            // Minor loop: transfer one word per request
            Self::set_minor_loop_ct_no_offsets(t, byte_size as u32);

            // Major loop count = buffer size
            let count = buf.len() as u16;
            Self::set_major_loop_ct_elinkno(t, count);

            // After major loop: reset destination to buffer start (circular)
            let buf_bytes = (buf.len() * byte_size) as i32;
            t.tcd_slast_sda().write(|w| w.slast_sda().bits(0)); // Source doesn't change
            t.tcd_dlast_sga().write(|w| w.dlast_sga().bits((-buf_bytes) as u32));

            // Control/status: enable both half and complete interrupts, NO DREQ (continuous)
            t.tcd_csr().write(|w| {
                w.intmajor()
                    .enable()
                    .inthalf()
                    .enable()
                    .dreq()
                    .channel_not_affected() // Don't clear ERQ on complete (circular)
                    .esg()
                    .normal_format()
                    .majorelink()
                    .disable()
                    .eeop()
                    .disable()
                    .esda()
                    .disable()
                    .bwc()
                    .no_stall()
            });

            cortex_m::asm::dsb();

            // Enable the channel request
            t.ch_csr().modify(|_, w| w.erq().enable());

            // Enable NVIC interrupt for this channel so async wakeups work
            self.enable_interrupt();

            RingBuffer::new(self.as_any(), buf)
        }
    }
}

// ============================================================================
// Scatter-Gather Builder
// ============================================================================

/// Maximum number of TCDs in a scatter-gather chain.
pub const MAX_SCATTER_GATHER_TCDS: usize = 16;

/// A builder for constructing scatter-gather DMA transfer chains.
///
/// This provides a type-safe way to build TCD chains for scatter-gather
/// transfers without manual TCD manipulation.
///
/// # Example
///
/// ```no_run
/// use embassy_mcxa::dma::{DmaChannel, ScatterGatherBuilder};
///
/// let mut builder = ScatterGatherBuilder::<u32>::new();
///
/// // Add transfer segments
/// builder.add_transfer(&src1, &mut dst1);
/// builder.add_transfer(&src2, &mut dst2);
/// builder.add_transfer(&src3, &mut dst3);
///
/// // Build and execute
/// let transfer = unsafe { builder.build(&dma_ch).unwrap() };
/// transfer.await;
/// ```
pub struct ScatterGatherBuilder<'a, W: Word> {
    /// TCD pool (must be 32-byte aligned)
    tcds: [Tcd; MAX_SCATTER_GATHER_TCDS],
    /// Number of TCDs configured
    count: usize,
    /// Phantom marker for word type
    _phantom: core::marker::PhantomData<W>,

    _plt: core::marker::PhantomData<&'a mut W>,
}

impl<'a, W: Word> ScatterGatherBuilder<'a, W> {
    /// Create a new scatter-gather builder.
    pub fn new() -> Self {
        ScatterGatherBuilder {
            tcds: [Tcd::default(); MAX_SCATTER_GATHER_TCDS],
            count: 0,
            _phantom: core::marker::PhantomData,
            _plt: core::marker::PhantomData,
        }
    }

    /// Add a memory-to-memory transfer segment to the chain.
    ///
    /// # Arguments
    ///
    /// * `src` - Source buffer for this segment
    /// * `dst` - Destination buffer for this segment
    ///
    /// # Panics
    ///
    /// Panics if the maximum number of segments (16) is exceeded.
    pub fn add_transfer<'b: 'a>(&mut self, src: &'b [W], dst: &'b mut [W]) -> &mut Self {
        assert!(self.count < MAX_SCATTER_GATHER_TCDS, "Too many scatter-gather segments");
        assert!(!src.is_empty());
        assert!(dst.len() >= src.len());

        let size = W::size();
        let byte_size = size.bytes();
        let hw_size = size.to_hw_size();
        let nbytes = (src.len() * byte_size) as u32;

        // Build the TCD for this segment
        self.tcds[self.count] = Tcd {
            saddr: src.as_ptr() as u32,
            soff: byte_size as i16,
            attr: ((hw_size as u16) << 8) | (hw_size as u16), // SSIZE | DSIZE
            nbytes,
            slast: 0,
            daddr: dst.as_mut_ptr() as u32,
            doff: byte_size as i16,
            citer: 1,
            dlast_sga: 0, // Will be filled in by build()
            csr: 0x0002,  // INTMAJOR only (ESG will be set for non-last TCDs)
            biter: 1,
        };

        self.count += 1;
        self
    }

    /// Get the number of transfer segments added.
    pub fn segment_count(&self) -> usize {
        self.count
    }

    /// Build the scatter-gather chain and start the transfer.
    ///
    /// # Arguments
    ///
    /// * `channel` - The DMA channel to use for the transfer
    ///
    /// # Returns
    ///
    /// A `Transfer` future that completes when the entire chain has executed.
    pub fn build<C: Channel>(&mut self, channel: &DmaChannel<C>) -> Result<Transfer<'a>, Error> {
        if self.count == 0 {
            return Err(Error::Configuration);
        }

        // Link TCDs together
        //
        // CSR bit definitions:
        // - START = bit 0 = 0x0001 (triggers transfer when set)
        // - INTMAJOR = bit 1 = 0x0002 (interrupt on major loop complete)
        // - ESG = bit 4 = 0x0010 (enable scatter-gather, loads next TCD on complete)
        //
        // When hardware loads a TCD via scatter-gather (ESG), it copies the TCD's
        // CSR directly into the hardware register. If START is not set in that CSR,
        // the hardware will NOT auto-execute the loaded TCD.
        //
        // Strategy:
        // - First TCD: ESG | INTMAJOR (no START - we add it manually after loading)
        // - Middle TCDs: ESG | INTMAJOR | START (auto-execute when loaded via S/G)
        // - Last TCD: INTMAJOR | START (auto-execute, no further linking)
        for i in 0..self.count {
            let is_first = i == 0;
            let is_last = i == self.count - 1;

            if is_first {
                if is_last {
                    // Only one TCD - no ESG, no START (we add START manually)
                    self.tcds[i].dlast_sga = 0;
                    self.tcds[i].csr = 0x0002; // INTMAJOR only
                } else {
                    // First of multiple - ESG to link, no START (we add START manually)
                    self.tcds[i].dlast_sga = &self.tcds[i + 1] as *const Tcd as i32;
                    self.tcds[i].csr = 0x0012; // ESG | INTMAJOR
                }
            } else if is_last {
                // Last TCD (not first) - no ESG, but START so it auto-executes
                self.tcds[i].dlast_sga = 0;
                self.tcds[i].csr = 0x0003; // INTMAJOR | START
            } else {
                // Middle TCD - ESG to link, and START so it auto-executes
                self.tcds[i].dlast_sga = &self.tcds[i + 1] as *const Tcd as i32;
                self.tcds[i].csr = 0x0013; // ESG | INTMAJOR | START
            }
        }

        let t = channel.tcd();

        // Reset channel state - clear DONE, disable requests, clear errors
        // This ensures the channel is in a clean state before loading the TCD
        DmaChannel::<C>::reset_channel_state(t);

        // Memory barrier to ensure channel state is reset before loading TCD
        cortex_m::asm::dsb();

        // Load first TCD into hardware
        unsafe {
            channel.load_tcd(&self.tcds[0]);
        }

        // Memory barrier before setting START
        cortex_m::asm::dsb();

        // Start the transfer
        t.tcd_csr().modify(|_, w| w.start().channel_started());

        Ok(Transfer::new(channel.as_any()))
    }

    /// Reset the builder for reuse.
    pub fn clear(&mut self) {
        self.count = 0;
    }
}

impl<W: Word> Default for ScatterGatherBuilder<'_, W> {
    fn default() -> Self {
        Self::new()
    }
}

/// A completed scatter-gather transfer result.
///
/// This type is returned after a scatter-gather transfer completes,
/// providing access to any error information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScatterGatherResult {
    /// Number of segments successfully transferred
    pub segments_completed: usize,
    /// Error if any occurred
    pub error: Option<Error>,
}

// ============================================================================
// Interrupt Handler
// ============================================================================

/// Interrupt handler helper.
///
/// Call this from your interrupt handler to clear the interrupt flag and wake the waker.
/// This handles both half-transfer and complete-transfer interrupts.
///
/// # Safety
/// Must be called from the correct DMA channel interrupt context.
pub unsafe fn on_interrupt(ch_index: usize) {
    crate::perf_counters::incr_interrupt_edma0();
    unsafe {
        let p = pac::Peripherals::steal();
        let edma = &p.edma_0_tcd0;
        let t = edma.tcd(ch_index);

        // Read TCD CSR to determine interrupt source
        let csr = t.tcd_csr().read();

        // Check if this is a half-transfer interrupt
        // INTHALF is set and we're at or past the half-way point
        if csr.inthalf().bit_is_set() {
            let biter = t.tcd_biter_elinkno().read().biter().bits();
            let citer = t.tcd_citer_elinkno().read().citer().bits();
            let half_point = biter / 2;

            if citer <= half_point && citer > 0 {
                // Half-transfer interrupt - wake half_waker
                crate::perf_counters::incr_interrupt_edma0_wake();
                half_waker(ch_index).wake();
            }
        }

        // Clear INT flag
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // If DONE is set, this is a complete-transfer interrupt
        // Only wake the full-transfer waker when the transfer is actually complete
        if t.ch_csr().read().done().bit_is_set() {
            crate::perf_counters::incr_interrupt_edma0_wake();
            waker(ch_index).wake();
        }
    }
}

// ============================================================================
// Type-level Interrupt Handlers
// ============================================================================

/// Macro to generate DMA channel interrupt handlers.
macro_rules! impl_dma_interrupt_handler {
    ($irq:ident, $ch:expr) => {
        #[interrupt]
        fn $irq() {
            unsafe {
                on_interrupt($ch);
            }
        }
    };
}

use crate::pac::interrupt;

impl_dma_interrupt_handler!(DMA_CH0, 0);
impl_dma_interrupt_handler!(DMA_CH1, 1);
impl_dma_interrupt_handler!(DMA_CH2, 2);
impl_dma_interrupt_handler!(DMA_CH3, 3);
impl_dma_interrupt_handler!(DMA_CH4, 4);
impl_dma_interrupt_handler!(DMA_CH5, 5);
impl_dma_interrupt_handler!(DMA_CH6, 6);
impl_dma_interrupt_handler!(DMA_CH7, 7);
