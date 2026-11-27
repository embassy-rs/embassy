//! Direct memory access (DMA) driver for the ASR6601.
//!
//! The two DMA controllers are Synopsys DesignWare AHB DMAC instances. Each
//! controller has four channels and one shared interrupt.
//!
//! PAC field accessors exist for controller-level registers (DMACFGREG,
//! CHENREG, CLEAR_TFR, CLEAR_BLOCK, CLEAR_ERR, MASK_TFR, MASK_BLOCK,
//! MASK_ERR, STATUS_TFR, STATUS_BLOCK, STATUS_ERR, STATUS_SRC_TRAN,
//! STATUS_DST_TRAN, CFG0-CFG3 per channel) and are used below. The
//! per-channel register fields that select a channel dynamically are written
//! with raw bit masks through those accessors.
//!
//! The PAC models each channel register as a single `u64` while the vendor
//! driver addresses split low/high halves; channel-register helpers below
//! bridge that gap.

use core::fmt;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicU8, Ordering, compiler_fence};
use core::task::{Context, Poll};

use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const CONTROLLER_COUNT: usize = 2;
const CHANNELS_PER_CONTROLLER: usize = 4;
const CHANNEL_COUNT: usize = CONTROLLER_COUNT * CHANNELS_PER_CONTROLLER;
// Hardware implements 11 BLOCK_TS bits; bit 43 reads zero and counts above
// 2047 truncate (2048 transfers zero items, 4095 transfers 2047).
const MAX_TRANSFER_COUNT: usize = 0x07ff;

// Bound for the suspend/disable acknowledges in `abort`. The normal path
// answers in a few cycles; the bound only converts a wedged channel (which
// today hangs forever inside `Drop`) into best-effort teardown.
const ABORT_POLL_LIMIT: u32 = 1_000_000;

// The channel registers are accessed as the vendor driver's split 32-bit
// low/high halves. The PAC describes the same bits as fields on the full
// 64-bit register, so these masks stay local to the split-register helpers.
const CTL_INT_ENABLE: u32 = 1 << 0;
const CTL_LLP_DEST_ENABLE: u32 = 1 << 27;
const CTL_LLP_SOURCE_ENABLE: u32 = 1 << 28;
const CTL_H_DONE: u32 = 1 << 12;

const CFG_L_CHANNEL_SUSPEND: u32 = 1 << 8;
const CFG_L_FIFO_EMPTY: u32 = 1 << 9;
const CFG_L_SOURCE_RELOAD: u32 = 1 << 30;
const CFG_L_DESTINATION_RELOAD: u32 = 1 << 31;

// Internal software state-machine values, not register fields.
const RESULT_IDLE: u8 = 0;
const RESULT_ACTIVE: u8 = 1;
const RESULT_COMPLETE: u8 = 2;
const RESULT_ERROR: u8 = 3;

const EVENT_TRANSFER: u8 = 0;
const EVENT_BLOCK: u8 = 1;

/// Width of one DMA transfer item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DataWidth {
    /// 8-bit transfer.
    Bits8 = 0,
    /// 16-bit transfer.
    Bits16 = 1,
    /// 32-bit transfer.
    Bits32 = 2,
}

impl DataWidth {
    const fn bytes(self) -> usize {
        1 << self as usize
    }
}

/// Number of items in a DMA burst.
///
/// These are the burst sizes documented by the ASR vendor driver.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BurstSize {
    /// One item per burst.
    #[default]
    One = 0,
    /// Four items per burst.
    Four = 1,
    /// Eight items per burst.
    Eight = 2,
}

/// Address update performed after each transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AddressMode {
    /// Increment by the configured transfer width.
    Increment = 0,
    /// Decrement by the configured transfer width.
    Decrement = 1,
    /// Keep the address fixed.
    Fixed = 2,
}

/// DMA transfer direction and flow controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Direction {
    /// Memory to memory, with the DMA as flow controller.
    MemoryToMemory = 0,
    /// Memory to peripheral, with the DMA as flow controller.
    MemoryToPeripheral = 1,
    /// Peripheral to memory, with the DMA as flow controller.
    PeripheralToMemory = 2,
    /// Peripheral to peripheral, with the DMA as flow controller.
    PeripheralToPeripheral = 3,
}

/// ASR6601 DMA request mux selection.
///
/// Shared with [`crate::syscfg::DmaRequest`] so the request table is defined once.
pub use crate::syscfg::DmaRequest as Request;

/// Automatic source and destination reload configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Reload {
    /// Reload the source address after each block.
    pub source: bool,
    /// Reload the destination address after each block.
    pub destination: bool,
}

impl Reload {
    const fn enabled(self) -> bool {
        self.source || self.destination
    }
}

/// Complete configuration for one DMA block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferConfig {
    /// Transfer direction.
    pub direction: Direction,
    /// Peripheral request. This must be `None` for memory-to-memory transfers.
    pub request: Option<Request>,
    /// Source item width.
    pub source_width: DataWidth,
    /// Destination item width.
    pub destination_width: DataWidth,
    /// Source burst size.
    pub source_burst: BurstSize,
    /// Destination burst size.
    pub destination_burst: BurstSize,
    /// Source address update.
    pub source_address: AddressMode,
    /// Destination address update.
    pub destination_address: AddressMode,
    /// Automatic block reload.
    pub reload: Reload,
}

impl TransferConfig {
    /// Create a memory-to-memory configuration.
    pub const fn memory_to_memory(width: DataWidth) -> Self {
        Self {
            direction: Direction::MemoryToMemory,
            request: None,
            source_width: width,
            destination_width: width,
            source_burst: BurstSize::One,
            destination_burst: BurstSize::One,
            source_address: AddressMode::Increment,
            destination_address: AddressMode::Increment,
            reload: Reload {
                source: false,
                destination: false,
            },
        }
    }

    /// Create a memory-to-peripheral configuration.
    pub const fn memory_to_peripheral(width: DataWidth, request: Request) -> Self {
        Self {
            direction: Direction::MemoryToPeripheral,
            request: Some(request),
            source_width: width,
            destination_width: width,
            source_burst: BurstSize::One,
            destination_burst: BurstSize::One,
            source_address: AddressMode::Increment,
            destination_address: AddressMode::Fixed,
            reload: Reload {
                source: false,
                destination: false,
            },
        }
    }

    /// Create a peripheral-to-memory configuration.
    pub const fn peripheral_to_memory(width: DataWidth, request: Request) -> Self {
        Self {
            direction: Direction::PeripheralToMemory,
            request: Some(request),
            source_width: width,
            destination_width: width,
            source_burst: BurstSize::One,
            destination_burst: BurstSize::One,
            source_address: AddressMode::Fixed,
            destination_address: AddressMode::Increment,
            reload: Reload {
                source: false,
                destination: false,
            },
        }
    }

    /// Create a peripheral-to-peripheral configuration.
    pub const fn peripheral_to_peripheral(width: DataWidth, request: Request) -> Self {
        Self {
            direction: Direction::PeripheralToPeripheral,
            request: Some(request),
            source_width: width,
            destination_width: width,
            source_burst: BurstSize::One,
            destination_burst: BurstSize::One,
            source_address: AddressMode::Fixed,
            destination_address: AddressMode::Fixed,
            reload: Reload {
                source: false,
                destination: false,
            },
        }
    }
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self::memory_to_memory(DataWidth::Bits8)
    }
}

/// Burst options used by the typed convenience methods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TransferOptions {
    /// Source burst size.
    pub source_burst: BurstSize,
    /// Destination burst size.
    pub destination_burst: BurstSize,
}

/// DMA configuration or transfer error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The transfer contains no items.
    EmptyTransfer,
    /// A block exceeds the hardware limit of 2047 source transfers.
    TooManyTransfers,
    /// The source address is not aligned to the source width.
    SourceMisaligned,
    /// The destination address is not aligned to the destination width.
    DestinationMisaligned,
    /// A peripheral transfer was configured without a request.
    MissingRequest,
    /// A memory-to-memory transfer was configured with a request.
    UnexpectedRequest,
    /// A pointer cannot be represented by the ASR6601's 32-bit DMA.
    AddressOutOfRange,
    /// The selected channel is already enabled.
    ChannelBusy,
    /// Source and destination slice lengths differ.
    LengthMismatch,
    /// The linked-list descriptor and block slice lengths differ.
    LinkedListLengthMismatch,
    /// A linked-list transfer has incompatible per-block routing settings.
    IncompatibleLinkedListConfig,
    /// Reload and linked-list modes were requested together.
    ReloadWithLinkedList,
    /// The selected channel cannot perform linked-list transfers.
    LinkedListUnsupported,
    /// The DMA reported a bus or descriptor error.
    Bus,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTransfer => f.write_str("empty DMA transfer"),
            Self::TooManyTransfers => f.write_str("DMA block exceeds 2047 transfers"),
            Self::SourceMisaligned => f.write_str("DMA source address is misaligned"),
            Self::DestinationMisaligned => f.write_str("DMA destination address is misaligned"),
            Self::MissingRequest => f.write_str("DMA peripheral request is missing"),
            Self::UnexpectedRequest => f.write_str("memory-to-memory DMA cannot use a peripheral request"),
            Self::AddressOutOfRange => f.write_str("DMA address is outside the 32-bit address space"),
            Self::ChannelBusy => f.write_str("DMA channel is busy"),
            Self::LengthMismatch => f.write_str("DMA source and destination lengths differ"),
            Self::LinkedListLengthMismatch => f.write_str("DMA linked-list lengths differ"),
            Self::IncompatibleLinkedListConfig => f.write_str("DMA linked-list routing differs between blocks"),
            Self::ReloadWithLinkedList => f.write_str("DMA reload and linked-list modes cannot be combined"),
            Self::LinkedListUnsupported => f.write_str("DMA linked-list transfers require channel 0"),
            Self::Bus => f.write_str("DMA bus or descriptor error"),
        }
    }
}

impl core::error::Error for Error {}

trait SealedWord {}

/// A type that can be transferred by the DMA.
#[allow(private_bounds)]
pub trait Word: SealedWord + Copy + 'static {
    /// DMA width for this type.
    const WIDTH: DataWidth;
}

impl SealedWord for u8 {}
impl Word for u8 {
    const WIDTH: DataWidth = DataWidth::Bits8;
}

impl SealedWord for u16 {}
impl Word for u16 {
    const WIDTH: DataWidth = DataWidth::Bits16;
}

impl SealedWord for u32 {}
impl Word for u32 {
    const WIDTH: DataWidth = DataWidth::Bits32;
}

/// One block used to build a linked-list transfer.
#[derive(Clone, Copy, Debug)]
pub struct LinkedListBlock {
    /// Source address used for the block.
    pub source: *const u8,
    /// Destination address used for the block.
    pub destination: *mut u8,
    /// Number of source transfers in the block.
    pub transfer_count: usize,
    /// Block configuration.
    pub config: TransferConfig,
}

impl LinkedListBlock {
    /// Create a linked-list block from raw addresses.
    pub const fn new(source: *const u8, destination: *mut u8, transfer_count: usize, config: TransferConfig) -> Self {
        Self {
            source,
            destination,
            transfer_count,
            config,
        }
    }
}

/// Hardware linked-list descriptor.
///
/// Keep descriptors in DMA-accessible RAM and do not move them while a linked
/// transfer is active. The transfer APIs enforce this for the descriptor slice.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(4))]
pub struct LinkedListItem {
    source: u32,
    destination: u32,
    next: u32,
    control_low: u32,
    control_high: u32,
    source_status: u32,
    destination_status: u32,
}

impl LinkedListItem {
    /// Create an empty descriptor suitable for array initialization.
    pub const fn new() -> Self {
        Self {
            source: 0,
            destination: 0,
            next: 0,
            control_low: 0,
            control_high: 0,
            source_status: 0,
            destination_status: 0,
        }
    }
}

impl Default for LinkedListItem {
    fn default() -> Self {
        Self::new()
    }
}

struct ChannelState {
    waker: AtomicWaker,
    result: AtomicU8,
    event: AtomicU8,
}

impl ChannelState {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            result: AtomicU8::new(RESULT_IDLE),
            event: AtomicU8::new(EVENT_TRANSFER),
        }
    }
}

static STATE: [ChannelState; CHANNEL_COUNT] = [const { ChannelState::new() }; CHANNEL_COUNT];

trait SealedControllerInstance {}

trait SealedChannelInstance {}

/// DMA controller instance.
#[allow(private_bounds)]
pub trait ControllerInstance: SealedControllerInstance + PeripheralType + 'static + Send {
    /// Shared interrupt for this controller.
    type Interrupt: TypelevelInterrupt;

    /// Controller number, zero or one.
    const NUMBER: usize;
}

/// DMA channel singleton instance.
#[allow(private_bounds)]
pub trait ChannelInstance: SealedChannelInstance + PeripheralType + 'static + Send {
    /// DMA controller containing this channel.
    type Controller: ControllerInstance;

    /// Channel number within the controller.
    const NUMBER: usize;
}

/// Interrupt handler for one DMA controller.
pub struct InterruptHandler<T: ControllerInstance> {
    _marker: PhantomData<T>,
}

impl<T: ControllerInstance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::NUMBER);
    }
}

/// Owned DMA channel.
pub struct Channel<'d, M: Mode> {
    controller: usize,
    channel: usize,
    _lifetime: PhantomData<&'d mut M>,
}

impl<'d> Channel<'d, Blocking> {
    /// Create a channel for polling and blocking transfers.
    ///
    /// # Safety contract
    ///
    /// The channel token proves exclusive ownership, but this driver erases
    /// it to controller/channel indices and cannot distinguish two tokens
    /// for the same channel minted via `Peripherals::steal`. Do not
    /// construct two live `Channel`s for one hardware channel; concurrent
    /// use programs the same registers and corrupts both transfers.
    pub fn new_blocking<T: ChannelInstance>(_channel: Peri<'d, T>) -> Self {
        Self::from_instance::<T>()
    }

    /// Start a raw transfer without enabling its interrupt.
    ///
    /// # Safety
    ///
    /// The source and destination must remain valid, correctly sized, and
    /// accessible by DMA until the returned transfer is dropped or completes.
    /// The caller must also uphold any peripheral-specific DMA requirements.
    pub unsafe fn start_transfer_raw<'a>(
        &'a mut self,
        source: *const u8,
        destination: *mut u8,
        transfer_count: usize,
        config: TransferConfig,
    ) -> Result<Transfer<'a, Blocking>, Error> {
        self.start_raw(source, destination, transfer_count, config, false)
    }

    /// Execute a raw transfer and wait for it to finish.
    ///
    /// # Safety
    ///
    /// The same requirements as [`Self::start_transfer_raw`] apply.
    pub unsafe fn blocking_transfer_raw(
        &mut self,
        source: *const u8,
        destination: *mut u8,
        transfer_count: usize,
        config: TransferConfig,
    ) -> Result<(), Error> {
        unsafe { self.start_transfer_raw(source, destination, transfer_count, config) }?.blocking_wait()
    }

    /// Start a memory-to-memory copy.
    pub fn start_copy<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: &'a mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'a, Blocking>, Error> {
        self.start_copy_inner(source, destination, options, false)
    }

    /// Copy a slice and wait for completion.
    pub fn blocking_copy<W: Word>(
        &mut self,
        source: &[W],
        destination: &mut [W],
        options: TransferOptions,
    ) -> Result<(), Error> {
        self.start_copy(source, destination, options)?.blocking_wait()
    }

    /// Start a peripheral-to-memory transfer.
    ///
    /// # Safety
    ///
    /// `source` must be a readable peripheral register of width `W` and must
    /// remain valid for the transfer.
    pub unsafe fn start_read<'a, W: Word>(
        &'a mut self,
        source: *const W,
        destination: &'a mut [W],
        request: Request,
        options: TransferOptions,
    ) -> Result<Transfer<'a, Blocking>, Error> {
        unsafe { self.start_read_inner(source, destination, request, options, false) }
    }

    /// Start a memory-to-peripheral transfer.
    ///
    /// # Safety
    ///
    /// `destination` must be a writable peripheral register of width `W` and
    /// must remain valid for the transfer.
    pub unsafe fn start_write<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: *mut W,
        request: Request,
        options: TransferOptions,
    ) -> Result<Transfer<'a, Blocking>, Error> {
        unsafe { self.start_write_inner(source, destination, request, options, false) }
    }

    /// Start a raw linked-list transfer without enabling its interrupt.
    ///
    /// # Safety
    ///
    /// Every block address must remain valid and DMA-accessible until the
    /// returned transfer is dropped or completes. Descriptors must be in
    /// DMA-accessible RAM.
    pub unsafe fn start_linked_transfer_raw<'a>(
        &'a mut self,
        descriptors: &'a mut [LinkedListItem],
        blocks: &'a [LinkedListBlock],
    ) -> Result<Transfer<'a, Blocking>, Error> {
        self.start_linked_raw(descriptors, blocks, false)
    }
}

impl<'d> Channel<'d, Async> {
    /// Create an interrupt-driven DMA channel.
    ///
    /// Same ownership contract as [`Channel::new_blocking`]: one live
    /// `Channel` per hardware channel.
    pub fn new<T: ChannelInstance>(
        _channel: Peri<'d, T>,
        _irq: impl Binding<<T::Controller as ControllerInstance>::Interrupt, InterruptHandler<T::Controller>> + 'd,
    ) -> Self {
        unsafe {
            <T::Controller as ControllerInstance>::Interrupt::enable();
        }
        Self::from_instance::<T>()
    }

    /// Start an asynchronous raw transfer.
    ///
    /// # Safety
    ///
    /// The source and destination must remain valid, correctly sized, and
    /// accessible by DMA until the returned future is dropped or completes.
    /// The caller must also uphold any peripheral-specific DMA requirements.
    pub unsafe fn transfer_raw<'a>(
        &'a mut self,
        source: *const u8,
        destination: *mut u8,
        transfer_count: usize,
        config: TransferConfig,
    ) -> Result<Transfer<'a, Async>, Error> {
        self.start_raw(source, destination, transfer_count, config, true)
    }

    /// Start an asynchronous memory-to-memory copy.
    pub fn copy<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: &'a mut [W],
        options: TransferOptions,
    ) -> Result<Transfer<'a, Async>, Error> {
        self.start_copy_inner(source, destination, options, true)
    }

    /// Start an asynchronous peripheral-to-memory transfer.
    ///
    /// # Safety
    ///
    /// `source` must be a readable peripheral register of width `W` and must
    /// remain valid for the transfer.
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        source: *const W,
        destination: &'a mut [W],
        request: Request,
        options: TransferOptions,
    ) -> Result<Transfer<'a, Async>, Error> {
        unsafe { self.start_read_inner(source, destination, request, options, true) }
    }

    /// Start an asynchronous memory-to-peripheral transfer.
    ///
    /// # Safety
    ///
    /// `destination` must be a writable peripheral register of width `W` and
    /// must remain valid for the transfer.
    pub unsafe fn write<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: *mut W,
        request: Request,
        options: TransferOptions,
    ) -> Result<Transfer<'a, Async>, Error> {
        unsafe { self.start_write_inner(source, destination, request, options, true) }
    }

    /// Start an asynchronous raw linked-list transfer.
    ///
    /// # Safety
    ///
    /// Every block address must remain valid and DMA-accessible until the
    /// returned future is dropped or completes. Descriptors must be in
    /// DMA-accessible RAM.
    pub unsafe fn linked_transfer_raw<'a>(
        &'a mut self,
        descriptors: &'a mut [LinkedListItem],
        blocks: &'a [LinkedListBlock],
    ) -> Result<Transfer<'a, Async>, Error> {
        self.start_linked_raw(descriptors, blocks, true)
    }
}

/// Generate low/high-half writers for one DMA channel register.
///
/// The regenerated PAC models each channel register (`SARn`, `DARn`, `LLPn`,
/// `CTLn`, `CFGn`) as a single `u64` with only the low 32 bits described,
/// replacing the old split `_l`/`_h` 32-bit registers. These helpers preserve
/// the old split-access semantics bit-for-bit via read-modify-write, so
/// existing write sequences (including write-low-half-only ones) behave
/// exactly as before.
macro_rules! ch_reg_w {
    ($write_lo:ident, $write_hi:ident, $r0:ident, $r1:ident, $r2:ident, $r3:ident) => {
        fn $write_lo(&self, value: u32) {
            let regs = self.regs();
            let value = u64::from(value);
            unsafe {
                match self.channel {
                    0 => regs.$r0().modify(|r, w| w.bits(r.bits() & !0xffff_ffff | value)),
                    1 => regs.$r1().modify(|r, w| w.bits(r.bits() & !0xffff_ffff | value)),
                    2 => regs.$r2().modify(|r, w| w.bits(r.bits() & !0xffff_ffff | value)),
                    _ => regs.$r3().modify(|r, w| w.bits(r.bits() & !0xffff_ffff | value)),
                };
            }
        }

        fn $write_hi(&self, value: u32) {
            let regs = self.regs();
            let value = u64::from(value) << 32;
            unsafe {
                match self.channel {
                    0 => regs.$r0().modify(|r, w| w.bits(r.bits() & 0xffff_ffff | value)),
                    1 => regs.$r1().modify(|r, w| w.bits(r.bits() & 0xffff_ffff | value)),
                    2 => regs.$r2().modify(|r, w| w.bits(r.bits() & 0xffff_ffff | value)),
                    _ => regs.$r3().modify(|r, w| w.bits(r.bits() & 0xffff_ffff | value)),
                };
            }
        }
    };
}

impl<'d, M: Mode> Channel<'d, M> {
    fn from_instance<T: ChannelInstance>() -> Self {
        Self {
            controller: T::Controller::NUMBER,
            channel: T::NUMBER,
            _lifetime: PhantomData,
        }
    }

    /// Controller number, zero or one.
    pub fn controller(&self) -> usize {
        self.controller
    }

    /// Channel number within the controller.
    pub fn number(&self) -> usize {
        self.channel
    }

    /// Reborrow this channel.
    pub fn reborrow(&mut self) -> Channel<'_, M> {
        Channel {
            controller: self.controller,
            channel: self.channel,
            _lifetime: PhantomData,
        }
    }

    fn state_index(&self) -> usize {
        self.controller * CHANNELS_PER_CONTROLLER + self.channel
    }

    fn state(&self) -> &'static ChannelState {
        &STATE[self.state_index()]
    }

    fn regs(&self) -> &'static pac::dmac0::RegisterBlock {
        controller_regs(self.controller)
    }

    ch_reg_w!(ch_sar_lo, ch_sar_hi, sar0, sar1, sar2, sar3);
    ch_reg_w!(ch_dar_lo, ch_dar_hi, dar0, dar1, dar2, dar3);
    ch_reg_w!(ch_llp_lo, ch_llp_hi, llp0, llp1, llp2, llp3);
    ch_reg_w!(ch_ctl_lo, ch_ctl_hi, ctl0, ctl1, ctl2, ctl3);

    fn ch_cfg_read(&self) -> u64 {
        match self.channel {
            0 => self.regs().cfg0().read().bits(),
            1 => self.regs().cfg1().read().bits(),
            2 => self.regs().cfg2().read().bits(),
            _ => self.regs().cfg3().read().bits(),
        }
    }

    ch_reg_w!(ch_cfg_lo, ch_cfg_hi, cfg0, cfg1, cfg2, cfg3);

    fn is_enabled(&self) -> bool {
        self.regs().chenreg().read().bits() & u64::from(channel_bit(self.channel)) != 0
    }

    fn start_copy_inner<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: &'a mut [W],
        options: TransferOptions,
        interrupts: bool,
    ) -> Result<Transfer<'a, M>, Error> {
        if source.len() != destination.len() {
            return Err(Error::LengthMismatch);
        }
        let mut config = TransferConfig::memory_to_memory(W::WIDTH);
        config.source_burst = options.source_burst;
        config.destination_burst = options.destination_burst;
        self.start_raw(
            source.as_ptr().cast(),
            destination.as_mut_ptr().cast(),
            source.len(),
            config,
            interrupts,
        )
    }

    unsafe fn start_read_inner<'a, W: Word>(
        &'a mut self,
        source: *const W,
        destination: &'a mut [W],
        request: Request,
        options: TransferOptions,
        interrupts: bool,
    ) -> Result<Transfer<'a, M>, Error> {
        let mut config = TransferConfig::peripheral_to_memory(W::WIDTH, request);
        config.source_burst = options.source_burst;
        config.destination_burst = options.destination_burst;
        self.start_raw(
            source.cast(),
            destination.as_mut_ptr().cast(),
            destination.len(),
            config,
            interrupts,
        )
    }

    unsafe fn start_write_inner<'a, W: Word>(
        &'a mut self,
        source: &'a [W],
        destination: *mut W,
        request: Request,
        options: TransferOptions,
        interrupts: bool,
    ) -> Result<Transfer<'a, M>, Error> {
        let mut config = TransferConfig::memory_to_peripheral(W::WIDTH, request);
        config.source_burst = options.source_burst;
        config.destination_burst = options.destination_burst;
        self.start_raw(
            source.as_ptr().cast(),
            destination.cast(),
            source.len(),
            config,
            interrupts,
        )
    }

    fn start_raw<'a>(
        &'a mut self,
        source: *const u8,
        destination: *mut u8,
        transfer_count: usize,
        config: TransferConfig,
        interrupts: bool,
    ) -> Result<Transfer<'a, M>, Error> {
        let event = if config.reload.enabled() {
            EVENT_BLOCK
        } else {
            EVENT_TRANSFER
        };
        self.prepare(source, destination, transfer_count, config, event, interrupts)?;
        self.enable();
        Ok(Transfer {
            channel: self.reborrow(),
            reload: config.reload.enabled(),
        })
    }

    fn start_linked_raw<'a>(
        &'a mut self,
        descriptors: &'a mut [LinkedListItem],
        blocks: &'a [LinkedListBlock],
        interrupts: bool,
    ) -> Result<Transfer<'a, M>, Error> {
        // The ASR6601 implements LLP only on channel 0 of each controller;
        // LLP1-3 read as zero and do not follow a descriptor chain.
        if self.channel != 0 {
            return Err(Error::LinkedListUnsupported);
        }
        if descriptors.len() != blocks.len() {
            return Err(Error::LinkedListLengthMismatch);
        }
        let first = blocks.first().ok_or(Error::EmptyTransfer)?;
        if first.config.reload.enabled() {
            return Err(Error::ReloadWithLinkedList);
        }

        for block in blocks {
            if block.config.reload.enabled() {
                return Err(Error::ReloadWithLinkedList);
            }
            if block.config.direction != first.config.direction || block.config.request != first.config.request {
                return Err(Error::IncompatibleLinkedListConfig);
            }
            validate_transfer(block.source, block.destination, block.transfer_count, block.config)?;
        }

        let descriptor_base = address_to_u32(descriptors.as_ptr() as usize)?;
        if descriptor_base & 0x03 != 0 {
            return Err(Error::AddressOutOfRange);
        }

        for index in 0..blocks.len() {
            let block = &blocks[index];
            let last = index + 1 == blocks.len();
            let next = if last {
                0
            } else {
                address_to_u32((&descriptors[index + 1]) as *const LinkedListItem as usize)?
            };
            descriptors[index] = LinkedListItem {
                source: address_to_u32(block.source as usize)?,
                destination: address_to_u32(block.destination as usize)?,
                next,
                control_low: control_low(block.config, !last),
                control_high: control_high(block.transfer_count),
                source_status: 0,
                destination_status: 0,
            };
        }

        self.prepare(
            first.source,
            first.destination,
            first.transfer_count,
            first.config,
            EVENT_TRANSFER,
            interrupts,
        )?;

        self.ch_llp_lo(descriptor_base);
        self.ch_llp_hi(0);
        self.ch_ctl_lo(control_low(first.config, true));
        compiler_fence(Ordering::SeqCst);
        self.enable();

        Ok(Transfer {
            channel: self.reborrow(),
            reload: false,
        })
    }

    fn prepare(
        &self,
        source: *const u8,
        destination: *mut u8,
        transfer_count: usize,
        config: TransferConfig,
        event: u8,
        interrupts: bool,
    ) -> Result<(), Error> {
        validate_transfer(source, destination, transfer_count, config)?;
        if self.is_enabled() {
            return Err(Error::ChannelBusy);
        }

        if let Some(request) = config.request {
            route_request(self.controller, self.channel, request);
        }

        self.mask_interrupts();
        self.clear_pending();

        let state = self.state();
        state.event.store(event, Ordering::Relaxed);
        state.result.store(RESULT_ACTIVE, Ordering::Release);

        let source_peripheral = if matches!(
            config.direction,
            Direction::PeripheralToMemory | Direction::PeripheralToPeripheral
        ) {
            self.channel as u32
        } else {
            0
        };
        let destination_peripheral = if matches!(
            config.direction,
            Direction::MemoryToPeripheral | Direction::PeripheralToPeripheral
        ) {
            self.channel as u32
        } else {
            0
        };

        let mut cfg_low = 0;
        if config.reload.source {
            cfg_low |= CFG_L_SOURCE_RELOAD;
        }
        if config.reload.destination {
            cfg_low |= CFG_L_DESTINATION_RELOAD;
        }
        let cfg_high = (source_peripheral << 7) | (destination_peripheral << 11);

        let source_addr = address_to_u32(source as usize)?;
        let destination_addr = address_to_u32(destination as usize)?;
        self.ch_sar_lo(source_addr);
        self.ch_sar_hi(0);
        self.ch_dar_lo(destination_addr);
        self.ch_dar_hi(0);
        self.ch_llp_lo(0);
        self.ch_llp_hi(0);
        self.ch_ctl_lo(control_low(config, false));
        self.ch_ctl_hi(control_high(transfer_count));
        self.ch_cfg_lo(cfg_low);
        self.ch_cfg_hi(cfg_high);

        compiler_fence(Ordering::SeqCst);

        if interrupts {
            set_mask(self.controller, MaskRegister::Error, self.channel, true);
            set_mask(
                self.controller,
                if event == EVENT_BLOCK {
                    MaskRegister::Block
                } else {
                    MaskRegister::Transfer
                },
                self.channel,
                true,
            );
        }

        Ok(())
    }

    fn enable(&self) {
        compiler_fence(Ordering::SeqCst);
        let regs = self.regs();
        if regs.dmacfgreg().read().bits() & 1 == 0 {
            unsafe {
                regs.dmacfgreg().write_with_zero(|w| w.bits(1));
            }
        }
        let bit = channel_bit(self.channel);
        unsafe {
            regs.chenreg()
                .write_with_zero(|w| w.bits(u64::from(bit) | (u64::from(bit) << 8)));
        }
        compiler_fence(Ordering::SeqCst);
    }

    fn poll_result(&self) -> Option<Result<(), Error>> {
        match self.state().result.load(Ordering::Acquire) {
            RESULT_COMPLETE => return Some(Ok(())),
            RESULT_ERROR => return Some(Err(Error::Bus)),
            _ => {}
        }

        let bit = channel_bit(self.channel);
        let bit64 = u64::from(bit);
        let regs = self.regs();
        if regs.status_err().read().bits() as u32 & bit != 0 {
            unsafe { regs.clear_err().write_with_zero(|w| w.bits(bit64)) };
            self.state().result.store(RESULT_ERROR, Ordering::Release);
            return Some(Err(Error::Bus));
        }

        let block_event = self.state().event.load(Ordering::Relaxed) == EVENT_BLOCK;
        let complete = if block_event {
            regs.status_block().read().bits() & u64::from(bit) != 0
        } else {
            regs.status_tfr().read().bits() & u64::from(bit) != 0
        };

        if complete || !self.is_enabled() {
            unsafe {
                if block_event {
                    regs.clear_block().write_with_zero(|w| w.bits(bit64))
                } else {
                    regs.clear_tfr().write_with_zero(|w| w.bits(bit64))
                }
            };
            self.state().result.store(RESULT_COMPLETE, Ordering::Release);
            return Some(Ok(()));
        }

        None
    }

    fn mask_interrupts(&self) {
        set_mask(self.controller, MaskRegister::Transfer, self.channel, false);
        set_mask(self.controller, MaskRegister::Block, self.channel, false);
        set_mask(self.controller, MaskRegister::Error, self.channel, false);
    }

    fn clear_pending(&self) {
        let bit = u64::from(channel_bit(self.channel));
        let regs = self.regs();
        unsafe {
            regs.clear_tfr().write_with_zero(|w| w.bits(bit));
            regs.clear_block().write_with_zero(|w| w.bits(bit));
            regs.clear_src_tran().write_with_zero(|w| w.bits(bit));
            regs.clear_dst_tran().write_with_zero(|w| w.bits(bit));
            regs.clear_err().write_with_zero(|w| w.bits(bit));
        }
    }

    /// Stop the channel and wait until it can no longer access memory.
    ///
    /// Best-effort on wedged hardware: the suspend/disable acknowledges are
    /// bounded, so a stuck channel cannot hang `Drop` forever.
    pub fn abort(&mut self) {
        self.mask_interrupts();

        if self.is_enabled() {
            self.ch_cfg_lo(self.ch_cfg_read() as u32 | CFG_L_CHANNEL_SUSPEND);
            for _ in 0..ABORT_POLL_LIMIT {
                if self.ch_cfg_read() & u64::from(CFG_L_FIFO_EMPTY) != 0 {
                    break;
                }
            }

            let bit = channel_bit(self.channel);
            unsafe {
                self.regs().chenreg().write_with_zero(|w| w.bits(u64::from(bit) << 8));
            }
            for _ in 0..ABORT_POLL_LIMIT {
                if !self.is_enabled() {
                    break;
                }
            }

            self.ch_cfg_lo(self.ch_cfg_read() as u32 & !CFG_L_CHANNEL_SUSPEND);
        }

        self.clear_pending();
        self.state().result.store(RESULT_IDLE, Ordering::Release);
        compiler_fence(Ordering::SeqCst);
    }
}

/// An in-progress DMA transfer.
///
/// Dropping this value is cancellation-safe: the channel is suspended, its FIFO
/// is drained, and the channel is disabled before borrowed memory is released.
#[must_use = "a DMA transfer stops when dropped"]
pub struct Transfer<'a, M: Mode> {
    channel: Channel<'a, M>,
    reload: bool,
}

impl<'a, M: Mode> Transfer<'a, M> {
    /// Return whether the transfer has reached its completion event.
    ///
    /// Idempotent peek behind a shared reference: `poll_result` caches the
    /// terminal outcome in channel state, so calling this and then awaiting
    /// the transfer reports the same result (no event is consumed).
    pub fn is_finished(&self) -> bool {
        self.channel.poll_result().is_some()
    }

    /// Wait synchronously for completion.
    ///
    /// With automatic reload enabled, completion means the first block has
    /// completed; this method then stops the repeating transfer before
    /// returning so the borrowed buffers are safe to release.
    pub fn blocking_wait(self) -> Result<(), Error> {
        compiler_fence(Ordering::SeqCst);
        let result = loop {
            if let Some(result) = self.channel.poll_result() {
                break result;
            }
        };
        compiler_fence(Ordering::SeqCst);
        if !self.reload {
            // A non-reloading channel disables itself on completion. Forget
            // instead of dropping so `Drop::abort` does not re-mask interrupts
            // and clear flags on a finished channel.
            core::mem::forget(self);
        }
        // For reloading transfers, `self` drops here and `Drop::abort` stops
        // the still-running channel before the borrow ends.
        result
    }

    /// Explicitly cancel the transfer.
    pub fn cancel(mut self) {
        self.channel.abort();
    }
}

impl<'a, M: Mode> Drop for Transfer<'a, M> {
    fn drop(&mut self) {
        self.channel.abort();
    }
}

impl<'a> Unpin for Transfer<'a, Async> {}

impl<'a> Future for Transfer<'a, Async> {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.channel.state();
        state.waker.register(cx.waker());
        compiler_fence(Ordering::SeqCst);

        match self.channel.poll_result() {
            Some(result) => Poll::Ready(result),
            None => Poll::Pending,
        }
    }
}

fn validate_transfer(
    source: *const u8,
    destination: *mut u8,
    transfer_count: usize,
    config: TransferConfig,
) -> Result<(), Error> {
    if transfer_count == 0 {
        return Err(Error::EmptyTransfer);
    }
    if transfer_count > MAX_TRANSFER_COUNT {
        return Err(Error::TooManyTransfers);
    }
    if !(source as usize).is_multiple_of(config.source_width.bytes()) {
        return Err(Error::SourceMisaligned);
    }
    if !(destination as usize).is_multiple_of(config.destination_width.bytes()) {
        return Err(Error::DestinationMisaligned);
    }
    address_to_u32(source as usize)?;
    address_to_u32(destination as usize)?;

    match (config.direction, config.request) {
        (Direction::MemoryToMemory, Some(_)) => Err(Error::UnexpectedRequest),
        (Direction::MemoryToMemory, None) => Ok(()),
        (_, None) => Err(Error::MissingRequest),
        (_, Some(_)) => Ok(()),
    }
}

fn address_to_u32(address: usize) -> Result<u32, Error> {
    u32::try_from(address).map_err(|_| Error::AddressOutOfRange)
}

fn control_low(config: TransferConfig, linked: bool) -> u32 {
    let (source_master, destination_master) = masters(config.direction);
    let mut value = CTL_INT_ENABLE
        | ((config.destination_width as u32) << 1)
        | ((config.source_width as u32) << 4)
        | ((config.destination_address as u32) << 7)
        | ((config.source_address as u32) << 9)
        | ((config.destination_burst as u32) << 11)
        | ((config.source_burst as u32) << 14)
        | ((config.direction as u32) << 20)
        | ((destination_master as u32) << 23)
        | ((source_master as u32) << 25);
    if linked {
        value |= CTL_LLP_DEST_ENABLE | CTL_LLP_SOURCE_ENABLE;
    }
    value
}

fn control_high(transfer_count: usize) -> u32 {
    transfer_count as u32 | CTL_H_DONE
}

fn masters(direction: Direction) -> (u8, u8) {
    match direction {
        Direction::MemoryToMemory => (1, 1),
        Direction::MemoryToPeripheral => (1, 0),
        Direction::PeripheralToMemory | Direction::PeripheralToPeripheral => (0, 0),
    }
}

fn route_request(controller: usize, channel: usize, request: Request) {
    let shift = 8 * (CHANNELS_PER_CONTROLLER - channel - 1);
    let mask = 0x3f_u32 << shift;
    let value = (request as u32) << shift;
    critical_section::with(|_| {
        let syscfg = unsafe { pac::Syscfg::steal() };
        if controller == 0 {
            syscfg
                .cr0()
                .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | value) });
        } else {
            syscfg
                .cr1()
                .modify(|r, w| unsafe { w.bits((r.bits() & !mask) | value) });
        }
    });
}

fn controller_regs(controller: usize) -> &'static pac::dmac0::RegisterBlock {
    let ptr = match controller {
        0 => pac::Dmac0::PTR,
        1 => pac::Dmac1::PTR,
        _ => unreachable!(),
    };
    unsafe { &*ptr }
}

fn channel_bit(channel: usize) -> u32 {
    1 << channel
}

#[derive(Clone, Copy)]
enum MaskRegister {
    Transfer,
    Block,
    Error,
}

fn set_mask(controller: usize, register: MaskRegister, channel: usize, enabled: bool) {
    let regs = controller_regs(controller);
    let bit = channel_bit(channel);
    // DW_ahb_dmac mask registers use bits 8..11 as write enables.
    let value = u64::from((bit << 8) | if enabled { bit } else { 0 });
    unsafe {
        match register {
            MaskRegister::Transfer => regs.mask_tfr().write_with_zero(|w| w.bits(value)),
            MaskRegister::Block => regs.mask_block().write_with_zero(|w| w.bits(value)),
            MaskRegister::Error => regs.mask_err().write_with_zero(|w| w.bits(value)),
        };
    }
}

fn on_interrupt(controller: usize) {
    let regs = controller_regs(controller);
    let errors = regs.status_err().read().bits() as u32 & 0x0f;
    let transfers = regs.status_tfr().read().bits() & 0x0f;
    let blocks = regs.status_block().read().bits() & 0x0f;

    if errors != 0 {
        unsafe { regs.clear_err().write_with_zero(|w| w.bits(u64::from(errors))) };
    }
    if transfers != 0 {
        unsafe { regs.clear_tfr().write_with_zero(|w| w.bits(transfers)) };
    }
    if blocks != 0 {
        unsafe { regs.clear_block().write_with_zero(|w| w.bits(blocks)) };
    }

    for channel in 0..CHANNELS_PER_CONTROLLER {
        let bit = channel_bit(channel);
        let state = &STATE[controller * CHANNELS_PER_CONTROLLER + channel];
        let complete = if state.event.load(Ordering::Relaxed) == EVENT_BLOCK {
            blocks & u64::from(bit) != 0
        } else {
            transfers & u64::from(bit) != 0
        };

        if errors & bit != 0 {
            state.result.store(RESULT_ERROR, Ordering::Release);
        } else if complete {
            state.result.store(RESULT_COMPLETE, Ordering::Release);
        } else {
            continue;
        }

        set_mask(controller, MaskRegister::Transfer, channel, false);
        set_mask(controller, MaskRegister::Block, channel, false);
        set_mask(controller, MaskRegister::Error, channel, false);
        state.waker.wake();
    }
}

/// Initialize both ASR6601 DMA controllers.
///
/// # Safety
///
/// This must be called exactly once, before any DMA channel is constructed.
pub(crate) unsafe fn init() {
    let rcc = unsafe { pac::Rcc::steal() };
    rcc.cgr0().modify(|_, w| {
        w.dmac0_clk_en()
            .set_bit()
            .dmac1_clk_en()
            .set_bit()
            .syscfg_clk_en()
            .set_bit()
    });

    // Reset is active low. Reset both controllers before exposing channels.
    rcc.rst1()
        .modify(|_, w| w.dmac0_rst_n().clear_bit().dmac1_rst_n().clear_bit());
    rcc.rst1()
        .modify(|_, w| w.dmac0_rst_n().set_bit().dmac1_rst_n().set_bit());

    pac::Interrupt::DMAC0.disable();
    pac::Interrupt::DMAC1.disable();

    for controller in 0..CONTROLLER_COUNT {
        let regs = controller_regs(controller);
        unsafe {
            // Disable all four channels using the CH_EN write-enable bits.
            regs.chenreg().write_with_zero(|w| w.bits(0x0f00));
        }
        while regs.chenreg().read().bits() & 0x0f != 0 {}

        set_mask(controller, MaskRegister::Transfer, 0, false);
        set_mask(controller, MaskRegister::Transfer, 1, false);
        set_mask(controller, MaskRegister::Transfer, 2, false);
        set_mask(controller, MaskRegister::Transfer, 3, false);
        set_mask(controller, MaskRegister::Block, 0, false);
        set_mask(controller, MaskRegister::Block, 1, false);
        set_mask(controller, MaskRegister::Block, 2, false);
        set_mask(controller, MaskRegister::Block, 3, false);
        set_mask(controller, MaskRegister::Error, 0, false);
        set_mask(controller, MaskRegister::Error, 1, false);
        set_mask(controller, MaskRegister::Error, 2, false);
        set_mask(controller, MaskRegister::Error, 3, false);

        unsafe {
            regs.clear_tfr().write_with_zero(|w| w.bits(0x0f));
            regs.clear_block().write_with_zero(|w| w.bits(0x0f));
            regs.clear_src_tran().write_with_zero(|w| w.bits(0x0f));
            regs.clear_dst_tran().write_with_zero(|w| w.bits(0x0f));
            regs.clear_err().write_with_zero(|w| w.bits(0x0f));
        }

        unsafe {
            regs.dmacfgreg().write_with_zero(|w| w.bits(1));
        }
    }

    for state in &STATE {
        state.result.store(RESULT_IDLE, Ordering::Relaxed);
        state.event.store(EVENT_TRANSFER, Ordering::Relaxed);
    }

    pac::Interrupt::DMAC0.unpend();
    pac::Interrupt::DMAC1.unpend();
    pac::Interrupt::DMAC0.set_priority(Priority::P2);
    pac::Interrupt::DMAC1.set_priority(Priority::P2);
}

macro_rules! controller {
    ($name:ident, $number:expr, $interrupt:ident) => {
        impl SealedControllerInstance for peripherals::$name {}

        impl ControllerInstance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$interrupt;
            const NUMBER: usize = $number;
        }
    };
}

macro_rules! channel {
    ($name:ident, $controller:ident, $number:expr) => {
        impl SealedChannelInstance for peripherals::$name {}

        impl ChannelInstance for peripherals::$name {
            type Controller = peripherals::$controller;
            const NUMBER: usize = $number;
        }
    };
}

controller!(DMA0, 0, DMAC0);
controller!(DMA1, 1, DMAC1);

channel!(DMA0_CH0, DMA0, 0);
channel!(DMA0_CH1, DMA0, 1);
channel!(DMA0_CH2, DMA0, 2);
channel!(DMA0_CH3, DMA0, 3);
channel!(DMA1_CH0, DMA1, 0);
channel!(DMA1_CH1, DMA1, 1);
channel!(DMA1_CH2, DMA1, 2);
channel!(DMA1_CH3, DMA1, 3);
