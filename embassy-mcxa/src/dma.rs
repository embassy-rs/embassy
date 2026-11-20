//! DMA driver.
//!
//! This module provides a typed channel abstraction over the EDMA_TCD0 array
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
//! ## Borrowed Async Transfer API
//!
//! These low-level starters are unsafe: the caller must keep DMA storage valid
//! and exclusive until hardware has actually finished accessing it, even if a
//! transfer is forgotten. See each method's safety contract. A borrowed future
//! alone does not establish a leak-safe or cancellation-safe ownership boundary.
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
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! # extern crate panic_halt;
//! # extern crate embassy_mcxa;
//! # extern crate embassy_executor;
//! # use panic_halt as _;
//! use embassy_executor::Spawner;
//! use embassy_mcxa::clocks::config::Div8;
//! use embassy_mcxa::config::Config;
//! use embassy_mcxa::dma::{DmaChannel, TransferOptions};
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let mut config = Config::default();
//!     config.clock_cfg.sirc.fro_12m_enabled = true;
//!     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
//!
//!     let p = embassy_mcxa::init(config);
//!
//!     let src = [0xa5u8; 4];
//!     let mut dst = [0u8; 4];
//!
//!     let mut ch0 = DmaChannel::new(p.DMA_CH0);
//!     let options = TransferOptions::COMPLETE_INTERRUPT;
//!     // SAFETY: aligned DMA-accessible SRAM, source immutable and destination
//!     // exclusive until normal single-descriptor completion; no cancellation.
//!     let transfer = unsafe { ch0.mem_to_mem(&src, &mut dst, options) }.unwrap();
//!     transfer.await.unwrap();
//! }
//! ```
//!
//! ## Scatter-Gather Builder (For Chained Transfers)
//!
//! Use [`ScatterGatherBuilder`] for complex multi-segment transfers:
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! # extern crate panic_halt;
//! # extern crate embassy_mcxa;
//! # extern crate embassy_executor;
//! # use panic_halt as _;
//! use embassy_executor::Spawner;
//! use embassy_mcxa::clocks::config::Div8;
//! use embassy_mcxa::config::Config;
//! use embassy_mcxa::dma::{DmaChannel, ScatterGatherBuilder};
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let mut config = Config::default();
//!     config.clock_cfg.sirc.fro_12m_enabled = true;
//!     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
//!
//!     let p = embassy_mcxa::init(config);
//!
//!     let src1 = [0x55u8; 4];
//!     let src2 = [0xaau8; 4];
//!     let src3 = [0x5au8; 4];
//!     let mut dst1 = [0u8; 4];
//!     let mut dst2 = [0u8; 4];
//!     let mut dst3 = [0u8; 4];
//!
//!     let mut ch0 = DmaChannel::new(p.DMA_CH0);
//!     let mut builder = ScatterGatherBuilder::<u8>::new();
//!
//!     builder.add_transfer(&src1, &mut dst1);
//!     builder.add_transfer(&src2, &mut dst2);
//!     builder.add_transfer(&src3, &mut dst3);
//!
//!     // SAFETY: aligned DMA-accessible storage; builder stays live and unmoved,
//!     // sources immutable and destinations exclusive until the whole chain
//!     // retires. This example does not forget the retirement owner.
//!     let transfer = unsafe { builder.build(ch0) }.unwrap();
//!     transfer.blocking_wait().unwrap();
//! }
//! ```

#![allow(dead_code)]

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering, fence};
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

pub(crate) use crate::_generated::DmaRequest;
use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::NoConfig;
use crate::dma::sealed::SealedChannel;
use crate::pac::edma_tcd::{Bwc, Size, TcdAttr, TcdBiterElinkno, TcdCiterElinkno, TcdCsr};
use crate::pac::{self, Interrupt};
use crate::peripherals::DMA0;
#[cfg(feature = "_mcxa5xx")]
use crate::peripherals::DMA1;

/// Initialize DMA controller (clock enabled, reset released, controller configured).
///
/// This function is intended to be called ONCE during HAL initialization (`hal::init()`).
///
/// The function enables the DMA0 clock, releases reset, and configures the controller
/// for normal operation with round-robin arbitration.
pub(crate) fn init() {
    // Enable DMA0 clock and release reset
    let _ = unsafe { enable_and_reset::<DMA0>(&NoConfig) };

    // Configure DMA controller
    pac::DMA0.mp_csr().modify(|w| {
        w.set_edbg(true);
        w.set_erca(true);
        w.set_halt(false);
        w.set_gclc(true);
        w.set_gmrc(true);
    });

    // Enable DMA1 clock, release reset, and configure its management page.
    // Without this, any access to the DMA1 TCD registers faults because the
    // peripheral is unclocked and held in reset.
    #[cfg(feature = "_mcxa5xx")]
    {
        let _ = unsafe { enable_and_reset::<DMA1>(&NoConfig) };

        pac::DMA1.mp_csr().modify(|w| {
            w.set_edbg(true);
            w.set_erca(true);
            w.set_halt(false);
            w.set_gclc(true);
            w.set_gmrc(true);
        });
    }

    // Enable all DMA request lines for non-secure access.
    #[cfg(all(feature = "_mcxa5xx", feature = "dma-ipd-req"))]
    for sec_gp_i in 0..=9 {
        crate::pac::AHBSC.sec_gp_reg(sec_gp_i).write(|w| w.0 = 0xffff_ffff);
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

impl Priority {
    pub const LOWEST: Priority = Priority::P7;
    pub const HIGHEST: Priority = Priority::P0;
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
    /// Largest [WordSize] supported by this HAL driver.
    pub const LARGEST: WordSize = WordSize::FourBytes;

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
            WordSize::OneByte => Size::EightBit,
            WordSize::TwoBytes => Size::SixteenBit,
            WordSize::FourBytes => Size::ThirtytwoBit,
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

/// DMA word types: `u8`, `u16`, and `u32`.
///
/// This trait is sealed so transfer widths always match the Rust object's layout.
#[allow(private_bounds)]
pub trait Word: sealed::SealedWord + Copy + 'static {
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
    /// Enable interrupt on transfer complete. Starters returning [`Transfer`]
    /// always enable this notification, even if this field is false.
    pub complete_transfer_interrupt: bool,
    /// Transfer priority
    pub priority: Priority,
}

impl TransferOptions {
    /// Short-hand to specify that no options should be configured.
    pub const NO_INTERRUPTS: Self = Self {
        half_transfer_interrupt: false,
        complete_transfer_interrupt: false,
        priority: Priority::LOWEST,
    };

    /// Short-hand to specify that only the complete transfer interrupt should be triggered.
    pub const COMPLETE_INTERRUPT: Self = Self {
        half_transfer_interrupt: false,
        complete_transfer_interrupt: true,
        priority: Priority::LOWEST,
    };
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

/// Selected buffer in a two-buffer ping-pong DMA transfer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum PingPongSelector {
    /// DMA buffer A selected.
    BufferA,
    /// DMA buffer B selected.
    BufferB,
}

impl PingPongSelector {
    const fn other(self) -> Self {
        match self {
            Self::BufferA => Self::BufferB,
            Self::BufferB => Self::BufferA,
        }
    }
}

/// Ownership status of one ping-pong DMA buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum BufferStatus {
    /// Buffer is free for DMA to use.
    Committed,
    /// Buffer is granted to the peripheral driver and is not ready for DMA reuse.
    Granted,
}

/// Snapshot of the ownership state for one ping-pong DMA channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct PingPongStatus {
    /// Buffer currently selected for DMA writes.
    pub(crate) current: PingPongSelector,
    /// Ownership status of buffer A.
    pub(crate) buffer_a_status: BufferStatus,
    /// Ownership status of buffer B.
    pub(crate) buffer_b_status: BufferStatus,
    /// Sticky error set when DMA reuses an uncommitted buffer.
    ///
    /// Cleared only when the channel is stopped and configured again.
    pub(crate) overrun_error: bool,
}

impl PingPongStatus {
    const ACTIVE: u32 = 1 << 0;
    const CURRENT_B: u32 = 1 << 1;
    const BUFFER_A_GRANTED: u32 = 1 << 2;
    const BUFFER_B_GRANTED: u32 = 1 << 3;
    const OVERRUN: u32 = 1 << 4;

    const fn new() -> Self {
        Self {
            current: PingPongSelector::BufferA,
            buffer_a_status: BufferStatus::Committed,
            buffer_b_status: BufferStatus::Committed,
            overrun_error: false,
        }
    }

    fn encode(self) -> u32 {
        let mut value = Self::ACTIVE;
        if self.current == PingPongSelector::BufferB {
            value |= Self::CURRENT_B;
        }
        if self.buffer_a_status == BufferStatus::Granted {
            value |= Self::BUFFER_A_GRANTED;
        }
        if self.buffer_b_status == BufferStatus::Granted {
            value |= Self::BUFFER_B_GRANTED;
        }
        if self.overrun_error {
            value |= Self::OVERRUN;
        }
        value
    }

    fn decode(value: u32) -> Option<Self> {
        if value & Self::ACTIVE == 0 {
            return None;
        }

        Some(Self {
            current: if value & Self::CURRENT_B == 0 {
                PingPongSelector::BufferA
            } else {
                PingPongSelector::BufferB
            },
            buffer_a_status: if value & Self::BUFFER_A_GRANTED == 0 {
                BufferStatus::Committed
            } else {
                BufferStatus::Granted
            },
            buffer_b_status: if value & Self::BUFFER_B_GRANTED == 0 {
                BufferStatus::Committed
            } else {
                BufferStatus::Granted
            },
            overrun_error: value & Self::OVERRUN != 0,
        })
    }

    fn buffer_status(&self, buffer: PingPongSelector) -> BufferStatus {
        match buffer {
            PingPongSelector::BufferA => self.buffer_a_status,
            PingPongSelector::BufferB => self.buffer_b_status,
        }
    }

    fn set_buffer_status(&mut self, buffer: PingPongSelector, status: BufferStatus) {
        match buffer {
            PingPongSelector::BufferA => self.buffer_a_status = status,
            PingPongSelector::BufferB => self.buffer_b_status = status,
        }
    }

    /// Return the single buffer currently granted to the peripheral driver.
    pub(crate) fn granted_buffer(&self) -> Result<Option<PingPongSelector>, Error> {
        if self.overrun_error {
            return Err(Error::Overrun);
        }

        match (self.buffer_a_status, self.buffer_b_status) {
            (BufferStatus::Committed, BufferStatus::Committed) => Ok(None),
            (BufferStatus::Granted, BufferStatus::Committed) => Ok(Some(PingPongSelector::BufferA)),
            (BufferStatus::Committed, BufferStatus::Granted) => Ok(Some(PingPongSelector::BufferB)),
            (BufferStatus::Granted, BufferStatus::Granted) => Err(Error::Overrun),
        }
    }

    fn set_overrun(&mut self) {
        self.overrun_error = true;
    }

    /// Grant the current buffer after hardware reaches its boundary.
    ///
    /// The observed hardware phase must agree with `current`. The current
    /// buffer becomes granted and DMA moves to the other buffer, which must
    /// already have been committed by the peripheral driver.
    fn grant_completed(&mut self, observed_completed: PingPongSelector) -> Result<(), Error> {
        if observed_completed != self.current {
            self.set_overrun();
            return Err(Error::Overrun);
        }
        self.set_buffer_status(observed_completed, BufferStatus::Granted);
        let next = observed_completed.other();
        if self.buffer_status(next) != BufferStatus::Committed {
            self.set_overrun();
            return Err(Error::Overrun);
        }

        self.current = next;
        Ok(())
    }

    /// Return a granted buffer to DMA after its contents have been copied.
    fn commit(&mut self, buffer: PingPongSelector) -> Result<(), Error> {
        if self.overrun_error || self.buffer_status(buffer) != BufferStatus::Granted {
            self.set_overrun();
            return Err(Error::Overrun);
        }

        self.set_buffer_status(buffer, BufferStatus::Committed);
        Ok(())
    }
}

// Because mcxa2xx only has 1 DMA instance, we need to conditionally set the instance count.
const DMA_INSTANCE_COUNT: usize = if cfg!(feature = "_mcxa5xx") { 2 } else { 1 };
const DMA_CHANNEL_COUNT: usize = 12;

fn ping_pong_status(dma: usize, channel: usize) -> Option<PingPongStatus> {
    let value = STATES[dma][channel].ping_pong_status.load(Ordering::Acquire);
    PingPongStatus::decode(value)
}

fn set_ping_pong_status(dma: usize, channel: usize, status: Option<PingPongStatus>) {
    let value = status.map(PingPongStatus::encode).unwrap_or(0);
    STATES[dma][channel].ping_pong_status.store(value, Ordering::Release);
}

fn update_ping_pong_status(
    dma: usize,
    channel: usize,
    update: impl Fn(&mut PingPongStatus) -> Result<(), Error>,
) -> Result<(), Error> {
    let state = &STATES[dma][channel].ping_pong_status;
    let mut result = Ok(());

    state
        .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            let mut status = PingPongStatus::decode(current)?;
            result = update(&mut status);
            Some(status.encode())
        })
        .map_err(|_| Error::Configuration)?;

    result
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

pub(crate) mod sealed {
    pub trait SealedWord {}

    impl SealedWord for u8 {}
    impl SealedWord for u16 {}
    impl SealedWord for u32 {}

    /// Sealed trait for DMA channels.
    pub trait SealedChannel {
        /// The dma instance number
        fn dma(&self) -> usize;

        /// Zero-based channel index into the TCD array.
        fn channel(&self) -> usize;

        /// Interrupt vector for this channel.
        fn interrupt(&self) -> crate::interrupt::Interrupt;
    }
}

/// Marker trait implemented by HAL peripheral tokens that map to a DMA0
/// channel backed by one EDMA_TCD0 TCD slot.
#[allow(private_bounds)]
pub trait Channel: sealed::SealedChannel + PeripheralType + Into<AnyChannel> + 'static {}

/// Type-erased DMA channel peripheral.
///
/// This allows storing DMA channels in a uniform way regardless of their
/// concrete type, useful for async transfer futures and runtime channel selection.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// #
/// # extern crate panic_halt;
/// # extern crate embassy_mcxa;
/// # extern crate embassy_executor;
/// # use panic_halt as _;
/// # use embassy_executor::Spawner;
/// # use embassy_hal_internal::Peri;
/// # use embassy_mcxa::clocks::config::Div8;
/// # use embassy_mcxa::config::Config;
/// # use embassy_mcxa::dma::{DmaChannel, AnyChannel};
/// #
/// # #[embassy_executor::main]
/// # async fn main(_spawner: Spawner) {
/// #     let mut config = Config::default();
/// #     config.clock_cfg.sirc.fro_12m_enabled = true;
/// #     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
/// #
/// #     let p = embassy_mcxa::init(config);
///     let anychannel: Peri<'static, AnyChannel> = p.DMA_CH0.into();
///     DmaChannel::new(anychannel);
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct AnyChannel {
    pub(crate) dma: u8,
    pub(crate) channel: u8,
    pub(crate) interrupt: Interrupt,
}

impl PeripheralType for AnyChannel {}
impl sealed::SealedChannel for AnyChannel {
    fn dma(&self) -> usize {
        self.dma as usize
    }

    fn channel(&self) -> usize {
        self.channel as usize
    }

    fn interrupt(&self) -> Interrupt {
        self.interrupt
    }
}

impl Channel for AnyChannel {}

#[doc(hidden)]
#[macro_export]
/// Macro to implement Channel trait for a peripheral.
macro_rules! impl_dma_channel {
    ($peri:ident, $dma:expr, $channel:expr, $irq:ident) => {
        impl $crate::dma::sealed::SealedChannel for $crate::peripherals::$peri {
            fn dma(&self) -> usize {
                $dma
            }

            fn channel(&self) -> usize {
                $channel
            }

            fn interrupt(&self) -> nxp_pac::Interrupt {
                nxp_pac::Interrupt::$irq
            }
        }
        impl $crate::dma::Channel for $crate::peripherals::$peri {}

        impl From<$crate::peripherals::$peri> for $crate::dma::AnyChannel {
            fn from(_: $crate::peripherals::$peri) -> Self {
                $crate::dma::AnyChannel {
                    dma: $dma,
                    channel: $channel,
                    interrupt: nxp_pac::Interrupt::$irq,
                }
            }
        }
    };
}

/// Parameters used to configure a 'typical' DMA transfer in [DmaChannel::setup_typical].
struct DmaTransferParameters<WSRC: Word, WDST: Word> {
    /// Number of words (with size WDST) that should be transferred to the destination.
    dst_count: usize,
    /// Source pointer. If incrementing, the backing memory region should be at least as large as `count`.
    src_ptr: *const WSRC,
    /// Destination pointer. If incrementing, the backing memory region should be at least as large as `count`.
    dst_ptr: *mut WDST,
    /// Whether the source pointer should be incremented.
    src_incr: bool,
    /// Whether the destination pointer should be incremented.
    dst_incr: bool,
    /// Perform circular DMA.
    ///
    /// After each loop, will reset the current pointer to the starting addresses, both for src and dest.
    circular: bool,
    /// The transfer will be requested and managed by software.
    ///
    /// This implies that the transfer is completed in a single major loop iteration.
    /// The transfer is also started from software, instead of using target DMA peripheral request signal.
    software: bool,
    /// Public facing transfer options that might be relevant.
    options: TransferOptions,
}

/// A single operation in a peripheral transfer sequence for memory-to-peripheral writes.
pub struct PeripheralWriteOperation<'buf> {
    /// TCD memory that will be loaded into the DMA channel.
    tcd: Tcd,
    /// PhantomData to tie the lifetime of the operation to the source buffer.
    _buffer: PhantomData<&'buf [u8]>,
}

impl<'buf> PeripheralWriteOperation<'buf> {
    /// Create a new PeripheralWriteOperation for a memory-to-peripheral write.
    ///
    /// # Arguments
    ///
    /// `src` - Source buffer slice.
    /// `dst` - Destination peripheral register pointer.
    pub fn new<W: Word>(src: &'buf [W], dst: *mut W) -> Result<Self, InvalidParameters> {
        if src.is_empty() || src.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        Ok(Self {
            tcd: Self::build_tcd(src, dst),
            _buffer: PhantomData,
        })
    }

    /// Create a sequence of write operations from `(src, dst)` pairs.
    ///
    /// # Arguments
    ///
    /// `ops` - An array of `(src, dst)` pairs, where `src` is a source buffer slice and `dst` is a destination peripheral register pointer.
    pub fn new_sequence<W: Word, const N: usize>(
        ops: [(&'buf [W], *mut W); N],
    ) -> Result<[Self; N], InvalidParameters> {
        const { assert!(N > 0, "Sequence must have at least one operation") };

        // Validate parameters for each segment in the sequence
        for (src, _) in &ops {
            if src.is_empty() || src.len() > DMA_MAX_TRANSFER_SIZE {
                return Err(InvalidParameters);
            }
        }

        Ok(ops.map(|(src, dst)| Self {
            tcd: Self::build_tcd(src, dst),
            _buffer: PhantomData,
        }))
    }

    /// Build a TCD for a peripheral write operation.
    ///
    /// # Arguments
    ///
    /// `src` - Source buffer slice.
    /// `dst` - Destination peripheral register pointer.
    fn build_tcd<W: Word>(src: &'buf [W], dst: *mut W) -> Tcd {
        let size = W::size();
        let byte_size = size.bytes();
        let hw_size = size.to_hw_size() as u16;
        let src_buffer_size = src.len() as u16;
        Tcd {
            saddr: src.as_ptr() as u32,
            soff: byte_size as i16,
            attr: (hw_size << 8) | hw_size,
            nbytes: byte_size as u32,
            slast: 0,
            daddr: dst as u32,
            doff: 0,
            citer: src_buffer_size,
            dlast_sga: 0,
            csr: 0,
            biter: src_buffer_size,
        }
    }
}

/// A single operation in a peripheral transfer sequence for peripheral-to-memory reads.
pub struct PeripheralReadOperation<'buf> {
    /// TCD memory that will be loaded into the DMA channel.
    tcd: Tcd,
    /// PhantomData to tie the lifetime of the operation to the destination buffer.
    _buffer: PhantomData<&'buf mut [u8]>,
}

impl<'buf> PeripheralReadOperation<'buf> {
    /// Create a new PeripheralReadOperation for a peripheral-to-memory read.
    ///
    /// # Arguments
    ///
    /// `src` - Source peripheral register pointer.
    /// `dst` - Destination buffer slice.
    pub fn new<W: Word>(src: *const W, dst: &'buf mut [W]) -> Result<Self, InvalidParameters> {
        if dst.is_empty() || dst.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        Ok(Self {
            tcd: Self::build_tcd(src, dst),
            _buffer: PhantomData,
        })
    }

    /// Create a sequence of read operations from `(src, dst)` pairs.
    ///
    /// # Arguments
    ///
    /// `ops` - An array of `(src, dst)` pairs, where `src` is a source peripheral register pointer and `dst` is a destination buffer slice.
    pub fn new_sequence<W: Word, const N: usize>(
        ops: [(*const W, &'buf mut [W]); N],
    ) -> Result<[Self; N], InvalidParameters> {
        // Validate parameters for each segment in the sequence
        for (_, dst) in &ops {
            if dst.is_empty() || dst.len() > DMA_MAX_TRANSFER_SIZE {
                return Err(InvalidParameters);
            }
        }

        Ok(ops.map(|(src, dst)| Self {
            tcd: Self::build_tcd(src, dst),
            _buffer: PhantomData,
        }))
    }

    /// Build a TCD for a peripheral read operation.
    ///
    /// # Arguments
    ///
    /// `src` - Source peripheral register pointer.
    /// `dst` - Destination buffer slice.
    fn build_tcd<W: Word>(src: *const W, dst: &'buf mut [W]) -> Tcd {
        let size = W::size();
        let byte_size = size.bytes();
        let hw_size = size.to_hw_size() as u16;
        let dst_buffer_size = dst.len() as u16;
        Tcd {
            saddr: src as u32,
            soff: 0,
            attr: (hw_size << 8) | hw_size,
            nbytes: byte_size as u32,
            slast: 0,
            daddr: dst.as_mut_ptr() as u32,
            doff: byte_size as i16,
            citer: dst_buffer_size,
            dlast_sga: 0,
            csr: 0,
            biter: dst_buffer_size,
        }
    }
}

/// Internal access to TCD storage used when constructing an SG chain.
trait TcdStorage {
    fn tcd_mut(&mut self) -> &mut Tcd;
}

impl TcdStorage for PeripheralWriteOperation<'_> {
    fn tcd_mut(&mut self) -> &mut Tcd {
        &mut self.tcd
    }
}

impl TcdStorage for PeripheralReadOperation<'_> {
    fn tcd_mut(&mut self) -> &mut Tcd {
        &mut self.tcd
    }
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

    /// Channel index in the EDMA_TCD0 array.
    #[inline]
    pub(crate) fn channel(&self) -> usize {
        self.channel.channel()
    }

    /// Dma index
    #[inline]
    pub(crate) fn dma(&self) -> usize {
        self.channel.dma()
    }

    /// Return a reference to the underlying TCD register block.
    #[inline]
    pub(crate) fn tcd(&self) -> pac::edma_tcd::Tcd {
        match self.dma() {
            0 => pac::EDMA_TCD0.tcd(self.channel.channel()),
            #[cfg(feature = "_mcxa5xx")]
            1 => pac::EDMA_TCD1.tcd(self.channel.channel()),
            _ => unreachable!(),
        }
    }

    /// set a manual callback to be called AFTER the DMA interrupt is processed. Will be called in the DMA interrupt
    /// context.
    ///
    /// SAFETY: This must only be called on an owned DmaChannel, as there is only a single
    /// callback slot, and calling this will invalidate any previously set callbacks.
    pub(crate) unsafe fn set_callback(&mut self, f: fn()) {
        // See https://doc.rust-lang.org/std/primitive.fn.html#casting-to-and-from-integers
        let cb = f as *mut ();
        CALLBACKS[self.dma()][self.channel()].store(cb, Ordering::Release);
    }

    /// Unset the callback, causing no method to be called after DMA completion.
    ///
    /// SAFETY: This must only be called on an owned DmaChannel, as there is only a single
    /// callback slot, and calling this will invalidate any previously set callbacks.
    pub(crate) unsafe fn clear_callback(&mut self) {
        CALLBACKS[self.dma()][self.channel()].store(core::ptr::null_mut(), Ordering::Release);
    }

    /// Access TCD DADDR field
    pub(crate) fn daddr(&self) -> u32 {
        self.tcd().tcd_daddr().read().daddr()
    }

    /// Return the current ping-pong ownership state for this channel.
    pub(crate) fn ping_pong_status(&self) -> Option<PingPongStatus> {
        ping_pong_status(self.dma(), self.channel())
    }

    /// Return whether an unhandled ping-pong boundary is currently pending.
    ///
    /// The DMA ISR clears this latch, so callers detecting a boundary across a
    /// non-atomic operation must also compare ping-pong status snapshots.
    pub(crate) fn ping_pong_boundary_pending(&self) -> bool {
        self.tcd().ch_int().read().int()
    }

    /// Commit a completed buffer after the peripheral driver has copied it.
    ///
    /// Committing a buffer that is not currently granted, or committing after the
    /// state has already gone sticky-overrun, leaves the channel in sticky error.
    /// The peripheral owner must stop its source and retire accepted work.
    pub(crate) fn commit_ping_pong_buffer(&self, buffer: PingPongSelector) -> Result<(), Error> {
        fence(Ordering::Release);
        let result = update_ping_pong_status(self.dma(), self.channel(), |status| status.commit(buffer));
        result
    }

    fn clear_tcd(t: &pac::edma_tcd::Tcd) {
        // Full TCD reset following NXP SDK pattern (EDMA_TcdResetExt).
        // Reset ALL TCD registers to 0 to clear any stale configuration from
        // previous transfers. This is critical when reusing a channel.
        t.tcd_saddr().write(|w| w.set_saddr(0));
        t.tcd_soff().write(|w| w.set_soff(0));
        t.tcd_attr().write(|w| *w = TcdAttr(0));
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(0));
        t.tcd_daddr().write(|w| w.set_daddr(0));
        t.tcd_doff().write(|w| w.set_doff(0));
        t.tcd_citer_elinkno().write(|w| *w = TcdCiterElinkno(0));
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
        t.tcd_csr().write(|w| *w = TcdCsr(0)); // Clear CSR completely
        t.tcd_biter_elinkno().write(|w| *w = TcdBiterElinkno(0));
    }

    #[inline]
    fn set_major_loop_ct_elinkno(t: &pac::edma_tcd::Tcd, count: u16) {
        t.tcd_biter_elinkno().write(|w| w.set_biter(count));
        t.tcd_citer_elinkno().write(|w| w.set_citer(count));
    }

    #[inline]
    fn set_major_loop_nbytes_without_minor(t: &pac::edma_tcd::Tcd, count: u32) {
        t.tcd_nbytes_mloffno().write(|w| {
            w.set_smloe(false);
            w.set_dmloe(false);
            w.set_nbytes(count)
        });
    }

    #[inline]
    fn set_no_final_adjustments(t: &pac::edma_tcd::Tcd) {
        // No source/dest adjustment after major loop
        t.tcd_slast_sda().write(|w| w.set_slast_sda(0));
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));
    }

    #[inline]
    fn set_source_ptr<T>(t: &pac::edma_tcd::Tcd, p: *const T) {
        t.tcd_saddr().write(|w| w.set_saddr(p as u32));
    }

    #[inline]
    fn set_source_increment(t: &pac::edma_tcd::Tcd, sz: WordSize) {
        t.tcd_soff().write(|w| w.set_soff(sz.bytes() as u16));
    }

    #[inline]
    fn set_source_fixed(t: &pac::edma_tcd::Tcd) {
        t.tcd_soff().write(|w| w.set_soff(0));
    }

    #[inline]
    fn set_dest_ptr<T>(t: &pac::edma_tcd::Tcd, p: *mut T) {
        t.tcd_daddr().write(|w| w.set_daddr(p as u32));
    }

    #[inline]
    fn set_dest_increment(t: &pac::edma_tcd::Tcd, sz: WordSize) {
        t.tcd_doff().write(|w| w.set_doff(sz.bytes() as u16));
    }

    #[inline]
    fn set_dest_fixed(t: &pac::edma_tcd::Tcd) {
        t.tcd_doff().write(|w| w.set_doff(0));
    }

    #[inline]
    fn set_fixed_priority(t: &pac::edma_tcd::Tcd, p: Priority) {
        t.ch_pri().write(|w| {
            w.set_dpa(false);
            w.set_ecp(true);
            w.set_apl(p as u8);
        });
    }

    #[inline]
    fn reset_channel_state(t: &pac::edma_tcd::Tcd) {
        // CSR: Resets to all zeroes (disabled), "done" is cleared by writing 1
        t.ch_csr().write(|w| w.set_done(true));
        // ES: Resets to all zeroes (disabled), "err" is cleared by writing 1
        t.ch_es().write(|w| w.set_err(true));
        // INT: Resets to all zeroes (disabled), "int" is cleared by writing 1
        t.ch_int().write(|w| w.set_int(true));
        // Clear stale error evidence before enabling its interrupt source.
        t.ch_csr().modify(|w| w.set_eei(true));
    }

    /// Start an async transfer.
    ///
    /// The channel must already be configured. This enables the channel
    /// request and returns a `Transfer` future that resolves when the
    /// DMA transfer completes.
    ///
    /// # Safety
    ///
    /// The caller must ensure the DMA channel is quiescent and properly configured
    /// and that source/destination buffers remain valid for the duration
    /// of the transfer.
    #[allow(unused)]
    pub(crate) unsafe fn start_transfer(&mut self) -> Transfer<'_> {
        // Clear any previous DONE/INT flags
        let t = self.tcd();
        t.ch_csr().modify(|w| w.set_done(true));
        t.ch_int().write(|w| w.set_int(true));
        t.tcd_csr().modify(|w| w.set_intmajor(true));

        // Enable the channel request
        t.ch_csr().modify(|w| {
            w.set_erq(true);
            w.set_earq(true);
        });

        Transfer::new(self.reborrow())
    }

    /// Setup a typical DMA transfer.
    ///
    /// A minor loop iteration transfers to a single word before yielding to arbitrage.
    /// A major loop iteration transfers the entire buffer.
    ///
    /// # Safety
    ///
    /// Requires that the source/destination buffers remain valid for the duration
    /// of the transfer.
    unsafe fn setup_transfers<WSRC: Word, WDST: Word>(&self, params: DmaTransferParameters<WSRC, WDST>) {
        let byte_count = (params.dst_count * WDST::size().bytes()) as u32;

        let t = self.tcd();
        self.prepare_regular_transfer();

        // Reset channel state - clear DONE, disable requests, clear errors
        Self::reset_channel_state(&t);

        // Memory & compiler barrier to ensure channel state is fully reset before touching TCD
        fence(Ordering::Release);

        Self::clear_tcd(&t);

        Self::set_source_ptr(&t, params.src_ptr);
        if params.src_incr {
            Self::set_source_increment(&t, WSRC::size());
        } else {
            Self::set_source_fixed(&t);
        }

        Self::set_dest_ptr(&t, params.dst_ptr);
        if params.dst_incr {
            Self::set_dest_increment(&t, WDST::size());
        } else {
            Self::set_dest_fixed(&t);
        }

        // Transfer attributes (size)
        t.tcd_attr().write(|w| {
            w.set_ssize(WSRC::size().to_hw_size());
            w.set_dsize(WDST::size().to_hw_size());
        });

        let (minor, major) = if params.software {
            // When called from software, transfer all bytes in a single major loop iteration.
            (byte_count, 1)
        } else {
            // When driven by hardware, transfer a DST word per major loop iteration.
            // When the DMA channel requests (ERQ) is driven by a peripheral, this is desirable.
            (WDST::size().bytes() as u32, params.dst_count as u16)
        };

        // Set the amount of bytes to be transferred per major loop iteration without minor loop offsets.
        Self::set_major_loop_nbytes_without_minor(&t, minor);
        Self::set_major_loop_ct_elinkno(&t, major);

        // Configure channel to be interruptible, to interrupt, with a set priority.
        Self::set_fixed_priority(&t, params.options.priority);

        if params.circular {
            let byte_diff = -(byte_count as i32); // Decrement the address pointers (if incrementing & not fixed).
            let byte_diff_reg = byte_diff as u32; // Cast as u32 so that it can be stored in the register.

            t.tcd_slast_sda()
                .write(|w| w.set_slast_sda(if params.src_incr { byte_diff_reg } else { 0 }));
            t.tcd_dlast_sga()
                .write(|w| w.set_dlast_sga(if params.dst_incr { byte_diff_reg } else { 0 }));
        } else {
            Self::set_no_final_adjustments(&t);
        }

        // Memory & compiler barrier before setting START
        fence(Ordering::Release);

        // Control/status: interrupt on major complete, start
        // Write this last after all other TCD registers are configured
        t.tcd_csr().write(|w| {
            w.set_intmajor(params.options.complete_transfer_interrupt);
            // RM: INTHALF is not supported when BITER is one.
            w.set_inthalf(params.options.half_transfer_interrupt && major > 1);
            w.set_start(params.software);
            w.set_esg(false);
            w.set_majorelink(false);
            w.set_eeop(false);
            w.set_esda(false);
            w.set_bwc(Bwc::NoStall);

            w.set_dreq(!params.circular);
        });

        // Memory & compiler barrier after setting START
        fence(Ordering::Release);
    }

    // ========================================================================
    // Type-Safe Transfer Methods (Embassy-style API)
    // ========================================================================

    /// Start a memory-to-memory DMA transfer of `src.len()` words.
    ///
    /// This is a type-safe wrapper that uses the `Word` trait to determine
    /// the correct transfer width automatically. Uses the global eDMA TCD
    /// register accessor internally.
    /// A larger destination is allowed; its tail is left unchanged. DMA starts
    /// before this function returns, not on the first poll of the future.
    ///
    /// # Arguments
    ///
    /// * `src` - Source buffer
    /// * `dst` - Destination buffer (must be at least as large as src)
    /// * `options` - Transfer configuration options
    ///
    /// # Safety
    ///
    /// - Source and destination storage must remain live, properly aligned,
    ///   DMA-accessible and cache-coherent until all hardware accesses retire.
    /// - The source must remain immutable and the destination exclusively owned
    ///   by this transfer: no CPU or other DMA access to the destination, nor
    ///   writes to the source, while hardware can access them.
    /// - These obligations survive dropping or forgetting the returned handle.
    ///   Normal destruction drains pending software work before returning, but
    ///   forgetting a borrowed handle does not stop DMA or retain its storage.
    pub unsafe fn mem_to_mem<'b, W: Word>(
        &'b mut self,
        src: &'b [W],
        dst: &'b mut [W],
        mut options: TransferOptions,
    ) -> Result<Transfer<'b>, InvalidParameters> {
        if src.is_empty() || src.len() > dst.len() || src.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        options.complete_transfer_interrupt = true;
        unsafe {
            self.setup_transfers(DmaTransferParameters {
                src_ptr: src.as_ptr(),
                dst_ptr: dst.as_mut_ptr(),
                dst_count: src.len(),
                src_incr: true,
                dst_incr: true,
                circular: false,
                software: true,
                options,
            });
        }

        Ok(Transfer::new(self.reborrow()))
    }

    /// Fill a memory buffer with a pattern value (memset).
    ///
    /// This performs a DMA transfer where the source address remains fixed
    /// (pattern value) while the destination address increments through the buffer.
    /// It's useful for quickly filling large memory regions with a constant value.
    /// DMA starts before this function returns, not on the first poll.
    ///
    /// # Safety
    ///
    /// - The pattern and destination must remain live, properly aligned,
    ///   DMA-accessible and cache-coherent until all hardware accesses retire.
    /// - The pattern must remain immutable; the destination must remain
    ///   exclusive to this transfer, with no CPU or other DMA access.
    /// - These obligations apply even if the handle is dropped or forgotten.
    ///   Normal destruction drains pending software work before returning;
    ///   forgetting the handle does not stop DMA or retain its storage.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Reference to the pattern value (will be read repeatedly)
    /// * `dst` - Destination buffer to fill
    /// * `options` - Transfer configuration options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    ///
    /// # extern crate panic_halt;
    /// # extern crate embassy_mcxa;
    /// # extern crate embassy_executor;
    /// # use panic_halt as _;
    /// # use embassy_executor::Spawner;
    /// # use embassy_mcxa::clocks::config::Div8;
    /// # use embassy_mcxa::config::Config;
    /// # use embassy_mcxa::dma::{DmaChannel, TransferOptions};
    ///
    /// # #[embassy_executor::main]
    /// # async fn main(_spawner: Spawner) {
    ///     let mut config = Config::default();
    ///     config.clock_cfg.sirc.fro_12m_enabled = true;
    ///     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
    ///
    ///     let p = embassy_mcxa::init(config);
    ///
    ///     let mut ch0 = DmaChannel::new(p.DMA_CH0);
    ///     let pattern = 0xDEADBEEF_u32;
    ///     let mut buffer = [0u32; 256];
    ///
    ///     // SAFETY: aligned DMA-accessible SRAM; pattern immutable and buffer
    ///     // exclusive until normal single-descriptor completion, no cancellation.
    ///     let transfer = unsafe {
    ///         ch0.memset(&pattern, &mut buffer, TransferOptions::COMPLETE_INTERRUPT)
    ///     }.unwrap();
    ///     transfer.await.unwrap();
    /// # }
    /// ```
    pub unsafe fn memset<'b, W: Word>(
        &'b mut self,
        pattern: &'b W,
        dst: &'b mut [W],
        mut options: TransferOptions,
    ) -> Result<Transfer<'b>, InvalidParameters> {
        if dst.is_empty() || dst.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        options.complete_transfer_interrupt = true;
        unsafe {
            self.setup_transfers(DmaTransferParameters {
                src_ptr: pattern,
                dst_ptr: dst.as_mut_ptr(),
                dst_count: dst.len(),
                src_incr: false,
                dst_incr: true,
                circular: false,
                software: true,
                options,
            });
        }

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
    /// - Keep storage live, DMA-accessible, coherent and exclusive through
    ///   retirement, even if the handle is forgotten. Before cancellation,
    ///   disable the peripheral's DMA request and read that register back;
    ///   this channel-only handle cannot shut down the peripheral for you.
    pub unsafe fn write_to_peripheral<W: Word>(
        &mut self,
        buf: &[W],
        peri_addr: *mut W,
        mut options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        options.complete_transfer_interrupt = true;
        unsafe {
            self.setup_write_to_peripheral(buf, peri_addr, false, options)?;
            self.enable_request();
        };

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
    /// - Keep storage live, DMA-accessible, coherent and exclusive through
    ///   retirement, even if the handle is forgotten. Before cancellation,
    ///   disable the peripheral's DMA request and read that register back;
    ///   this channel-only handle cannot shut down the peripheral for you.
    pub unsafe fn read_from_peripheral<W: Word>(
        &mut self,
        peri_addr: *const W,
        buf: &mut [W],
        mut options: TransferOptions,
    ) -> Result<Transfer<'_>, InvalidParameters> {
        options.complete_transfer_interrupt = true;
        unsafe {
            self.setup_read_from_peripheral(peri_addr, buf, false, options)?;
            self.enable_request();
        };

        Ok(Transfer::new(self.reborrow()))
    }

    /// Common function to set up a sequence of DMA transfer operations.
    ///
    /// # Arguments
    ///
    /// * `sequence` - Sequence of DMA transfer operations to chain together.
    ///
    /// # Safety
    /// - The sequence storage must not be moved, dropped or modified for the duration of the transfer.
    /// - The source/destination buffers in the sequence must remain valid for the duration of the transfer.
    /// - Every register's address in the sequence must be valid for read/writes.
    /// - All operations must be compatible with the configured DMA request
    unsafe fn setup_sequence<T: TcdStorage>(&mut self, sequence: &mut [T]) -> Result<(), InvalidParameters> {
        if sequence.is_empty() {
            return Err(InvalidParameters);
        }

        // Link the peripheral-paced TCDs. Intermediate TCDs only load the next
        // descriptor; the final TCD disables requests and raises completion.
        let mut remaining = &mut *sequence;
        while let Some((current, tail)) = remaining.split_first_mut() {
            let mut csr = TcdCsr(0);

            let current_tcd = current.tcd_mut();
            if let Some(next) = tail.first_mut() {
                // For all but the last TCD, set dlast_sga to point to the next TCD in the chain.
                let next_tcd = next.tcd_mut();
                current_tcd.dlast_sga = next_tcd as *const Tcd as i32;

                // Set ESG=ScatterGatherFormat for all but the last TCD to enable chaining.
                csr.set_esg(true);
            } else {
                // Last TCD in the sequence, no next TCD.
                current_tcd.dlast_sga = 0;

                // For the last TCD, clear DREQ to auto-disable requests on completion.
                csr.set_dreq(true);
                // Enable INTMAJOR to fire a completion interrupt when the last TCD finishes.
                csr.set_intmajor(true);
            }

            current_tcd.csr = csr.0;
            remaining = tail;
        }

        // Reset channel state - clear DONE, disable requests, clear errors
        // This ensures the channel is in a clean state before loading the TCD
        Self::reset_channel_state(&self.tcd());
        self.prepare_peripheral_scatter_gather();

        // Ensure the completed in-memory chain is visible before loading its head.
        fence(Ordering::Release);

        let head = sequence.first_mut().ok_or(InvalidParameters)?;
        unsafe {
            self.load_tcd(head.tcd_mut());
        }

        // Memory barrier before enabling request
        fence(Ordering::Release);

        Ok(())
    }

    /// Configure a memory-to-peripheral scatter-gather sequence without starting it.
    ///
    /// This chains and loads the sequence's TCDs into the channel but does NOT
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
    /// * `sequence` - Sequence of peripheral write operations to chain together.
    ///
    /// # Safety
    ///
    /// - The sequence storage must not be moved, dropped or modified for the duration of the transfer.
    /// - Every destination address in the sequence must be valid for writes.
    /// - All operations must be compatible with the configured DMA request
    pub(crate) unsafe fn setup_write_to_peripheral_scatter_gather(
        &mut self,
        sequence: &mut [PeripheralWriteOperation<'_>],
    ) -> Result<(), InvalidParameters> {
        if sequence.is_empty() {
            return Err(InvalidParameters);
        }

        unsafe { self.setup_sequence(sequence) }
    }

    /// Start a memory-to-peripheral scatter-gather sequence.
    ///
    /// This chains and loads the sequence, then enables the channel request (ERQ) and
    /// returns a [`Transfer`]. The caller is still responsible for enabling the
    /// peripheral's own DMA request so it actually drives the channel.
    ///
    /// # Arguments
    ///
    /// * `sequence` - Sequence of peripheral write operations to chain together.
    ///
    /// # Safety
    ///
    /// - Every destination address in the sequence must be valid for writes.
    /// - All operations must be compatible with the configured DMA request.
    /// - If the returned transfer is forgotten, the sequence must remain valid until the transfer
    ///   completes.
    /// - Before cancellation, disable and read back the peripheral's DMA request
    ///   source. Retain the sequence at its current address, and all buffers,
    ///   until accepted requests and descriptor fetches have retired.
    pub unsafe fn write_to_peripheral_scatter_gather<'seq>(
        &'seq mut self,
        sequence: &'seq mut [PeripheralWriteOperation<'_>],
    ) -> Result<Transfer<'seq>, InvalidParameters> {
        unsafe {
            self.setup_write_to_peripheral_scatter_gather(sequence)?;
            self.enable_request();
        }

        Ok(Transfer::new(self.reborrow()))
    }

    /// Configure a peripheral-to-memory scatter-gather sequence without starting it.
    ///
    /// This chains and loads the sequence's TCDs into the channel but does NOT
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
    /// * `sequence` - Sequence of peripheral read operations to chain together.
    ///
    /// # Safety
    ///
    /// - The sequence storage must not be moved, dropped or modified for the duration of the transfer.
    /// - Every source address in the sequence must be valid for reads.
    /// - All operations must be compatible with the configured DMA request
    pub(crate) unsafe fn setup_read_from_peripheral_scatter_gather(
        &mut self,
        sequence: &mut [PeripheralReadOperation<'_>],
    ) -> Result<(), InvalidParameters> {
        if sequence.is_empty() {
            return Err(InvalidParameters);
        }

        unsafe { self.setup_sequence(sequence) }
    }

    /// Start a peripheral-to-memory scatter-gather sequence.
    ///
    /// This chains and loads the sequence, then enables the channel request (ERQ) and
    /// returns a [`Transfer`]. The caller is still responsible for enabling the
    /// peripheral's own DMA request so it actually drives the channel.
    ///
    /// # Arguments
    ///
    /// * `sequence` - Sequence of peripheral read operations to chain together.
    ///
    /// # Safety
    ///
    /// - Every source address in the sequence must be valid for reads.
    /// - All operations must be compatible with the configured DMA request.
    /// - If the returned transfer is forgotten, the sequence must remain valid and inaccessible
    ///   until the transfer completes.
    /// - Before cancellation, disable and read back the peripheral's DMA request
    ///   source. Retain the sequence at its current address, and all buffers,
    ///   until accepted requests and descriptor fetches have retired.
    pub unsafe fn read_from_peripheral_scatter_gather<'seq>(
        &'seq mut self,
        sequence: &'seq mut [PeripheralReadOperation<'_>],
    ) -> Result<Transfer<'seq>, InvalidParameters> {
        unsafe {
            self.setup_read_from_peripheral_scatter_gather(sequence)?;
            self.enable_request();
        }

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

        #[repr(C, align(4))]
        struct Zero([u8; WordSize::LARGEST.bytes()]);

        // Static mut so that this is allocated in RAM.
        static mut DUMMY: Zero = Zero([0u8; WordSize::LARGEST.bytes()]);

        unsafe {
            self.setup_transfers(DmaTransferParameters {
                // Unsafe: cast to W is safe as DUMMY is aligned and guaranteed to be as least as large.
                src_ptr: core::ptr::addr_of_mut!(DUMMY) as *const W,
                dst_ptr: peri_addr,
                dst_count: count,
                src_incr: false,
                dst_incr: false,
                circular: false,
                software: false,
                options,
            });
        }

        Ok(())
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
    /// * `software` - Use software start for the transfer; otherwise use hardware ERQ to drive the transfer.
    ///   Should be `false` unless your peripheral does not support hardware ERQ.
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for writes.
    pub(crate) unsafe fn setup_write_to_peripheral<WSRC: Word, WDST: Word>(
        &self,
        buf: &[WSRC],
        peri_addr: *mut WDST,
        software: bool,
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_transfers(DmaTransferParameters {
                src_ptr: buf.as_ptr(),
                dst_ptr: peri_addr,
                dst_count: buf.len() * WSRC::size().bytes() / WDST::size().bytes(),
                src_incr: true,
                dst_incr: false,
                circular: false,
                software,
                options,
            });
        }

        Ok(())
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
    /// * `software` - Use software start for the transfer; otherwise use hardware ERQ to drive the transfer.
    ///   Should be `false` unless your peripheral does not support hardware ERQ.
    /// * `enable_interrupt` - Whether to enable interrupt on completion
    ///
    /// # Safety
    ///
    /// - The buffer must remain valid for the duration of the transfer.
    /// - The peripheral address must be valid for reads.
    pub(crate) unsafe fn setup_read_from_peripheral<WSRC: Word, WDST: Word>(
        &self,
        peri_addr: *const WSRC,
        buf: &mut [WDST],
        software: bool,
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_transfers(DmaTransferParameters {
                src_ptr: peri_addr,
                dst_ptr: buf.as_mut_ptr(),
                dst_count: buf.len(),
                src_incr: false,
                dst_incr: true,
                circular: false,
                software,
                options,
            });
        }

        Ok(())
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
    /// ```rust,ignore
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
    /// This does not retire accepted work. Disable and read back the peripheral
    /// request source, then use `stop` before releasing storage or rewriting TCDs.
    pub(crate) unsafe fn disable_request(&self) {
        let t = self.tcd();
        t.ch_csr().modify(|w| {
            w.set_erq(false);
            w.set_earq(false);
        });
    }

    /// Retire accepted work before releasing buffers, descriptors, or grants.
    ///
    /// The peripheral owner must first disable its request source and read the
    /// peripheral register back. No other channel may subsequently link to this
    /// channel. We wait for HRS to deassert before clearing ERQ (A2 RM
    /// 15.5.9.1 / A5 RM 23.5.9.1), then drain START as well as ACTIVE (A2
    /// 15.5.6.1 / A5 23.5.6.1). Disabling ERQ does not cancel software START.
    ///
    /// A finite software SG chain drains to its final interrupt or an error:
    /// idle gaps between descriptors are not a retirement boundary. This can
    /// wait for the whole chain, not just one minor loop. It deliberately does
    /// not use global CX/HALT, which would affect independently owned channels.
    /// A stalled bus can prevent retirement; a timeout must not release memory
    /// while hardware can still access it.
    pub(crate) fn stop(&self) {
        let t = self.tcd();
        let state = &STATES[self.dma()][self.channel()];

        if t.ch_csr().read().erq() {
            while self.hardware_request_pending() {
                core::hint::spin_loop();
            }
            t.ch_csr().modify(|w| {
                w.set_erq(false);
                w.set_earq(false);
            });
        }

        if state.software_scatter_gather.load(Ordering::Acquire) {
            while self.errors().is_none() && !self.is_done() {
                core::hint::spin_loop();
            }
        }

        // Read START first: arbitration clears it as execution begins. The
        // following ACTIVE read catches that transition. No new request source
        // remains, and software SG has reached its terminal event above.
        while t.tcd_csr().read().start() || t.ch_csr().read().active() {
            core::hint::spin_loop();
        }

        critical_section::with(|_| {
            // TCD writes are prohibited while ACTIVE. Only mask TCD interrupts
            // after retirement, and preserve error evidence before W1C.
            let _ = self.errors();
            t.ch_csr().modify(|w| {
                w.set_erq(false);
                w.set_earq(false);
                w.set_eei(false);
                w.set_done(true);
            });
            t.tcd_csr().modify(|w| {
                w.set_intmajor(false);
                w.set_inthalf(false);
            });
            t.ch_es().write(|w| w.set_err(true));
            t.ch_int().write(|w| w.set_int(true));
            cortex_m::peripheral::NVIC::unpend(self.channel.interrupt());
            state.peripheral_scatter_gather_active.store(false, Ordering::Release);
            state.peripheral_scatter_gather_done.store(false, Ordering::Release);
            state.software_scatter_gather.store(false, Ordering::Release);
            set_ping_pong_status(self.dma(), self.channel(), None);
        });

        cortex_m::asm::dsb();
        fence(Ordering::Acquire);
    }

    fn hardware_request_pending(&self) -> bool {
        let hrs = match self.dma() {
            0 => pac::DMA0.mp_hrs().read().hrs(),
            #[cfg(feature = "_mcxa5xx")]
            1 => pac::DMA1.mp_hrs().read().hrs(),
            _ => unreachable!(),
        };
        (hrs & (1 << self.channel())) != 0
    }

    /// Preserve error-only terminal outcomes independently of DONE and W1C.
    pub(crate) fn errors(&self) -> Option<TransferErrors> {
        let state = &STATES[self.dma()][self.channel()];
        let es = self.tcd().ch_es().read();
        if es.err() {
            state.errors.fetch_or(es.0, Ordering::AcqRel);
        }
        let errors = state.errors.load(Ordering::Acquire);
        // ERR can be asserted without a low-byte detail (e.g. cancellation).
        (errors != 0).then_some(TransferErrors(errors as u8))
    }

    /// Register before checking predicates, consuming stored wakes until a
    /// future interrupt is guaranteed to have a registered waiter.
    pub(crate) fn poll_complete(&self, cx: &mut Context<'_>) -> Poll<Result<(), TransferErrors>> {
        loop {
            let registered = self.wait_cell().poll_wait(cx).is_pending();
            if let Some(errors) = self.errors() {
                return Poll::Ready(Err(errors));
            }
            if self.is_done() {
                fence(Ordering::Acquire);
                return Poll::Ready(self.errors().map_or(Ok(()), Err));
            }
            if registered {
                return Poll::Pending;
            }
        }
    }

    /// Return true if the current transfer has completed.
    ///
    /// Regular transfers use the hardware DONE flag. Final-interrupt SG uses
    /// a completion latch set by its final-only interrupt.
    pub(crate) fn is_done(&self) -> bool {
        let state = &STATES[self.dma()][self.channel()];
        if state.peripheral_scatter_gather_active.load(Ordering::Acquire) {
            self.is_peripheral_scatter_gather_done(state)
        } else {
            self.tcd().ch_csr().read().done()
        }
    }

    fn is_peripheral_scatter_gather_done(&self, state: &State) -> bool {
        if state.peripheral_scatter_gather_done.load(Ordering::Acquire) {
            return true;
        }

        let interrupted = self.tcd().ch_int().read().int();
        if interrupted {
            state.peripheral_scatter_gather_done.store(true, Ordering::Release);
        }

        // Observe a peripheral SG ISR that latched final completion and cleared
        // CH_INT concurrently with the checks above.
        state.peripheral_scatter_gather_done.load(Ordering::Acquire)
    }

    /// Reset the channel state for a regular transfer.
    fn prepare_regular_transfer(&self) {
        let state = &STATES[self.dma()][self.channel()];
        state.errors.store(0, Ordering::Release);
        state.software_scatter_gather.store(false, Ordering::Release);
        state.peripheral_scatter_gather_active.store(false, Ordering::Release);
        state.peripheral_scatter_gather_done.store(false, Ordering::Release);
        set_ping_pong_status(self.dma(), self.channel(), None);
    }

    /// Select final-interrupt completion tracking for a finite SG sequence.
    fn prepare_peripheral_scatter_gather(&self) {
        let state = &STATES[self.dma()][self.channel()];
        state.errors.store(0, Ordering::Release);
        state.software_scatter_gather.store(false, Ordering::Release);
        state.peripheral_scatter_gather_done.store(false, Ordering::Release);
        state.peripheral_scatter_gather_active.store(true, Ordering::Release);
        set_ping_pong_status(self.dma(), self.channel(), None);
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
        STATES[self.dma()][self.channel()]
            .peripheral_scatter_gather_done
            .store(false, Ordering::Release);
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
        t.tcd_csr().modify(|w| w.set_start(true));
    }

    /// Get the wait cell for this channel
    pub(crate) fn wait_cell(&self) -> &'static WaitCell {
        &STATES[self.channel.dma()][self.channel.channel()].waker
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
    /// WaitCell for transfer complete interrupt
    waker: WaitCell,
    /// WaitCell for half-transfer interrupt
    half_waker: WaitCell,
    /// Raw CH_ES including ERR; kept until the next transfer is configured.
    errors: AtomicU32,
    /// Finite auto-started memory SG must retire the entire chain on Drop.
    software_scatter_gather: AtomicBool,
    /// Packed ping-pong ownership state shared by DMA and peripheral ISRs.
    /// Zero means the channel is not configured for ping-pong operation.
    ping_pong_status: AtomicU32,
    /// Whether this channel currently uses final-interrupt SG completion.
    peripheral_scatter_gather_active: AtomicBool,
    /// Set when final SG completion is observed.
    peripheral_scatter_gather_done: AtomicBool,
}

impl State {
    const fn new() -> Self {
        Self {
            waker: WaitCell::new(),
            half_waker: WaitCell::new(),
            errors: AtomicU32::new(0),
            software_scatter_gather: AtomicBool::new(false),
            ping_pong_status: AtomicU32::new(0),
            peripheral_scatter_gather_active: AtomicBool::new(false),
            peripheral_scatter_gather_done: AtomicBool::new(false),
        }
    }
}

static STATES: [[State; DMA_CHANNEL_COUNT]; DMA_INSTANCE_COUNT] =
    [const { [const { State::new() }; DMA_CHANNEL_COUNT] }; DMA_INSTANCE_COUNT];

pub(crate) fn waker(dma: usize, channel: usize) -> &'static WaitCell {
    &STATES[dma][channel].waker
}

pub(crate) fn half_waker(dma: usize, channel: usize) -> &'static WaitCell {
    &STATES[dma][channel].half_waker
}

// ============================================================================
// Async Transfer Future
// ============================================================================

/// An in-progress DMA transfer.
///
/// This type implements `Future` and can be `.await`ed to wait for the
/// transfer to complete. Hardware is already armed when this handle is returned.
/// Dropping it retires accepted work. Software scatter-gather cancellation may
/// drain the entire finite chain. Peripheral users must disable and read back
/// their request source before dropping the handle. Forgetting it does not stop
/// DMA; the starter's storage obligations remain in force until retirement.
#[must_use = "DMA is already armed; retain storage until actual hardware completion or retirement"]
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
        self.channel.errors().is_none() && !self.channel.is_done()
    }

    /// Get the remaining transfer count.
    pub fn remaining(&self) -> u16 {
        let t = self.channel.tcd();
        t.tcd_citer_elinkno().read().citer()
    }

    /// Block until completion or an error, retire accepted work, and return its
    /// result. A permanently stalled bus can prevent retirement and block here.
    pub fn blocking_wait(self) -> Result<(), TransferErrors> {
        while self.is_running() {
            core::hint::spin_loop();
        }
        // Drop retires accepted work, including error paths, before the caller
        // can inspect or reuse storage. Do not forget the retirement owner.
        self.channel.stop();
        let result = self.channel.errors().map_or(Ok(()), Err);
        drop(self);
        result
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
            loop {
                let state = &STATES[self.channel.dma()][self.channel.channel()];

                // Register the half-transfer waker
                let registered = state.half_waker.poll_wait(cx).is_pending();

                // Check if there's an error
                let t = self.channel.tcd();
                if let Some(errors) = self.channel.errors() {
                    return Poll::Ready(Err(errors));
                }

                // Check if we're past the half-way point
                let biter = t.tcd_biter_elinkno().read().biter();
                let citer = t.tcd_citer_elinkno().read().citer();
                let half_point = biter / 2;

                if self.channel.is_done() {
                    // Transfer completed before half-transfer
                    return Poll::Ready(Ok(false));
                } else if biter > 1 && citer <= half_point {
                    // We're past the half-way point
                    fence(Ordering::SeqCst);
                    return Poll::Ready(Ok(true));
                } else if registered {
                    return Poll::Pending;
                }
            }
        })
        .await
    }

    /// Abort the transfer and leave the channel ready for reuse.
    fn abort(&mut self) {
        self.channel.stop();
    }
}

/// A collection of [TransferError] returned by any transfer.
///
/// Each error variant can be queried separately, or all errors can be iterated by using [TransferErrors::into_iter].
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        self.channel.poll_complete(cx)
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        // `abort` retires accepted work and clears DONE/INT/NVIC bookkeeping.
        // Stored WaitCell wakes may remain; waiter registration consumes them
        // before waiting for a future event. Cheap after normal completion.
        self.abort();

        fence(Ordering::Release);
    }
}

/// Storage for finite, peripheral-paced DMA receive windows.
///
/// Can only be constructed by drivers in this HAL, and not from the application.
///
/// Hardware stops at the end of the allocation. The peripheral owner must
/// disable/read back its request source and pause DMA before inspecting or
/// copying data. Only a fully consumed or explicitly cleared window is reused.
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
    /// Positions in the current window, never reduced modulo capacity.
    read_pos: usize,
    write_pos: usize,
    /// Peripheral source retained for rearming the unused tail.
    peri_addr: *const W,
    /// No CPU slice may be created while hardware owns the window.
    armed: bool,
    /// Phantom data to tie the lifetime to the original buffer
    _lt: PhantomData<&'buf mut [W]>,
}

impl<'channel, 'buf, W: Word> RingBuffer<'channel, 'buf, W> {
    /// Create a new ring buffer for the given channel and buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The channel and peripheral request source are stopped
    /// - The buffer remains valid for the lifetime of the ring buffer
    /// - Only one RingBuffer exists per DMA channel at a time
    pub(crate) unsafe fn new(channel: DmaChannel<'channel>, buf: &'buf mut [W], peri_addr: *const W) -> Self {
        let buf_len = buf.len();
        Self {
            channel,
            buf: NonNull::from(buf),
            buf_len,
            read_pos: 0,
            write_pos: 0,
            peri_addr,
            armed: false,
            _lt: PhantomData,
        }
    }

    pub(crate) fn available(&self) -> usize {
        assert!(!self.armed);
        self.write_pos - self.read_pos
    }

    /// The peripheral owner must disable and read back its request source first.
    pub(crate) fn pause(&mut self) -> Result<(), Error> {
        if !self.armed {
            return Ok(());
        }
        self.channel.stop();
        self.armed = false;
        if self.channel.errors().is_some() {
            return Err(Error::BusError);
        }
        // A finite window leaves DADDR one-past-end on full completion. Never
        // reduce it modulo capacity: full and empty must remain distinguishable.
        let daddr = self.channel.tcd().tcd_daddr().read().daddr() as usize;
        let base = self.buf.as_ptr() as *const W as usize;
        let bytes = daddr.checked_sub(base).ok_or(Error::Configuration)?;
        let width = core::mem::size_of::<W>();
        if !bytes.is_multiple_of(width) {
            return Err(Error::Configuration);
        }
        let written = bytes / width;
        if written < self.write_pos || written > self.buf_len {
            return Err(Error::Configuration);
        }
        self.write_pos = written;
        Ok(())
    }

    /// Rearm only unused storage; do not recycle a full window's unread tail.
    ///
    /// # Safety
    /// The peripheral request source is disabled and read back. `peri_addr`
    /// remains valid and no CPU references into the allocation may escape.
    pub(crate) unsafe fn resume(&mut self) -> Result<bool, InvalidParameters> {
        assert!(!self.armed);
        if self.read_pos == self.buf_len {
            self.read_pos = 0;
            self.write_pos = 0;
        }
        if self.write_pos == self.buf_len {
            return Ok(false);
        }
        // SAFETY: DMA is retired, no source slice escapes read_quiescent, and
        // only the never-written tail is borrowed for the next finite window.
        let tail = unsafe {
            core::slice::from_raw_parts_mut(
                (self.buf.as_ptr() as *mut W).add(self.write_pos),
                self.buf_len - self.write_pos,
            )
        };
        unsafe {
            self.channel.setup_read_from_peripheral(
                self.peri_addr,
                tail,
                false,
                TransferOptions {
                    half_transfer_interrupt: true,
                    complete_transfer_interrupt: true,
                    priority: Priority::default(),
                },
            )?;
            // setup_read_from_peripheral uses DREQ and no destination rewind:
            // hardware stops at capacity even if no interrupt is serviced.
            self.channel.enable_interrupt();
            self.channel.enable_request();
        }
        self.armed = true;
        Ok(true)
    }

    pub(crate) fn read_quiescent(&mut self, dst: &mut [W]) -> usize {
        assert!(!self.armed);
        let n = dst.len().min(self.available());
        // SAFETY: pause retired DMA, and no new request can access this buffer
        // until resume. Unlike an optimistic copy/recheck, there is no race.
        let buf = unsafe { self.buf.as_ref() };
        dst[..n].copy_from_slice(&buf[self.read_pos..self.read_pos + n]);
        self.read_pos += n;
        n
    }

    pub(crate) fn clear_quiescent(&mut self) {
        assert!(!self.armed);
        self.read_pos = 0;
        self.write_pos = 0;
    }

    pub(crate) fn poll_wait(&self, cx: &mut Context<'_>) -> bool {
        let state = &STATES[self.channel.dma()][self.channel.channel()];
        let complete_registered = state.waker.poll_wait(cx).is_pending();
        let half_registered = state.half_waker.poll_wait(cx).is_pending();
        complete_registered && half_registered
    }
}

impl<W: Word> Drop for RingBuffer<'_, '_, W> {
    fn drop(&mut self) {
        // The peripheral wrapper disables and reads back its request first.
        self.channel.stop();
    }
}

impl<'a> DmaChannel<'a> {
    /// Configure a peripheral-paced circular read with two DMA-owned halves.
    ///
    /// The DMA ISR grants each completed buffer to the peripheral driver. The
    /// driver must call [`DmaChannel::commit_ping_pong_buffer`] after copying it,
    /// before DMA reaches the next buffer boundary. Failure to do so sets the
    /// sticky overrun field in [`PingPongStatus`] and stops hardware requests.
    /// The even buffer length becomes the major-loop iteration count; its midpoint
    /// raises the Buffer A interrupt and major completion raises the Buffer B interrupt.
    ///
    /// # Safety
    ///
    /// - `peri_addr` must remain valid while the transfer is active.
    /// - `buf` must remain valid and inaccessible through normal Rust references
    ///   until the channel is stopped.
    /// - The caller must disable the peripheral request source and stop the
    ///   channel before releasing or reusing `buf`.
    pub(crate) unsafe fn setup_ping_pong_read_from_peripheral<W: Word>(
        &mut self,
        peri_addr: *const W,
        buf: &mut [W],
        priority: Priority,
    ) -> Result<(), InvalidParameters> {
        if buf.len() < 2 || !buf.len().is_multiple_of(2) || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_circular_read_from_peripheral(
                peri_addr,
                buf,
                false,
                TransferOptions {
                    half_transfer_interrupt: true,
                    complete_transfer_interrupt: true,
                    priority,
                },
            )?;
        }

        set_ping_pong_status(self.dma(), self.channel(), Some(PingPongStatus::new()));
        Ok(())
    }

    /// Configure a circular DMA read from a peripheral without taking ownership of
    /// the channel or destination buffer.
    ///
    /// The destination address wraps after the major loop while the request remains
    /// enabled. Half and major-loop interrupts may therefore be used as ping-pong
    /// publication points without stopping the channel between buffer halves.
    ///
    /// # Safety
    ///
    /// - `peri_addr` must remain valid for peripheral reads while the transfer is active.
    /// - `buf` must remain valid and inaccessible through normal Rust references while
    ///   DMA is active.
    /// - The caller must stop the channel before releasing or reusing `buf`.
    pub(crate) unsafe fn setup_circular_read_from_peripheral<W: Word>(
        &mut self,
        peri_addr: *const W,
        buf: &mut [W],
        software: bool,
        options: TransferOptions,
    ) -> Result<(), InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }

        unsafe {
            self.setup_transfers(DmaTransferParameters {
                src_ptr: peri_addr,
                dst_ptr: buf.as_mut_ptr(),
                dst_count: buf.len(),
                src_incr: false,
                dst_incr: true,
                circular: true,
                software,
                options,
            });
        }

        // Half/major completion notifications are delivered through the channel IRQ.
        self.enable_interrupt();

        Ok(())
    }

    /// Create a receive window. The owner calls `resume` before enabling its
    /// peripheral request, and `pause` before copying or reusing storage.
    ///
    /// # Arguments
    ///
    /// * `peri_addr` - Peripheral register address to read from
    /// * `buf` - Nonempty destination buffer, at most DMA_MAX_TRANSFER_SIZE words
    ///
    /// # Returns
    ///
    /// A [`RingBuffer`] that can be used to read received data.
    ///
    /// # Safety
    ///
    /// - The peripheral address must be valid for reads.
    /// - The peripheral's DMA request must be disabled and read back, with the
    ///   channel stopped. Its mux must select the corresponding request source.
    pub(crate) unsafe fn setup_buffered_read<'buf, W: Word>(
        &mut self,
        peri_addr: *const W,
        buf: &'buf mut [W],
    ) -> Result<RingBuffer<'_, 'buf, W>, InvalidParameters> {
        if buf.is_empty() || buf.len() > DMA_MAX_TRANSFER_SIZE {
            return Err(InvalidParameters);
        }
        Ok(unsafe { RingBuffer::new(self.reborrow(), buf, peri_addr) })
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
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// #
/// # extern crate panic_halt;
/// # extern crate embassy_mcxa;
/// # extern crate embassy_executor;
/// # use panic_halt as _;
/// # use embassy_executor::Spawner;
/// # use embassy_hal_internal::Peri;
/// # use embassy_mcxa::clocks::config::Div8;
/// # use embassy_mcxa::config::Config;
/// # use embassy_mcxa::dma::{DmaChannel, ScatterGatherBuilder};
/// #
/// # #[embassy_executor::main]
/// # async fn main(_spawner: Spawner) {
/// #     let mut config = Config::default();
/// #     config.clock_cfg.sirc.fro_12m_enabled = true;
/// #     config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
/// #
/// #     let p = embassy_mcxa::init(config);
///     let src1 = [0x00u32; 128];
///     let src2 = [0xffu32; 128];
///     let src3 = [0x55u32; 128];
///
///     let mut dst1 = [0x00u32; 128];
///     let mut dst2 = [0x00u32; 128];
///     let mut dst3 = [0x00u32; 128];
///
///     let mut ch0 = DmaChannel::new(p.DMA_CH0);
///     let mut builder = ScatterGatherBuilder::<u32>::new();
///
///     // Add transfer segments
///     builder.add_transfer(&src1, &mut dst1);
///     builder.add_transfer(&src2, &mut dst2);
///     builder.add_transfer(&src3, &mut dst3);
///
///     // Build and execute
///     // SAFETY: aligned DMA-accessible storage stays live; the builder remains
///     // unmoved, sources immutable and destinations exclusive until the whole
///     // chain retires. This example does not forget the handle.
///     let transfer = unsafe { builder.build(ch0).unwrap() };
///     transfer.await.unwrap();
/// # }
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
    /// A `Transfer` future tracking final-chain completion or an error.
    ///
    /// # Safety
    ///
    /// - The builder's aligned TCD storage must be DMA-accessible and coherent.
    ///   Keep the builder at this exact address, live and unmodified: do not move,
    ///   drop, clear, rebuild, or otherwise mutate it while hardware can fetch
    ///   any descriptor in the chain.
    /// - All segment buffers must remain live, aligned, DMA-accessible and
    ///   coherent, with sources immutable and destinations exclusive to DMA.
    /// - Keep these guarantees until the entire chain and all accepted work
    ///   actually retire, including if the returned handle is dropped or
    ///   forgotten. Its borrow is only an aid, not a leak-safety argument.
    /// - Normal destruction drains the finite software-started chain, which may
    ///   wait for all segments. Forgetting the handle performs no retirement.
    pub unsafe fn build<'b>(&'b mut self, channel: DmaChannel<'b>) -> Result<Transfer<'b>, Error> {
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
        // - First TCD: ESG (no START - we add it manually after loading)
        // - Middle TCDs: ESG | START (auto-execute when loaded via S/G)
        // - Last TCD: INTMAJOR | START (auto-execute, no further linking)
        for i in 0..self.count {
            let is_first = i == 0;
            let is_last = i == self.count - 1;

            if is_first {
                if is_last {
                    // Only one TCD - no ESG, no START (we add START manually)
                    self.tcds[i].dlast_sga = 0;
                    self.tcds[i].csr = 0x0002; // INTMAJOR
                } else {
                    // First of multiple - ESG to link, no START (we add START manually)
                    self.tcds[i].dlast_sga = &self.tcds[i + 1] as *const Tcd as i32;
                    self.tcds[i].csr = 0x0010; // ESG; only the final TCD interrupts
                }
            } else if is_last {
                // Last TCD (not first) - no ESG, but START so it auto-executes
                self.tcds[i].dlast_sga = 0;
                self.tcds[i].csr = 0x0003; // INTMAJOR | START
            } else {
                // Middle TCD - ESG to link, and START so it auto-executes
                self.tcds[i].dlast_sga = &self.tcds[i + 1] as *const Tcd as i32;
                self.tcds[i].csr = 0x0011; // ESG | START
            }
        }

        channel.prepare_peripheral_scatter_gather();
        STATES[channel.dma()][channel.channel()]
            .software_scatter_gather
            .store(true, Ordering::Release);
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
        t.tcd_csr().modify(|w| w.set_start(true));

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
pub(crate) unsafe fn on_interrupt(dma: usize, channel: usize) {
    // Serialize the entire status/acknowledgement/latch transaction against
    // channel retirement and reuse in a higher-priority interrupt executor.
    critical_section::with(|_| {
        crate::perf_counters::incr_interrupt_edma0();

        let t = match dma {
            0 => pac::EDMA_TCD0.tcd(channel),
            #[cfg(feature = "_mcxa5xx")]
            1 => pac::EDMA_TCD1.tcd(channel),
            _ => unreachable!(),
        };

        let state = &STATES[dma][channel];
        let es = t.ch_es().read();
        if es.err() {
            // Error-only interrupts need not set INT or DONE. Preserve the entire
            // outcome before acknowledging anything. Keep ERR asserted until the
            // owner shuts down the peripheral and retires the channel; masking EEI
            // prevents an interrupt storm while retaining the hardware evidence.
            state.errors.fetch_or(es.0, Ordering::AcqRel);
            t.ch_csr().modify(|w| w.set_eei(false));
            waker(dma, channel).wake();
            half_waker(dma, channel).wake();
        }

        if !t.ch_int().read().int() {
            // The interrupt has been cleared, ignore this spurious interrupt.
            return;
        }

        if ping_pong_status(dma, channel).is_some() {
            // Clear the observed boundary before sampling progress. If another
            // boundary occurs while this ISR runs, hardware will latch CH_INT again
            // instead of that newer event being lost in this W1C operation.
            t.ch_int().write(|w| w.set_int(true));

            let done = t.ch_csr().read().done();
            let biter = t.tcd_biter_elinkno().read().biter();
            let citer = t.tcd_citer_elinkno().read().citer();

            let valid = biter >= 2 && biter.is_multiple_of(2) && citer <= biter;
            let another_boundary = t.ch_int().read().int();
            // `current` independently validates this phase classification. If ISR
            // latency spans another boundary, the sampled buffer will be unexpected
            // (or a previous grant will remain), deliberately producing an overrun.
            let completed = if !valid || another_boundary {
                None
            } else if done {
                // If DONE is set, it means Buffer B has completed.
                Some(PingPongSelector::BufferB)
            } else if citer > 0 && citer <= biter / 2 {
                // If CITER is in the first half, it means Buffer A is currently being transferred.
                Some(PingPongSelector::BufferA)
            } else if citer > biter / 2 {
                // B completed, but a new A request already cleared DONE.
                Some(PingPongSelector::BufferB)
            } else {
                None
            };

            let granted = match completed {
                Some(buffer) => update_ping_pong_status(dma, channel, |status| status.grant_completed(buffer)),
                None => Err(Error::Overrun),
            };
            if granted.is_err() {
                let _ = update_ping_pong_status(dma, channel, |status| {
                    status.set_overrun();
                    Ok(())
                });
                // The callback pends the owning peripheral ISR, which disables its
                // source, reads it back and retires this channel before reuse. Do
                // not clear ERQ ahead of the documented source/HRS handshake here.
            }

            return;
        }

        // Acknowledge the observed event before sampling terminal state. A later
        // completion either appears in the samples below or leaves INT asserted
        // for another dispatch; it cannot be erased by a trailing W1C.
        t.ch_int().write(|w| w.set_int(true));

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
                half_waker(dma, channel).wake();
            }
        }

        // Finite SG enables INTMAJOR only on its final TCD, so CH_INT
        // can be latched as whole-chain completion while that mode is active.
        let wake = if state.peripheral_scatter_gather_active.load(Ordering::Acquire) {
            state.peripheral_scatter_gather_done.store(true, Ordering::Release);
            true
        } else {
            t.ch_csr().read().done()
        };

        // If the transfer is done or if we're in peripheral
        if wake {
            crate::perf_counters::incr_interrupt_edma0_wake();
            waker(dma, channel).wake();
            half_waker(dma, channel).wake();
        }
    });
}

#[doc(hidden)]
#[macro_export]
/// Macro to generate DMA channel interrupt handlers.
macro_rules! impl_dma_interrupt_handler {
    ($irq:ident, $dma:expr, $ch:expr) => {
        #[cortex_m_rt::interrupt]
        fn $irq() {
            // SAFETY: The correct $ch is called as generated, We check that
            // the given callback is non-null before calling.
            unsafe {
                $crate::dma::on_interrupt($dma, $ch);

                // See https://doc.rust-lang.org/std/primitive.fn.html#casting-to-and-from-integers
                let cb: *mut () = $crate::dma::CALLBACKS[$dma][$ch].load(core::sync::atomic::Ordering::Acquire);
                if !cb.is_null() {
                    let cb: fn() = core::mem::transmute(cb);
                    (cb)();
                }
            }
        }
    };
}

// TODO(AJM): This is a gross, gross hack. This implements optional callbacks
// for DMA completion interrupts. This should go away once we switch to
// "in-band" DMA interrupt binding with `bind_interrupts!`.
pub(crate) static CALLBACKS: [[AtomicPtr<()>; DMA_CHANNEL_COUNT]; DMA_INSTANCE_COUNT] =
    [const { [const { AtomicPtr::new(core::ptr::null_mut()) }; DMA_CHANNEL_COUNT] }; DMA_INSTANCE_COUNT];
