//! USB device driver for LPC55 (ip3511).
//!
//! The LPC55 carries two independent device controllers, both instances of the
//! same ip3511 design and both served by this driver through the [`Instance`]
//! trait:
//!
//! | Peripheral | Speed                  | EVK connector | Logical endpoints |
//! |------------|------------------------|---------------|-------------------|
//! | [`USB0`]   | full-speed, 12 Mbps    | P10           | 5 (EP0 + 4)       |
//! | [`USBHSD`] | high-speed, 480 Mbps   | P9            | 6 (EP0 + 5)       |
//!
//! [`USB0`]: crate::peripherals::USB0
//! [`USBHSD`]: crate::peripherals::USBHSD
//!
//! # Clocking
//!
//! The high-speed controller needs a system (main) clock of at least 96 MHz —
//! see [`crate::config::MainClock::FroHf96`] — and a 16 MHz crystal for its
//! PHY PLL. The full-speed controller needs neither: it drives itself from the
//! FRO-HF 96 MHz output divided to 48 MHz, which [`Driver::new`] enables
//! itself, so it works under [`crate::config::Config::default()`].
//!
//! # Maximum packet sizes
//!
//! | Endpoint type   | USB0 (FS) | USBHSD (HS) |
//! |-----------------|-----------|-------------|
//! | Bulk            | 64        | 512         |
//! | Interrupt       | 64        | 1024        |
//! | Isochronous OUT | 1023      | 1024        |
//! | Isochronous IN  | 1023      | 1023        |
//!
//! Control endpoints beyond EP0 are not allocatable; `embassy-usb` never asks
//! for one.
//!
//! # Transfer granularity
//!
//! [`driver::EndpointOut::read`] returns exactly one packet, as its contract
//! requires, and OUT slots are always armed for one packet.
//!
//! The high-speed controller can packetize a single endpoint command entry,
//! streaming several kilobytes without CPU involvement, and IN endpoints use
//! that: [`driver::EndpointIn::write`] accepts up to a slot's capacity
//! ([`Instance::BULK_IN_SLOT_LEN`], 3584 B) rather than rejecting anything over
//! the max packet size, and [`driver::EndpointIn::write_transfer`] chunks by
//! that capacity instead of by packet. Measured on an LPC55S69 at 480 Mbps over
//! CDC-ACM bulk, that is 44 MB/s against 23 MB/s for packet-at-a-time writes.
//! Accepting an oversized `write` is an extension beyond the trait contract, so
//! portable code should prefer `write_transfer`.
//!
//! OUT deliberately forgoes the same trick even though it would roughly double
//! the 18 MB/s that packet-at-a-time reads reach. A slot's window length is
//! committed when the slot is armed, which double buffering does one packet
//! ahead of the caller, whereas the room left in the caller's buffer is only
//! known when that slot is drained; the two cannot be reconciled without an
//! internal holding buffer. [`driver::EndpointOut::read_transfer`] therefore
//! keeps the trait's per-packet default.
//!
//! The full-speed controller has no packetizing capability at all: an entry
//! whose NBytes exceeds the max packet size never completes, so both directions
//! are one packet per call there.
//!
//! # Endpoint memory
//!
//! Each controller needs a region for its endpoint command/status list and
//! data buffers, chosen explicitly via [`Memory`]. [`Memory::usb1_sram`] hands
//! over the dedicated 16 KiB USB1 SRAM; [`Memory::buffer`] takes a
//! caller-owned buffer in main SRAM, which is what lets both controllers run
//! at the same time. `buffer` aligns the base itself, so a plain `[u8; N]`
//! will do; no `#[repr(align(..))]` newtype is needed.
//!
//! # Silicon errata (ES_LPC55S6x, applies to revisions 0A and 1B)
//!
//! - **USB.3** (§3.11): locking the FRO to USB SOF packets, which the
//!   full-speed bring-up enables via `FRO192M_CTRL.USBCLKADJ`, is not
//!   functional when the device is reached through more than one hub. Direct
//!   connections and single-hub paths are unaffected.
//! - **USB.5** (§3.22): a high-speed OUT transfer whose length is not a
//!   multiple of 8 has extra bytes written past its end, because the
//!   controller always writes 8 bytes at a time. Every endpoint buffer
//!   allocation is rounded up to 64 bytes, which absorbs the overrun.
//! - **USB.6** (§3.23): a high-speed isochronous IN endpoint sending a
//!   1024-byte packet raises no endpoint interrupt and leaves its
//!   command/status entry untouched. `alloc_endpoint` therefore caps
//!   high-speed isochronous IN at 1023 bytes, which is NXP's documented
//!   workaround.
//!
//! # Known limitations
//!
//! [`driver::Bus::poll`] derives [`Event::PowerDetected`] and
//! [`Event::PowerRemoved`] from `DEVCMDSTAT.VBUS_DEBOUNCED`, which is only
//! sampled when the controller raises a device-status interrupt. Losing VBUS
//! forces a connect-state change (hardware gates the D+ pull-up on
//! `VBUS_DEBOUNCED`), so that interrupt does arrive, but the driver never
//! polls VBUS on its own.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac;

/// Largest [`Instance::EP_COUNT`] across LPC55 controllers; sizes the static
/// per-endpoint arrays in [`State`].
const MAX_EP_COUNT: usize = 6;

/// Dedicated USB1 SRAM, usable as endpoint memory by either controller.
const USB1_SRAM_ADDR: u32 = 0x4010_0000;
const USB1_SRAM_SIZE: u32 = 0x4000;

/// Bytes reserved at the start of the endpoint memory for the command/status
/// list. Word layout per logical EP i: `[out_buf0, out_buf1, in_buf0,
/// in_buf1]`, so the list itself needs `EP_COUNT * 16` bytes; rounding the
/// reservation up to `MAX_EP_COUNT * 16` = 96 -> 128 keeps the first data
/// buffer 64-byte aligned for every instance. EP0's `out_buf1` slot holds the
/// SETUP buffer pointer rather than a command word.
const EP_LIST_LEN: u32 = 0x80;
const EP0_SETUP_OFFSET: u32 = EP_LIST_LEN;
const EP0_OUT_OFFSET: u32 = EP0_SETUP_OFFSET + 64;
const EP0_IN_OFFSET: u32 = EP0_OUT_OFFSET + 64;
const DATA_BUFFER_OFFSET: u32 = EP0_IN_OFFSET + 64;
const SLOT_RECLAIM_SPINS: usize = 1_000_000;

fn poll_bounded(iterations: usize, mut ready: impl FnMut() -> bool) -> bool {
    (0..iterations).any(|_| ready())
}

// Endpoint command/status word encoding. The bit flags below are common to
// both controllers; the NBytes and AddressOffset field positions differ and
// live on `Instance`.
const CMD_A: u32 = 1 << 31; // Active (HW clears on completion)
const CMD_D: u32 = 1 << 30; // Disabled
const CMD_S: u32 = 1 << 29; // Stall
const CMD_TR: u32 = 1 << 28; // Toggle Reset (write-1)
#[allow(dead_code)] // documented for completeness of the CS-word encoding
const CMD_TV: u32 = 1 << 27; // RF/TV: data toggle value for bulk/interrupt
const CMD_T: u32 = 1 << 26; // Endpoint type: 0 = generic, 1 = isochronous

/// A slot carrying either of these bits must not be armed or treated as
/// complete.
///
/// `S` and `A` share one command word on ip3511, so a host-requested stall
/// looks exactly like a completed transfer to a `poll_fn` that only tests `A`.
/// Re-arming such a slot would overwrite `S` and silently un-halt the endpoint,
/// so every read and write path reports [`EndpointError::Disabled`] instead -
/// the trait's only "not usable right now" signal. Note that `wait_enabled`
/// deliberately keeps testing `D` alone: a stalled endpoint is still enabled.
const CMD_UNUSABLE: u32 = CMD_D | CMD_S;

/// Per-endpoint state flags, kept in an `AtomicU8`.
///
/// Double-buffer bookkeeping for data endpoints is shared between [`Bus`]
/// (reset/enable/stall paths) and the endpoint futures. Hardware consumes the
/// two command/status entries of a double-buffered endpoint in strict EPINUSE
/// ping-pong order, so a 1-bit cursor kept in lockstep (resynced to slot 0
/// whenever EPINUSE is cleared) tracks the hardware exactly.
const SLOT: u8 = 1 << 0; // next slot to arm (IN) / consume (OUT)
const PRIMED0: u8 = 1 << 1; // OUT slot armed or holding an unread packet (`PRIMED0 << slot`)
const TR_PENDING: u8 = 1 << 3; // reset the data toggle on the next arm
const ISO: u8 = 1 << 4; // isochronous endpoint; survives enable/stall/reset

struct EndpointState {
    wakers: [AtomicWaker; MAX_EP_COUNT],
    state: [AtomicU8; MAX_EP_COUNT],
    generation: [AtomicU32; MAX_EP_COUNT],
}

impl EndpointState {
    const fn new() -> Self {
        Self {
            wakers: [const { AtomicWaker::new() }; MAX_EP_COUNT],
            state: [const { AtomicU8::new(0) }; MAX_EP_COUNT],
            generation: [const { AtomicU32::new(0) }; MAX_EP_COUNT],
        }
    }

    fn wake_all(&self) {
        for waker in &self.wakers {
            waker.wake();
        }
    }
}

#[derive(Clone, Copy)]
struct TransferGeneration {
    lifecycle: u32,
    endpoint: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ControlGeneration {
    lifecycle: u32,
    out: u32,
    in_: u32,
}

/// Shared driver state for one controller instance.
pub(crate) struct State {
    bus_waker: AtomicWaker,
    ep_in: EndpointState,
    ep_out: EndpointState,
    alive: AtomicBool,
    setup_pending: AtomicBool,
    lifecycle_generation: AtomicU32,
}

impl State {
    const fn new() -> Self {
        Self {
            bus_waker: AtomicWaker::new(),
            ep_in: EndpointState::new(),
            ep_out: EndpointState::new(),
            alive: AtomicBool::new(false),
            setup_pending: AtomicBool::new(false),
            lifecycle_generation: AtomicU32::new(0),
        }
    }

    fn endpoint(&self, dir: Direction) -> &EndpointState {
        match dir {
            Direction::Out => &self.ep_out,
            Direction::In => &self.ep_in,
        }
    }

    fn ep_state(&self, index: usize, dir: Direction) -> &AtomicU8 {
        &self.endpoint(dir).state[index]
    }

    fn ep_waker(&self, index: usize, dir: Direction) -> &AtomicWaker {
        &self.endpoint(dir).wakers[index]
    }

    fn ep_generation(&self, index: usize, dir: Direction) -> &AtomicU32 {
        &self.endpoint(dir).generation[index]
    }

    fn transfer_generation(&self, index: usize, dir: Direction) -> TransferGeneration {
        TransferGeneration {
            lifecycle: self.lifecycle_generation.load(Ordering::Relaxed),
            endpoint: self.ep_generation(index, dir).load(Ordering::Relaxed),
        }
    }

    fn transfer_generation_valid(&self, index: usize, dir: Direction, generation: TransferGeneration) -> bool {
        generation.lifecycle == self.lifecycle_generation.load(Ordering::Relaxed)
            && generation.endpoint == self.ep_generation(index, dir).load(Ordering::Relaxed)
    }

    fn control_generation(&self) -> ControlGeneration {
        ControlGeneration {
            lifecycle: self.lifecycle_generation.load(Ordering::Relaxed),
            out: self.ep_generation(0, Direction::Out).load(Ordering::Relaxed),
            in_: self.ep_generation(0, Direction::In).load(Ordering::Relaxed),
        }
    }

    fn wake_all(&self) {
        self.bus_waker.wake();
        self.ep_in.wake_all();
        self.ep_out.wake_all();
    }

    fn invalidate_controller(&self) {
        self.lifecycle_generation.fetch_add(1, Ordering::Relaxed);
        self.setup_pending.store(false, Ordering::Release);
        self.wake_all();
    }

    fn invalidate_endpoint(&self, index: usize, dir: Direction) {
        self.ep_generation(index, dir).fetch_add(1, Ordering::Relaxed);
        self.ep_waker(index, dir).wake();
    }

    fn note_setup(&self) {
        if !self.setup_pending.swap(true, Ordering::AcqRel) {
            self.ep_generation(0, Direction::Out).fetch_add(1, Ordering::Relaxed);
            self.ep_generation(0, Direction::In).fetch_add(1, Ordering::Relaxed);
            self.ep_waker(0, Direction::Out).wake();
            self.ep_waker(0, Direction::In).wake();
        }
    }
}

fn nbytes<T: Instance>(n: u32) -> u32 {
    (n & T::NBYTES_MASK) << T::NBYTES_SHIFT
}

fn remaining_bytes<T: Instance>(word: u32) -> u32 {
    (word >> T::NBYTES_SHIFT) & T::NBYTES_MASK
}

/// AddressOffset field: buffer address >> 6, masked to the instance's field
/// width. The upper address bits come from DATABUFSTART, which is why the
/// whole endpoint memory region must sit inside one DATABUFSTART window.
fn addroff<T: Instance>(buf_addr: u32) -> u32 {
    debug_assert_eq!(buf_addr & 0x3F, 0);
    (buf_addr >> 6) & T::ADDROFF_MASK
}

/// EPBUFCFG / EPINUSE cover the physical endpoints above EP0, two per logical
/// endpoint.
fn ep_buf_mask<T: Instance>() -> u16 {
    ((1u32 << (2 * (T::EP_COUNT - 1))) - 1) as u16
}

/// INTEN.EP_INT_EN covers every physical endpoint including EP0's pair.
fn ep_int_mask<T: Instance>() -> u16 {
    ((1u32 << (2 * T::EP_COUNT)) - 1) as u16
}

/// Volatile command/status word for one physical endpoint buffer slot.
#[derive(Clone, Copy)]
struct EndpointCommand(*mut u32);

impl EndpointCommand {
    fn new(ep_list: u32, index: usize, dir: Direction, slot: usize) -> Self {
        let direction = match dir {
            Direction::Out => 0,
            Direction::In => 2,
        };
        Self((ep_list as usize + index * 16 + (direction + slot) * 4) as *mut u32)
    }

    fn read(self) -> u32 {
        unsafe { self.0.read_volatile() }
    }

    fn write(self, word: u32) {
        unsafe { self.0.write_volatile(word) }
    }

    fn modify(self, f: impl FnOnce(u32) -> u32) {
        critical_section::with(|_| self.write(f(self.read())));
    }

    /// Arms one buffer slot of a data endpoint.
    ///
    /// The command word is written whole: the controller tracks the
    /// bulk/interrupt data toggle internally per endpoint (read-only EPTOGGLE
    /// register), not in the entry's TV bit, which is only consumed together
    /// with TR. `tr` requests that toggle reset; it is set exactly once for the
    /// first transfer after enable/unstall (the NXP SDK's deferred toggle-reset
    /// discipline).
    ///
    /// Isochronous endpoints are marked with `T` and never get `TR`: they have
    /// no data toggle to reset.
    fn arm<T: Instance>(self, len: u32, buf_addr: u32, tr: bool, iso: bool) {
        let flags = if iso {
            CMD_T
        } else if tr {
            CMD_TR
        } else {
            0
        };
        self.write(CMD_A | nbytes::<T>(len) | addroff::<T>(buf_addr) | flags);
    }
}

/// Fixed command-list and endpoint-zero buffer addresses.
#[derive(Clone, Copy)]
struct EndpointLayout {
    list: u32,
    setup: u32,
    out: u32,
    in_: u32,
}

impl EndpointLayout {
    fn new(base: u32) -> Self {
        Self {
            list: base,
            setup: base + EP0_SETUP_OFFSET,
            out: base + EP0_OUT_OFFSET,
            in_: base + EP0_IN_OFFSET,
        }
    }

    fn command(self, index: usize, dir: Direction, slot: usize) -> EndpointCommand {
        EndpointCommand::new(self.list, index, dir, slot)
    }
}

/// Waits for a slot to become inactive after registering its interrupt waker.
///
/// `validate` rechecks the transfer generation and software-selected slot
/// inside the same critical section as the hardware command word.
async fn wait_slot<T: Instance, R>(
    command: EndpointCommand,
    index: usize,
    dir: Direction,
    mut validate: impl FnMut() -> Result<R, EndpointError>,
) -> Result<(u32, R), EndpointError> {
    poll_fn(|cx| {
        let shared = T::state();
        shared.ep_waker(index, dir).register(cx.waker());

        critical_section::with(|_| {
            if !shared.alive.load(Ordering::Acquire) {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let value = match validate() {
                Ok(value) => value,
                Err(error) => return Poll::Ready(Err(error)),
            };
            let word = command.read();
            if word & CMD_UNUSABLE != 0 {
                Poll::Ready(Err(EndpointError::Disabled))
            } else if word & CMD_A == 0 {
                Poll::Ready(Ok((word, value)))
            } else {
                Poll::Pending
            }
        })
    })
    .await
}
fn valid_endpoint<T: Instance>(ep_addr: EndpointAddress) -> Option<(usize, Direction)> {
    let index = ep_addr.index();
    (index < T::EP_COUNT).then_some((index, ep_addr.direction()))
}

const DIRECTIONS: [Direction; 2] = [Direction::Out, Direction::In];

fn select_slot<T: Instance>(index: usize, dir: Direction, slot: usize) {
    if index == 0 {
        return;
    }
    let bit = 1 << (2 * index + (dir == Direction::In) as usize - 2);
    T::regs().epinuse().modify(|w| {
        let selected = if slot == 0 { w.buf() & !bit } else { w.buf() | bit };
        w.set_buf(selected);
    });
}

fn reclaim_slot<T: Instance>(ep_list: u32, index: usize, dir: Direction, slot: usize) {
    let regs = T::regs();
    let phy_ep = 2 * index + (dir == Direction::In) as usize;
    select_slot::<T>(index, dir, slot);
    if EndpointCommand::new(ep_list, index, dir, slot).read() & CMD_A == 0 {
        return;
    }

    let mask = 1 << phy_ep;
    regs.epskip().write_value(pac::usbhsd::regs::Epskip(mask));
    if poll_bounded(SLOT_RECLAIM_SPINS, || {
        regs.epskip().read().0 & mask == 0 && EndpointCommand::new(ep_list, index, dir, slot).read() & CMD_A == 0
    }) {
        return;
    }

    let direction = if dir == Direction::In { "IN" } else { "OUT" };
    panic!(
        "USB endpoint {} {} slot {} did not release: command={}, EPSKIP={}, EPINUSE={}, DEVCMDSTAT={}",
        index,
        direction,
        slot,
        EndpointCommand::new(ep_list, index, dir, slot).read(),
        regs.epskip().read().0,
        regs.epinuse().read().0,
        regs.devcmdstat().read().0
    );
}

fn reclaim_slots<T: Instance>(ep_list: u32, index: usize, dir: Direction) {
    let regs = T::regs();
    let phy_ep = 2 * index + (dir == Direction::In) as usize;
    if index == 0 {
        reclaim_slot::<T>(ep_list, index, dir, 0);
    } else {
        let bit = 1 << (phy_ep - 2);
        let current = ((regs.epinuse().read().buf() & bit) != 0) as usize;
        reclaim_slot::<T>(ep_list, index, dir, current);
        reclaim_slot::<T>(ep_list, index, dir, current ^ 1);
    }
    regs.intstat().write_value(pac::usbhsd::regs::Intstat(1 << phy_ep));
    select_slot::<T>(index, dir, 0);
}

fn reset_endpoint<T: Instance>(ep_list: u32, index: usize, dir: Direction) {
    EndpointCommand::new(ep_list, index, dir, 0).write(CMD_D);
    EndpointCommand::new(ep_list, index, dir, 1).write(CMD_D);
    let state = T::state().ep_state(index, dir);
    state.store(state.load(Ordering::Relaxed) & ISO, Ordering::Relaxed);
}

/// Copy into endpoint buffer memory using 32-bit accesses.
///
/// Endpoint memory sits behind the AHB matrix, so each access is a full bus
/// transaction: byte-wise copies are 4x the transactions and dominate the
/// per-packet cost at high speed. Endpoint buffers are 64-byte aligned and
/// rounded up to 64-byte multiples (see `alloc_buffer`), so word-aligned
/// stores plus a full-word tail never leave the allocation.
fn copy_to_ep_buffer(buf_addr: u32, data: &[u8]) {
    debug_assert!(buf_addr % 4 == 0);
    compiler_fence(Ordering::SeqCst);
    let mut dst = buf_addr as *mut u32;
    let mut chunks = data.chunks_exact(4);
    for c in &mut chunks {
        unsafe {
            dst.write_volatile(u32::from_le_bytes(c.try_into().unwrap()));
            dst = dst.add(1);
        }
    }
    let rem = chunks.remainder();
    if !rem.is_empty() {
        let mut tail = [0u8; 4];
        tail[..rem.len()].copy_from_slice(rem);
        unsafe { dst.write_volatile(u32::from_le_bytes(tail)) };
    }
    compiler_fence(Ordering::SeqCst);
}

/// Copy out of endpoint buffer memory using 32-bit accesses; see
/// [`copy_to_ep_buffer`] for the alignment/over-read reasoning.
fn copy_from_ep_buffer(buf_addr: u32, data: &mut [u8]) {
    debug_assert!(buf_addr % 4 == 0);
    compiler_fence(Ordering::SeqCst);
    let mut src = buf_addr as *const u32;
    let mut chunks = data.chunks_exact_mut(4);
    for c in &mut chunks {
        unsafe {
            c.copy_from_slice(&src.read_volatile().to_le_bytes());
            src = src.add(1);
        }
    }
    let rem = chunks.into_remainder();
    if !rem.is_empty() {
        let tail = unsafe { src.read_volatile() }.to_le_bytes();
        rem.copy_from_slice(&tail[..rem.len()]);
    }
    compiler_fence(Ordering::SeqCst);
}

/// RMW DEVCMDSTAT without accidentally acknowledging pending events.
///
/// `SETUP`, `DCON_C`, `DSUS_C` and `DRES_C` are write-1-to-clear: a naive
/// read-modify-write would clear whichever of them is currently pending.
/// This helper zeroes them in the read value before applying `f`, so a set
/// bit in the written value is always an intentional acknowledge by `f`.
fn devcmdstat_modify(regs: pac::usbhsd::Usbhsd, f: impl FnOnce(&mut pac::usbhsd::regs::Devcmdstat)) {
    critical_section::with(|_| {
        let r = regs.devcmdstat();
        let mut val = r.read();
        val.set_setup(false);
        val.set_dcon_c(false);
        val.set_dsus_c(false);
        val.set_dres_c(false);
        f(&mut val);
        r.write_value(val);
    });
}

/// Program the controller registers that describe the endpoint memory and
/// enable the device block. Run after every [`SealedInstance::power_up`].
fn program_controller<T: Instance>(ep_list: u32) {
    let regs = T::regs();

    // Both registers take the full region base; the hardware ignores the low
    // bits it does not implement. `write_value` keeps one code path for the
    // two instances, whose field decompositions differ.
    regs.databufstart()
        .write_value(pac::usbhsd::regs::Databufstart(ep_list));
    regs.epliststart().write_value(pac::usbhsd::regs::Epliststart(ep_list));

    // Non-control endpoints run double-buffered: EPBUFCFG has one bit per
    // physical endpoint (EP0 excluded), and hardware ping-pongs the two
    // command/status entries via EPINUSE.
    regs.epinuse().write(|w| w.set_buf(0));
    regs.epbufcfg().write(|w| w.set_buf_sb(ep_buf_mask::<T>()));
    regs.epskip().write_value(pac::usbhsd::regs::Epskip(0));

    // Enable the device controller, but do not connect yet. Preserve the
    // reset defaults of the other fields (notably LPM_SUP).
    devcmdstat_modify(regs, |w| w.set_dev_en(true));

    regs.intstat().write_value(pac::usbhsd::regs::Intstat(!0));
    regs.inten().write(|w| {
        w.set_ep_int_en(ep_int_mask::<T>());
        w.set_dev_int_en(true);
    });
}

trait SealedInstance {
    fn regs() -> pac::usbhsd::Usbhsd;
    fn state() -> &'static State;
    /// Power up clocks and the PHY for this controller.
    fn power_up(uses_usb1_sram: bool);
    /// Power the PHY down and gate this controller's clocks.
    fn power_down();
}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: crate::interrupt::typelevel::Interrupt;

    /// Number of logical endpoints (EP0 plus data endpoints).
    const EP_COUNT: usize;
    /// Whether this instance accepts only high-speed negotiation.
    const HIGH_SPEED: bool;
    /// NBytes field of the command/status word.
    const NBYTES_SHIFT: u32;
    /// Mask of the NBytes field, before shifting.
    const NBYTES_MASK: u32;
    /// AddressOffset field of the command/status word, at bit 0.
    const ADDROFF_MASK: u32;
    /// `log2` of the address window DATABUFSTART spans; the whole endpoint
    /// memory region must lie inside one such window.
    const DATABUF_WINDOW_SHIFT: u32;
    /// Largest max-packet-size accepted by `alloc_endpoint`, for bulk.
    const MAX_BULK_MPS: u16;
    /// Largest max-packet-size accepted by `alloc_endpoint`, for interrupt.
    const MAX_INTERRUPT_MPS: u16;
    /// Largest max-packet-size accepted by `alloc_endpoint`, for isochronous OUT.
    const MAX_ISO_OUT_MPS: u16;
    /// Largest max-packet-size accepted by `alloc_endpoint`, for isochronous IN.
    const MAX_ISO_IN_MPS: u16;
    /// Whether one command/status entry may span several max-packet-size
    /// packets.
    ///
    /// The ip3511-**HS** packetizes a multi-packet entry in hardware, streaming
    /// every packet without CPU involvement. The full-speed block has no such
    /// capability: an entry whose NBytes exceeds the max packet size simply
    /// never completes, so full-speed endpoints must be armed one packet at a
    /// time.
    ///
    /// Only IN endpoints use it. OUT slots are always armed for one packet, so
    /// that `read` can honour its single-packet contract.
    const MULTI_PACKET: bool;
    /// Multi-packet slot capacity for bulk IN endpoints. Inert when
    /// [`Instance::MULTI_PACKET`] is `false`.
    const BULK_IN_SLOT_LEN: u16;
}

impl SealedInstance for crate::peripherals::USBHSD {
    fn regs() -> pac::usbhsd::Usbhsd {
        pac::USBHSD
    }

    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }

    fn power_up(uses_usb1_sram: bool) {
        use pac::syscon::vals::{Usb1DevRst, Usb1HostRst, Usb1PhyRst, Usb1RamRst};

        critical_section::with(|_| {
            let users = USB1_SRAM_USERS.load(Ordering::Relaxed);
            let reset_ram = users == 0 || (uses_usb1_sram && users == 1);
            USBHSD_ACTIVE.store(true, Ordering::Release);
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_ram(true));

            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::Asserted));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::Released));
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::Asserted);
                if reset_ram {
                    w.set_usb1_ram_rst(Usb1RamRst::Asserted);
                }
            });
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::Released);
                if reset_ram {
                    w.set_usb1_ram_rst(Usb1RamRst::Released);
                }
            });
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::Asserted));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::Released));
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_host(true));
        });

        // Hand the shared USB1 port to the device controller; verify the
        // write sticks (it is silently dropped while the block is still
        // completing its reset). Use absolute writes: a read-modify-write on
        // the still-resetting block would write garbage read values back.
        let handed_over = poll_bounded(10_000, || {
            let mut pm = pac::usbhsh::regs::Portmode(0);
            pm.set_dev_enable(true);
            // Keep the PHY power-down input (PDCOM) under software control
            // (reset default), released. If hardware control is left to the
            // clock-gated host block, it powers the PHY down.
            pm.set_sw_ctrl_pdcom(true);
            pac::USBHSH.portmode().write_value(pm);
            pac::USBHSH.portmode().read().dev_enable()
        });
        if !handed_over {
            warn!("USB1 port handover to device controller failed");
        }
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_host(false));
        });

        // Power on the 32 MHz crystal + its LDO and route it to the USB PLL.
        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0(PDEN_XTAL32M | PDEN_LDOXO32M));
        pac::ANACTRL.xo32m_ctrl().modify(|w| w.set_enable_pll_usb_out(true));

        // Power on the HS PHY + its LDO.
        pac::PMC
            .pdruncfgclr0()
            .write(|w| w.set_pdruncfgclr0(PDEN_USBHSPHY | PDEN_LDOUSBHS));
        // >= 5 ms for the PHY to be ready (1.5M cycles = 5 ms at a worst-case
        // 300 MHz; proportionally longer at lower clocks, which is fine).
        cortex_m::asm::delay(1_500_000);

        // PHY register clock on, only now that the analog is powered.
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_phy(true));
        });

        // PHY initialization (order mirrors lpc55-hal `enabled_as_device`).
        let phy = pac::USBPHY;
        // Leave soft reset and open the clock gate in one full write, before
        // touching the PLL (lpc55-hal writes CTRL = 0 here).
        phy.ctrl().write_value(pac::usbphy::regs::Ctrl(0));
        phy.pll_sic().modify(|w| {
            // Divide-by-30: 16 MHz crystal * 30 = 480 MHz (verified on this
            // board; lpc55-hal uses the same divider).
            w.set_pll_div_sel(pac::usbphy::vals::PllSicPllDivSel::Value6);
            w.set_pll_reg_enable(true);
        });
        // Undocumented bit the NXP SDK clears after enabling the PLL regulator.
        phy.pll_sic_clr().write_value(pac::usbphy::regs::PllSicClr(1 << 16));
        // >= 15 us for the PLL regulator to stabilize.
        cortex_m::asm::delay(5_000);
        phy.pll_sic().modify(|w| {
            w.set_pll_power(true);
            w.set_pll_en_usb_clks(true);
        });
        // Bounded wait for PLL lock (~100 ms worth); carry on regardless, the
        // device simply won't enumerate if the PLL never locks.
        let locked = poll_bounded(1_000_000, || {
            phy.pll_sic().read().pll_lock() == pac::usbphy::vals::PllSicPllLock::Value1
        });
        if !locked {
            warn!("USB1 PHY PLL failed to lock");
        }
        phy.ctrl().modify(|w| {
            w.set_clkgate(false);
            w.set_enautoclr_clkgate(true);
            // On suspend the controller powers the PHY down (hardware sets the PWD
            // TX/RX bits); this makes wakeup auto-clear them again. Without it the
            // PHY receiver stays powered off after the first bus-idle suspend and
            // the controller never decodes another packet (SDK USB_EhciPhyInit
            // sets this too).
            w.set_enautoclr_phy_pwd(true);
        });
        // Power up everything in the PHY.
        phy.pwd().write_value(pac::usbphy::regs::Pwd(0));

        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_dev(true));
        });
    }

    fn power_down() {
        // Reverse of `power_up`: PHY analog off, PHY register clock gated,
        // analog supplies powered down, then the block clocks.
        pac::USBPHY.pwd().write_value(pac::usbphy::regs::Pwd(!0));
        pac::USBPHY.ctrl().modify(|w| w.set_clkgate(true));
        pac::PMC
            .pdruncfgset0()
            .write(|w| w.set_pdruncfgset0(PDEN_USBHSPHY | PDEN_LDOUSBHS));
        critical_section::with(|_| {
            USBHSD_ACTIVE.store(false, Ordering::Release);
            let keep_ram = USB1_SRAM_USERS.load(Ordering::Relaxed) != 0;
            pac::SYSCON.ahbclkctrl2().modify(|w| {
                w.set_usb1_dev(false);
                w.set_usb1_ram(keep_ram);
                w.set_usb1_phy(false);
            });
        });
    }
}

impl Instance for crate::peripherals::USBHSD {
    type Interrupt = crate::interrupt::typelevel::USB1;

    const EP_COUNT: usize = 6;
    const HIGH_SPEED: bool = true;
    const NBYTES_SHIFT: u32 = 11;
    const NBYTES_MASK: u32 = 0x7FFF;
    const ADDROFF_MASK: u32 = 0x7FF;
    const DATABUF_WINDOW_SHIFT: u32 = 17;
    const MAX_BULK_MPS: u16 = 512;
    const MAX_INTERRUPT_MPS: u16 = 1024;
    const MAX_ISO_OUT_MPS: u16 = 1024;
    /// 1023, not 1024, per erratum **USB.6** (ES_LPC55S6x §3.23, revisions 0A
    /// and 1B, no silicon fix): a high-speed isochronous IN endpoint sending a
    /// 1024-byte packet raises no endpoint interrupt and does not update its
    /// command/status entry, so [`driver::EndpointIn::write`] would never
    /// complete. NXP's documented workaround is exactly this cap.
    const MAX_ISO_IN_MPS: u16 = 1023;
    const MULTI_PACKET: bool = true;
    /// Per-slot buffer capacity for HS bulk IN endpoints, in bytes.
    ///
    /// The ip3511-HS packetizes a single command/status entry: NBytes (15
    /// bits) may span multiple max-packet-size packets and hardware streams
    /// them all without CPU involvement, so larger slots amortize the ISR +
    /// executor + re-arm turnaround over many packets.
    ///
    /// 3584 (7 x 512) leaves both directions plus EP0 and one interrupt
    /// endpoint inside the 16 KiB USB SRAM.
    const BULK_IN_SLOT_LEN: u16 = 3584;
}

impl SealedInstance for crate::peripherals::USB0 {
    fn regs() -> pac::usbhsd::Usbhsd {
        // USB0 and USBHSD are the same ip3511 register map: every shared
        // register sits at the same offset and every shared field at the same
        // bit position. The HS block is the field-width superset, and this
        // driver masks writes to the FS widths (`ep_buf_mask`, `ep_int_mask`),
        // so nothing ever reaches an FS-reserved bit. See the module docs.
        unsafe { pac::usbhsd::Usbhsd::from_ptr(pac::USB0.as_ptr()) }
    }

    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }

    fn power_up(_uses_usb1_sram: bool) {
        use pac::syscon::vals::{Usb0DevRst, Usb0clkdivHalt, Usb0clkdivReqflag, Usb0clkselSel};

        // The FS controller clocks off FRO-HF regardless of
        // `config::MainClock`, so it must enable the 96 MHz output itself
        // rather than assume `init` did.
        pac::ANACTRL.fro192m_ctrl().modify(|w| w.set_ena_96mhzclk(true));

        // FRO 96 MHz / 2 = the 48 MHz the FS controller needs. Program the
        // divider before selecting the source, then wait for it to settle.
        pac::SYSCON.usb0clkdiv().modify(|w| w.set_div(1));
        pac::SYSCON.usb0clkdiv().modify(|w| w.set_halt(Usb0clkdivHalt::Run));
        pac::SYSCON.usb0clksel().write(|w| w.set_sel(Usb0clkselSel::Enum0x3));
        let stable = poll_bounded(1_000_000, || {
            pac::SYSCON.usb0clkdiv().read().reqflag() == Usb0clkdivReqflag::Stable
        });
        if !stable {
            warn!("USB0 clock divider did not stabilize");
        }

        // Power on the FS PHY. `pdruncfgclr0` is write-1-to-clear over a
        // single wide field, so use the same raw-bit style as the HS path.
        pac::PMC.pdruncfgclr0().write(|w| w.set_pdruncfgclr0(PDEN_USBFSPHY));

        // Reset the device block, then clock it.
        critical_section::with(|_| {
            pac::SYSCON
                .presetctrl1()
                .modify(|w| w.set_usb0_dev_rst(Usb0DevRst::Asserted));
            pac::SYSCON
                .presetctrl1()
                .modify(|w| w.set_usb0_dev_rst(Usb0DevRst::Released));
            pac::SYSCON.ahbclkctrl1().modify(|w| w.set_usb0_dev(true));
        });

        // Hand the shared USB0 port to the device controller (reset default is
        // host). The USB0 clock must already be running for this write, which
        // is why it comes after the divider and the device clock: lpc55-hal
        // flags it as a debugger-killer otherwise.
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb0_hosts(true));
        });
        pac::USBFSH.portmode().modify(|w| w.set_dev_enable(true));
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb0_hosts(false));
        });

        // Lock the FRO to USB SOF packets for full-speed timing accuracy,
        // which is what makes crystal-less FS operation viable. Not functional
        // through more than one hub — erratum USB.3, see the module docs.
        pac::ANACTRL.fro192m_ctrl().modify(|w| w.set_usbclkadj(true));
        let adjusted = poll_bounded(1_000_000, || !pac::ANACTRL.fro192m_ctrl().read().usbmodchg());
        if !adjusted {
            warn!("USB0 FRO clock adjustment did not settle");
        }
    }

    fn power_down() {
        use pac::syscon::vals::Usb0clkdivHalt;

        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl1().modify(|w| w.set_usb0_dev(false));
        });
        pac::PMC.pdruncfgset0().write(|w| w.set_pdruncfgset0(PDEN_USBFSPHY));
        // Leave USBCLKADJ set: the FRO trim it trained is still valid, and
        // retraining costs another SOF lock on the next `power_up`.
        pac::SYSCON.usb0clkdiv().modify(|w| w.set_halt(Usb0clkdivHalt::Halt));
    }
}

impl Instance for crate::peripherals::USB0 {
    type Interrupt = crate::interrupt::typelevel::USB0;

    /// EP0 plus 4 data endpoints. The FS register field widths give 10
    /// physical endpoints (`epbufcfg.buf_sb` / `epinuse.buf` are 8 bits above
    /// EP0's pair, `epskip.skip` / `inten.ep_int_en` are 10 bits), matching
    /// lpc55-hal's endpoint-list layout of `ep0out, setup, ep0in, _, eps[4]`.
    const EP_COUNT: usize = 5;
    const HIGH_SPEED: bool = false;
    const NBYTES_SHIFT: u32 = 16;
    const NBYTES_MASK: u32 = 0x3FF;
    const ADDROFF_MASK: u32 = 0xFFFF;
    const DATABUF_WINDOW_SHIFT: u32 = 22;
    const MAX_BULK_MPS: u16 = 64;
    const MAX_INTERRUPT_MPS: u16 = 64;
    const MAX_ISO_OUT_MPS: u16 = 1023;
    const MAX_ISO_IN_MPS: u16 = 1023;
    /// The full-speed block does not packetize a command entry: an entry whose
    /// NBytes exceeds the max packet size never completes (verified on
    /// hardware - a 512-byte IN entry on a 64-byte endpoint delivers nothing,
    /// and `lpc55-hal`'s full-speed driver likewise never sets NBytes above
    /// the max packet size). Endpoints are therefore armed one packet at a
    /// time and [`Instance::BULK_IN_SLOT_LEN`] is inert.
    const MULTI_PACKET: bool = false;
    const BULK_IN_SLOT_LEN: u16 = 64;
}

// PDRUNCFG bit positions (1 = powered down; writing 1 to the CLR register
// clears the bit, i.e. powers the block ON).
const PDEN_XTAL32M: u32 = 1 << 8;
const PDEN_USBFSPHY: u32 = 1 << 11;
const PDEN_USBHSPHY: u32 = 1 << 12;
const PDEN_LDOUSBHS: u32 = 1 << 18;
const PDEN_LDOXO32M: u32 = 1 << 20;

trait SealedVbusPin {}

/// `USB0_VBUS` sense pin. Only `PIO0_22` carries this function on LPC55.
#[allow(private_bounds)]
pub trait VbusPin: SealedVbusPin + crate::gpio::Pin {}

impl SealedVbusPin for crate::peripherals::PIO0_22 {}
impl VbusPin for crate::peripherals::PIO0_22 {}

/// Where a USB controller's endpoint command list and data buffers live.
pub struct Memory<'d> {
    base: u32,
    len: u32,
    lease: MemoryLease<'d>,
}

static USB1_SRAM_USERS: AtomicU8 = AtomicU8::new(0);
static USBHSD_ACTIVE: AtomicBool = AtomicBool::new(false);

struct Usb1SramLease;

impl Usb1SramLease {
    fn new() -> Self {
        critical_section::with(|_| {
            if USB1_SRAM_USERS.load(Ordering::Relaxed) != 0 {
                panic!("USB1 SRAM already in use");
            }
            USB1_SRAM_USERS.store(1, Ordering::Relaxed);
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_ram(true));
        });
        Self
    }

    fn clone_lease(&self) -> Self {
        critical_section::with(|_| {
            let users = USB1_SRAM_USERS.load(Ordering::Relaxed);
            let next = users.checked_add(1).expect("USB1 SRAM lease count overflow");
            USB1_SRAM_USERS.store(next, Ordering::Relaxed);
        });
        Self
    }
}

impl Drop for Usb1SramLease {
    fn drop(&mut self) {
        critical_section::with(|_| {
            let users = USB1_SRAM_USERS.load(Ordering::Relaxed);
            debug_assert!(users != 0);
            let remaining = users - 1;
            USB1_SRAM_USERS.store(remaining, Ordering::Relaxed);
            if remaining == 0 && !USBHSD_ACTIVE.load(Ordering::Acquire) {
                pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_ram(false));
            }
        });
    }
}

enum MemoryLease<'d> {
    Usb1Sram(Usb1SramLease),
    Buffer(PhantomData<&'d mut [u8]>),
}

impl<'d> MemoryLease<'d> {
    fn clone_lease(&self) -> Self {
        match self {
            Self::Usb1Sram(lease) => Self::Usb1Sram(lease.clone_lease()),
            Self::Buffer(_) => Self::Buffer(PhantomData),
        }
    }

    fn uses_usb1_sram(&self) -> bool {
        matches!(self, Self::Usb1Sram(_))
    }
}

impl<'d> Memory<'d> {
    /// The dedicated 16 KiB USB1 SRAM at `0x4010_0000`.
    ///
    /// Usable by either controller, but only by one endpoint-memory owner at a
    /// time. The claim is released after the last split driver object is
    /// dropped.
    pub fn usb1_sram() -> Self {
        Self {
            base: USB1_SRAM_ADDR,
            len: USB1_SRAM_SIZE,
            lease: MemoryLease::Usb1Sram(Usb1SramLease::new()),
        }
    }

    /// A caller-owned buffer, typically a local in `main`.
    ///
    /// A local is not a stack allocation: the body of `#[embassy_executor::main]`
    /// is a task, so its locals live in the task's storage, which is a
    /// `static`. A `static_cell::ConstStaticCell` works too.
    ///
    /// `buf` needs no particular alignment. `EPLISTSTART` reserves the low 8
    /// bits of the address, so up to 255 bytes at the front of `buf` are
    /// skipped; at least 512 bytes must be left after that. USB0 can use this
    /// memory while USBHSD uses [`Memory::usb1_sram`]. USBHSD itself cannot
    /// use this memory because its command/status list is fixed to USB1 SRAM.
    pub fn buffer(buf: &'d mut [u8]) -> Self {
        let raw_base = buf.as_mut_ptr() as u32;
        let base = raw_base.next_multiple_of(256);
        let len = (buf.len() as u32).saturating_sub(base - raw_base);
        assert!(
            len >= 512,
            "USB endpoint memory must leave at least 512 bytes after alignment"
        );
        Self {
            base,
            len,
            lease: MemoryLease::Buffer(PhantomData),
        }
    }

    /// Base address of the region, after alignment.
    ///
    /// This is the address the driver programs into `EPLISTSTART` and
    /// `DATABUFSTART`, and where the endpoint command/status list starts.
    pub fn base(&self) -> u32 {
        self.base
    }
}

/// Marker type for IN endpoints.
pub enum In {}
/// Marker type for OUT endpoints.
pub enum Out {}

trait SealedDir {
    fn dir() -> Direction;
}

/// Endpoint direction marker.
#[allow(private_bounds)]
pub trait Dir: SealedDir {}

impl SealedDir for In {
    fn dir() -> Direction {
        Direction::In
    }
}
impl Dir for In {}

impl SealedDir for Out {
    fn dir() -> Direction {
        Direction::Out
    }
}
impl Dir for Out {}

/// USB interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let state = T::state();
        if !state.alive.load(Ordering::Acquire) {
            return;
        }
        let regs = T::regs();
        let stat = regs.intstat().read();
        // W1C exactly the bits we saw. Later-arriving bits stay pending and
        // re-fire the interrupt. Losing an edge is safe by design: the futures
        // re-check level state (eplist Active bit / DEVCMDSTAT.SETUP).
        regs.intstat().write_value(stat);

        for i in 0..T::EP_COUNT {
            if stat.0 & (1 << (2 * i)) != 0 {
                state.ep_waker(i, Direction::Out).wake();
            }
            if stat.0 & (1 << (2 * i + 1)) != 0 {
                state.ep_waker(i, Direction::In).wake();
            }
        }

        if regs.devcmdstat().read().setup() {
            state.note_setup();
        }

        // dev_int: DEVCMDSTAT change bits are consumed by `Bus::poll`.
        if stat.0 & (1 << 31) != 0 {
            state.bus_waker.wake();
        }
    }
}
fn shutdown_controller<T: Instance>(ep_list: u32) {
    let state = T::state();
    if !state.alive.swap(false, Ordering::AcqRel) {
        return;
    }
    state.invalidate_controller();

    critical_section::with(|_| {
        let regs = T::regs();
        devcmdstat_modify(regs, |w| {
            w.set_dcon(false);
            w.set_dev_en(false);
        });
        for index in 0..T::EP_COUNT {
            for dir in DIRECTIONS {
                reclaim_slots::<T>(ep_list, index, dir);
                reset_endpoint::<T>(ep_list, index, dir);
            }
        }
        regs.inten().write_value(pac::usbhsd::regs::Inten(0));
        regs.intstat()
            .write_value(pac::usbhsd::regs::Intstat(ep_int_mask::<T>() as u32 | (1 << 31)));
    });

    T::Interrupt::disable();
    T::Interrupt::unpend();
    state.wake_all();
    T::power_down();
}

/// LPC55 USB device driver.
pub struct Driver<'d, T: Instance> {
    _usb: Option<Peri<'d, T>>,
    _vbus: Option<Peri<'d, crate::gpio::AnyPin>>,
    memory: Option<MemoryLease<'d>>,
    ep_in_used: [bool; MAX_EP_COUNT],
    ep_out_used: [bool; MAX_EP_COUNT],
    /// Command list and fixed endpoint-zero buffers.
    layout: EndpointLayout,
    /// One past the end of the endpoint memory.
    mem_end: u32,
    /// Bump allocator offset, relative to `layout.list`.
    alloc_offset: u32,
}

impl<'d> Driver<'d, crate::peripherals::USBHSD> {
    /// Create a new USB1 high-speed driver.
    ///
    /// This powers up and configures the USB1 PHY (expects a 16 MHz crystal)
    /// and the device controller. It does not connect to the bus yet;
    /// [`driver::Bus::enable`] controls the soft-connect.
    ///
    /// USBHSD is high-speed-only in this release. The driver disconnects and
    /// panics if the host negotiates a fallback speed.
    ///
    /// Requires a system clock of at least 96 MHz, see
    /// [`crate::config::MainClock::FroHf96`].
    ///
    /// `memory` must be [`Memory::usb1_sram`]. The USBHSD controller cannot
    /// write its command/status list to main SRAM.
    pub fn new(
        usb: Peri<'d, crate::peripherals::USBHSD>,
        _irq: impl Binding<crate::interrupt::typelevel::USB1, InterruptHandler<crate::peripherals::USBHSD>> + 'd,
        memory: Memory<'d>,
    ) -> Self {
        Self::new_inner(usb, None, memory)
    }
}

impl<'d> Driver<'d, crate::peripherals::USB0> {
    /// Create a new USB0 full-speed driver.
    ///
    /// This brings up the controller's own 48 MHz clock and the full-speed
    /// PHY, but does not connect to the bus yet; the soft-connect happens in
    /// [`driver::Bus::enable`].
    ///
    /// `vbus` must be `PIO0_22`; it is routed to `USB0_VBUS` (IOCON FUNC7).
    /// The pin is required because hardware gates the D+ pull-up on
    /// `DEVCMDSTAT.VBUS_DEBOUNCED && DCON`, so the device cannot signal a
    /// connect without it.
    pub fn new(
        usb: Peri<'d, crate::peripherals::USB0>,
        _irq: impl Binding<crate::interrupt::typelevel::USB0, InterruptHandler<crate::peripherals::USB0>> + 'd,
        vbus: Peri<'d, impl VbusPin>,
        memory: Memory<'d>,
    ) -> Self {
        use pac::iocon::vals::{PioDigimode, PioFunc, PioMode, PioOd, PioSlew};

        vbus.pio().modify(|w| {
            w.set_func(PioFunc::Alt7);
            w.set_mode(PioMode::Inactive);
            w.set_slew(PioSlew::Standard);
            w.set_invert(false);
            w.set_digimode(PioDigimode::Digital);
            w.set_od(PioOd::Normal);
        });

        Self::new_inner(usb, Some(vbus.into()), memory)
    }
}

impl<'d, T: Instance> Driver<'d, T> {
    fn new_inner(usb: Peri<'d, T>, vbus: Option<Peri<'d, crate::gpio::AnyPin>>, memory: Memory<'d>) -> Self {
        let Memory { base, len, lease } = memory;
        let uses_usb1_sram = lease.uses_usb1_sram();
        assert!(
            !T::HIGH_SPEED || uses_usb1_sram,
            "USBHSD endpoint memory must use Memory::usb1_sram()"
        );
        assert!(
            len >= DATA_BUFFER_OFFSET,
            "USB endpoint memory is too small for EP0 storage"
        );
        assert!(
            base >> T::DATABUF_WINDOW_SHIFT == (base + len - 1) >> T::DATABUF_WINDOW_SHIFT,
            "endpoint memory crosses the DATABUFSTART window"
        );

        T::power_up(uses_usb1_sram);

        for i in 0..T::EP_COUNT * 4 {
            unsafe { ((base as usize + i * 4) as *mut u32).write_volatile(0) };
        }
        let layout = EndpointLayout::new(base);
        unsafe {
            (layout.setup as *mut u32).write_volatile(0);
            (layout.setup as *mut u32).add(1).write_volatile(0);
            layout.command(0, Direction::Out, 1).write(addroff::<T>(layout.setup));
        }

        program_controller::<T>(layout.list);
        let state = T::state();
        state.invalidate_controller();
        for index in 0..T::EP_COUNT {
            state.ep_state(index, Direction::In).store(0, Ordering::Relaxed);
            state.ep_state(index, Direction::Out).store(0, Ordering::Relaxed);
        }
        state.alive.store(true, Ordering::Release);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _usb: Some(usb),
            _vbus: vbus,
            memory: Some(lease),
            ep_in_used: [false; MAX_EP_COUNT],
            ep_out_used: [false; MAX_EP_COUNT],
            layout,
            mem_end: base + len,
            alloc_offset: DATA_BUFFER_OFFSET,
        }
    }

    /// Carve a 64-byte-aligned buffer out of the endpoint memory.
    ///
    /// The 64-byte rounding is load-bearing beyond alignment: erratum USB.5
    /// has the high-speed controller write up to 7 bytes past an OUT transfer
    /// whose length is not a multiple of 8, and a 64-byte granule keeps that
    /// overrun inside the allocation. Do not tighten it.
    fn alloc_buffer(&mut self, len: u16) -> Result<u32, EndpointAllocError> {
        let len = (len as u32).div_ceil(64) * 64;
        if self.layout.list + self.alloc_offset + len > self.mem_end {
            warn!("USB endpoint memory full");
            return Err(EndpointAllocError);
        }
        let addr = self.layout.list + self.alloc_offset;
        self.alloc_offset += len;
        Ok(addr)
    }

    /// Allocates both slots atomically with respect to the bump pointer.
    fn alloc_slots(&mut self, len: u16) -> Result<[u32; 2], EndpointAllocError> {
        let mark = self.alloc_offset;
        let result = match self.alloc_buffer(len) {
            Ok(first) => self.alloc_buffer(len).map(|second| [first, second]),
            Err(error) => Err(error),
        };
        if result.is_err() {
            self.alloc_offset = mark;
        }
        result
    }

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Endpoint<'d, T, D>, EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval_ms={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval_ms,
            D::dir()
        );

        let mps_limit = match ep_type {
            // Control is only used internally for EP0; embassy-usb never
            // allocates extra control endpoints.
            EndpointType::Control => return Err(EndpointAllocError),
            EndpointType::Isochronous => match D::dir() {
                Direction::Out => T::MAX_ISO_OUT_MPS,
                Direction::In => T::MAX_ISO_IN_MPS,
            },
            EndpointType::Bulk => T::MAX_BULK_MPS,
            EndpointType::Interrupt => T::MAX_INTERRUPT_MPS,
        };
        if max_packet_size > mps_limit {
            warn!("max_packet_size too high: {}", max_packet_size);
            return Err(EndpointAllocError);
        }

        let used = match D::dir() {
            Direction::Out => &mut self.ep_out_used,
            Direction::In => &mut self.ep_in_used,
        };
        let index = match ep_addr {
            Some(addr) => {
                let i = addr.index();
                if i == 0 || i >= T::EP_COUNT || used[i] {
                    return Err(EndpointAllocError);
                }
                i
            }
            None => (1..T::EP_COUNT).find(|&i| !used[i]).ok_or(EndpointAllocError)?,
        };

        // Two buffer slots per endpoint (double buffering): software fills or
        // drains one while the other is on the wire. Bulk IN endpoints get a
        // multi-packet slot where hardware packetizes a command entry (see
        // BULK_IN_SLOT_LEN), and fall back to a single-packet slot if the
        // endpoint memory cannot fit the large one.
        //
        // Everything else is one packet per slot: OUT because `read` owes the
        // caller a single packet, isochronous because there is one packet per
        // (micro)frame anyway.
        let want_len = match (ep_type, D::dir()) {
            (EndpointType::Bulk, Direction::In) if T::MULTI_PACKET => T::BULK_IN_SLOT_LEN.max(max_packet_size),
            _ => max_packet_size,
        };
        let (buf_addrs, slot_len) = match self.alloc_slots(want_len) {
            Ok(addrs) => (addrs, want_len),
            Err(_) if want_len > max_packet_size => {
                // The multi-packet slots did not fit; single-packet slots may.
                (self.alloc_slots(max_packet_size)?, max_packet_size)
            }
            Err(error) => return Err(error),
        };

        // Only now that the allocation succeeded does the index get consumed:
        // a rejected allocation must leave the driver untouched.
        match D::dir() {
            Direction::Out => self.ep_out_used[index] = true,
            Direction::In => self.ep_in_used[index] = true,
        }

        // Isochronous endpoints keep a state flag: `Bus` only ever sees an
        // `EndpointAddress`, and the stall path has to treat them differently.
        let iso = ep_type == EndpointType::Isochronous;
        T::state()
            .ep_state(index, D::dir())
            .store(if iso { ISO } else { 0 }, Ordering::Relaxed);

        // Disabled until `endpoint_set_enabled`.
        self.layout.command(index, D::dir(), 0).write(CMD_D);
        self.layout.command(index, D::dir(), 1).write(CMD_D);

        trace!("  index={} addr={:08x}", index, buf_addrs[0]);

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
            ep_list: self.layout.list,
            buf_addrs,
            slot_len,
            armed_len: [0; 2],
            _memory: self.memory.as_ref().ok_or(EndpointAllocError)?.clone_lease(),
        })
    }
}
impl<'d, T: Instance> Drop for Driver<'d, T> {
    fn drop(&mut self) {
        if self._usb.is_some() {
            shutdown_controller::<T>(self.layout.list);
        }
        drop(self.memory.take());
    }
}

impl<'d, T: Instance> driver::Driver<'d> for Driver<'d, T> {
    type EndpointOut = Endpoint<'d, T, Out>;
    type EndpointIn = Endpoint<'d, T, In>;
    type ControlPipe = ControlPipe<'d, T>;
    type Bus = Bus<'d, T>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn start(mut self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        assert!(control_max_packet_size <= 64);
        let layout = self.layout;
        let Some(usb) = self._usb.take() else {
            unreachable!("USB peripheral token missing at start");
        };
        let vbus_pin = self._vbus.take();
        let Some(memory) = self.memory.take() else {
            unreachable!("USB endpoint memory lease missing at start");
        };
        let bus_memory = memory.clone_lease();
        let control_memory = memory.clone_lease();

        critical_section::with(|_| {
            layout
                .command(0, Direction::Out, 0)
                .arm::<T>(control_max_packet_size as u32, layout.out, false, false);
            layout.command(0, Direction::In, 0).write(addroff::<T>(layout.in_));
            let vbus = T::regs().devcmdstat().read().vbus_debounced();
            devcmdstat_modify(T::regs(), |w| {
                w.set_dev_en(true);
                w.set_dcon(!vbus);
            });
        });

        trace!("started");

        (
            Bus {
                _usb: Some(usb),
                _vbus: vbus_pin,
                memory: Some(bus_memory),
                vbus: false,
                needs_reset: false,
                layout,
                ep0_mps: control_max_packet_size,
            },
            ControlPipe {
                _phantom: PhantomData,
                _memory: control_memory,
                max_packet_size: control_max_packet_size,
                layout,
                accepted_generation: None,
            },
        )
    }
}

/// USB bus.
pub struct Bus<'d, T: Instance> {
    _usb: Option<Peri<'d, T>>,
    _vbus: Option<Peri<'d, crate::gpio::AnyPin>>,
    memory: Option<MemoryLease<'d>>,
    vbus: bool,
    needs_reset: bool,
    layout: EndpointLayout,
    ep0_mps: u16,
}

impl<'d, T: Instance> Bus<'d, T> {
    fn reset_endpoints(&mut self) {
        critical_section::with(|_| {
            let regs = T::regs();
            let state = T::state();
            state.invalidate_controller();

            state.ep_state(0, Direction::Out).store(0, Ordering::Relaxed);
            state.ep_state(0, Direction::In).store(0, Ordering::Relaxed);
            for index in 0..T::EP_COUNT {
                for dir in DIRECTIONS {
                    reclaim_slots::<T>(self.layout.list, index, dir);
                }
            }
            for index in 1..T::EP_COUNT {
                for dir in DIRECTIONS {
                    reset_endpoint::<T>(self.layout.list, index, dir);
                }
            }
            regs.epinuse().write(|w| w.set_buf(0));
            regs.epbufcfg().write(|w| w.set_buf_sb(ep_buf_mask::<T>()));
            self.layout
                .command(0, Direction::Out, 0)
                .arm::<T>(self.ep0_mps as u32, self.layout.out, false, false);
            self.layout.command(0, Direction::In, 0).write(0);
            self.layout
                .command(0, Direction::Out, 1)
                .write(addroff::<T>(self.layout.setup));
        });
    }

    fn disable_data_endpoints(&mut self) {
        for index in 1..T::EP_COUNT {
            for dir in DIRECTIONS {
                reclaim_slots::<T>(self.layout.list, index, dir);
                reset_endpoint::<T>(self.layout.list, index, dir);
            }
        }
    }

    fn shutdown(&mut self) {
        if self._usb.is_none() {
            return;
        }
        shutdown_controller::<T>(self.layout.list);
        drop(self.memory.take());
        self._usb.take();
        self._vbus.take();
    }
}

impl<'d, T: Instance> Drop for Bus<'d, T> {
    fn drop(&mut self) {
        self.shutdown();
    }
}

const fn negotiated_speed_valid(high_speed: bool, speed: u8) -> bool {
    // USB0 leaves the SPEED field reserved and reads it as zero.
    speed == if high_speed { 0b10 } else { 0b00 }
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    async fn enable(&mut self) {
        critical_section::with(|_| {
            // `start` already programs and arms EP0. Reclaiming that slot on
            // the first enable would cancel the request that waits for reset.
            if self.needs_reset {
                for index in 0..T::EP_COUNT {
                    for dir in DIRECTIONS {
                        reclaim_slots::<T>(self.layout.list, index, dir);
                    }
                }
                program_controller::<T>(self.layout.list);
                self.reset_endpoints();
                self.needs_reset = false;
            }
            devcmdstat_modify(T::regs(), |w| {
                w.set_dev_en(true);
                w.set_dcon(true);
            });
        });
        T::state().bus_waker.wake();
    }

    async fn disable(&mut self) {
        critical_section::with(|_| {
            let regs = T::regs();
            let vbus = regs.devcmdstat().read().vbus_debounced();
            T::state().invalidate_controller();
            // EPSKIP only runs while the endpoint engine is connected.
            self.disable_data_endpoints();
            devcmdstat_modify(regs, |w| {
                w.set_setup(true);
                w.set_dev_en(true);
                w.set_dcon(!vbus);
            });
            self.vbus = false;
            self.needs_reset = true;
        });
    }

    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            let state = T::state();
            state.bus_waker.register(cx.waker());
            if !state.alive.load(Ordering::Acquire) {
                return Poll::Pending;
            }

            let regs = T::regs();
            let dcs = regs.devcmdstat().read();
            let vbus = dcs.vbus_debounced();
            if vbus != self.vbus {
                self.vbus = vbus;
                let event = if vbus {
                    Event::PowerDetected
                } else {
                    critical_section::with(|_| state.invalidate_controller());
                    Event::PowerRemoved
                };
                trace!("{}", event);
                return Poll::Ready(event);
            }

            if dcs.dres_c() {
                let speed = dcs.speed();
                if !negotiated_speed_valid(T::HIGH_SPEED, speed) {
                    critical_section::with(|_| {
                        state.invalidate_controller();
                        devcmdstat_modify(regs, |w| w.set_dcon(false));
                    });
                    if T::HIGH_SPEED {
                        panic!("USBHSD negotiated unsupported speed {}", speed);
                    }
                    panic!("USB0 negotiated unsupported speed {}", speed);
                }

                devcmdstat_modify(regs, |w| {
                    w.set_dres_c(true);
                    w.set_dev_addr(0);
                });
                self.reset_endpoints();
                trace!("RESET");
                return Poll::Ready(Event::Reset);
            }

            if dcs.dsus_c() {
                devcmdstat_modify(regs, |w| w.set_dsus_c(true));
                let event = if dcs.dsus() { Event::Suspend } else { Event::Resume };
                trace!("{}", event);
                return Poll::Ready(event);
            }

            if dcs.dcon_c() {
                devcmdstat_modify(regs, |w| w.set_dcon_c(true));
            }

            Poll::Pending
        })
        .await
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:?} {}", ep_addr, enabled);
        let Some((index, dir)) = valid_endpoint::<T>(ep_addr) else {
            return;
        };
        if index == 0 {
            return;
        }

        critical_section::with(|_| {
            let shared = T::state();
            let state = shared.ep_state(index, dir);
            let iso = state.load(Ordering::Relaxed) & ISO;
            reclaim_slots::<T>(self.layout.list, index, dir);
            shared.invalidate_endpoint(index, dir);

            if enabled {
                self.layout
                    .command(index, dir, 0)
                    .write(if iso != 0 { 0 } else { CMD_TR });
                self.layout.command(index, dir, 1).write(0);
                select_slot::<T>(index, dir, 0);
                state.store(iso | if iso == 0 { TR_PENDING } else { 0 }, Ordering::Relaxed);
            } else {
                reset_endpoint::<T>(self.layout.list, index, dir);
            }
        });
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        trace!("set_stalled {:?} {}", ep_addr, stalled);
        let Some((index, dir)) = valid_endpoint::<T>(ep_addr) else {
            return;
        };

        critical_section::with(|_| {
            let shared = T::state();
            let iso = index != 0 && shared.ep_state(index, dir).load(Ordering::Relaxed) & ISO != 0;
            reclaim_slots::<T>(self.layout.list, index, dir);
            shared.invalidate_endpoint(index, dir);

            if stalled {
                if iso {
                    self.layout.command(index, dir, 0).write(0);
                    self.layout.command(index, dir, 1).write(0);
                    shared.ep_state(index, dir).store(ISO, Ordering::Relaxed);
                } else {
                    self.layout.command(index, dir, 0).write(CMD_S);
                    if index != 0 {
                        self.layout.command(index, dir, 1).write(CMD_S);
                    }
                }
            } else if index == 0 {
                self.layout.command(index, dir, 0).write(CMD_TR);
            } else {
                self.layout.command(index, dir, 0).write(if iso { 0 } else { CMD_TR });
                self.layout.command(index, dir, 1).write(0);
                let state = shared.ep_state(index, dir);
                let keep = state.load(Ordering::Relaxed) & ISO;
                state.store(keep | if keep == 0 { TR_PENDING } else { 0 }, Ordering::Relaxed);
            }
        });
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let Some((index, dir)) = valid_endpoint::<T>(ep_addr) else {
            return false;
        };
        critical_section::with(|_| {
            T::state().alive.load(Ordering::Acquire) && self.layout.command(index, dir, 0).read() & CMD_S != 0
        })
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        devcmdstat_modify(T::regs(), |w| w.set_dsus(false));
        Ok(())
    }

    fn force_reset(&mut self) -> Result<(), Unsupported> {
        critical_section::with(|_| {
            T::state().invalidate_controller();
            devcmdstat_modify(T::regs(), |w| w.set_dcon(false));
        });
        cortex_m::asm::delay(10_000_000);
        critical_section::with(|_| {
            if T::state().alive.load(Ordering::Acquire) {
                devcmdstat_modify(T::regs(), |w| w.set_dcon(true));
            }
        });
        Ok(())
    }
}

/// USB endpoint.
pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    ep_list: u32,
    buf_addrs: [u32; 2],
    /// Per-slot buffer capacity (multi-packet for bulk endpoints).
    slot_len: u16,
    /// Length each OUT slot was last armed with; needed to compute the
    /// received count from the NBytes-remaining field on completion.
    armed_len: [u16; 2],
    _memory: MemoryLease<'d>,
}

impl<'d, T: Instance, D: Dir> Endpoint<'d, T, D> {
    fn is_iso(&self) -> bool {
        self.info.ep_type == EndpointType::Isochronous
    }

    fn command(&self, slot: usize) -> EndpointCommand {
        EndpointCommand::new(self.ep_list, self.info.addr.index(), D::dir(), slot)
    }

    fn read_slot(&self, slot: usize) -> u32 {
        self.command(slot).read()
    }

    fn arm_slot(&mut self, slot: usize, len: u32, tr: bool) {
        self.command(slot)
            .arm::<T>(len, self.buf_addrs[slot], tr, self.is_iso());
        self.armed_len[slot] = len as u16;
    }
}

impl<'d, T: Instance, D: Dir> driver::Endpoint for Endpoint<'d, T, D> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        let index = self.info.addr.index();
        poll_fn(|cx| {
            let shared = T::state();
            shared.ep_waker(index, D::dir()).register(cx.waker());
            critical_section::with(|_| {
                if !shared.alive.load(Ordering::Acquire) {
                    return Poll::Pending;
                }
                if self.read_slot(0) & CMD_D == 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        })
        .await;
        trace!("wait_enabled {:?} OK", self.info.addr);
    }
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    /// Receives exactly one packet, per the trait contract.
    ///
    /// Slots are always armed for a single max-packet-size packet. Arming a
    /// multi-packet window would let hardware coalesce packets into one command
    /// entry - roughly twice the OUT rate - but the window length is committed
    /// when a slot is armed, one packet ahead of the caller, while the room left
    /// in the caller's buffer is only known when that slot is drained. Any
    /// mismatch overflows, so the fast path is not expressible without an
    /// internal holding buffer. `read_transfer` therefore keeps the trait's
    /// per-packet default.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        let arm_len = self.info.max_packet_size as u32;
        let shared = T::state();
        let state = shared.ep_state(index, Direction::Out);

        let s = critical_section::with(|_| {
            if !shared.alive.load(Ordering::Acquire) {
                return Err(EndpointError::Disabled);
            }
            let generation = shared.transfer_generation(index, Direction::Out);
            let mut s = state.load(Ordering::Relaxed);
            for slot in 0..2 {
                if s & (PRIMED0 << slot) == 0 {
                    if self.read_slot(slot) & CMD_UNUSABLE != 0 {
                        return Err(EndpointError::Disabled);
                    }
                    self.arm_slot(slot, arm_len, s & TR_PENDING != 0);
                    s = (s | (PRIMED0 << slot)) & !TR_PENDING;
                    state.store(s, Ordering::Relaxed);
                }
            }
            if !shared.transfer_generation_valid(index, Direction::Out, generation) {
                return Err(EndpointError::Disabled);
            }
            Ok((s, generation))
        })?;

        let (s, generation) = s;
        let slot = (s & SLOT) as usize;
        let (word, snapshot_state) = wait_slot::<T, _>(self.command(slot), index, Direction::Out, || {
            if !shared.transfer_generation_valid(index, Direction::Out, generation) {
                return Err(EndpointError::Disabled);
            }
            let snapshot_state = state.load(Ordering::Relaxed);
            if (snapshot_state & SLOT) as usize != slot || snapshot_state & (PRIMED0 << slot) == 0 {
                return Err(EndpointError::Disabled);
            }
            Ok(snapshot_state)
        })
        .await?;

        let rx_len = (self.armed_len[slot] as u32 - remaining_bytes::<T>(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_ep_buffer(self.buf_addrs[slot], &mut buf[..rx_len]);

        critical_section::with(|_| {
            if !shared.alive.load(Ordering::Acquire)
                || !shared.transfer_generation_valid(index, Direction::Out, generation)
            {
                return Err(EndpointError::Disabled);
            }
            let current = state.load(Ordering::Relaxed);
            let command = self.read_slot(slot);
            if (current & SLOT) as usize != slot || current & (PRIMED0 << slot) == 0 || command & CMD_UNUSABLE != 0 {
                return Err(EndpointError::Disabled);
            }
            self.arm_slot(slot, arm_len, false);
            state.store((snapshot_state ^ SLOT) | (PRIMED0 << slot), Ordering::Relaxed);
            Ok(())
        })?;

        trace!("READ {:?} rx_len = {}", self.info.addr, rx_len);
        Ok(rx_len)
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    /// Sends `buf` as one endpoint command entry.
    ///
    /// Beyond the trait's single-packet contract this accepts up to the
    /// endpoint's slot capacity, which hardware then packetizes on its own
    /// where [`Instance::MULTI_PACKET`] holds. Code relying on that is not
    /// portable to drivers without it; [`driver::EndpointIn::write_transfer`]
    /// is, and is just as fast here.
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        if buf.len() > self.slot_len as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let index = self.info.addr.index();
        let shared = T::state();
        let state = shared.ep_state(index, Direction::In);
        let (slot, generation) = critical_section::with(|_| {
            if !shared.alive.load(Ordering::Acquire) {
                return Err(EndpointError::Disabled);
            }
            let slot = (state.load(Ordering::Relaxed) & SLOT) as usize;
            Ok((slot, shared.transfer_generation(index, Direction::In)))
        })?;

        let (_, snapshot_state) = wait_slot::<T, _>(self.command(slot), index, Direction::In, || {
            if !shared.transfer_generation_valid(index, Direction::In, generation) {
                return Err(EndpointError::Disabled);
            }
            let snapshot_state = state.load(Ordering::Relaxed);
            if (snapshot_state & SLOT) as usize != slot {
                return Err(EndpointError::Disabled);
            }
            Ok(snapshot_state)
        })
        .await?;

        copy_to_ep_buffer(self.buf_addrs[slot], buf);
        critical_section::with(|_| {
            if !shared.alive.load(Ordering::Acquire)
                || !shared.transfer_generation_valid(index, Direction::In, generation)
            {
                return Err(EndpointError::Disabled);
            }
            let current = state.load(Ordering::Relaxed);
            let command = self.read_slot(slot);
            if (current & SLOT) as usize != slot || command & (CMD_UNUSABLE | CMD_A) != 0 {
                return Err(EndpointError::Disabled);
            }
            self.arm_slot(slot, buf.len() as u32, snapshot_state & TR_PENDING != 0);
            state.store((snapshot_state ^ SLOT) & !TR_PENDING, Ordering::Relaxed);
            Ok(())
        })?;

        trace!("WRITE {:?} len = {}", self.info.addr, buf.len());
        Ok(())
    }

    /// Sends all of `buf`, then a zero-length packet if the transfer needs one
    /// to terminate.
    ///
    /// Chunks by slot capacity rather than by max packet size, so hardware
    /// packetizes each chunk instead of taking one interrupt per packet.
    async fn write_transfer(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointError> {
        let chunk_len = self.slot_len as usize;
        for chunk in buf.chunks(chunk_len) {
            self.write(chunk).await?;
        }
        // A transfer ending on a max-packet-size boundary is only complete once
        // a short packet follows it; any other length already ended on one.
        if needs_zlp && buf.len() % self.info.max_packet_size as usize == 0 {
            self.write(&[]).await?;
        }
        Ok(())
    }
}

/// USB control pipe (endpoint 0, both directions).
pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    _memory: MemoryLease<'d>,
    max_packet_size: u16,
    layout: EndpointLayout,
    accepted_generation: Option<ControlGeneration>,
}

impl<'d, T: Instance> ControlPipe<'d, T> {
    fn request_valid(&self) -> bool {
        let state = T::state();
        let Some(generation) = self.accepted_generation else {
            return false;
        };
        if !state.alive.load(Ordering::Acquire) || generation != state.control_generation() {
            return false;
        }
        if T::regs().devcmdstat().read().setup() {
            state.note_setup();
            return false;
        }
        true
    }

    fn command(&self, dir: Direction) -> EndpointCommand {
        self.layout.command(0, dir, 0)
    }

    fn arm(&self, dir: Direction, len: u32) {
        let addr = match dir {
            Direction::Out => self.layout.out,
            Direction::In => self.layout.in_,
        };
        self.command(dir).arm::<T>(len, addr, false, false);
    }

    fn validate_request(&self) -> Result<(), EndpointError> {
        self.request_valid().then_some(()).ok_or(EndpointError::Disabled)
    }
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        self.max_packet_size as usize
    }

    async fn setup(&mut self) -> [u8; 8] {
        let shared = T::state();
        poll_fn(|cx| {
            shared.ep_waker(0, Direction::Out).register(cx.waker());
            critical_section::with(|_| {
                if !shared.alive.load(Ordering::Acquire) {
                    return Poll::Pending;
                }
                if T::regs().devcmdstat().read().setup() {
                    shared.note_setup();
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        })
        .await;

        let buf = critical_section::with(|_| {
            self.command(Direction::Out).modify(|w| w & !(CMD_A | CMD_S));
            self.command(Direction::In).modify(|w| w & !(CMD_A | CMD_S));

            let mut buf = [0; 8];
            copy_from_ep_buffer(self.layout.setup, &mut buf);
            devcmdstat_modify(T::regs(), |w| w.set_setup(true));
            self.layout
                .command(0, Direction::Out, 1)
                .write(addroff::<T>(self.layout.setup));
            self.arm(Direction::Out, self.max_packet_size as u32);

            self.accepted_generation = Some(shared.control_generation());
            shared.setup_pending.store(false, Ordering::Release);
            buf
        });

        trace!("SETUP {=[u8]:x}", buf);
        buf
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        let mps = self.max_packet_size as u32;
        let (word, ()) = wait_slot::<T, _>(self.command(Direction::Out), 0, Direction::Out, || {
            self.validate_request()
        })
        .await?;

        let rx_len = (mps - remaining_bytes::<T>(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_ep_buffer(self.layout.out, &mut buf[..rx_len]);

        critical_section::with(|_| {
            self.validate_request()?;
            if self.command(Direction::Out).read() & CMD_UNUSABLE != 0 {
                return Err(EndpointError::Disabled);
            }
            self.arm(Direction::Out, mps);
            Ok(())
        })?;

        trace!("control: data_out rx_len = {}", rx_len);
        Ok(rx_len)
    }

    async fn data_in(&mut self, data: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in len = {} last = {}", data.len(), last);
        if data.len() > self.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }

        if !critical_section::with(|_| self.request_valid()) {
            return Err(EndpointError::Disabled);
        }
        copy_to_ep_buffer(self.layout.in_, data);
        critical_section::with(|_| {
            self.validate_request()?;
            if self.command(Direction::In).read() & CMD_UNUSABLE != 0 {
                return Err(EndpointError::Disabled);
            }
            self.arm(Direction::In, data.len() as u32);
            Ok(())
        })?;

        wait_slot::<T, _>(self.command(Direction::In), 0, Direction::In, || {
            self.validate_request()
        })
        .await?;

        let _ = last;
        Ok(())
    }

    async fn accept(&mut self) {
        trace!("control: accept");
        let armed = critical_section::with(|_| {
            if !self.request_valid() {
                return false;
            }
            self.arm(Direction::In, 0);
            true
        });
        if !armed {
            return;
        }

        let _ = wait_slot::<T, _>(self.command(Direction::In), 0, Direction::In, || {
            self.validate_request()
        })
        .await;
    }

    async fn reject(&mut self) {
        trace!("control: reject");
        critical_section::with(|_| {
            if !self.request_valid() {
                return;
            }
            self.command(Direction::Out).modify(|w| w | CMD_S);
            self.command(Direction::In).modify(|w| w | CMD_S);
        });
    }

    async fn accept_set_address(&mut self, addr: u8) {
        trace!("setting addr: {}", addr);
        let valid = critical_section::with(|_| {
            if !self.request_valid() {
                return false;
            }
            devcmdstat_modify(T::regs(), |w| w.set_dev_addr(addr));
            true
        });
        if valid {
            self.accept().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::negotiated_speed_valid;

    #[test]
    fn usb0_accepts_its_full_speed_encoding() {
        assert!(negotiated_speed_valid(false, 0b00));
        assert!(!negotiated_speed_valid(false, 0b01));
        assert!(!negotiated_speed_valid(false, 0b10));
    }

    #[test]
    fn usbhsd_accepts_only_high_speed() {
        assert!(negotiated_speed_valid(true, 0b10));
        assert!(!negotiated_speed_valid(true, 0b00));
        assert!(!negotiated_speed_valid(true, 0b01));
    }
}
