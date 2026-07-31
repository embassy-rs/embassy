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
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering, compiler_fence};
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

/// Shared driver state for one controller instance.
pub(crate) struct State {
    bus_waker: AtomicWaker,
    ep_in_wakers: [AtomicWaker; MAX_EP_COUNT],
    ep_out_wakers: [AtomicWaker; MAX_EP_COUNT],
    ep_in_state: [AtomicU8; MAX_EP_COUNT],
    ep_out_state: [AtomicU8; MAX_EP_COUNT],
}

impl State {
    const fn new() -> Self {
        Self {
            bus_waker: AtomicWaker::new(),
            ep_in_wakers: [const { AtomicWaker::new() }; MAX_EP_COUNT],
            ep_out_wakers: [const { AtomicWaker::new() }; MAX_EP_COUNT],
            ep_in_state: [const { AtomicU8::new(0) }; MAX_EP_COUNT],
            ep_out_state: [const { AtomicU8::new(0) }; MAX_EP_COUNT],
        }
    }

    fn ep_state(&self, index: usize, dir: Direction) -> &AtomicU8 {
        match dir {
            Direction::Out => &self.ep_out_state[index],
            Direction::In => &self.ep_in_state[index],
        }
    }

    fn ep_waker(&self, index: usize, dir: Direction) -> &AtomicWaker {
        match dir {
            Direction::Out => &self.ep_out_wakers[index],
            Direction::In => &self.ep_in_wakers[index],
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

/// Pointer to the command/status word of a physical endpoint buffer slot.
fn ep_cmd_ptr(ep_list: u32, index: usize, dir: Direction, buf1: bool) -> *mut u32 {
    let slot = match dir {
        Direction::Out => 0,
        Direction::In => 2,
    } + buf1 as usize;
    (ep_list as usize + index * 16 + slot * 4) as *mut u32
}

fn ep_cmd_read(ep_list: u32, index: usize, dir: Direction, slot: usize) -> u32 {
    unsafe { ep_cmd_ptr(ep_list, index, dir, slot != 0).read_volatile() }
}

fn ep_cmd_write(ep_list: u32, index: usize, dir: Direction, slot: usize, word: u32) {
    unsafe { ep_cmd_ptr(ep_list, index, dir, slot != 0).write_volatile(word) }
}

fn ep_cmd_modify(ep_list: u32, index: usize, dir: Direction, slot: usize, f: impl FnOnce(u32) -> u32) {
    critical_section::with(|_| {
        let p = ep_cmd_ptr(ep_list, index, dir, slot != 0);
        unsafe { p.write_volatile(f(p.read_volatile())) }
    });
}

/// Arm one buffer slot of a data endpoint.
///
/// The command word is written whole: the controller tracks the bulk/interrupt
/// data toggle internally per endpoint (read-only EPTOGGLE register), not in
/// the entry's TV bit, which is only consumed together with TR. `tr` requests
/// that toggle reset; it is set exactly once for the first transfer after
/// enable/unstall (the NXP SDK's deferred toggle-reset discipline).
///
/// Isochronous endpoints are marked with `T` and never get `TR`: they have no
/// data toggle to reset.
fn ep_arm_slot<T: Instance>(
    ep_list: u32,
    index: usize,
    dir: Direction,
    slot: usize,
    len: u32,
    buf_addr: u32,
    tr: bool,
    iso: bool,
) {
    let flags = if iso {
        CMD_T
    } else if tr {
        CMD_TR
    } else {
        0
    };
    ep_cmd_write(
        ep_list,
        index,
        dir,
        slot,
        CMD_A | nbytes::<T>(len) | addroff::<T>(buf_addr) | flags,
    );
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
    /// Power up clocks + PHY for this controller. Idempotent.
    fn power_up();
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

    fn power_up() {
        use pac::syscon::vals::{Usb1DevRst, Usb1HostRst, Usb1PhyRst, Usb1RamRst};

        // Reset the USB1 blocks sequentially, per block, with their clocks
        // still off — the exact order lpc55-hal's `enabled_as_device` uses
        // (host, then device + RAM, then PHY). Clocks are enabled per block
        // further down, right before each block is first used.
        critical_section::with(|_| {
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::Asserted));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::Released));
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::Asserted);
                w.set_usb1_ram_rst(Usb1RamRst::Asserted);
            });
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::Released);
                w.set_usb1_ram_rst(Usb1RamRst::Released);
            });
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::Asserted));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::Released));

            // Host block clock on, only briefly, to hand the shared port over.
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_host(true));
        });

        // Hand the shared USB1 port to the device controller; verify the
        // write sticks (it is silently dropped while the block is still
        // completing its reset). Use absolute writes: a read-modify-write on
        // the still-resetting block would write garbage read values back.
        let mut handed_over = false;
        for _ in 0..10_000 {
            let mut pm = pac::usbhsh::regs::Portmode(0);
            pm.set_dev_enable(true);
            // Keep the PHY power-down input (PDCOM) under software control
            // (reset default), released. If hardware control is left to the
            // clock-gated host block, it powers the PHY down.
            pm.set_sw_ctrl_pdcom(true);
            pac::USBHSH.portmode().write_value(pm);
            if pac::USBHSH.portmode().read().dev_enable() {
                handed_over = true;
                break;
            }
        }
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
        let mut locked = false;
        for _ in 0..1_000_000 {
            if phy.pll_sic().read().pll_lock() == pac::usbphy::vals::PllSicPllLock::Value1 {
                locked = true;
                break;
            }
        }
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

        // Device controller + dedicated USB RAM clocks on, last (mirrors
        // lpc55-hal's `enable_clock(USB1)` which gates both).
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| {
                w.set_usb1_dev(true);
                w.set_usb1_ram(true);
            });
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
            pac::SYSCON.ahbclkctrl2().modify(|w| {
                w.set_usb1_dev(false);
                w.set_usb1_ram(false);
                w.set_usb1_phy(false);
            });
        });
    }
}

impl Instance for crate::peripherals::USBHSD {
    type Interrupt = crate::interrupt::typelevel::USB1;

    const EP_COUNT: usize = 6;
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

    fn power_up() {
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
        let mut stable = false;
        for _ in 0..1_000_000 {
            if pac::SYSCON.usb0clkdiv().read().reqflag() == Usb0clkdivReqflag::Stable {
                stable = true;
                break;
            }
        }
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
        let mut adjusted = false;
        for _ in 0..1_000_000 {
            if !pac::ANACTRL.fro192m_ctrl().read().usbmodchg() {
                adjusted = true;
                break;
            }
        }
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
    _lifetime: PhantomData<&'d mut ()>,
}

static USB1_SRAM_TAKEN: AtomicBool = AtomicBool::new(false);

impl<'d> Memory<'d> {
    /// The dedicated 16 KiB USB1 SRAM at `0x4010_0000`.
    ///
    /// Usable by either controller, but only by one at a time: constructing a
    /// second one panics. There is no release path; the region is held for the
    /// lifetime of the driver.
    pub fn usb1_sram() -> Self {
        if USB1_SRAM_TAKEN.swap(true, Ordering::Relaxed) {
            panic!("USB1 SRAM already in use");
        }
        // The dedicated RAM has its own clock gate, independent of whichever
        // controller ends up addressing it.
        critical_section::with(|_| {
            pac::SYSCON.ahbclkctrl2().modify(|w| w.set_usb1_ram(true));
        });
        Self {
            base: USB1_SRAM_ADDR,
            len: USB1_SRAM_SIZE,
            _lifetime: PhantomData,
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
    /// skipped; at least 512 bytes must be left after that. Using this instead
    /// of [`Memory::usb1_sram`] is what allows USB0 and USB1 to run at the same
    /// time.
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
            _lifetime: PhantomData,
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
        let regs = T::regs();
        let state = T::state();
        let stat = regs.intstat().read();
        // W1C exactly the bits we saw. Later-arriving bits stay pending and
        // re-fire the interrupt. Losing an edge is safe by design: the futures
        // re-check level state (eplist Active bit / DEVCMDSTAT.SETUP).
        regs.intstat().write_value(stat);

        for i in 0..T::EP_COUNT {
            if stat.0 & (1 << (2 * i)) != 0 {
                state.ep_out_wakers[i].wake();
            }
            if stat.0 & (1 << (2 * i + 1)) != 0 {
                state.ep_in_wakers[i].wake();
            }
        }

        // A pending SETUP must abort in-flight control transfers in either
        // direction, so wake both EP0 futures.
        if regs.devcmdstat().read().setup() {
            state.ep_out_wakers[0].wake();
            state.ep_in_wakers[0].wake();
        }

        // dev_int: DEVCMDSTAT change bits are consumed by `Bus::poll`.
        if stat.0 & (1 << 31) != 0 {
            state.bus_waker.wake();
        }
    }
}

/// LPC55 USB device driver.
pub struct Driver<'d, T: Instance> {
    _usb: Peri<'d, T>,
    _vbus: Option<Peri<'d, crate::gpio::AnyPin>>,
    ep_in_used: [bool; MAX_EP_COUNT],
    ep_out_used: [bool; MAX_EP_COUNT],
    /// Base of the endpoint memory: the command/status list starts here.
    ep_list: u32,
    /// One past the end of the endpoint memory.
    mem_end: u32,
    /// Bump allocator offset, relative to `ep_list`.
    alloc_offset: u32,
    /// Address of the 8-byte (64-byte aligned) SETUP buffer.
    setup_addr: u32,
}

impl<'d> Driver<'d, crate::peripherals::USBHSD> {
    /// Create a new USB1 high-speed driver.
    ///
    /// This powers up and configures the USB1 PHY (expects a 16 MHz crystal)
    /// and the device controller, but does not connect to the bus yet; the
    /// soft-connect happens in [`driver::Bus::enable`].
    ///
    /// Requires a system clock of at least 96 MHz, see
    /// [`crate::config::MainClock::FroHf96`].
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
        let Memory { base, len, .. } = memory;
        assert!(
            len >= EP_LIST_LEN + 64,
            "USB endpoint memory is too small for the command list and SETUP buffer"
        );
        // AddressOffset only carries the low bits of a buffer address; the
        // rest comes from DATABUFSTART, so the whole region must sit inside
        // one DATABUFSTART window.
        assert!(
            base >> T::DATABUF_WINDOW_SHIFT == (base + len - 1) >> T::DATABUF_WINDOW_SHIFT,
            "endpoint memory crosses the DATABUFSTART window"
        );

        T::power_up();

        // Zero the endpoint command/status list and reserve the SETUP buffer.
        for i in 0..T::EP_COUNT * 4 {
            unsafe { ((base as usize + i * 4) as *mut u32).write_volatile(0) };
        }
        let setup_addr = base + EP_LIST_LEN;
        unsafe {
            (setup_addr as *mut u32).write_volatile(0);
            (setup_addr as *mut u32).add(1).write_volatile(0);
            // EP0 out_buf1 slot holds the SETUP buffer pointer.
            ep_cmd_ptr(base, 0, Direction::Out, true).write_volatile(addroff::<T>(setup_addr));
        }

        program_controller::<T>(base);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _usb: usb,
            _vbus: vbus,
            ep_in_used: [false; MAX_EP_COUNT],
            ep_out_used: [false; MAX_EP_COUNT],
            ep_list: base,
            mem_end: base + len,
            alloc_offset: EP_LIST_LEN + 64,
            setup_addr,
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
        if self.ep_list + self.alloc_offset + len > self.mem_end {
            warn!("USB endpoint memory full");
            return Err(EndpointAllocError);
        }
        let addr = self.ep_list + self.alloc_offset;
        self.alloc_offset += len;
        Ok(addr)
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
        // Every failure path restores the bump allocator, so a rejected
        // allocation consumes neither memory nor an endpoint index.
        let mark = self.alloc_offset;
        let (buf_addrs, slot_len) = match (self.alloc_buffer(want_len), self.alloc_buffer(want_len)) {
            (Ok(a), Ok(b)) => ([a, b], want_len),
            _ => {
                self.alloc_offset = mark;
                if want_len <= max_packet_size {
                    return Err(EndpointAllocError);
                }
                // The multi-packet slots did not fit; single-packet slots may.
                match (self.alloc_buffer(max_packet_size), self.alloc_buffer(max_packet_size)) {
                    (Ok(a), Ok(b)) => ([a, b], max_packet_size),
                    _ => {
                        self.alloc_offset = mark;
                        return Err(EndpointAllocError);
                    }
                }
            }
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
        ep_cmd_write(self.ep_list, index, D::dir(), 0, CMD_D);
        ep_cmd_write(self.ep_list, index, D::dir(), 1, CMD_D);

        trace!("  index={} addr={:08x}", index, buf_addrs[0]);

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
            ep_list: self.ep_list,
            buf_addrs,
            slot_len,
            armed_len: [0; 2],
        })
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
        let ep_list = self.ep_list;
        let ep0_out_addr = self.alloc_buffer(control_max_packet_size).unwrap();
        let ep0_in_addr = self.alloc_buffer(control_max_packet_size).unwrap();

        // EP0 OUT stays armed at all times (lpc55-hal discipline): the ip3511
        // needs an active EP0 OUT entry to accept SETUP and OUT packets on the
        // control pipe. `setup()`/`data_out()` re-arm it after every consumed
        // packet.
        ep_cmd_write(
            ep_list,
            0,
            Direction::Out,
            0,
            CMD_A | nbytes::<T>(control_max_packet_size as u32) | addroff::<T>(ep0_out_addr),
        );
        ep_cmd_write(ep_list, 0, Direction::In, 0, addroff::<T>(ep0_in_addr));

        trace!("started");

        (
            Bus {
                _phantom: PhantomData,
                _vbus: self._vbus,
                // Deliberately `false` rather than sampled from the register:
                // a `false` start is what makes the first `poll()` emit
                // `PowerDetected` for an already-attached cable.
                vbus: false,
                powered: true,
                ep_list,
                setup_addr: self.setup_addr,
                ep0_out_addr,
                ep0_mps: control_max_packet_size,
            },
            ControlPipe {
                _phantom: PhantomData,
                max_packet_size: control_max_packet_size,
                ep_list,
                setup_addr: self.setup_addr,
                ep0_out_addr,
                ep0_in_addr,
            },
        )
    }
}

/// USB bus.
pub struct Bus<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    _vbus: Option<Peri<'d, crate::gpio::AnyPin>>,
    vbus: bool,
    powered: bool,
    ep_list: u32,
    setup_addr: u32,
    ep0_out_addr: u32,
    ep0_mps: u16,
}

impl<'d, T: Instance> Bus<'d, T> {
    /// Re-apply the controller programming `Driver::new_inner` does, after a
    /// `power_up` that reset the block.
    fn reinit(&mut self) {
        program_controller::<T>(self.ep_list);
        self.reset_endpoints();
    }

    /// Disable all data endpoints, reset the double-buffer bookkeeping and
    /// re-arm EP0. Shared by the bus-reset path and `reinit`.
    fn reset_endpoints(&mut self) {
        let regs = T::regs();
        let state = T::state();

        for i in 1..T::EP_COUNT {
            for slot in 0..2 {
                ep_cmd_write(self.ep_list, i, Direction::Out, slot, CMD_D);
                ep_cmd_write(self.ep_list, i, Direction::In, slot, CMD_D);
            }
            // Keep ISO: it describes the endpoint, not its transfer state.
            for dir in [Direction::Out, Direction::In] {
                let s = state.ep_state(i, dir);
                s.store(s.load(Ordering::Relaxed) & ISO, Ordering::Relaxed);
            }
        }
        regs.epinuse().write(|w| w.set_buf(0));
        regs.epbufcfg().write(|w| w.set_buf_sb(ep_buf_mask::<T>()));

        // EP0 OUT stays armed at all times; keep the SETUP pointer.
        ep_cmd_write(
            self.ep_list,
            0,
            Direction::Out,
            0,
            CMD_A | nbytes::<T>(self.ep0_mps as u32) | addroff::<T>(self.ep0_out_addr),
        );
        ep_cmd_write(self.ep_list, 0, Direction::In, 0, 0);
        unsafe {
            ep_cmd_ptr(self.ep_list, 0, Direction::Out, true).write_volatile(addroff::<T>(self.setup_addr));
        }

        // Unstick any pending endpoint futures; they will observe the
        // disabled/reset state.
        for w in &state.ep_in_wakers {
            w.wake();
        }
        for w in &state.ep_out_wakers {
            w.wake();
        }
    }
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    async fn enable(&mut self) {
        if !self.powered {
            T::power_up();
            self.reinit();
            self.powered = true;
        }
        // Soft-connect.
        devcmdstat_modify(T::regs(), |w| w.set_dcon(true));
    }

    async fn disable(&mut self) {
        devcmdstat_modify(T::regs(), |w| w.set_dcon(false));
        devcmdstat_modify(T::regs(), |w| w.set_dev_en(false));
        T::power_down();
        self.powered = false;
    }

    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            T::state().bus_waker.register(cx.waker());

            let regs = T::regs();
            let dcs = regs.devcmdstat().read();

            // VBUS level changes are reported before anything else: a lost
            // cable invalidates every other event.
            let vbus = dcs.vbus_debounced();
            if vbus != self.vbus {
                self.vbus = vbus;
                let event = if vbus {
                    Event::PowerDetected
                } else {
                    Event::PowerRemoved
                };
                trace!("{}", event);
                return Poll::Ready(event);
            }

            if dcs.dres_c() {
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
                // Acknowledge only. Hardware gates the D+ pull-up on
                // VBUS_DEBOUNCED, so a VBUS change forces a connect-state
                // change: this is the wake that carries it, and the level was
                // already picked up above.
                devcmdstat_modify(regs, |w| w.set_dcon_c(true));
            }

            Poll::Pending
        })
        .await
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:?} {}", ep_addr, enabled);
        let index = ep_addr.index();
        if index == 0 {
            return;
        }
        let dir = ep_addr.direction();
        let state = T::state().ep_state(index, dir);
        let iso = state.load(Ordering::Relaxed) & ISO;

        if enabled {
            // Both slots idle (NAK until armed) with the data toggle reset,
            // which USB requires whenever a SET_CONFIGURATION or SET_INTERFACE
            // enables an endpoint. Hardware must also restart on slot 0 to match
            // the software cursor.
            //
            // `TR` has to be in the word here and not only deferred to the first
            // armed transfer via `TR_PENDING`: the full-speed block otherwise
            // keeps the pre-disable toggle and silently drops the first packets
            // after an alternate setting switches the endpoint back on.
            //
            // Only slot 0 carries it. `TR` is honoured on an inactive entry, so
            // arming both slots with it lets hardware reset the toggle a second
            // time after the first packet has already flipped it, which corrupts
            // the very next transfer. Isochronous endpoints have no toggle at all.
            ep_cmd_write(self.ep_list, index, dir, 0, if iso != 0 { 0 } else { CMD_TR });
            ep_cmd_write(self.ep_list, index, dir, 1, 0);
            let phy_ep = 2 * index + (dir == Direction::In) as usize;
            T::regs()
                .epinuse()
                .modify(|w| w.set_buf(w.buf() & !(1 << (phy_ep - 2))));
            state.store(iso | TR_PENDING, Ordering::Relaxed);
        } else {
            ep_cmd_write(self.ep_list, index, dir, 0, CMD_D);
            ep_cmd_write(self.ep_list, index, dir, 1, CMD_D);
            state.store(iso, Ordering::Relaxed);
        }

        T::state().ep_waker(index, dir).wake();
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        trace!("set_stalled {:?} {}", ep_addr, stalled);
        let index = ep_addr.index();
        let dir = ep_addr.direction();
        let regs = T::regs();
        let phy_ep = 2 * index + (dir == Direction::In) as usize;
        // EP0's second OUT slot is the SETUP pointer: never touch it.
        let slots = if index == 0 { 0..1 } else { 0..2 };
        // Isochronous endpoints have no handshake phase, so the S bit is
        // meaningless for them: reclaim and resync, but never stall.
        let iso = index != 0 && T::state().ep_state(index, dir).load(Ordering::Relaxed) & ISO != 0;

        if stalled {
            for slot in slots.clone() {
                if ep_cmd_read(self.ep_list, index, dir, slot) & CMD_A != 0 {
                    // Reclaim the in-flight transfer via EPSKIP (skips the
                    // slot EPINUSE points at).
                    regs.epskip().write_value(pac::usbhsd::regs::Epskip(1 << phy_ep));
                    while regs.epskip().read().0 & (1 << phy_ep) != 0 {}
                    regs.intstat().write_value(pac::usbhsd::regs::Intstat(1 << phy_ep));
                }
            }
            if !iso {
                for slot in slots {
                    ep_cmd_modify(self.ep_list, index, dir, slot, |w| (w & !CMD_A) | CMD_S);
                }
            }
        } else if index == 0 {
            // Clearing a stall resets the data toggle to DATA0.
            ep_cmd_modify(self.ep_list, index, dir, 0, |w| (w & !CMD_S) | CMD_TR);
        } else {
            // Drop any buffered packets, resync the hardware slot cursor and
            // reset the data toggle, which USB requires when a halt is cleared.
            //
            // Both slot words are rewritten whole rather than having `S` and `A`
            // masked out: keeping the aborted transfer's NBytes and
            // AddressOffset leaves the full-speed block wedged - it acknowledges
            // the next OUT packets on the wire and discards them without ever
            // completing the entry.
            //
            // The toggle reset lands on slot 0 only, for the reason spelled out
            // in `endpoint_set_enabled`.
            for slot in slots {
                let word = if slot == 0 && !iso { CMD_TR } else { 0 };
                ep_cmd_write(self.ep_list, index, dir, slot, word);
            }
            regs.epinuse().modify(|w| w.set_buf(w.buf() & !(1 << (phy_ep - 2))));
            let state = T::state().ep_state(index, dir);
            let keep = state.load(Ordering::Relaxed) & ISO;
            state.store(keep | TR_PENDING, Ordering::Relaxed);
        }

        T::state().ep_waker(index, dir).wake();
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        // Isochronous endpoints are never stalled, and `endpoint_set_stalled`
        // never sets CMD_S on them, so this reports `false` for them.
        ep_cmd_read(self.ep_list, ep_addr.index(), ep_addr.direction(), 0) & CMD_S != 0
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        devcmdstat_modify(T::regs(), |w| w.set_dsus(false));
        Ok(())
    }

    fn force_reset(&mut self) -> Result<(), Unsupported> {
        devcmdstat_modify(T::regs(), |w| w.set_dcon(false));
        // The host needs >= 10 us of SE0 to see a disconnect. This is a
        // blocking trait method, so busy-wait: 10M cycles is 33-104 ms across
        // the supported main clocks (300 MHz down to 96 MHz) - generous, but
        // this path runs at most once per host-visible reconnect.
        cortex_m::asm::delay(10_000_000);
        devcmdstat_modify(T::regs(), |w| w.set_dcon(true));
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
}

impl<'d, T: Instance, D: Dir> Endpoint<'d, T, D> {
    fn is_iso(&self) -> bool {
        self.info.ep_type == EndpointType::Isochronous
    }
}

impl<'d, T: Instance, D: Dir> driver::Endpoint for Endpoint<'d, T, D> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        let index = self.info.addr.index();
        poll_fn(|cx| {
            T::state().ep_waker(index, D::dir()).register(cx.waker());
            if ep_cmd_read(self.ep_list, index, D::dir(), 0) & CMD_D == 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
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
        let state = T::state().ep_state(index, Direction::Out);
        let iso = self.is_iso();
        // Keep both slots armed so the endpoint does not NAK between packets
        // while the host streams. A slot with PRIMED set and Active clear
        // holds a received, not-yet-consumed packet.
        let mut s = state.load(Ordering::Relaxed);
        for slot in 0..2 {
            if s & (PRIMED0 << slot) == 0 {
                if ep_cmd_read(self.ep_list, index, Direction::Out, slot) & CMD_UNUSABLE != 0 {
                    return Err(EndpointError::Disabled);
                }
                ep_arm_slot::<T>(
                    self.ep_list,
                    index,
                    Direction::Out,
                    slot,
                    arm_len,
                    self.buf_addrs[slot],
                    s & TR_PENDING != 0,
                    iso,
                );
                self.armed_len[slot] = arm_len as u16;
                s = (s | (PRIMED0 << slot)) & !TR_PENDING;
                state.store(s, Ordering::Relaxed);
            }
        }

        // Consume in hardware ping-pong order.
        let slot = (s & SLOT) as usize;
        let word = poll_fn(|cx| {
            T::state().ep_out_wakers[index].register(cx.waker());
            let word = ep_cmd_read(self.ep_list, index, Direction::Out, slot);
            if word & CMD_UNUSABLE != 0 {
                // Disabled by a bus reset, or stalled by the host, while waiting.
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if state.load(Ordering::Relaxed) & (PRIMED0 << slot) == 0 {
                // A re-enable or unstall cancelled the armed transfer.
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if word & CMD_A == 0 {
                Poll::Ready(Ok(word))
            } else {
                Poll::Pending
            }
        })
        .await?;

        // A dropped or errored isochronous packet completes the entry with
        // NBytes untouched, which this arithmetic turns into `Ok(0)`. That is
        // the correct report for isochronous: there is no retry, and the
        // caller sees a lost frame rather than an error. Do not "fix" it.
        let rx_len = (self.armed_len[slot] as u32 - remaining_bytes::<T>(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_ep_buffer(self.buf_addrs[slot], &mut buf[..rx_len]);

        // A host halt landing between the completion poll and here would be
        // erased by the re-arm below, so drop the packet instead: the host asked
        // for the endpoint to stop.
        if ep_cmd_read(self.ep_list, index, Direction::Out, slot) & CMD_S != 0 {
            return Err(EndpointError::Disabled);
        }

        // Hand the drained slot straight back to hardware and advance the
        // consume cursor.
        ep_arm_slot::<T>(
            self.ep_list,
            index,
            Direction::Out,
            slot,
            arm_len,
            self.buf_addrs[slot],
            false,
            iso,
        );
        self.armed_len[slot] = arm_len as u16;
        state.store((s ^ SLOT) | (PRIMED0 << slot), Ordering::Relaxed);

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
        let state = T::state().ep_state(index, Direction::In);
        let iso = self.is_iso();

        // Fill slots in hardware ping-pong order, returning as soon as the
        // packet is armed: the wire transmits from one slot while software
        // fills the other, so a saturating writer never lets the endpoint
        // NAK an IN token.
        loop {
            let s = state.load(Ordering::Relaxed);
            let slot = (s & SLOT) as usize;

            poll_fn(|cx| {
                T::state().ep_in_wakers[index].register(cx.waker());
                let word = ep_cmd_read(self.ep_list, index, Direction::In, slot);
                if word & CMD_UNUSABLE != 0 {
                    return Poll::Ready(Err(EndpointError::Disabled));
                }
                if word & CMD_A == 0 {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            })
            .await?;

            // A reset or re-enable while waiting may have moved the slot
            // cursor; re-derive and retry if so.
            let s = state.load(Ordering::Relaxed);
            if (s & SLOT) as usize != slot {
                continue;
            }

            // Same hazard as in `read`: a host halt landing here would be
            // erased by the arm below.
            if ep_cmd_read(self.ep_list, index, Direction::In, slot) & CMD_S != 0 {
                return Err(EndpointError::Disabled);
            }

            copy_to_ep_buffer(self.buf_addrs[slot], buf);
            ep_arm_slot::<T>(
                self.ep_list,
                index,
                Direction::In,
                slot,
                buf.len() as u32,
                self.buf_addrs[slot],
                s & TR_PENDING != 0,
                iso,
            );
            state.store((s ^ SLOT) & !TR_PENDING, Ordering::Relaxed);

            trace!("WRITE {:?} len = {}", self.info.addr, buf.len());
            return Ok(());
        }
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
    max_packet_size: u16,
    ep_list: u32,
    setup_addr: u32,
    ep0_out_addr: u32,
    ep0_in_addr: u32,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        self.max_packet_size as usize
    }

    async fn setup(&mut self) -> [u8; 8] {
        let regs = T::regs();

        poll_fn(|cx| {
            T::state().ep_out_wakers[0].register(cx.waker());
            if regs.devcmdstat().read().setup() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // UM procedure: clear Active and Stall on both EP0 directions BEFORE
        // acknowledging the SETUP bit.
        ep_cmd_modify(self.ep_list, 0, Direction::Out, 0, |w| w & !(CMD_A | CMD_S));
        ep_cmd_modify(self.ep_list, 0, Direction::In, 0, |w| w & !(CMD_A | CMD_S));

        let mut buf = [0; 8];
        copy_from_ep_buffer(self.setup_addr, &mut buf);

        devcmdstat_modify(regs, |w| w.set_setup(true));

        // Hardware may overwrite the SETUP slot's address field: re-write it,
        // and re-arm EP0 OUT for the next packet.
        unsafe {
            ep_cmd_ptr(self.ep_list, 0, Direction::Out, true).write_volatile(addroff::<T>(self.setup_addr));
        }
        ep_cmd_write(
            self.ep_list,
            0,
            Direction::Out,
            0,
            CMD_A | nbytes::<T>(self.max_packet_size as u32) | addroff::<T>(self.ep0_out_addr),
        );

        trace!("SETUP {=[u8]:x}", buf);
        buf
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        let regs = T::regs();
        let mps = self.max_packet_size as u32;

        // EP0 OUT is already armed (from `setup()` or the previous
        // `data_out()`); just wait for the packet.

        let word = poll_fn(|cx| {
            T::state().ep_out_wakers[0].register(cx.waker());
            // A new SETUP aborts the transfer (trait contract).
            if regs.devcmdstat().read().setup() {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let word = ep_cmd_read(self.ep_list, 0, Direction::Out, 0);
            if word & CMD_A == 0 {
                Poll::Ready(Ok(word))
            } else {
                Poll::Pending
            }
        })
        .await?;

        let rx_len = (mps - remaining_bytes::<T>(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_ep_buffer(self.ep0_out_addr, &mut buf[..rx_len]);

        // Re-arm for the next OUT packet (or the status stage).
        ep_cmd_write(
            self.ep_list,
            0,
            Direction::Out,
            0,
            CMD_A | nbytes::<T>(mps) | addroff::<T>(self.ep0_out_addr),
        );

        trace!("control: data_out rx_len = {}", rx_len);
        Ok(rx_len)
    }

    async fn data_in(&mut self, data: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in len = {} last = {}", data.len(), last);
        if data.len() > self.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let regs = T::regs();
        if regs.devcmdstat().read().setup() {
            return Err(EndpointError::Disabled);
        }

        copy_to_ep_buffer(self.ep0_in_addr, data);
        ep_cmd_write(
            self.ep_list,
            0,
            Direction::In,
            0,
            CMD_A | nbytes::<T>(data.len() as u32) | addroff::<T>(self.ep0_in_addr),
        );

        poll_fn(|cx| {
            T::state().ep_in_wakers[0].register(cx.waker());
            if regs.devcmdstat().read().setup() {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if ep_cmd_read(self.ep_list, 0, Direction::In, 0) & CMD_A == 0 {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await?;

        // The status-stage OUT ZLP is accepted by the always-armed EP0 OUT
        // buffer; nothing else to do for `last`.
        let _ = last;

        Ok(())
    }

    async fn accept(&mut self) {
        trace!("control: accept");

        // Zero-length status-stage IN packet.
        ep_cmd_write(
            self.ep_list,
            0,
            Direction::In,
            0,
            CMD_A | addroff::<T>(self.ep0_in_addr),
        );

        poll_fn(|cx| {
            T::state().ep_in_wakers[0].register(cx.waker());
            // Bail out if the host abandoned the request with a new SETUP.
            if T::regs().devcmdstat().read().setup() {
                return Poll::Ready(());
            }
            if ep_cmd_read(self.ep_list, 0, Direction::In, 0) & CMD_A == 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    async fn reject(&mut self) {
        trace!("control: reject");
        // Stall both EP0 directions; `setup()` clears the stalls on the next
        // SETUP packet.
        ep_cmd_modify(self.ep_list, 0, Direction::Out, 0, |w| w | CMD_S);
        ep_cmd_modify(self.ep_list, 0, Direction::In, 0, |w| w | CMD_S);
    }

    async fn accept_set_address(&mut self, addr: u8) {
        trace!("setting addr: {}", addr);
        // ip3511 quirk (UM11126): the new device address must be programmed
        // BEFORE the status stage completes, contrary to the USB spec's
        // "after the status stage" wording. lpc55-hal encodes the same order
        // via usb-device's `QUIRK_SET_ADDRESS_BEFORE_STATUS`.
        devcmdstat_modify(T::regs(), |w| w.set_dev_addr(addr));
        self.accept().await;
    }
}
