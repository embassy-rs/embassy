//! DMA driver.
//!
//! This module provides a typed channel abstraction over the EDMA_0_TCD0 array
//! and helpers for configuring the channel MUX. The driver supports
//! higher-level async transfer APIs.
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
//! | [`DmaChannel::write_to_peripheral()`] | Memory-to-peripheral (TX) |
//! | [`DmaChannel::read_from_peripheral()`] | Peripheral-to-memory (RX) |
//!
//! These return a [`Transfer`] future that can be `.await`ed:
//!
//! ```no_run
//! # use embassy_mcxa::dma::{DmaChannel, TransferOptions};
//! let dma_ch = DmaChannel::new(p.DMA_CH0);
//! # let src = [0u32; 4];
//! # let mut dst = [0u32; 4];
//! dma_ch.mem_to_mem(&src, &mut dst, TransferOptions::default()).await;
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
//! let transfer = builder.build(&dma_ch).unwrap();
//! transfer.await;
//! ```

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering, fence};
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::Gate;
use crate::dma::sealed::SealedChannel;
use crate::pac::dma::vals::Halt;
use crate::pac::edma_0_tcd::regs::{TcdAttr, TcdBiterElinkno, TcdCiterElinkno, TcdCsr};
use crate::pac::edma_0_tcd::vals::{Bwc, Dpa, Dreq, Ecp, Esg, Size, Start};
use crate::pac::{self, Interrupt};
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
        pac::DMA0.mp_csr().modify(|w| {
            w.set_edbg(true);
            w.set_erca(true);
            w.set_halt(Halt::NORMAL_OPERATION);
            w.set_gclc(true);
            w.set_gmrc(true);
        });
    }
}

/// DMA transfer priority.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Priority {
    /// Highest priority.
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
    P4 = 4,
    P5 = 5,
    P6 = 6,
    /// Lowest priority.
    #[default]
    P7 = 7,
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
    pub const fn to_hw_size(self) -> Size {
        match self {
            WordSize::OneByte => Size::EIGHT_BIT,
            WordSize::TwoBytes => Size::SIXTEEN_BIT,
            WordSize::FourBytes => Size::THIRTYTWO_BIT,
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

/// Trait for word-sizes that are supported.
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Enable interrupt on half transfer complete.
    pub half_transfer_interrupt: bool,
    /// Enable interrupt on transfer complete.
    pub complete_transfer_interrupt: bool,
    /// Transfer priority
    pub priority: Priority,
}

/// Typical variants of [TransferOptions] to be used as shorthands.
pub mod transfer_opts {
    use crate::dma::{Priority, TransferOptions};

    /// Short-hand to specify that no options should be configured.
    pub struct NoInterrupt;

    /// Short-hand to specify that only the complete transfer interrupt should be triggered.
    pub struct EnableComplete;

    impl From<NoInterrupt> for TransferOptions {
        fn from(_value: NoInterrupt) -> Self {
            TransferOptions {
                half_transfer_interrupt: false,
                complete_transfer_interrupt: false,
                priority: Priority::default(),
            }
        }
    }

    impl From<EnableComplete> for TransferOptions {
        fn from(_value: EnableComplete) -> Self {
            TransferOptions {
                half_transfer_interrupt: false,
                complete_transfer_interrupt: true,
                priority: Priority::default(),
            }
        }
    }
}

/// General DMA error types.
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

/// An error that can occur if the parameters passed were invalid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidParameters;

/// Maximum bytes per DMA transfer (eDMA4 CITER/BITER are 15-bit fields).
///
/// This is a hardware limitation of the eDMA4 controller. Transfers larger
/// than this must be split into multiple DMA operations.
pub const DMA_MAX_TRANSFER_SIZE: usize = 0x7FFF;

/// DMA request sources
///
/// (from MCXA266 reference manual PDF attachment "DMA_Configuration.xml")
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub(crate) enum DmaRequest {
    WUU0WakeUpEvent = 1,
    CAN0 = 2,
    LPI2C2Rx = 3,
    LPI2C2Tx = 4,
    LPI2C3Rx = 5,
    LPI2C3Tx = 6,
    I3C0Rx = 7,
    I3C0Tx = 8,
    LPI2C0Rx = 11,
    LPI2C0Tx = 12,
    LPI2C1Rx = 13,
    LPI2C1Tx = 14,
    LPSPI0Rx = 15,
    LPSPI0Tx = 16,
    LPSPI1Rx = 17,
    LPSPI1Tx = 18,
    LPUART0Rx = 21,
    LPUART0Tx = 22,
    LPUART1Rx = 23,
    LPUART1Tx = 24,
    LPUART2Rx = 25,
    LPUART2Tx = 26,
    LPUART3Rx = 27,
    LPUART3Tx = 28,
    LPUART4Rx = 29,
    LPUART4Tx = 30,
    Ctimer0M0 = 31,
    Ctimer0M1 = 32,
    Ctimer1M0 = 33,
    Ctimer1M1 = 34,
    Ctimer2M0 = 35,
    Ctimer2M1 = 36,
    Ctimer3M0 = 37,
    Ctimer3M1 = 38,
    Ctimer4M0 = 39,
    Ctimer4M1 = 40,
    FlexPWM0Capt0 = 41,
    FlexPWM0Capt1 = 42,
    FlexPWM0Capt2 = 43,
    FlexPWM0Capt3 = 44,
    FlexPWM0Val0 = 45,
    FlexPWM0Val1 = 46,
    FlexPWM0Val2 = 47,
    FlexPWM0Val3 = 48,
    LPTMR0CounterMatchEvent = 49,
    ADC0FifoRequest = 51,
    ADC1FifoRequest = 52,
    CMP0 = 53,
    CMP1 = 54,
    CMP2 = 55,
    DAC0FifoRequest = 56,
    GPIO0Pin = 60,
    GPIO1Pin = 61,
    GPIO2Pin = 62,
    GPIO3Pin = 63,
    GPIO4Pin = 64,
    QDC0 = 65,
    QDC1 = 66,
    FlexIO0SR0 = 71,
    FlexIO0SR1 = 72,
    FlexIO0SR2 = 73,
    FlexIO0SR3 = 74,
    FlexPWM1ReqCapt0 = 79,
    FlexPWM1ReqCapt1 = 80,
    FlexPWM1ReqCapt2 = 81,
    FlexPWM1ReqCapt3 = 82,
    FlexPWM1ReqVal0 = 83,
    FlexPWM1ReqVal1 = 84,
    FlexPWM1ReqVal2 = 85,
    FlexPWM1ReqVal3 = 86,
    CAN1 = 87,
    LPUART5Rx = 102,
    LPUART5Tx = 103,
    MAU0MAU = 115,
    SGI0ReqIdat = 119,
    SGI0ReqOdat = 120,
    ADC2FifoRequest = 123,
    ADC3FifoRequest = 124,
}

impl DmaRequest {
    /// Convert enumerated value into a raw integer
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// Convert a raw integer into an enumerated value
    ///
    /// ## SAFETY
    ///
    /// The given number MUST be one of the defined variant, e.g. a number
    /// derived from [`Self::number()`], otherwise it is immediate undefined behavior.
    pub unsafe fn from_number_unchecked(num: u8) -> Self {
        unsafe { core::mem::transmute(num) }
    }
}

mod sealed {
    /// Sealed trait for DMA channels.
    pub trait SealedChannel {
        /// Zero-based channel index into the TCD array.
        fn index(&self) -> usize;

        /// Interrupt vector for this channel.
        fn interrupt(&self) -> crate::interrupt::Interrupt;
    }
}

/// Marker trait implemented by HAL peripheral tokens that map to a DMA0
/// channel backed by one EDMA_0_TCD0 TCD slot.
#[allow(private_bounds)]
pub trait Channel: sealed::SealedChannel + PeripheralType + Into<AnyChannel> + 'static {}

/// Type-erased DMA channel peripheral.
///
/// This allows storing DMA channels in a uniform way regardless of their
/// concrete type, useful for async transfer futures and runtime channel selection.
///
/// ```no_run
/// let anychannel: Peri<'static, AnyChannel> = p.DMA_CH0.into();
/// DmaChannel::new(anychannel);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AnyChannel {
    index: usize,
    interrupt: Interrupt,
}

impl PeripheralType for AnyChannel {}
impl sealed::SealedChannel for AnyChannel {
    fn index(&self) -> usize {
        self.index
    }

    fn interrupt(&self) -> Interrupt {
        self.interrupt
    }
}

impl Channel for AnyChannel {}

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
        impl Channel for crate::peripherals::$peri {}

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

/// Parameters used to configure a 'typical' DMA transfer in [DmaChannel::setup_typical].
struct DmaTransferParameters<W> {
    /// Number of words that should be transferred.
    count: usize,
    /// Source pointer. If incrementing, the backing memory region should be at least as large as `count`.
    src_ptr: *const W,
    /// Destination pointer. If incrementing, the backing memory region should be at least as large as `count`.
    dst_ptr: *mut W,
    /// Whether the source pointer should be incremented.
    src_incr: bool,
    /// Whether the destination pointer should be incremented.
    dst_incr: bool,
    /// Perform circular DMA.
    ///
    /// After each loop, will reset the current pointer to the starting addresses, both for src and dest.
    circular: bool,
    /// Public facing transfer options that might be relevant.
    options: TransferOptions,
}

/// DMA channel driver.
pub struct DmaChannel<'a> {
    channel: Peri<'a, AnyChannel>,
}

impl<'a> DmaChannel<'a> {
    /// Wrap a DMA channel token (takes ownership of the Peri wrapper).
    ///
    /// Note: DMA is initialized during `hal::init()` via `dma::init()`.
    #[inline]
    pub fn new<C: Channel>(channel: embassy_hal_internal::Peri<'a, C>) -> Self {
        unsafe {
            cortex_m::peripheral::NVIC::unmask(channel.interrupt());
        }
        Self {
            channel: channel.into(),
        }
    }
}

impl DmaChannel<'_> {
    /// Reborrow the DmaChannel with a shorter lifetime.
    pub fn reborrow(&mut self) -> DmaChannel<'_> {
        DmaChannel {
            channel: self.channel.reborrow(),
        }
    }

    /// Channel index in the EDMA_0_TCD0 array.
    #[inline]
    pub(crate) fn index(&self) -> usize {
        self.channel.index()
    }

    /// Return a reference to the underlying TCD register block.
    #[inline]
    pub(crate) fn tcd(&self) -> pac::edma_0_tcd::Tcd {
        // Safety: MCXA276 has a single eDMA instance
        pac::EDMA_0_TCD0.tcd(self.channel.index())
    }

    /// set a manual callback to be called AFTER the DMA interrupt is processed. Will be called in the DMA interrupt
    /// context.
    ///
    /// SAFETY: This must only be called on an owned DmaChannel, as there is only a single
    /// callback slot, and calling this will invalidate any previously set callbacks.
    pub(crate) unsafe fn set_callback(&mut self, f: fn()) {
        // See https://doc.rust-lang.org/std/primitive.fn.html#casting-to-and-from-integers
        let cb = f as *mut ();
        CALLBACKS[self.index()].store(cb, Ordering::Release);
    }

    /// Unset the callback, causing no method to be called after DMA completion.
    ///
    /// SAFETY: This must only be called on an owned DmaChannel, as there is only a single
    /// callback slot, and calling this will invalidate any previously set callbacks.
    pub(crate) unsafe fn clear_callback(&mut self) {
        CALLBACKS[self.index()].store(core::ptr::null_mut(), Ordering::Release);
    }

    /// Access TCD DADDR field
    pub(crate) fn daddr(&self) -> u32 {
        self.tcd().tcd_daddr().read().daddr()
    }

    fn clear_tcd(t: &pac::edma_0_tcd::Tcd) {
        // Full TCD reset following NXP SDK pattern (EDMA_TcdResetExt).
        // Reset ALL TCD registers to 0 to clear any stale configuration from
        // previous transfers. This is critical when reusing a channel.
        t.tcd_saddr().write(|w| w.set_saddr(0));
        t.tcd_soff().write(|w| w.set_soff(0));
        t.tcd_attr().write(|w| *w = TcdAttr(0));
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(0));
        t.tcd_slast_sda().write(|w| w.set_slast_sda(0));
        t.tcd_daddr().write(|w| w.set_daddr(0));
        t.tcd_doff().write(|w| w.set_doff(0));
        t.tcd_citer_elinkno().write(|w| *w = TcdCiterElinkno(0));
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
        t.tcd_csr().write(|w| *w = TcdCsr(0)); // Clear CSR completly
        t.tcd_biter_elinkno().write(|w| *w = TcdBiterElinkno(0));
    }

    #[inline]
    fn set_major_loop_ct_elinkno(t: &pac::edma_0_tcd::Tcd, count: u16) {
        t.tcd_biter_elinkno().write(|w| w.set_biter(count));
        t.tcd_citer_elinkno().write(|w| w.set_citer(count));
    }

    #[inline]
    fn set_minor_loop_ct_no_offsets(t: &pac::edma_0_tcd::Tcd, count: u32) {
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(count));
    }

    #[inline]
    fn set_no_final_adjustments(t: &pac::edma_0_tcd::Tcd) {
        // No source/dest adjustment after major loop
        t.tcd_slast_sda().write(|w| w.set_slast_sda(0));
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
    }

    #[inline]
    fn set_source_ptr<T>(t: &pac::edma_0_tcd::Tcd, p: *const T) {
        t.tcd_saddr().write(|w| w.set_saddr(p as u32));
    }

    #[inline]
    fn set_source_increment(t: &pac::edma_0_tcd::Tcd, sz: WordSize) {
        t.tcd_soff().write(|w| w.set_soff(sz.bytes() as u16));
    }

    #[inline]
    fn set_source_fixed(t: &pac::edma_0_tcd::Tcd) {
        t.tcd_soff().write(|w| w.set_soff(0));
    }

    #[inline]
    fn set_dest_ptr<T>(t: &pac::edma_0_tcd::Tcd, p: *mut T) {
        t.tcd_daddr().write(|w| w.set_daddr(p as u32));
    }

    #[inline]
    fn set_dest_increment(t: &pac::edma_0_tcd::Tcd, sz: WordSize) {
        t.tcd_doff().write(|w| w.set_doff(sz.bytes() as u16));
    }

    #[inline]
    fn set_dest_fixed(t: &pac::edma_0_tcd::Tcd) {
        t.tcd_doff().write(|w| w.set_doff(0));
    }

    #[inline]
    fn set_fixed_priority(t: &pac::edma_0_tcd::Tcd, p: Priority) {
        t.ch_pri().write(|w| {
            w.set_dpa(Dpa::SUSPEND);
            w.set_ecp(Ecp::SUSPEND);
            w.set_apl(p as u8);
        });
    }

    #[inline]
    fn set_even_transfer_size(t: &pac::edma_0_tcd::Tcd, sz: WordSize) {
        let hw_size = sz.to_hw_size();
        t.tcd_attr().write(|w| {
            w.set_ssize(hw_size);
            w.set_dsize(hw_size);
        });
    }

    #[inline]
    fn reset_channel_state(t: &pac::edma_0_tcd::Tcd) {
        // CSR: Resets to all zeroes (disabled), "done" is cleared by writing 1
        t.ch_csr().write(|w| w.set_done(true));
        // ES: Resets to all zeroes (disabled), "err" is cleared by writing 1
        t.ch_es().write(|w| w.set_err(true));
        // INT: Resets to all zeroes (disabled), "int" is cleared by writing 1
        t.ch_int().write(|w| w.set_int(true));
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
    #[allow(unused)]
    pub(crate) unsafe fn start_transfer(&mut self) -> Transfer<'_> {
        // Clear any previous DONE/INT flags
        let t = self.tcd();
        t.ch_csr().modify(|w| w.set_done(true));
        t.ch_int().write(|w| w.set_int(true));

        // Enable the channel request
        t.ch_csr().modify(|w| {
            w.set_erq(true);
            w.set_earq(true);
        });

        Transfer::new(self.reborrow())
    }

    /// Setup a typical DMA transfer.
    ///
    /// # Safety
    ///
    /// Requires that the source/destination buffers remain valid for the duration
    /// of the transfer.
    unsafe fn setup_typical<W: Word>(&self, params: DmaTransferParameters<W>) -> Result<(), InvalidParameters> {
        let size = W::size();
        let byte_count = (params.count * size.bytes()) as u32;

        let t = self.tcd();

        // Reset channel state - clear DONE, disable requests, clear errors
        Self::reset_channel_state(&t);

        // Memory & compiler barrier to ensure channel state is fully reset before touching TCD
        fence(Ordering::Release);

        Self::clear_tcd(&t);

        // Note: Priority is managed by round-robin arbitration (set in init())
        // Per-channel priority can be configured via ch_pri() if needed

        // Now configure the new transfer

        // Source address and increment
        Self::set_source_ptr(&t, params.src_ptr);

        if params.src_incr {
            Self::set_source_increment(&t, size);
        } else {
            Self::set_source_fixed(&t);
        }

        // Destination address and increment
        Self::set_dest_ptr(&t, params.dst_ptr);

        if params.dst_incr {
            Self::set_dest_increment(&t, size);
        } else {
            Self::set_dest_fixed(&t);
        }

        // Transfer attributes (size)
        Self::set_even_transfer_size(&t, size);

        // Minor loop: transfer all bytes in one minor loop
        Self::set_minor_loop_ct_no_offsets(&t, byte_count);

        // No source/dest adjustment after major loop
        Self::set_no_final_adjustments(&t);

        // Major loop count = 1 (single major loop)
        // Write BITER first, then CITER (CITER must match BITER at start)
        Self::set_major_loop_ct_elinkno(&t, 1);

        // Configure channel to be interruptable, to interrupt, with a set priority.
        Self::set_fixed_priority(&t, params.options.priority);

        if params.circular {
            let byte_diff = -(byte_count as i32); // Decrement the address pointers (if incrementing & not fixed).
            let byte_diff_reg = byte_diff as u32; // Cast as u32 so that it can be stored in the register.

            t.tcd_slast_sda()
                .write(|w| w.set_slast_sda(params.src_incr.then_some(byte_diff_reg).unwrap_or(0)));
            t.tcd_dlast_sga()
                .write(|w| w.set_dlast_sga(params.dst_incr.then_some(byte_diff_reg).unwrap_or(0)));
        }

        // Memory & compiler barrier before setting START
        fence(Ordering::Release);

        // Control/status: interrupt on major complete, start
        // Write this last after all other TCD registers are configured
        t.tcd_csr().write(|w| {
            w.set_intmajor(params.options.complete_transfer_interrupt);
            w.set_inthalf(params.options.half_transfer_interrupt);
            w.set_start(Start::CHANNEL_STARTED); // Start the channel
            w.set_esg(Esg::NORMAL_FORMAT);
            w.set_majorelink(false);
            w.set_eeop(false);
            w.set_esda(false);
            w.set_bwc(Bwc::NO_STALL);

            w.set_dreq(if params.circular {
                Dreq::CHANNEL_NOT_AFFECTED // Don't clear ERQ on complete (circular)
            } else {
                Dreq::ERQ_FIELD_CLEAR // Auto-disable request after major loop
            });
        });

        Ok(())
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
        &mut self,
        src: &[W],
        dst: &mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        if src.is_empty() || src.len() > dst.len() || src.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_typical(DmaTransferParameters {
                src_ptr: src.as_ptr(),
                dst_ptr: dst.as_mut_ptr(),
                count: src.len(),
                src_incr: true,
                dst_incr: true,
                circular: false,
                options,
            })?
        };

        Ok(Transfer::new(self.reborrow()))
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
    pub fn memset<W: Word>(
        &mut self,
        pattern: &W,
        dst: &mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        if dst.is_empty() || dst.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_typical(DmaTransferParameters {
                src_ptr: pattern,
                dst_ptr: dst.as_mut_ptr(),
                count: dst.len(),
                src_incr: false,
                dst_incr: true,
                circular: false,
                options,
            })?
        };

        Ok(Transfer::new(self.reborrow()))
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
        &mut self,
        buf: &[W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        unsafe { self.setup_write_to_peripheral(buf, peri_addr, options)? };
        Ok(Transfer::new(self.reborrow()))
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
        &mut self,
        peri_addr: *const W,
        buf: &mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        unsafe { self.setup_read_from_peripheral(peri_addr, buf, options)? };
        Ok(Transfer::new(self.reborrow()))
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
    /// * `peri_addr` - Peripheral register address
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub(crate) unsafe fn setup_write_zeros_to_peripheral<W: Word>(
        &self,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if count > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        // Static mut so that this is allocated in RAM.
        static mut DUMMY: u32 = 0;

        unsafe {
            self.setup_typical(DmaTransferParameters {
                src_ptr: core::ptr::addr_of_mut!(DUMMY) as *const W,
                dst_ptr: peri_addr,
                count,
                src_incr: false,
                dst_incr: false,
                circular: false,
                options,
            })
        }
    }

    /// Produce the number of bytes transferred at the time of calling
    /// this function.
    pub fn transferred_bytes(&self) -> usize {
        critical_section::with(|_| {
            let t = self.tcd();
            let biter = t.tcd_biter_elinkno().read().biter() as usize;
            let citer = t.tcd_citer_elinkno().read().citer() as usize;
            let minor = t.tcd_nbytes_mloffno().read().nbytes() as usize;
            (biter - citer) * minor
        })
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
    pub(crate) unsafe fn setup_write_to_peripheral<W: Word>(
        &self,
        buf: &[W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_typical(DmaTransferParameters {
                src_ptr: buf.as_ptr(),
                dst_ptr: peri_addr,
                count: buf.len(),
                src_incr: true,
                dst_incr: false,
                circular: false,
                options,
            })
        }
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
    pub(crate) unsafe fn setup_read_from_peripheral<W: Word>(
        &self,
        peri_addr: *const W,
        buf: &mut [W],
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_typical(DmaTransferParameters {
                src_ptr: peri_addr,
                dst_ptr: buf.as_mut_ptr(),
                count: buf.len(),
                src_incr: false,
                dst_incr: true,
                circular: false,
                options,
            })
        }
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
    /// unsafe {
    ///     channel.set_request_source(Lpuart2RxRequest::REQUEST_NUMBER);
    /// }
    /// ```
    #[inline]
    pub(crate) unsafe fn set_request_source(&self, source: DmaRequest) {
        // Two-step write per NXP SDK: clear to 0, then set actual source.
        self.tcd().ch_mux().write(|w| w.set_src(0));
        cortex_m::asm::dsb(); // Ensure the clear completes before setting new source
        self.tcd().ch_mux().write(|w| w.set_src(source.number()));
    }

    /// Enable hardware requests for this channel (ERQ=1).
    ///
    /// # Safety
    ///
    /// The channel must be properly configured before enabling requests.
    pub(crate) unsafe fn enable_request(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|w| {
            w.set_erq(true);
            w.set_earq(true);
        });
    }

    /// Disable hardware requests for this channel (ERQ=0).
    ///
    /// # Safety
    ///
    /// Disabling requests on an active transfer may leave the transfer incomplete.
    pub(crate) unsafe fn disable_request(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|w| {
            w.set_erq(false);
            w.set_earq(false);
        });
    }

    /// Return true if the channel's DONE flag is set.
    pub(crate) fn is_done(&self) -> bool {
        let t = self.tcd();
        t.ch_csr().read().done()
    }

    /// Clear the DONE flag for this channel.
    ///
    /// Uses modify to preserve other bits (especially ERQ) unlike write
    /// which would clear ERQ and halt an active transfer.
    ///
    /// # Safety
    ///
    /// Clearing DONE while a transfer is in progress may cause undefined behavior.
    pub(crate) unsafe fn clear_done(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|w| w.set_done(true));
    }

    /// Clear the channel interrupt flag (CH_INT.INT).
    ///
    /// # Safety
    ///
    /// Must be called from the correct interrupt context or with interrupts disabled.
    pub(crate) unsafe fn clear_interrupt(&self) {
        let t = self.tcd();
        t.ch_int().write(|w| w.set_int(true));
    }

    /// Trigger a software start for this channel.
    ///
    /// # Safety
    ///
    /// The channel must be properly configured with a valid TCD before triggering.
    #[allow(unused)]
    pub(crate) unsafe fn trigger_start(&self) {
        let t = self.tcd();
        t.tcd_csr().modify(|w| w.set_start(Start::CHANNEL_STARTED));
    }

    /// Get the waker for this channel
    pub(crate) fn waker(&self) -> &'static AtomicWaker {
        &STATES[self.channel.index()].waker
    }

    /// Enable the interrupt for this channel in the NVIC.
    pub(crate) fn enable_interrupt(&self) {
        unsafe {
            cortex_m::peripheral::NVIC::unmask(self.channel.interrupt());
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
    #[allow(unused)]
    pub(crate) unsafe fn set_major_link(&self, link_ch: usize) {
        let t = self.tcd();
        t.tcd_csr().modify(|w| {
            w.set_majorelink(true);
            w.set_majorlinkch(link_ch as u8)
        });
    }

    /// Disable Major Loop Linking.
    ///
    /// Removes any major loop channel linking previously configured.
    ///
    /// # Safety
    ///
    /// The caller must ensure this doesn't disrupt an active transfer that
    /// depends on the linking.
    #[allow(unused)]
    pub(crate) unsafe fn clear_major_link(&self) {
        let t = self.tcd();
        t.tcd_csr().modify(|w| w.set_majorelink(false));
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
    #[allow(unused)]
    pub(crate) unsafe fn set_minor_link(&self, link_ch: usize) {
        let t = self.tcd();

        // Read current CITER (assuming ELINKNO format initially)
        let current_citer = t.tcd_citer_elinkno().read().citer();
        let current_biter = t.tcd_biter_elinkno().read().biter();

        // Write back using ELINKYES format
        t.tcd_citer_elinkyes().write(|w| {
            w.set_citer(current_citer);
            w.set_elink(true);
            w.set_linkch(link_ch as u8);
        });

        t.tcd_biter_elinkyes().write(|w| {
            w.set_biter(current_biter);
            w.set_elink(true);
            w.set_linkch(link_ch as u8);
        });
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
    #[allow(unused)]
    pub(crate) unsafe fn clear_minor_link(&self) {
        let t = self.tcd();

        // Read current CITER (could be in either format, but we only need the count)
        // Note: In ELINKYES format, citer is 9 bits; in ELINKNO, it's 15 bits.
        // We read from ELINKNO which will give us the combined value.
        let current_citer = t.tcd_citer_elinkno().read().citer();
        let current_biter = t.tcd_biter_elinkno().read().biter();

        // Write back using ELINKNO format (disabling link)
        t.tcd_citer_elinkno().write(|w| {
            w.set_citer(current_citer);
            w.set_elink(false);
        });

        t.tcd_biter_elinkno().write(|w| {
            w.set_biter(current_biter);
            w.set_elink(false);
        });
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
    unsafe fn load_tcd(&self, tcd: &Tcd) {
        let t = self.tcd();
        t.tcd_saddr().write(|w| w.set_saddr(tcd.saddr));
        t.tcd_soff().write(|w| w.set_soff(tcd.soff as u16));
        t.tcd_attr().write(|w| w.0 = tcd.attr);
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(tcd.nbytes));
        t.tcd_slast_sda().write(|w| w.set_slast_sda(tcd.slast as u32));
        t.tcd_daddr().write(|w| w.set_daddr(tcd.daddr));
        t.tcd_doff().write(|w| w.set_doff(tcd.doff as u16));
        t.tcd_citer_elinkno().write(|w| w.set_citer(tcd.citer));
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(tcd.dlast_sga as u32));
        t.tcd_csr().write(|w| w.0 = tcd.csr);
        t.tcd_biter_elinkno().write(|w| w.set_biter(tcd.biter));
    }
}

/// In-memory representation of a Transfer Control Descriptor (TCD).
///
/// This matches the hardware layout (32 bytes).
#[repr(C, align(32))]
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Tcd {
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
    channel: DmaChannel<'a>,
}

impl<'a> Transfer<'a> {
    /// Create a new transfer for the given channel.
    ///
    /// The caller must have already configured and started the DMA channel.
    pub(crate) fn new(channel: DmaChannel<'a>) -> Self {
        Self { channel }
    }

    /// Check if the transfer is still running.
    pub fn is_running(&self) -> bool {
        !self.channel.is_done()
    }

    /// Get the remaining transfer count.
    pub fn remaining(&self) -> u16 {
        let t = self.channel.tcd();
        t.tcd_citer_elinkno().read().citer()
    }

    /// Block until the transfer completes.
    pub fn blocking_wait(self) {
        while self.is_running() {
            core::hint::spin_loop();
        }

        // Ensure all DMA writes are visible
        fence(Ordering::Release);

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
    pub async fn wait_half(&mut self) -> Result<bool, TransferErrors> {
        use core::future::poll_fn;

        poll_fn(|cx| {
            let state = &STATES[self.channel.index()];

            // Register the half-transfer waker
            state.half_waker.register(cx.waker());

            // Check if there's an error
            let t = self.channel.tcd();
            let es = t.ch_es().read();
            if es.err() {
                // Currently, all error fields are in the lowest 8 bits, as-casting truncates
                let errs = es.0 as u8;
                return Poll::Ready(Err(TransferErrors(errs)));
            }

            // Check if we're past the half-way point
            let biter = t.tcd_biter_elinkno().read().biter();
            let citer = t.tcd_citer_elinkno().read().citer();
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
        t.ch_csr().modify(|w| {
            w.set_erq(false);
            w.set_earq(false);
        });

        // Clear any pending interrupt
        t.ch_int().write(|w| w.set_int(true));

        // Clear DONE flag
        t.ch_csr().modify(|w| w.set_done(true));

        fence(Ordering::SeqCst);
    }
}

/// A collection of [TransferError] returned by any transfer.
///
/// Each error variant can be queried separately, or all errors can be iterated by using [TransferErrors::into_iter].
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct TransferErrors(u8);

/// Iterator to extract all [TransferError]s using [TransferErrors::into_iter].
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct TransferErrorIter(u8);

impl TransferErrors {
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

impl IntoIterator for TransferErrors {
    type Item = TransferError;

    type IntoIter = TransferErrorIter;

    fn into_iter(self) -> Self::IntoIter {
        TransferErrorIter(self.0)
    }
}

impl Iterator for TransferErrorIter {
    type Item = TransferError;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        for (mask, var) in TransferErrors::MAP {
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

/// An error that can be returned as the result of a failed transfer.
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
    type Output = Result<(), TransferErrors>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = &STATES[self.channel.index()];
        state.waker.register(cx.waker());
        if self.channel.is_done() {
            // Ensure all DMA writes are visible before returning
            fence(Ordering::Acquire);

            let es = self.channel.tcd().ch_es().read();
            if es.err() {
                // Currently, all error fields are in the lowest 8 bits, as-casting truncates
                let errs = es.0 as u8;
                Poll::Ready(Err(TransferErrors(errs)))
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

        fence(Ordering::Release);
    }
}

/// A ring buffer for continuous DMA reception.
///
/// Can only be constructed by drivers in this HAL, and not from the application.
///
/// This structure manages a circular DMA transfer, allowing continuous
/// reception of data without losing bytes between reads. It uses both
/// half-transfer and complete-transfer interrupts to track available data.
pub struct RingBuffer<'channel, 'buf, W: Word> {
    /// Reference to the DmaChannel for the duration of the DMA transfer.
    ///
    /// When this RingBuffer is dropped, the DmaChannel becomes usable again.
    channel: DmaChannel<'channel>,
    /// Buffer pointer. We use NonNull instead of &mut because DMA acts like
    /// a separate thread writing to this buffer, and &mut claims exclusive
    /// access which the compiler could optimize incorrectly.
    buf: NonNull<[W]>,
    /// Buffer length cached for convenience
    buf_len: usize,
    /// Read position in the buffer (consumer side)
    read_pos: AtomicUsize,
    /// Phantom data to tie the lifetime to the original buffer
    _lt: PhantomData<&'buf mut [W]>,
}

impl<'channel, 'buf, W: Word> RingBuffer<'channel, 'buf, W> {
    /// Create a new ring buffer for the given channel and buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The DMA channel has been configured for circular transfer
    /// - The buffer remains valid for the lifetime of the ring buffer
    /// - Only one RingBuffer exists per DMA channel at a time
    pub(crate) unsafe fn new(channel: DmaChannel<'channel>, buf: &'buf mut [W]) -> Self {
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
        let daddr = t.tcd_daddr().read().daddr() as usize;
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

    /// Enable the DMA channel request.
    ///
    /// Call this to start continuous reception.
    /// This is separated from setup to allow for any additional configuration
    /// before starting the transfer.
    ///
    /// ## SAFETY
    ///
    /// The Dma Channel must have been setup with proper manual configuration prior to
    /// calling `enable_dma_request`. See safety requirements of the configuration methods
    /// for more details.
    pub(crate) unsafe fn enable_dma_request(&self) {
        unsafe {
            self.channel.enable_request();
        }
    }

    /// Stop the DMA transfer. Intended to be called by `stop()` or `Drop`.
    fn teardown(&mut self) -> usize {
        let available = self.available();

        // Disable the channel
        let t = self.channel.tcd();
        t.ch_csr().modify(|w| {
            w.set_erq(false);
            w.set_earq(false)
        });

        // Clear flags
        t.ch_int().write(|w| w.set_int(true));
        t.ch_csr().modify(|w| w.set_done(true));

        fence(Ordering::Release);

        available
    }
}

impl<W: Word> Drop for RingBuffer<'_, '_, W> {
    fn drop(&mut self) {
        self.teardown();
    }
}

impl<'a> DmaChannel<'a> {
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
    /// - The peripheral address must be valid for reads.
    /// - The peripheral's DMA request must be configured to trigger this channel.
    pub(crate) unsafe fn setup_circular_read<'buf, W: Word>(
        &mut self,
        peri_addr: *const W,
        buf: &'buf mut [W],
    ) -> Result<RingBuffer<'_, 'buf, W>, InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_typical(DmaTransferParameters {
                count: buf.len(),
                src_ptr: peri_addr,
                dst_ptr: buf.as_mut_ptr(),
                src_incr: false,
                dst_incr: true,
                circular: true,
                options: TransferOptions {
                    half_transfer_interrupt: true,
                    complete_transfer_interrupt: true,
                    priority: Priority::default(),
                },
            })?
        };

        // Enable NVIC interrupt for this channel so async wakeups work
        self.enable_interrupt();

        Ok(unsafe { RingBuffer::new(self.reborrow(), buf) })
    }
}

/// Maximum number of TCDs in a scatter-gather chain.
pub(crate) const MAX_SCATTER_GATHER_TCDS: usize = 16;

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
    pub fn build(&mut self, channel: DmaChannel<'a>) -> Result<Transfer<'a>, Error> {
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
        DmaChannel::reset_channel_state(&t);

        // Memory barrier to ensure channel state is reset before loading TCD
        cortex_m::asm::dsb();

        // Load first TCD into hardware
        unsafe {
            channel.load_tcd(&self.tcds[0]);
        }

        // Memory barrier before setting START
        cortex_m::asm::dsb();

        // Start the transfer
        t.tcd_csr().modify(|w| w.set_start(Start::CHANNEL_STARTED));

        Ok(Transfer::new(channel))
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

/// Interrupt handler helper.
///
/// Call this from your interrupt handler to clear the interrupt flag and wake the waker.
/// This handles both half-transfer and complete-transfer interrupts.
///
/// # Safety
/// Must be called from the correct DMA channel interrupt context.
unsafe fn on_interrupt(ch_index: usize) {
    crate::perf_counters::incr_interrupt_edma0();
    let edma = &pac::EDMA_0_TCD0;
    let t = edma.tcd(ch_index);

    // Read TCD CSR to determine interrupt source
    let csr = t.tcd_csr().read();

    // Check if this is a half-transfer interrupt
    // INTHALF is set and we're at or past the half-way point
    if csr.inthalf() {
        let biter = t.tcd_biter_elinkno().read().biter();
        let citer = t.tcd_citer_elinkno().read().citer();
        let half_point = biter / 2;

        if citer <= half_point && citer > 0 {
            // Half-transfer interrupt - wake half_waker
            crate::perf_counters::incr_interrupt_edma0_wake();
            half_waker(ch_index).wake();
        }
    }

    // Clear INT flag
    t.ch_int().write(|w| w.set_int(true));

    // If DONE is set, this is a complete-transfer interrupt
    // Only wake the full-transfer waker when the transfer is actually complete
    if t.ch_csr().read().done() {
        crate::perf_counters::incr_interrupt_edma0_wake();
        waker(ch_index).wake();
    }
}

/// Macro to generate DMA channel interrupt handlers.
macro_rules! impl_dma_interrupt_handler {
    ($irq:ident, $ch:expr) => {
        #[interrupt]
        fn $irq() {
            // SAFETY: The correct $ch is called as generated, We check that
            // the given callback is non-null before calling.
            unsafe {
                on_interrupt($ch);

                // See https://doc.rust-lang.org/std/primitive.fn.html#casting-to-and-from-integers
                let cb: *mut () = CALLBACKS[$ch].load(Ordering::Acquire);
                if cb != core::ptr::null_mut() {
                    let cb: fn() = core::mem::transmute(cb);
                    (cb)();
                }
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

// TODO(AJM): This is a gross, gross hack. This implements optional callbacks
// for DMA completion interrupts. This should go away once we switch to
// "in-band" DMA interrupt binding with `bind_interrupts!`.
static CALLBACKS: [AtomicPtr<()>; 8] = [const { AtomicPtr::new(core::ptr::null_mut()) }; 8];
