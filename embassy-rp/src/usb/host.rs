use core::cell::{Cell, RefCell};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::host::{
    DeviceEvent, HostError, PipeError, SplitInfo, SplitSpeed, TimeoutConfig, UsbHostAllocator, UsbHostController,
    UsbPipe, pipe,
};
use embassy_usb_driver::{EndpointInfo, EndpointType, Speed};

/// Reduce a [`SplitInfo`] to the legacy "emit PRE packet" bit used by this
/// full-speed only controller. USB 1.1 §11.8.6: PRE is required when the
/// target device is low-speed and reached through a (full-speed) hub.
fn split_to_pre(split: Option<SplitInfo>) -> bool {
    matches!(split, Some(s) if s.device_speed() == SplitSpeed::Low)
}
use rp_pac::usb_dpram::vals::EpControlEndpointType;

use super::{BUS_WAKER, DPRAM_DATA_OFFSET, EP_COUNT, EP_IN_WAKERS, EP_MEMORY, EndpointBuffer, Instance};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::interrupt::{self};
use crate::peripherals::USB;
use crate::usb::EP_MEMORY_SIZE;
use crate::{Peri, RegExt};

/// Root port reset drive time (USB 2.0 §7.1.7.5, TDRSTR).
const ROOT_RESET_MS: u64 = 50;
/// Reset recovery time (USB 2.0 §7.1.7.5, TRSTRCY). A device need not answer any request
/// until this has elapsed after reset is deasserted.
const RESET_RECOVERY_MS: u64 = 10;
/// Register writes need two clk_usb cycles to transfer; 12 clk_sys cycles cover this at the supported 150 MHz maximum.
const USB_CLOCK_DELAY_CYCLES: u32 = 12;
const SIE_START_DELAY_CYCLES: u32 = 12;

// DPRAM layout - `EP_COUNT` blocks for control and interrupts, then remaining blocks
// allocatable to EPX pipes.
const EPX_BLOCK_SIZE: usize = 64;

/// First EPX buffer after the control and interrupt buffers.
const EPX_BUFFER_OFFSET: u16 = DPRAM_DATA_OFFSET + ((EP_COUNT - 1) * EPX_BLOCK_SIZE) as u16;

/// Number of allocatable EPX blocks.
const EPX_NUM_BLOCKS: usize = (EP_MEMORY_SIZE - EPX_BUFFER_OFFSET as usize) / EPX_BLOCK_SIZE;

/// Maximum number of concurrently allocated EPX pipes.
const EPX_MAX_PIPES: usize = 16;

/// NAK retry interval, stretched so a NAKing transfer cannot retry within the frame.
#[cfg(feature = "rp2040")]
const NAK_POLL_DELAY_YIELD: u16 = 300;
/// NAK retry interval outside a yield.
#[cfg(feature = "rp2040")]
const NAK_POLL_DELAY_NORMAL: u16 = 16;

/// Bits reserved per EPX pipe in [`EpxArbiter::error`].
const EPX_ERROR_BITS: usize = 4;
/// Mask of one pipe's error field.
const EPX_ERROR_MASK: u64 = (1 << EPX_ERROR_BITS) - 1;

/// Terminal errors the interrupt handler can record, packed [`EPX_ERROR_BITS`] wide.
/// Zero means no error, so the whole field clears to "nothing recorded".
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
enum EpxError {
    BadResponse = 1,
    DataToggle = 2,
}

impl EpxError {
    fn from_code(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::BadResponse),
            2 => Some(Self::DataToggle),
            _ => None,
        }
    }
}

impl From<EpxError> for PipeError {
    fn from(error: EpxError) -> Self {
        match error {
            EpxError::BadResponse => PipeError::BadResponse,
            EpxError::DataToggle => PipeError::DataToggleError,
        }
    }
}

/// Default control timeout, overriding the trait's 50 ms: devices may overrun the
/// USB 2.0 §9.2.6.4 response requirement while configuring.
#[allow(clippy::field_reassign_with_default)]
fn default_timeout() -> TimeoutConfig {
    let mut timeout = TimeoutConfig::default();
    timeout.no_data_timeout = core::time::Duration::from_millis(500);
    timeout
}

/// Who holds EPX, who is queued for it, and what the interrupt recorded for a
/// transfer that was stopped.
struct EpxArbiter {
    /// Bitset of EPX pipes whose transaction was stopped at a NAK boundary.
    yielded: u16,
    /// Bitset of EPX pipes queued in `wait_ready_for_transaction`.
    waiting: u16,
    /// Whether EPX has a transaction armed. Framing errors come from the shared RX
    /// engine, so they are only charged to EPX while it is mid-transaction.
    armed: bool,
    /// Bitset of EPX pipes whose packet landed as the transfer was being stopped.
    /// `STOP_TRANS` suppresses `trans_complete`, so the completion is reported here.
    completed: u16,
    /// Terminal error per EPX pipe, [`EPX_ERROR_BITS`] wide. Clearing the status bit is
    /// what stops the interrupt re-firing, so the reason is carried here instead.
    error: u64,
    /// Slot that last held EPX. The queue is served starting after it, so a pipe
    /// that just released goes to the back.
    last: usize,
    /// RP2040 only: set on the first SOF of a contended EPX transfer, so the
    /// second SOF with no completion in between can stop it.
    #[cfg(feature = "rp2040")]
    switch_requested: bool,
}

impl EpxArbiter {
    /// Arbitration state with nothing queued, held or recorded.
    const fn new() -> Self {
        Self {
            yielded: 0,
            waiting: 0,
            armed: false,
            completed: 0,
            error: 0,
            last: 0,
            #[cfg(feature = "rp2040")]
            switch_requested: false,
        }
    }

    /// Slot whose turn it is to take a free EPX: the first queued pipe after the one
    /// that last held it. `None` when nobody is queued.
    fn turn(&self) -> Option<usize> {
        if self.waiting == 0 {
            return None;
        }
        (1..=EPX_MAX_PIPES)
            .map(|step| (self.last + step) % EPX_MAX_PIPES)
            .find(|slot| self.waiting & (1 << slot) != 0)
    }

    /// Queue a pipe for EPX, or take it out of the queue.
    fn set_waiting(&mut self, slot: usize, waiting: bool) {
        if waiting {
            self.waiting |= 1 << slot;
        } else {
            self.waiting &= !(1 << slot);
        }
    }

    fn mark_yielded(&mut self, slot: usize) {
        self.yielded |= 1 << slot;
    }

    fn take_yielded(&mut self, slot: usize) -> bool {
        let was = self.yielded & (1 << slot) != 0;
        self.yielded &= !(1 << slot);
        was
    }

    fn mark_completed(&mut self, slot: usize) {
        self.completed |= 1 << slot;
    }

    fn take_completed(&mut self, slot: usize) -> bool {
        let was = self.completed & (1 << slot) != 0;
        self.completed &= !(1 << slot);
        was
    }

    fn mark_error(&mut self, slot: usize, error: EpxError) {
        let shift = slot * EPX_ERROR_BITS;
        self.error = (self.error & !(EPX_ERROR_MASK << shift)) | ((error as u64) << shift);
    }

    fn take_error(&mut self, slot: usize) -> Option<PipeError> {
        let shift = slot * EPX_ERROR_BITS;
        let code = (self.error >> shift) & EPX_ERROR_MASK;
        self.error &= !(EPX_ERROR_MASK << shift);
        EpxError::from_code(code).map(PipeError::from)
    }

    /// Void any outcome the interrupt recorded for a pipe's finished attempt.
    fn discard_outcome(&mut self, slot: usize) {
        self.take_completed(slot);
        self.take_error(slot);
        self.take_yielded(slot);
    }
}

/// Allocation state for the dedicated interrupt endpoints.
struct InterruptPipeState {
    /// Bitset indexed by endpoint index (1..EP_COUNT).
    allocated: AtomicU16,
    /// Pipes whose cancelled transfer completed before its data toggle was advanced.
    toggle_owed: AtomicU16,
}

impl InterruptPipeState {
    const fn new() -> Self {
        Self {
            allocated: AtomicU16::new(0),
            toggle_owed: AtomicU16::new(0),
        }
    }

    fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.toggle_owed.store(0, Ordering::Relaxed);
    }

    fn allocate(&self) -> Result<usize, HostError> {
        critical_section::with(|_| {
            let allocated = self.allocated.load(Ordering::Relaxed);
            let index = (1..EP_COUNT)
                .find(|index| allocated & (1 << index) == 0)
                .ok_or(HostError::OutOfPipes)?;
            self.allocated.store(allocated | (1 << index), Ordering::Relaxed);
            Ok(index)
        })
    }

    fn free(&self, index: usize) {
        critical_section::with(|_| {
            let bit = 1 << index;
            let allocated = self.allocated.load(Ordering::Relaxed);
            self.allocated.store(allocated & !bit, Ordering::Relaxed);
            let owed = self.toggle_owed.load(Ordering::Relaxed);
            self.toggle_owed.store(owed & !bit, Ordering::Relaxed);
        });
    }

    fn mark_toggle_owed(&self, index: usize) {
        critical_section::with(|_| {
            let owed = self.toggle_owed.load(Ordering::Relaxed);
            self.toggle_owed.store(owed | (1 << index), Ordering::Relaxed);
        });
    }

    fn take_toggle_owed(&self, index: usize) -> bool {
        critical_section::with(|_| {
            let bit = 1 << index;
            let owed = self.toggle_owed.load(Ordering::Relaxed);
            self.toggle_owed.store(owed & !bit, Ordering::Relaxed);
            owed & bit != 0
        })
    }

    fn clear_toggle_owed(&self, index: usize) {
        critical_section::with(|_| {
            let owed = self.toggle_owed.load(Ordering::Relaxed);
            self.toggle_owed.store(owed & !(1 << index), Ordering::Relaxed);
        });
    }
}

/// Per-instance state shared between [`Driver`], [`Allocator`] and [`Channel`].
pub struct HostState {
    /// Current channel with ongoing non-interrupt transfer. `0` means None.
    current_channel: AtomicUsize,
    interrupt_pipes: InterruptPipeState,
    /// Bitset of allocated EPX pipes, indexed by `Channel::index - EP_COUNT`.
    allocated_epx: AtomicU16,
    /// One waiter per EPX pipe. Completion is routed to the pipe that owns EPX,
    /// while releasing it wakes every contender.
    epx_wakers: [AtomicWaker; EPX_MAX_PIPES],
    /// Bitmap of used EPX buffer blocks, one bit per [`EPX_BLOCK_SIZE`] block.
    used_blocks: critical_section::Mutex<Cell<u64>>,
    /// EPX arbitration. One lock keeps multi-field decisions consistent; it also masks
    /// the interrupt, so a re-entrant borrow cannot happen.
    arbiter: critical_section::Mutex<RefCell<EpxArbiter>>,
}

impl HostState {
    /// Run `f` with the arbiter locked.
    fn with_arbiter<R>(&self, f: impl FnOnce(&mut EpxArbiter) -> R) -> R {
        critical_section::with(|cs| f(&mut self.arbiter.borrow(cs).borrow_mut()))
    }

    /// Create a new, reset host state.
    pub const fn new() -> Self {
        Self {
            current_channel: AtomicUsize::new(0),
            interrupt_pipes: InterruptPipeState::new(),
            allocated_epx: AtomicU16::new(0),
            epx_wakers: [const { AtomicWaker::new() }; EPX_MAX_PIPES],
            used_blocks: critical_section::Mutex::new(Cell::new(0)),
            arbiter: critical_section::Mutex::new(RefCell::new(EpxArbiter::new())),
        }
    }

    fn reset(&self) {
        self.current_channel.store(0, Ordering::Relaxed);
        self.interrupt_pipes.reset();
        self.allocated_epx.store(0, Ordering::Relaxed);
        critical_section::with(|cs| {
            *self.arbiter.borrow(cs).borrow_mut() = EpxArbiter::new();
            self.used_blocks.borrow(cs).set(0);
        });
    }

    /// Slot of an EPX pipe in [`HostState::epx_wakers`], or `None` for the idle
    /// sentinel and for interrupt pipes.
    fn epx_slot(index: usize) -> Option<usize> {
        index.checked_sub(EP_COUNT).filter(|slot| *slot < EPX_MAX_PIPES)
    }

    /// Wake the pipe currently holding EPX.
    fn wake_current_epx(&self) {
        if let Some(slot) = Self::epx_slot(self.current_channel.load(Ordering::Acquire)) {
            self.epx_wakers[slot].wake();
        }
    }

    /// Wake every allocated EPX pipe, so whoever is queued can claim EPX.
    fn wake_all_epx(&self) {
        let allocated = self.allocated_epx.load(Ordering::Acquire);
        for slot in 0..EPX_MAX_PIPES {
            if allocated & (1 << slot) != 0 {
                self.epx_wakers[slot].wake();
            }
        }
    }

    /// Slot whose turn it is to take a free EPX. `None` when nobody is queued.
    fn epx_turn(&self) -> Option<usize> {
        self.with_arbiter(|a| a.turn())
    }

    /// Record that the pipe holding EPX had its transaction stopped.
    fn mark_epx_yielded(&self, index: usize) {
        if let Some(slot) = Self::epx_slot(index) {
            self.with_arbiter(|a| a.mark_yielded(slot));
        }
    }

    /// Record a completion the hardware could not signal because EPX was stopped.
    fn mark_epx_completed(&self, index: usize) {
        if let Some(slot) = Self::epx_slot(index) {
            self.with_arbiter(|a| a.mark_completed(slot));
        }
    }

    /// Clear a pipe's completion bit, returning whether it was set.
    fn take_epx_completed(&self, slot: usize) -> bool {
        self.with_arbiter(|a| a.take_completed(slot))
    }

    /// Record a terminal error against the pipe currently holding EPX.
    fn mark_epx_error(&self, index: usize, error: EpxError) {
        if let Some(slot) = Self::epx_slot(index) {
            self.with_arbiter(|a| a.mark_error(slot, error));
        }
    }

    /// Take any terminal error recorded for a pipe.
    fn take_epx_error(&self, slot: usize) -> Option<PipeError> {
        self.with_arbiter(|a| a.take_error(slot))
    }

    /// Clear a pipe's yield bit, returning whether it was set.
    fn take_epx_yielded(&self, slot: usize) -> bool {
        self.with_arbiter(|a| a.take_yielded(slot))
    }

    /// Void any outcome the interrupt recorded for a pipe's finished attempt.
    fn discard_epx_outcome(&self, slot: usize) {
        self.with_arbiter(|a| a.discard_outcome(slot));
    }
}

/// Sealed extension of [`Instance`] exposing the per-peripheral [`HostState`].
#[allow(private_bounds)]
pub trait SealedHostInstance: Instance {
    #[doc(hidden)]
    fn host_state() -> &'static HostState;
}

impl SealedHostInstance for crate::peripherals::USB {
    fn host_state() -> &'static HostState {
        static STATE: HostState = HostState::new();
        &STATE
    }
}

/// RP2040 USB host driver handle.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    connected: bool,
}

impl<'d, T: SealedHostInstance> Driver<'d, T> {
    /// Create a new USB driver.
    pub fn new(_usb: Peri<'d, USB>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>>) -> Self {
        let regs = T::regs();

        // Reset the peripheral: zeroing its registers by hand would clobber those that
        // do not reset to zero, notably `USBPHY_TRIM` and `NAK_POLL`.
        crate::pac::RESETS.reset().modify(|w| w.set_usbctrl(true));
        crate::pac::RESETS.reset().modify(|w| w.set_usbctrl(false));
        while !crate::pac::RESETS.reset_done().read().usbctrl() {}

        // DPRAM is outside the peripheral reset, so clear the control area by hand.
        unsafe {
            let p = EP_MEMORY as *mut u32;
            for i in 0..DPRAM_DATA_OFFSET as usize / 4 {
                p.add(i).write_volatile(0)
            }
        }

        regs.usb_muxing().modify(|w| {
            w.set_to_phy(true);
            w.set_softcon(true);
        });
        regs.usb_pwr().modify(|w| {
            w.set_vbus_detect(true);
            w.set_vbus_detect_override_en(true);
        });
        regs.main_ctrl().modify(|w| {
            w.set_controller_en(true);
            w.set_host_ndevice(true);
            // RP2350 resets PHY isolation asserted; the PHY is unusable until cleared (§12.7.2).
            #[cfg(feature = "_rp235x")]
            w.set_phy_iso(false);
        });
        regs.sie_ctrl().modify(|w| {
            w.set_sof_en(true);
            w.set_keep_alive_en(true);
            w.set_pulldown_en(true);
        });

        regs.inte().write(|w| {
            w.set_buff_status(true);
            w.set_host_resume(true);
            w.set_error_data_seq(true);
            w.set_error_crc(true);
            w.set_error_bit_stuff(true);
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // Initialize the bus so that it signals that power is available
        BUS_WAKER.wake();

        // Reset per-instance allocator state.
        T::host_state().reset();

        Self {
            phantom: PhantomData,
            connected: false,
        }
    }
}

/// USB endpoint.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Channel<'d, T: SealedHostInstance, E, D> {
    _phantom: PhantomData<(&'d mut T, E, D)>,
    index: usize,
    buf: EndpointBuffer<T>,
    dev_addr: u8,

    max_packet_size: u16,
    ep_addr: u8,

    /// Interrupt endpoint poll interval
    interval: u8,

    /// DATA0-DATA1 state
    pid: bool,
    /// Send PRE packet
    pre: bool,
    timeout: TimeoutConfig,
    /// Per-transaction control response budget, or `None` outside a control transfer.
    control_timeout_us: Option<u64>,
}

impl<'d, T: SealedHostInstance, E: pipe::Type, D: pipe::Direction> Channel<'d, T, E, D> {
    fn start_control_timeout(&mut self, has_data: bool) {
        let timeout = if has_data {
            self.timeout.data_timeout
        } else {
            self.timeout.no_data_timeout
        };
        self.control_timeout_us = Some(timeout.as_micros().min(u64::MAX as u128) as u64);
    }

    fn stop_timed_out_transaction(&self) -> TransactionStatus {
        let regs = T::regs();
        let setup = regs.sie_ctrl().read().send_setup();
        regs.sie_ctrl().modify(|w| w.set_stop_trans(true));
        while regs.sie_ctrl().read().stop_trans() {}

        if setup && regs.sie_status().read().trans_complete() {
            regs.sie_status().write_clear(|w| w.set_trans_complete(true));
            TransactionStatus::Complete
        } else if !setup && !yield_epx::<T>(self.index) {
            T::host_state().take_epx_completed(self.epx_slot());
            TransactionStatus::Complete
        } else {
            // Clear global status so the next pipe cannot consume this transaction's result.
            regs.sie_status().write_clear(|w| {
                w.set_trans_complete(true);
                w.set_stall_rec(true);
                w.set_rx_timeout(true);
                w.set_rx_overflow(true);
            });
            regs.buff_status().write_clear(|w| w.0 = 0b11);
            TransactionStatus::Timeout
        }
    }

    /// [EP_MEMORY]-relative address
    fn new(index: usize, buf_addr: u16, buf_len: u16, ep_info: &EndpointInfo, dev_addr: u8, pre: bool) -> Self {
        // TODO: assert only in debug?
        assert!(ep_info.ep_type == E::ep_type());
        assert!(buf_addr + buf_len <= EP_MEMORY_SIZE as u16);
        assert!(ep_info.max_packet_size <= buf_len);

        if ep_info.ep_type == EndpointType::Interrupt {
            assert!(index > 0 && index < EP_COUNT);
        } else {
            assert!(index >= EP_COUNT && index < EP_COUNT + EPX_MAX_PIPES);
        }

        Self {
            _phantom: PhantomData,
            index,
            dev_addr,
            buf: EndpointBuffer {
                addr: buf_addr,
                len: buf_len,
                _phantom: PhantomData,
            },
            max_packet_size: ep_info.max_packet_size,
            // The register carries the endpoint number; direction is configured separately.
            ep_addr: u8::from(ep_info.addr) & 0x0f,
            interval: ep_info.interval_ms,
            pid: false,
            pre,
            timeout: default_timeout(),
            control_timeout_us: None,
        }
    }
}

type BufferControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpBufferControl, rp_pac::common::RW>;
type EpControlReg = rp_pac::common::Reg<rp_pac::usb_dpram::regs::EpControl, rp_pac::common::RW>;
type AddrControlReg = rp_pac::common::Reg<rp_pac::usb::regs::AddrEndpX, rp_pac::common::RW>;

/// Stop the in-flight EPX transaction at its next safe boundary so a queued pipe can take
/// the endpoint. Armed only while contended, or an idle bulk IN would trip it on every NAK.
fn arm_epx_yield<T: SealedHostInstance>() {
    #[cfg(feature = "_rp235x")]
    {
        // RP235x stops EPX at its next NAK and raises an interrupt.
        T::regs().nak_poll().modify(|w| w.set_stop_epx_on_nak(true));
        T::regs().inte().modify(|w| w.set_epx_stopped_on_nak(true));
    }
    #[cfg(feature = "rp2040")]
    {
        // RP2040 has no stop-on-NAK bit, so stretch the retry interval and watch frame
        // boundaries instead, as TinyUSB's fallback does.
        T::regs().nak_poll().write(|w| {
            w.set_delay_fs(NAK_POLL_DELAY_YIELD);
            w.set_delay_ls(NAK_POLL_DELAY_YIELD);
        });
        T::regs().inte().modify(|w| w.set_host_sof(true));
    }
}

/// Stop asking for a yield: EPX is ours, or nobody is queued for it.
fn disarm_epx_yield<T: SealedHostInstance>() {
    #[cfg(feature = "_rp235x")]
    {
        T::regs().nak_poll().modify(|w| w.set_stop_epx_on_nak(false));
        T::regs().inte().modify(|w| w.set_epx_stopped_on_nak(false));
    }
    #[cfg(feature = "rp2040")]
    {
        T::host_state().with_arbiter(|a| a.switch_requested = false);
        T::regs().inte().modify(|w| w.set_host_sof(false));
        T::regs().nak_poll().write(|w| {
            w.set_delay_fs(NAK_POLL_DELAY_NORMAL);
            w.set_delay_ls(NAK_POLL_DELAY_NORMAL);
        });
    }
}

/// Note that EPX made progress, so a pending yield request starts counting again.
#[allow(unused_variables)]
fn note_epx_progress<T: SealedHostInstance>() {
    #[cfg(feature = "rp2040")]
    T::host_state().with_arbiter(|a| a.switch_requested = false);
}

/// Record a terminal error against the EPX owner and wake it.
fn record_epx_error<T: SealedHostInstance>(error: EpxError) {
    let state = T::host_state();

    // With no EPX transaction in flight the error belongs to a polled endpoint. One
    // raised while EPX is also active is still charged here; the source is unknowable.
    if !state.with_arbiter(|a| a.armed) {
        return;
    }

    state.mark_epx_error(state.current_channel.load(Ordering::Acquire), error);
    state.wake_current_epx();
}

/// Hand EPX from the pipe at `index` to whoever is queued for it. Returns `true` when the
/// transfer was stopped short; a packet landing as the stop takes effect stays with its owner.
fn yield_epx<T: SealedHostInstance>(index: usize) -> bool {
    let state = T::host_state();

    // Sample after the stop. `full` is set by the processor when arming an OUT and by the
    // controller on a received IN, so what counts as "nothing moved" depends on direction.
    let buf_ctrl = T::dpram().ep_in_buffer_control(0);
    let bc = buf_ctrl.read();
    let is_out = T::regs().sie_ctrl().read().send_data();
    let completed = !bc.available(0) && if is_out { !bc.full(0) } else { bc.full(0) };

    if completed {
        // The stop suppressed `trans_complete`, so report it in software and let the
        // owner collect its packet. It keeps EPX until the transfer finishes.
        state.mark_epx_completed(index);
        state.wake_current_epx();
        return false;
    }

    state.mark_epx_yielded(index);

    // Nothing was transferred. Revoke the buffer before another pipe installs its own.
    buf_ctrl.modify(|w| w.set_available(0, false));
    state.current_channel.store(0, Ordering::Release);
    state.wake_all_epx();

    true
}

/// Queue membership for a pipe waiting on EPX. Clearing the bit on drop keeps a
/// cancelled wait from stalling the rotation on a pipe that will never claim it.
struct WaitTicket<T: SealedHostInstance> {
    slot: usize,
    _phantom: PhantomData<T>,
}

impl<T: SealedHostInstance> WaitTicket<T> {
    fn new(slot: usize) -> Self {
        T::host_state().with_arbiter(|a| a.set_waiting(slot, true));

        Self {
            slot,
            _phantom: PhantomData,
        }
    }
}

impl<T: SealedHostInstance> Drop for WaitTicket<T> {
    fn drop(&mut self) {
        T::host_state().with_arbiter(|a| a.set_waiting(self.slot, false));
    }
}

/// Outcome of one EPX transaction attempt.
enum TransactionStatus {
    /// The transaction ran to completion.
    Complete,
    /// EPX was taken away at a NAK boundary; the caller should retry.
    NakYield,
    /// The software response budget expired.
    Timeout,
}

/// Stops a dedicated interrupt endpoint if its transfer future is cancelled.
struct InterruptTransferGuard<T: SealedHostInstance> {
    index: usize,
    is_out: bool,
    buffer_control: BufferControlReg,
    active: bool,
    _phantom: PhantomData<T>,
}

impl<T: SealedHostInstance> InterruptTransferGuard<T> {
    fn arm(&mut self) {
        self.active = true;
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl<T: SealedHostInstance> Drop for InterruptTransferGuard<T> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let regs = T::regs();
        regs.int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() & !(1 << (self.index - 1)));
        });

        let buffer = self.buffer_control.read();
        let moved = !buffer.available(0) && if self.is_out { !buffer.full(0) } else { buffer.full(0) };
        if moved {
            T::host_state().interrupt_pipes.mark_toggle_owed(self.index);
        }

        self.buffer_control.modify(|w| {
            w.set_available(0, false);
            w.set_full(0, false);
        });
        regs.buff_status().write_clear(|w| w.0 = 0b11 << (self.index * 2));
    }
}

struct TransactionGuard<T: SealedHostInstance> {
    state: &'static HostState,
    index: usize,
    ep_control: EpControlReg,
    buffer_control: BufferControlReg,
    transaction_active: bool,
    _phantom: PhantomData<T>,
}

impl<T: SealedHostInstance> TransactionGuard<T> {
    fn arm(&mut self) {
        self.transaction_active = true;
        self.state.with_arbiter(|a| a.armed = true);
    }

    fn disarm(&mut self) {
        self.transaction_active = false;
        self.state.with_arbiter(|a| a.armed = false);
    }
}

impl<T: SealedHostInstance> Drop for TransactionGuard<T> {
    fn drop(&mut self) {
        let selected = self.state.current_channel.load(Ordering::Relaxed);
        if selected == self.index || selected == 0 {
            // Any armed transaction must be stopped before the endpoint registers are
            // torn down below, or the SIE may still be driving the bus while they change.
            if self.transaction_active {
                let regs = T::regs();
                regs.sie_ctrl().modify(|w| w.set_stop_trans(true));
                while regs.sie_ctrl().read().stop_trans() {}
                regs.sie_status().write_clear(|w| {
                    w.set_trans_complete(true);
                    w.set_stall_rec(true);
                    w.set_rx_timeout(true);
                    w.set_rx_overflow(true);
                });
                regs.buff_status().write_clear(|w| w.0 = 0b11);
            }
            if let Some(slot) = HostState::epx_slot(self.index) {
                self.state.with_arbiter(|a| a.last = slot);

                // This attempt is over. A cancelled transfer never consumes its recorded
                // outcome, and the next transaction would take it as its own.
                self.state.discard_epx_outcome(slot);
            }
            self.state.with_arbiter(|a| a.armed = false);
            self.state.current_channel.store(0, Ordering::Release);
            disarm_epx_yield::<T>();
            self.state.wake_all_epx();

            self.ep_control.modify(|w| {
                w.set_interrupt_per_buff(false);
                w.set_enable(false);
            });
            self.buffer_control.modify(|w| w.set_available(0, false));
        }
    }
}

impl<'d, T: SealedHostInstance, E: pipe::Type, D: pipe::Direction> Channel<'d, T, E, D> {
    /// Get channel waker
    fn waker(&self) -> &AtomicWaker {
        if Self::is_interrupt() {
            &EP_IN_WAKERS[self.index]
        } else {
            &T::host_state().epx_wakers[self.epx_slot()]
        }
    }

    /// Whether this pipe was made to yield EPX since the last check.
    fn take_nak_yield(&self) -> bool {
        T::host_state().take_epx_yielded(self.epx_slot())
    }

    /// Get buffer control register
    fn buffer_control(&self) -> BufferControlReg {
        let index = if Self::is_interrupt() {
            // Validated 1-15
            self.index
        } else {
            0
        };
        T::dpram().ep_in_buffer_control(index)
    }

    /// Give the buffer to the USB controller only after the rest of its
    /// control fields have crossed into the USB clock domain.
    fn write_buffer_control(&self, f: impl FnOnce(&mut rp_pac::usb_dpram::regs::EpBufferControl)) {
        let reg = self.buffer_control();
        let mut value = Default::default();
        f(&mut value);

        let available = value.available(0);
        value.set_available(0, false);
        reg.write_value(value);

        if available {
            cortex_m::asm::delay(USB_CLOCK_DELAY_CYCLES);
            value.set_available(0, true);
            reg.write_value(value);
        }
    }

    /// Get endpoint control register
    fn ep_control(&self) -> EpControlReg {
        if Self::is_interrupt() {
            T::dpram().ep_in_control(self.index - 1)
        } else {
            T::dpram_epx_control()
        }
    }

    /// Get interrupt endpoint address control
    fn addr_endp_host(&self) -> AddrControlReg {
        assert!(Self::is_interrupt());
        T::regs().addr_endp_x(self.index - 1)
    }

    fn is_interrupt() -> bool {
        E::ep_type() == EndpointType::Interrupt
    }

    /// Wait for buffer to be available
    /// Returns stall status
    async fn wait_available(&self) -> bool {
        trace!("CHANNEL {} WAIT AVAILABLE", self.index);
        poll_fn(|cx| {
            // Both IN and OUT endpoints use IN registers on rp2040 in host mode
            self.waker().register(cx.waker());

            let reg = self.buffer_control().read();

            // If waiting on current tx, clear interrupts
            if self.is_ready_for_transaction() {
                self.clear_sie_status();
            }

            // FIXME: Stall derived from other place
            match reg.available(0) {
                true => Poll::Pending,
                false => Poll::Ready(false),
            }
        })
        .await
    }

    /// Is hardware configured to perform transaction with this buffer
    /// Always true for INTERRUPT channel
    fn is_ready_for_transaction(&self) -> bool {
        if Self::is_interrupt() {
            true
        } else {
            let state = T::host_state();
            let sel = state.current_channel.load(Ordering::Relaxed);
            if sel == self.index {
                return true;
            }
            if sel != 0 {
                return false;
            }

            // EPX is free: take it only when the queue says it is our turn, so a
            // pipe that keeps re-arming cannot monopolise the endpoint.
            match state.epx_turn() {
                Some(slot) => slot == self.epx_slot(),
                None => true,
            }
        }
    }

    async fn wait_ready_for_transaction(&self) {
        trace!("CHANNEL {} WAIT READY", self.index);

        // Join the queue for EPX. Dropped with this future, cancelled or not.
        let _ticket = (!Self::is_interrupt()).then(|| WaitTicket::<T>::new(self.epx_slot()));
        // Wait for other transaction end
        poll_fn(|cx| {
            self.waker().register(cx.waker());

            if self.is_ready_for_transaction() {
                #[cfg(feature = "_rp235x")]
                if !Self::is_interrupt() {
                    disarm_epx_yield::<T>();
                }

                return Poll::Ready(());
            }

            trace!("CHANNEL {} EPX contention: request yield", self.index);
            arm_epx_yield::<T>();

            Poll::Pending
        })
        .await;

        // Once this pipe owns EPX, wait for its transfer buffer to be free.
        self.wait_available().await;
    }

    // FIXME: RX Timeout with LS device on hub
    /// Start transaction and wait it to be complete
    async fn wait_transaction(&self) -> Result<TransactionStatus, PipeError> {
        assert!(!Self::is_interrupt());
        let regs = T::regs();

        // Enable error and cplt interrupts
        regs.inte().modify(|w| {
            w.set_trans_complete(true);
            w.set_stall(true);
            w.set_error_rx_timeout(false);
            w.set_error_rx_overflow(true);
            w.set_error_crc(true);
            w.set_error_bit_stuff(true);
            w.set_error_data_seq(true);
        });

        // START_TRANS is synchronized separately (RP2040 §4.1.2.9, RP2350 §12.7.3.9).
        cortex_m::asm::delay(SIE_START_DELAY_CYCLES);
        T::regs().sie_ctrl().modify(|w| {
            w.set_start_trans(true);
        });

        trace!("CHANNEL {} WAIT TRANSACTION", self.index);
        let res = poll_fn(|cx| {
            self.waker().register(cx.waker());

            if let Some(error) = T::host_state().take_epx_error(self.epx_slot()) {
                return Poll::Ready(Err(error));
            }
            if T::host_state().take_epx_completed(self.epx_slot()) {
                return Poll::Ready(Ok(TransactionStatus::Complete));
            }
            if self.take_nak_yield() {
                return Poll::Ready(Ok(TransactionStatus::NakYield));
            }

            let stat = regs.sie_status().read();
            if stat.trans_complete() {
                regs.sie_status().write_clear(|w| w.set_trans_complete(true));
                return Poll::Ready(Ok(TransactionStatus::Complete));
            }
            if stat.stall_rec() {
                regs.sie_status().write_clear(|w| w.set_stall_rec(true));
                return Poll::Ready(Err(PipeError::Stall));
            }
            if stat.rx_overflow() {
                regs.sie_status().write_clear(|w| w.set_rx_overflow(true));
                return Poll::Ready(Err(PipeError::BufferOverflow));
            }
            Poll::Pending
        })
        .await;

        res
    }

    /// Mark this channel as currently used and configure endpoint type
    ///
    /// Call once on creation for interrupt pipe
    fn set_current(&self) {
        let regs = T::regs();
        trace!(
            "SET CURRENT: {:?} CHANNEL {}: dev: {}, ep: {}, max_packet: {}, preamble: {}",
            E::ep_type(),
            self.index,
            self.dev_addr,
            self.ep_addr,
            self.max_packet_size,
            self.pre
        );
        if Self::is_interrupt() {
            self.ep_control().write(|w| {
                w.set_endpoint_type(EpControlEndpointType::Interrupt);
                w.set_interrupt_per_buff(true);

                // `host_poll_interval` (bits 16:25) has no PAC accessor and counts from
                // zero, so clamp: a descriptor may declare an interval of 0.
                let interval = self.interval.max(1) as u32 - 1;
                w.0 |= interval << 16;

                w.set_buffer_address(self.buf.addr);
                w.set_enable(true);
            });

            // FIXME: What is this for?
            regs.sie_ctrl().modify(|w| w.set_sof_sync(true));

            self.addr_endp_host().write(|w| {
                w.set_address(self.dev_addr);
                w.set_endpoint(self.ep_addr);
                w.set_intep_dir(D::is_out());
                w.set_intep_preamble(self.pre)
            });
        } else {
            T::host_state().current_channel.store(self.index, Ordering::Relaxed);

            T::regs().addr_endp().write(|w| {
                w.set_address(self.dev_addr);
                w.set_endpoint(self.ep_addr);
            });

            self.ep_control().modify(|w| {
                w.set_enable(true);
                w.set_interrupt_per_buff(true);
                w.set_buffer_address(self.buf.addr);

                let epty = match E::ep_type() {
                    EndpointType::Control => EpControlEndpointType::Control,
                    EndpointType::Isochronous => EpControlEndpointType::Isochronous,
                    EndpointType::Bulk => EpControlEndpointType::Bulk,
                    EndpointType::Interrupt => EpControlEndpointType::Interrupt,
                };

                w.set_endpoint_type(epty);
            });

            regs.sie_ctrl().modify(|w| w.set_preamble_en(self.pre));
        }
    }

    fn transaction_guard(&self) -> TransactionGuard<T> {
        debug_assert!(!Self::is_interrupt());
        TransactionGuard {
            state: T::host_state(),
            index: self.index,
            ep_control: self.ep_control(),
            buffer_control: self.buffer_control(),
            transaction_active: false,
            _phantom: PhantomData,
        }
    }

    fn interrupt_transfer_guard(&self) -> InterruptTransferGuard<T> {
        debug_assert!(Self::is_interrupt());
        InterruptTransferGuard {
            index: self.index,
            is_out: D::is_out(),
            buffer_control: self.buffer_control(),
            active: false,
            _phantom: PhantomData,
        }
    }

    /// Apply a toggle consumed by a packet that completed during cancellation.
    fn settle_interrupt_toggle(&mut self) {
        if T::host_state().interrupt_pipes.take_toggle_owed(self.index) {
            self.advance_pid();
        }
    }

    /// Copy setup packet to buffer and set SETUP transaction
    ///
    /// Set PID = 1 for next transaction
    fn set_setup_packet(&mut self, setup: &[u8; 8]) {
        assert!(E::ep_type() == EndpointType::Control);
        let dpram = T::dpram();
        let value = u16::from_le_bytes([setup[2], setup[3]]);
        let index = u16::from_le_bytes([setup[4], setup[5]]);
        let length = u16::from_le_bytes([setup[6], setup[7]]);
        dpram.setup_packet_low().write(|w| {
            w.set_bmrequesttype(setup[0]);
            w.set_brequest(setup[1]);
            w.set_wvalue(value);
        });
        dpram.setup_packet_high().write(|w| {
            w.set_windex(index);
            w.set_wlength(length);
        });
        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_receive_data(false);
            w.set_send_setup(true);
        });
    }

    /// Reload interrupt channel buffer register
    fn interrupt_reload(&mut self) {
        assert!(E::ep_type() == EndpointType::Interrupt);
        self.write_buffer_control(|w| {
            w.set_last(0, true);
            w.set_pid(0, self.pid);
            w.set_full(0, false);
            w.set_reset(true);
            w.set_length(0, self.max_packet_size);
            w.set_available(0, true);
        });

        cortex_m::asm::delay(USB_CLOCK_DELAY_CYCLES);
        T::regs().int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() | 1 << (self.index - 1));
        });
    }

    /// Load an interrupt OUT buffer and arm it for the next poll.
    fn interrupt_send(&mut self, data: &[u8]) -> usize {
        assert!(Self::is_interrupt() && D::is_out());
        let chunk = &data[..data.len().min(self.max_packet_size as usize)];
        self.buf.write(chunk);

        self.write_buffer_control(|w| {
            w.set_last(0, chunk.len() == data.len());
            w.set_pid(0, self.pid);
            w.set_full(0, true);
            w.set_reset(true);
            w.set_length(0, chunk.len() as u16);
            w.set_available(0, true);
        });

        cortex_m::asm::delay(USB_CLOCK_DELAY_CYCLES);
        T::regs().int_ep_ctrl().modify(|w| {
            w.set_int_ep_active(w.int_ep_active() | 1 << (self.index - 1));
        });

        chunk.len()
    }

    /// Read one packet from a dedicated interrupt endpoint.
    async fn interrupt_in_read(&mut self, buf: &mut [u8]) -> Result<usize, PipeError> {
        self.wait_ready_for_transaction().await;
        self.set_current();
        self.settle_interrupt_toggle();
        let mut guard = self.interrupt_transfer_guard();
        guard.arm();

        let ctrl = self.buffer_control().read();
        if ctrl.full(0) {
            // A packet arrived while nobody was reading. Take it as it stands:
            // re-arming would clear `full` and discard what the device already sent.
        } else if ctrl.available(0) {
            // Already armed, so the controller owns the buffer and may fill it at
            // any moment. Writing to it here would race that; just wait.
            trace!("CHANNEL {} WAIT FOR INTERRUPT", self.index);
            self.wait_available().await;
        } else {
            // Idle: not armed and holding nothing, so the controller cannot be
            // touching the buffer and it is safe to program.
            trace!("CHANNEL {} ARM INTERRUPT", self.index);
            self.interrupt_reload();
            self.wait_available().await;
        }

        let rx_len = self.buffer_control().read().length(0) as usize;
        trace!("CHANNEL {} READ DONE, rx_len = {}", self.index, rx_len);
        if rx_len > buf.len() {
            return Err(PipeError::BufferOverflow);
        }

        self.buf.read(&mut buf[..rx_len]);
        self.advance_pid();
        self.interrupt_reload();
        guard.disarm();
        Ok(rx_len)
    }

    /// Write to a dedicated interrupt endpoint, one packet per poll.
    async fn interrupt_out_write(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError> {
        self.wait_ready_for_transaction().await;
        self.set_current();
        self.settle_interrupt_toggle();
        let mut guard = self.interrupt_transfer_guard();
        guard.arm();

        // Run once so an empty buffer sends a ZLP.
        let mut count = 0;
        let mut packet;
        loop {
            trace!("CHANNEL {} ARM INTERRUPT OUT", self.index);
            packet = self.interrupt_send(&buf[count..]);
            self.wait_available().await;
            self.advance_pid();
            count += packet;
            if count >= buf.len() {
                break;
            }
        }

        if ensure_transaction_end && packet == self.max_packet_size as usize {
            trace!("CHANNEL {} ARM INTERRUPT OUT ZLP", self.index);
            self.interrupt_send(&[]);
            self.wait_available().await;
            self.advance_pid();
        }

        guard.disarm();
        Ok(())
    }

    /// Set DATA IN transaction
    ///
    fn set_data_in(&mut self, len: u16, pid: bool) {
        assert!(E::ep_type() != EndpointType::Interrupt);
        let pid = if E::ep_type() == EndpointType::Isochronous {
            false
        } else {
            pid
        };

        self.write_buffer_control(|w| {
            w.set_pid(0, pid);
            w.set_full(0, false);
            w.set_length(0, len);
            w.set_last(0, true);
            w.set_reset(true);
            w.set_available(0, true);
        });

        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(false);
            w.set_send_setup(false);
            w.set_receive_data(true);
        });
    }

    /// Set DATA OUT transaction and copy data to buffer
    /// Returns count of copied bytes
    fn set_data_out(&mut self, data: &[u8], pid: bool) -> usize {
        assert!(E::ep_type() != EndpointType::Interrupt);
        let pid = if E::ep_type() == EndpointType::Isochronous {
            false
        } else {
            pid
        };

        let chunk = if data.len() > 0 {
            data.chunks(self.max_packet_size as _).next().unwrap()
        } else {
            &[]
        };

        self.buf.write(&chunk);

        self.write_buffer_control(|w| {
            w.set_available(0, true);
            w.set_pid(0, pid);
            w.set_full(0, true);
            w.set_length(0, chunk.len() as _);
            w.set_last(0, true);
            w.set_reset(true);
        });

        T::regs().sie_ctrl().modify(|w| {
            w.set_send_data(true);
            w.set_send_setup(false);
            w.set_receive_data(false);
        });

        chunk.len()
    }

    fn advance_pid(&mut self) {
        if E::ep_type() != EndpointType::Isochronous {
            self.pid = !self.pid;
        }
    }

    /// Clear buffer interrupt bit
    fn clear_sie_status(&self) {
        if Self::is_interrupt() {
            T::regs().buff_status().write_clear(|w| w.0 = 0b11 << self.index * 2);
        } else {
            T::regs().buff_status().write_clear(|w| w.0 = 0b11);
        }
    }

    /// Claim EPX and run one transaction on it. `arm` programs the buffer once this
    /// pipe owns the endpoint.
    async fn run_transaction(&mut self, mut arm: impl FnMut(&mut Self)) -> Result<(), PipeError> {
        let deadline = self
            .control_timeout_us
            .map(|us| embassy_time::Instant::now() + embassy_time::Duration::from_micros(us));

        loop {
            self.wait_ready_for_transaction().await;
            self.set_current();
            let mut guard = self.transaction_guard();

            arm(self);
            guard.arm();
            let result = if let Some(deadline) = deadline {
                match embassy_time::with_deadline(deadline, self.wait_transaction()).await {
                    Ok(result) => result,
                    Err(_) => Ok(self.stop_timed_out_transaction()),
                }
            } else {
                self.wait_transaction().await
            };

            // Only a completed transaction is known to have released the SIE, so any other
            // outcome leaves the guard armed and its `Drop` issues STOP_TRANS.
            match result? {
                TransactionStatus::Complete => {
                    guard.disarm();
                    return Ok(());
                }
                // Isochronous data is only valid in its own frame, so a yielded
                // transfer is dropped rather than replayed later.
                TransactionStatus::NakYield if E::ep_type() == EndpointType::Isochronous => {
                    return Err(PipeError::Canceled);
                }
                TransactionStatus::NakYield => {}
                TransactionStatus::Timeout => {
                    guard.disarm();
                    return Err(PipeError::Timeout);
                }
            }

            drop(guard);
            embassy_futures::yield_now().await;
        }
    }

    /// Receive one packet, returning its length.
    async fn transfer_in_packet(&mut self, len: u16, pid: bool) -> Result<usize, PipeError> {
        self.run_transaction(|s| s.set_data_in(len, pid)).await?;

        Ok(self.buffer_control().read().length(0) as usize)
    }

    /// Send one packet, returning how much of `data` it carried.
    async fn transfer_out_packet(&mut self, data: &[u8], pid: bool) -> Result<usize, PipeError> {
        let mut len = 0;
        self.run_transaction(|s| len = s.set_data_out(data, pid)).await?;

        Ok(len)
    }

    /// Read over EPX until the caller's buffer fills or the device sends a short packet.
    async fn epx_read(&mut self, buf: &mut [u8]) -> Result<usize, PipeError> {
        let mut count: usize = 0;
        let res = loop {
            trace!("CHANNEL {} START READ, len = {}", self.index, buf.len());
            let packet_len = core::cmp::min(buf.len() - count, self.max_packet_size as usize);
            let rx_len = self.transfer_in_packet(packet_len as u16, self.pid).await?;
            self.advance_pid();

            let free = &mut buf[count..];
            trace!("CHANNEL {} READ DONE, rx_len = {}", self.index, rx_len);

            if rx_len > free.len() {
                break Err(PipeError::BufferOverflow);
            }

            self.buf.read(&mut free[..rx_len]);
            count += rx_len;

            // If transfer is smaller than max_packet_size, we are done
            // If we have read buf.len() bytes, we are done
            if count == buf.len() || rx_len < self.max_packet_size as usize {
                break Ok(count);
            }
        };

        res
    }

    /// Write over EPX until the caller's buffer is drained.
    async fn epx_write(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError> {
        let mut count = 0;
        let res = loop {
            trace!("CHANNEL {} START WRITE", self.index);
            let packet = self.transfer_out_packet(&buf[count..], self.pid).await?;
            self.advance_pid();

            trace!("WRITE DONE, tx_len = {}", packet);

            count += packet;

            if count == buf.len() {
                if packet == self.max_packet_size as usize && ensure_transaction_end {
                    trace!("CHANNEL {} START ZLP WRITE", self.index);
                    self.transfer_out_packet(&[], self.pid).await?;
                    self.advance_pid();
                    trace!("ZLP WRITE DONE");
                }
                break Ok(());
            }
        };

        res
    }

    /// Send SETUP packet
    ///
    /// WARNING: This flips PID
    async fn send_setup(&mut self, setup: &[u8; 8]) -> Result<(), PipeError> {
        trace!("SEND SETUP");
        self.run_transaction(|s| s.set_setup_packet(setup)).await?;
        self.pid = true;

        Ok(())
    }

    /// Send status packet
    async fn control_status(&mut self, active_direction_out: bool) -> Result<(), PipeError> {
        // Status packet always have DATA1
        trace!("SEND STATUS");
        self.run_transaction(|s| {
            if active_direction_out {
                s.set_data_in(0, true);
            } else {
                s.set_data_out(&[], true);
            }
        })
        .await?;
        self.pid = false;

        Ok(())
    }
}

impl<'d, T: SealedHostInstance, E: pipe::Type, D: pipe::Direction> UsbPipe<E, D> for Channel<'d, T, E, D> {
    async fn control_in(&mut self, setup: &[u8; 8], buf: &mut [u8]) -> Result<usize, PipeError>
    where
        E: pipe::IsControl,
        D: pipe::IsIn,
    {
        trace!("CONTROL IN: {:?}", setup);
        let length = u16::from_le_bytes([setup[6], setup[7]]) as usize;
        if length > buf.len() {
            return Err(PipeError::BufferOverflow);
        }

        self.start_control_timeout(length != 0);
        let result = async {
            self.send_setup(setup).await?;
            let read = if length > 0 {
                self.request_in(&mut buf[..length]).await?
            } else {
                0
            };
            self.control_status(false).await?;
            Ok(read)
        }
        .await;
        self.control_timeout_us = None;
        result
    }

    async fn control_out(&mut self, setup: &[u8; 8], buf: &[u8]) -> Result<(), PipeError>
    where
        E: pipe::IsControl,
        D: pipe::IsOut,
    {
        trace!("CONTROL OUT: {:?}", setup);
        let length = u16::from_le_bytes([setup[6], setup[7]]) as usize;
        if length > buf.len() {
            return Err(PipeError::BufferOverflow);
        }

        self.start_control_timeout(length != 0);
        let result = async {
            self.send_setup(setup).await?;
            if length > 0 {
                self.request_out(&buf[..length], false).await?;
            }
            self.control_status(true).await?;
            Ok(())
        }
        .await;
        self.control_timeout_us = None;
        result
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, PipeError>
    where
        D: pipe::IsIn,
    {
        if Self::is_interrupt() {
            self.interrupt_in_read(buf).await
        } else {
            self.epx_read(buf).await
        }
    }

    async fn request_out(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError>
    where
        D: pipe::IsOut,
    {
        if Self::is_interrupt() {
            self.interrupt_out_write(buf, ensure_transaction_end).await
        } else {
            self.epx_write(buf, ensure_transaction_end).await
        }
    }

    fn set_timeout(&mut self, timeout: TimeoutConfig) {
        self.timeout = timeout;
    }

    fn reset_data_toggle(&mut self) {
        self.pid = false;
        if Self::is_interrupt() {
            T::host_state().interrupt_pipes.clear_toggle_owed(self.index);
        }
    }
}

impl<'d, T: SealedHostInstance, E, D> Channel<'d, T, E, D> {
    /// This pipe's slot in the EPX arrays. Only meaningful for non-interrupt pipes.
    fn epx_slot(&self) -> usize {
        self.index - EP_COUNT
    }
}

impl<'d, T: SealedHostInstance, E, D> Drop for Channel<'d, T, E, D> {
    fn drop(&mut self) {
        if self.index < EP_COUNT {
            // Disarm and clear stale state so the interrupt slot can be reused safely.
            let regs = T::regs();
            let dpram = T::dpram();

            regs.int_ep_ctrl().modify(|w| {
                w.set_int_ep_active(w.int_ep_active() & !(1 << (self.index - 1)));
            });
            dpram.ep_in_control(self.index - 1).write(|w| w.0 = 0);
            dpram.ep_in_buffer_control(self.index).write(|w| w.0 = 0);
            regs.buff_status().write_clear(|w| w.0 = 0b11 << (self.index * 2));

            T::host_state().interrupt_pipes.free(self.index);
        } else {
            let state = T::host_state();
            // Return the EPX buffer and the pipe slot to the pool.
            free_epx_mem(state, self.buf.addr, self.buf.len);
            state.discard_epx_outcome(self.epx_slot());
            critical_section::with(|_| {
                let epx = &state.allocated_epx;
                epx.store(
                    epx.load(Ordering::Relaxed) & !(1 << (self.epx_slot())),
                    Ordering::Relaxed,
                );
            });
            debug!(
                "EPX pipe FREE  slot {} (bitset {:04x})",
                self.epx_slot(),
                state.allocated_epx.load(Ordering::Relaxed)
            );
        }
    }
}

/// Pipe allocator handle for [`Driver`].
pub struct Allocator<'d, T: Instance> {
    phantom: PhantomData<&'d T>,
}

impl<'d, T: Instance> Clone for Allocator<'d, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'d, T: Instance> Copy for Allocator<'d, T> {}

/// Number of [`EPX_BLOCK_SIZE`] blocks needed to hold `len` bytes.
fn blocks_for(len: u16) -> usize {
    (len as usize).div_ceil(EPX_BLOCK_SIZE)
}

/// `used_blocks` mask for a run of `blocks` blocks starting at `start`.
fn block_mask(start: usize, blocks: usize) -> u64 {
    ((1u64 << blocks) - 1) << start
}

/// Allocate `len` bytes of EPX buffer memory, returning its [`EP_MEMORY`]-relative byte address.
/// First-fit over contiguous free blocks.
fn alloc_epx_mem(state: &HostState, len: u16) -> Result<u16, ()> {
    let blocks = blocks_for(len);
    if blocks == 0 || blocks > EPX_NUM_BLOCKS {
        error!("EPX buffer request of {} bytes is too large", len);
        return Err(());
    }
    critical_section::with(|cs| {
        let used_blocks = state.used_blocks.borrow(cs);
        let used = used_blocks.get();
        for start in 0..=(EPX_NUM_BLOCKS - blocks) {
            let mask = block_mask(start, blocks);
            if used & mask == 0 {
                used_blocks.set(used | mask);
                return Ok(EPX_BUFFER_OFFSET + (start * EPX_BLOCK_SIZE) as u16);
            }
        }
        error!("EPX buffer memory full");
        Err(())
    })
}

/// Free EPX buffer memory previously returned by [`alloc_epx_mem`].
fn free_epx_mem(state: &HostState, addr: u16, len: u16) {
    let blocks = blocks_for(len);
    let start = (addr - EPX_BUFFER_OFFSET) as usize / EPX_BLOCK_SIZE;
    let mask = block_mask(start, blocks);
    critical_section::with(|cs| {
        let used_blocks = state.used_blocks.borrow(cs);
        used_blocks.set(used_blocks.get() & !mask);
    });
}

impl<'d, T: SealedHostInstance> UsbHostAllocator<'d> for Allocator<'d, T> {
    type Pipe<E: pipe::Type, D: pipe::Direction> = Channel<'d, T, E, D>;

    fn alloc_pipe<E: pipe::Type, D: pipe::Direction>(
        &self,
        dev_addr: u8,
        endpoint: &EndpointInfo,
        split: Option<SplitInfo>,
    ) -> Result<Self::Pipe<E, D>, HostError> {
        let state = T::host_state();
        let pre = split_to_pre(split);
        if E::ep_type() == EndpointType::Interrupt {
            if endpoint.max_packet_size == 0 || endpoint.max_packet_size as usize > EPX_BLOCK_SIZE {
                return Err(HostError::InvalidDescriptor);
            }
            let index = state.interrupt_pipes.allocate()?;
            // Fixed layout: pipe index 1..EP_COUNT maps to block 0..EP_COUNT-1.
            let addr = DPRAM_DATA_OFFSET + (index as u16 - 1) * EPX_BLOCK_SIZE as u16;

            Ok(Channel::new(
                index,
                addr,
                EPX_BLOCK_SIZE as u16,
                endpoint,
                dev_addr,
                pre,
            ))
        } else {
            let index = critical_section::with(|_| {
                let alloc = state.allocated_epx.load(Ordering::Relaxed);
                let slot = alloc.trailing_ones() as usize;
                if slot >= EPX_MAX_PIPES {
                    return Err(HostError::OutOfPipes);
                }
                state.allocated_epx.store(alloc | (1 << slot), Ordering::Relaxed);
                Ok(EP_COUNT + slot)
            })?;
            let slot = index - EP_COUNT;
            // One buffer per pipe: a parked transfer's data must survive another pipe
            // taking EPX in the meantime.
            let len = (blocks_for(endpoint.max_packet_size) * EPX_BLOCK_SIZE) as u16;
            let addr = match alloc_epx_mem(state, len) {
                Ok(addr) => addr,
                Err(()) => {
                    // Hand back the slot claimed above, or repeated buffer failures
                    // would exhaust the bitset and report `OutOfPipes` instead.
                    critical_section::with(|_| {
                        let epx = &state.allocated_epx;
                        epx.store(epx.load(Ordering::Relaxed) & !(1 << slot), Ordering::Relaxed);
                    });

                    return Err(HostError::InsufficientMemory);
                }
            };
            debug!(
                "EPX pipe ALLOC slot {} (bitset {:04x}) buf {:#x}+{}",
                slot,
                state.allocated_epx.load(Ordering::Relaxed),
                addr,
                len
            );

            Ok(Channel::new(index, addr, len, endpoint, dev_addr, pre))
        }
    }
}

impl<'d, T: SealedHostInstance> UsbHostController<'d> for Driver<'d, T> {
    type Allocator = Allocator<'d, T>;

    fn allocator(&self) -> Self::Allocator {
        Allocator { phantom: PhantomData }
    }

    async fn wait_for_device_event(&mut self) -> DeviceEvent {
        let is_connected = |status: u8| match status {
            0b01 | 0b10 => true,
            _ => false,
        };

        let was = self.connected;

        // Clear interrupt status
        T::regs().sie_status().write_clear(|w| {
            w.set_speed(0b11);
        });

        // Enable conn/dis irq
        T::regs().inte().modify(|w| {
            w.set_host_conn_dis(true);
        });
        let ev = poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());

            let now = T::regs().sie_status().read().speed();
            let speed_now: DeviceEvent = match now {
                0b01 => DeviceEvent::Connected(Speed::Low),
                0b10 => DeviceEvent::Connected(Speed::Full),
                _ => DeviceEvent::Disconnected,
            };
            match (was, is_connected(now)) {
                (true, false) => Poll::Ready(DeviceEvent::Disconnected),
                (false, true) => Poll::Ready(speed_now),
                _ => Poll::Pending,
            }
        })
        .await;

        self.connected = matches!(ev, DeviceEvent::Connected(_));

        // Per the `UsbHostController` contract, reset before reporting the attach so the
        // device leaves Powered for Default (USB 2.0 §9.1.2). Full-speed only, so no chirp.
        if matches!(ev, DeviceEvent::Connected(_)) {
            self.bus_reset().await;
        }
        ev
    }

    async fn bus_reset(&mut self) {
        T::regs().sie_ctrl().modify(|w| {
            w.set_reset_bus(true);
        });

        embassy_time::Timer::after_millis(ROOT_RESET_MS).await;

        T::regs().sie_ctrl().modify(|w| {
            w.set_reset_bus(false);
        });

        embassy_time::Timer::after_millis(RESET_RECOVERY_MS).await;
    }
}

/// Service the RP235x stop-on-NAK interrupt, handing EPX to whoever is queued.
/// Returns `true` when it consumed the interrupt; RP2040 has no such interrupt.
fn on_nak_stop<T: SealedHostInstance>() -> bool {
    #[cfg(feature = "_rp235x")]
    {
        let regs = T::regs();
        if T::regs().ints().read().epx_stopped_on_nak() {
            let index = T::host_state().current_channel.load(Ordering::Acquire);

            // Acknowledge, stop asking, then hand the endpoint over.
            regs.nak_poll().modify(|w| w.set_epx_stopped_on_nak(true));
            disarm_epx_yield::<T>();
            let yielded = yield_epx::<T>(index);
            if !yielded {
                trace!("USB IRQ: EPx completed as it was stopped; keeping the packet");
            }

            trace!("USB IRQ: EPx stopped on NAK, yielded channel {}", index);
            BUS_WAKER.wake();
            return true;
        }
    }
    false
}

/// Service the SOF interrupt. RP235x only silences it; RP2040 uses frame boundaries
/// to stop a NAKing holder so a queued pipe can take EPX.
fn on_host_sof<T: SealedHostInstance>() -> &'static str {
    #[allow(unused)]
    let regs = T::regs();
    #[cfg(feature = "_rp235x")]
    {
        // Prevent nonstop SOF interrupt
        T::regs().inte().write_clear(|w| w.set_host_sof(true));
        "sof"
    }
    #[cfg(feature = "rp2040")]
    {
        // Reading SOF_RD acknowledges HOST_SOF on RP2040.
        let _ = regs.sof_rd().read();
        let state = T::host_state();
        let index = state.current_channel.load(Ordering::Acquire);

        if state.with_arbiter(|a| a.waiting) == 0 {
            // Nobody is queued for EPX, so there is nothing to yield to.
            disarm_epx_yield::<T>();
            "sof idle"
        } else if index == 0 {
            // EPX is free; the queue will claim it without a yield.
            state.with_arbiter(|a| a.switch_requested = false);
            "sof free"
        } else if state.with_arbiter(|a| a.switch_requested) {
            // Two frame boundaries with no completion between them means the
            // holder is retrying NAKs: safe to stop it here.
            regs.sie_ctrl().modify(|w| w.set_stop_trans(true));
            while regs.sie_ctrl().read().stop_trans() {}

            state.with_arbiter(|a| a.switch_requested = false);
            if yield_epx::<T>(index) {
                "sof yielded EPx"
            } else {
                "sof stop raced a completion; packet kept"
            }
        } else {
            state.with_arbiter(|a| a.switch_requested = true);
            "sof armed EPx yield"
        }
    }
}

/// USB interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _usb: PhantomData<T>,
}

impl<T: SealedHostInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let ints = regs.ints().read();

        // Hand EPX over if the controller stopped it at a NAK boundary.
        if on_nak_stop::<T>() {
            return;
        }

        let ev = {
            if ints.host_conn_dis() {
                regs.inte().write_clear(|w| w.set_host_conn_dis(true));
                match regs.sie_status().read().speed() {
                    0b01 => "attached low speed",
                    0b10 => "attached full speed",
                    _ => "detached",
                }
            } else if ints.host_resume() {
                regs.sie_status().write_clear(|w| w.set_resume(true));
                "resume"
            } else if ints.stall() {
                regs.inte().write_clear(|w| w.set_stall(true));
                T::host_state().wake_current_epx();
                "stall"
            } else if ints.error_rx_overflow() {
                regs.inte().write_clear(|w| w.set_error_rx_overflow(true));
                T::host_state().wake_current_epx();
                "rx overflow"
            } else if ints.trans_complete() {
                regs.inte().write_clear(|w| w.set_trans_complete(true));
                note_epx_progress::<T>();
                T::host_state().wake_current_epx();
                "transaction complete"
            } else if ints.error_rx_timeout() {
                regs.inte().write_clear(|w| w.set_error_rx_timeout(true));
                T::host_state().wake_current_epx();
                "rx timeout"
            } else if ints.buff_status() {
                let status = regs.buff_status().read().0;
                // Bits 0 and 1 are EPX's IN/OUT pair; from bit 2 up they are the
                // dedicated interrupt endpoints, two bits per endpoint. Either direction
                // can signal, so clear both bits in the pair.
                if status & 0b11 != 0 {
                    regs.buff_status().write_clear(|w| w.0 = status & 0b11);
                    trace!("USB IRQ: EPx");
                    T::host_state().wake_current_epx();
                }

                for n in 1..EP_COUNT {
                    if status & (0b11 << (n * 2)) != 0 {
                        regs.buff_status().write_clear(|w| w.0 = 0b11 << (n * 2));
                        trace!("USB IRQ: Interrupt EP {}", n);
                        EP_IN_WAKERS[n].wake();
                    }
                }
                "^^^"
            } else if ints.error_crc() {
                regs.sie_status().write_clear(|w| w.set_crc_error(true));
                record_epx_error::<T>(EpxError::BadResponse);
                "crc error"
            } else if ints.error_bit_stuff() {
                regs.sie_status().write_clear(|w| w.set_bit_stuff_error(true));
                record_epx_error::<T>(EpxError::BadResponse);
                "bit stuff error"
            } else if ints.error_data_seq() {
                regs.sie_status().write_clear(|w| w.set_data_seq_error(true));
                record_epx_error::<T>(EpxError::DataToggle);
                "data sequence error"
            } else if ints.host_sof() {
                on_host_sof::<T>()
            } else {
                "???"
            }
        };

        trace!("USB IRQ: {:08x} :: {}", ints.0, ev);

        BUS_WAKER.wake();
    }
}
