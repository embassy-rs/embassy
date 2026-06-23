//! USB 2.0 device driver for the MCXA5xx USBHS controller.
//!
//! The MCXA5xx exposes a single Chipidea/EHCI USB-OTG core (`USB1_HS`) with an
//! integrated high-speed PHY. This driver operates that core in **device mode**
//! and forces **full speed** (12 Mbit/s) operation, which is the configuration
//! used by simple USB device classes such as HID.
//!
//! It implements the [`embassy_usb_driver`] traits so it can be used directly
//! with `embassy-usb` (e.g. the HID class).
//!
//! # Board clocking
//! The MCXA577 USBHS PHY expects the board clock tree to match the SDK/Zephyr
//! profile used by FRDM-MCXA577: over-drive voltage mode with the 24 MHz SOSC
//! crystal enabled. [`Driver::new`] returns an error if that clocking is not
//! available or if the USBPHY PLL does not lock.
//!
//! # Limitations
//! - Device mode only (no host / OTG role switching).
//! - Forced full speed; high-speed operation is not exposed.
//! - No hardware VBUS detection yet; the bus reports power detected at startup.
//! - Remote wakeup is not implemented.
//! - Isochronous endpoints and packet sizes greater than 64 bytes are not
//!   supported.
//! - One transfer descriptor in flight per endpoint direction at a time.
//!
//! # Register access
//! MMIO register blocks come from `nxp-pac`. The local [`ehci`] module only
//! contains the EHCI DMA queue-head and transfer-descriptor RAM formats that are
//! not MMIO registers.

mod clock;
mod ehci;

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU32, Ordering};
use core::task::Poll;

pub use clock::PhyConfig;
use cortex_m::asm;
use ehci::{
    DTD_BUFFER_PAGE_MASK, DTD_BUFFER_PAGE_SIZE, DTD_NEXT_TERMINATE, DTD_TOKEN_ACTIVE, DTD_TOKEN_ERROR_MASK,
    DTD_TOKEN_IOC, DTD_TOKEN_TOTAL_MASK, DTD_TOKEN_TOTAL_SHIFT, QH_CAP_IOS, QH_CAP_MAXLEN_SHIFT, QH_CAP_ZLT, QueueHead,
    TransferDescriptor,
};
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use crate::clocks::periph_helpers::UsbhsConfig;
use crate::clocks::{self, ClockError, Gate, WakeGuard};
use crate::interrupt;
use crate::interrupt::typelevel::{Binding, Interrupt};

/// USBHS device-driver configuration.
///
/// The controller is currently always placed in device mode and forced to
/// full-speed operation. The configuration carries board-specific PHY trim and
/// PLL settings so applications can use the FRDM-MCXA577 defaults or provide
/// calibrated values from their board support package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// USBHS clock-source and PHY reference divider settings.
    pub clock: UsbhsConfig,
    /// USBPHY trim and PLL settings.
    pub phy: PhyConfig,
}

impl From<PhyConfig> for Config {
    fn from(phy: PhyConfig) -> Self {
        Self { phy, ..Self::default() }
    }
}

/// Error returned while creating the USB driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum InitError {
    /// USB clock configuration failed.
    Clock(ClockError),
    /// PHY trim or PLL configuration is outside the hardware field range.
    PhyConfig,
    /// USBPHY PLL did not lock.
    PhyPllLock,
}

impl From<ClockError> for InitError {
    fn from(err: ClockError) -> Self {
        Self::Clock(err)
    }
}

/// Number of bidirectional endpoint pairs supported (EP0..EP{N-1}).
///
/// The MCXA577 USBHS controller exposes eight bidirectional endpoint pairs.
const EP_COUNT: usize = 8;

/// Maximum packet size for a full-speed endpoint.
const FS_MAX_PACKET: u16 = 64;
const ENDPOINT_TYPE_UNALLOCATED: u8 = 0xFF;
const ALL_USBSTS: u32 = 0xFFFF_FFFF;
const ALL_ENDPOINT_BITS: u32 = 0xFFFF_FFFF;
const DEFAULT_BURST_SIZE: u32 = (0x10 << 8) | 0x10;
const ENDPOINT_WAIT_ITERS: usize = 100_000;

// The PAC exposes generated field accessors for these registers. These raw
// masks stay local to the register wrapper because the endpoint registers are an
// indexed array in hardware but generated as distinct Rust types per endpoint.
const USBCMD_RS: u32 = 1 << 0;
const USBCMD_RST: u32 = 1 << 1;
const USBCMD_SUTW: u32 = 1 << 13;
const USBSTS_UI: u32 = 1 << 0;
const USBSTS_UEI: u32 = 1 << 1;
const USBSTS_PCI: u32 = 1 << 2;
const USBSTS_URI: u32 = 1 << 6;
const USBSTS_SLI: u32 = 1 << 8;
const USBMODE_CM_DEVICE: u32 = 0b10;
const USBMODE_SLOM: u32 = 1 << 3;
const PORTSC1_PFSC: u32 = 1 << 24;
const DEVICEADDR_USBADRA: u32 = 1 << 24;
const DEVICEADDR_USBADR_SHIFT: u32 = 25;
const EPCTRL_RXE: u32 = 1 << 7;
const EPCTRL_RXR: u32 = 1 << 6;
const EPCTRL_RXS: u32 = 1 << 0;
const EPCTRL_RXT_SHIFT: u32 = 2;
const EPCTRL_TXE: u32 = 1 << 23;
const EPCTRL_TXR: u32 = 1 << 22;
const EPCTRL_TXS: u32 = 1 << 16;
const EPCTRL_TXT_SHIFT: u32 = 18;

/// Static DMA memory cell.
///
/// The USBHS controller reads and writes these objects directly. This wrapper
/// keeps the global mutability localized and makes all access sites spell out
/// their raw-pointer safety instead of exposing `static mut` items throughout
/// the driver.
struct DmaCell<T>(UnsafeCell<T>);

impl<T> DmaCell<T> {
    const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    #[inline]
    fn as_ptr(&self) -> *mut T {
        self.0.get()
    }
}

// SAFETY: USB driver ownership plus per-endpoint transfer ownership serialize
// mutable access. The controller may access the memory concurrently as DMA, so
// all synchronization with hardware is done with explicit barriers.
unsafe impl<T> Sync for DmaCell<T> {}

// ---- DMA-visible controller structures (statically allocated) ----
//
// The USBHS controller reads and writes these structures directly via DMA. The
// current MCX-A target has no data cache and its SRAM is non-cacheable, so plain
// statics stay coherent with the controller using only the `dsb()` barriers in
// `dma_clean` / `dma_invalidate`. Those two functions are the single porting hook:
// a part with a data cache must grow real clean/invalidate operations there and
// place this state in a non-cacheable region. `Driver::new` debug-asserts that the
// data cache is disabled so the assumption cannot be silently violated.

/// The device queue-head list. Must be 2 KiB aligned and contain `2 * EP_COUNT`
/// entries (OUT at even indices, IN at odd indices).
#[repr(C, align(2048))]
struct QhList {
    qh: [QueueHead; EP_COUNT * 2],
}

/// One transfer descriptor per endpoint direction.
#[repr(C, align(32))]
struct DtdList {
    dtd: [TransferDescriptor; EP_COUNT * 2],
}

static QH_LIST: DmaCell<QhList> = DmaCell::new(QhList {
    qh: [QueueHead::new(); EP_COUNT * 2],
});
static DTD_LIST: DmaCell<DtdList> = DmaCell::new(DtdList {
    dtd: [TransferDescriptor::new(); EP_COUNT * 2],
});

/// Per-endpoint-direction transfer-complete wakers (index = `2*ep + dir`).
static EP_WAKERS: [AtomicWaker; EP_COUNT * 2] = [const { AtomicWaker::new() }; EP_COUNT * 2];
/// Waker for bus events (reset/suspend/resume) handled in [`Bus::poll`].
static BUS_WAKER: AtomicWaker = AtomicWaker::new();

/// Bus event flags set by the interrupt handler and consumed by [`Bus::poll`].
static FLAG_RESET: AtomicBool = AtomicBool::new(false);
static FLAG_SUSPEND: AtomicBool = AtomicBool::new(false);
static FLAG_RESUME: AtomicBool = AtomicBool::new(false);
static SUSPENDED: AtomicBool = AtomicBool::new(false);
static TRANSFER_ABORT_EPOCH: AtomicU32 = AtomicU32::new(0);

static ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
static RESET_COUNT: AtomicU32 = AtomicU32::new(0);
static SUSPEND_COUNT: AtomicU32 = AtomicU32::new(0);
static RESUME_COUNT: AtomicU32 = AtomicU32::new(0);

/// Per-endpoint-direction configuration captured at allocation time and applied
/// by [`Bus::endpoint_set_enabled`] (index = `2*ep + dir`).
static EP_MAX_PACKET: [AtomicU16; EP_COUNT * 2] = [const { AtomicU16::new(0) }; EP_COUNT * 2];
static EP_TYPE: [AtomicU8; EP_COUNT * 2] = [const { AtomicU8::new(ENDPOINT_TYPE_UNALLOCATED) }; EP_COUNT * 2];

/// Snapshot of USBHS diagnostic counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DiagnosticCounters {
    /// USB error interrupts.
    pub errors: u32,
    /// Bus reset events.
    pub resets: u32,
    /// Suspend events.
    pub suspends: u32,
    /// Resume events.
    pub resumes: u32,
}

/// Return the current USBHS diagnostic counters.
pub fn diagnostic_counters() -> DiagnosticCounters {
    DiagnosticCounters {
        errors: ERROR_COUNT.load(Ordering::Relaxed),
        resets: RESET_COUNT.load(Ordering::Relaxed),
        suspends: SUSPEND_COUNT.load(Ordering::Relaxed),
        resumes: RESUME_COUNT.load(Ordering::Relaxed),
    }
}

fn wake_all_endpoint_wakers() {
    for waker in &EP_WAKERS {
        waker.wake();
    }
}

fn abort_endpoint_transfers() {
    TRANSFER_ABORT_EPOCH.fetch_add(1, Ordering::Relaxed);
    wake_all_endpoint_wakers();
}

fn reset_static_state() {
    FLAG_RESET.store(false, Ordering::Relaxed);
    FLAG_SUSPEND.store(false, Ordering::Relaxed);
    FLAG_RESUME.store(false, Ordering::Relaxed);
    SUSPENDED.store(false, Ordering::Relaxed);
    TRANSFER_ABORT_EPOCH.fetch_add(1, Ordering::Relaxed);
    ERROR_COUNT.store(0, Ordering::Relaxed);
    RESET_COUNT.store(0, Ordering::Relaxed);
    SUSPEND_COUNT.store(0, Ordering::Relaxed);
    RESUME_COUNT.store(0, Ordering::Relaxed);

    for ep_type in &EP_TYPE {
        ep_type.store(ENDPOINT_TYPE_UNALLOCATED, Ordering::Relaxed);
    }
    for max_packet in &EP_MAX_PACKET {
        max_packet.store(0, Ordering::Relaxed);
    }

    // SAFETY: Driver construction has exclusive ownership of the peripheral.
    unsafe {
        let qh = qh_ptr();
        let dtd = dtd_ptr();
        for i in 0..EP_COUNT * 2 {
            (*qh)[i] = QueueHead::new();
            (*dtd)[i] = TransferDescriptor::new();
        }
        core::ptr::write_bytes(EP_BUFFERS.as_ptr(), 0, 1);
    }

    wake_all_endpoint_wakers();
}

#[derive(Clone, Copy)]
struct EndpointSlot {
    index: usize,
    dir: Direction,
}

impl EndpointSlot {
    #[inline]
    fn new(index: usize, dir: Direction) -> Self {
        Self { index, dir }
    }

    #[inline]
    fn from_addr(addr: EndpointAddress) -> Self {
        let dir = if addr.is_in() { Direction::In } else { Direction::Out };
        Self::new(addr.index(), dir)
    }

    #[inline]
    fn array_index(self) -> usize {
        self.index * 2 + self.dir_index()
    }

    #[inline]
    fn dir_index(self) -> usize {
        match self.dir {
            Direction::Out => 0,
            Direction::In => 1,
        }
    }

    #[inline]
    #[cfg_attr(not(feature = "defmt"), allow(dead_code))]
    fn dir_num(self) -> u8 {
        self.dir_index() as u8
    }

    #[inline]
    fn endpoint_bit(self) -> u32 {
        match self.dir {
            Direction::Out => 1 << self.index,
            Direction::In => 1 << (self.index + 16),
        }
    }

    #[inline]
    fn is_enabled(self, r: UsbRegs) -> bool {
        r.endptctrl(self.index) & endpoint_enable_bit(self) != 0
    }

    #[inline]
    fn waker(self) -> &'static AtomicWaker {
        &EP_WAKERS[self.array_index()]
    }
}

#[inline]
fn endpoint_type_bits(ep_type: EndpointType) -> u32 {
    match ep_type {
        EndpointType::Control => 0b00,
        EndpointType::Isochronous => 0b01,
        EndpointType::Bulk => 0b10,
        EndpointType::Interrupt => 0b11,
    }
}

#[inline]
fn endpoint_ctrl_enable_bits(slot: EndpointSlot, ep_type: EndpointType) -> (u32, u32) {
    let ty = endpoint_type_bits(ep_type);
    match slot.dir {
        Direction::Out => (
            0b11 << EPCTRL_RXT_SHIFT,
            (ty << EPCTRL_RXT_SHIFT) | EPCTRL_RXE | EPCTRL_RXR,
        ),
        Direction::In => (
            0b11 << EPCTRL_TXT_SHIFT,
            (ty << EPCTRL_TXT_SHIFT) | EPCTRL_TXE | EPCTRL_TXR,
        ),
    }
}

#[inline]
fn endpoint_stall_bit(slot: EndpointSlot) -> u32 {
    match slot.dir {
        Direction::Out => EPCTRL_RXS,
        Direction::In => EPCTRL_TXS,
    }
}

#[inline]
fn endpoint_enable_bit(slot: EndpointSlot) -> u32 {
    match slot.dir {
        Direction::Out => EPCTRL_RXE,
        Direction::In => EPCTRL_TXE,
    }
}

#[inline]
fn endpoint_reset_bit(slot: EndpointSlot) -> u32 {
    match slot.dir {
        Direction::Out => EPCTRL_RXR,
        Direction::In => EPCTRL_TXR,
    }
}

#[inline]
fn endpoint_max_packet(slot: EndpointSlot) -> Option<u16> {
    let max_packet = EP_MAX_PACKET[slot.array_index()].load(Ordering::Relaxed);
    (max_packet != 0).then_some(max_packet)
}

#[inline]
fn qh_ptr() -> *mut [QueueHead; EP_COUNT * 2] {
    // SAFETY: only constructs a raw pointer to DMA-owned static memory.
    unsafe { core::ptr::addr_of_mut!((*QH_LIST.as_ptr()).qh) }
}

#[inline]
fn dtd_ptr() -> *mut [TransferDescriptor; EP_COUNT * 2] {
    // SAFETY: only constructs a raw pointer to DMA-owned static memory.
    unsafe { core::ptr::addr_of_mut!((*DTD_LIST.as_ptr()).dtd) }
}

#[inline]
fn ep_buffer_ptr(slot: EndpointSlot) -> *mut u8 {
    // SAFETY: only constructs a raw pointer to the endpoint's dedicated bounce
    // buffer. The endpoint future owns that slot while the transfer is active.
    unsafe { core::ptr::addr_of_mut!((*EP_BUFFERS.as_ptr()).buf[slot.array_index()]) as *mut u8 }
}

/// Controller register handle.
#[derive(Clone, Copy)]
struct UsbRegs(crate::pac::usbhs::Usbhs);

impl UsbRegs {
    #[inline]
    #[cfg_attr(not(feature = "defmt"), allow(dead_code))]
    fn id(self) -> u32 {
        self.0.ID().read().0
    }

    #[inline]
    fn usbcmd(self) -> u32 {
        self.0.USBCMD().read().0
    }

    #[inline]
    fn set_usbcmd(self, bits: u32) {
        self.0.USBCMD().write(|w| w.0 = bits);
    }

    #[inline]
    fn modify_usbcmd(self, f: impl FnOnce(u32) -> u32) {
        self.set_usbcmd(f(self.usbcmd()));
    }

    #[inline]
    fn usbsts(self) -> u32 {
        self.0.USBSTS().read().0
    }

    #[inline]
    fn clear_usbsts(self, bits: u32) {
        self.0.USBSTS().write(|w| w.0 = bits);
    }

    #[inline]
    fn usbintr(self) -> u32 {
        self.0.USBINTR().read().0
    }

    #[inline]
    fn set_usbintr(self, bits: u32) {
        self.0.USBINTR().write(|w| w.0 = bits);
    }

    #[inline]
    fn set_deviceaddr(self, bits: u32) {
        self.0.DEVICEADDR().write(|w| w.0 = bits);
    }

    #[inline]
    fn set_endptlistaddr(self, bits: u32) {
        self.0.ENDPTLISTADDR().write(|w| w.0 = bits);
    }

    #[inline]
    fn set_burstsize(self, bits: u32) {
        self.0.BURSTSIZE().write(|w| w.0 = bits);
    }

    #[inline]
    fn portsc1(self) -> u32 {
        self.0.PORTSC1().read().0
    }

    #[inline]
    fn set_portsc1(self, bits: u32) {
        self.0.PORTSC1().write(|w| w.0 = bits);
    }

    #[inline]
    fn modify_portsc1(self, f: impl FnOnce(u32) -> u32) {
        self.set_portsc1(f(self.portsc1()));
    }

    #[inline]
    fn set_usbmode(self, bits: u32) {
        self.0.USBMODE().write(|w| w.0 = bits);
    }

    #[inline]
    fn endptsetupstat(self) -> u32 {
        self.0.ENDPTSETUPSTAT().read().0
    }

    #[inline]
    fn clear_endptsetupstat(self, bits: u32) {
        self.0.ENDPTSETUPSTAT().write(|w| w.0 = bits);
    }

    #[inline]
    fn endptprime(self) -> u32 {
        self.0.ENDPTPRIME().read().0
    }

    #[inline]
    fn set_endptprime(self, bits: u32) {
        self.0.ENDPTPRIME().write(|w| w.0 = bits);
    }

    #[inline]
    fn endptflush(self) -> u32 {
        self.0.ENDPTFLUSH().read().0
    }

    #[inline]
    fn set_endptflush(self, bits: u32) {
        self.0.ENDPTFLUSH().write(|w| w.0 = bits);
    }

    #[inline]
    fn endptcomplete(self) -> u32 {
        self.0.ENDPTCOMPLETE().read().0
    }

    #[inline]
    fn clear_endptcomplete(self, bits: u32) {
        self.0.ENDPTCOMPLETE().write(|w| w.0 = bits);
    }

    #[inline]
    fn endptctrl(self, index: usize) -> u32 {
        match index {
            0 => self.0.ENDPTCTRL0().read().0,
            1 => self.0.ENDPTCTRL1().read().0,
            2 => self.0.ENDPTCTRL2().read().0,
            3 => self.0.ENDPTCTRL3().read().0,
            4 => self.0.ENDPTCTRL4().read().0,
            5 => self.0.ENDPTCTRL5().read().0,
            6 => self.0.ENDPTCTRL6().read().0,
            7 => self.0.ENDPTCTRL7().read().0,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn set_endptctrl(self, index: usize, bits: u32) {
        match index {
            0 => self.0.ENDPTCTRL0().write(|w| w.0 = bits),
            1 => self.0.ENDPTCTRL1().write(|w| w.0 = bits),
            2 => self.0.ENDPTCTRL2().write(|w| w.0 = bits),
            3 => self.0.ENDPTCTRL3().write(|w| w.0 = bits),
            4 => self.0.ENDPTCTRL4().write(|w| w.0 = bits),
            5 => self.0.ENDPTCTRL5().write(|w| w.0 = bits),
            6 => self.0.ENDPTCTRL6().write(|w| w.0 = bits),
            7 => self.0.ENDPTCTRL7().write(|w| w.0 = bits),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn modify_endptctrl(self, index: usize, f: impl FnOnce(u32) -> u32) {
        self.set_endptctrl(index, f(self.endptctrl(index)));
    }
}

#[inline]
fn regs() -> UsbRegs {
    UsbRegs(crate::pac::USB1)
}

#[inline]
fn control_setup_pending() -> bool {
    regs().endptsetupstat() & 1 != 0
}

#[inline]
fn dma_clean(_ptr: *const u8, _len: usize) {
    // MCXA577 SRAM is currently non-cacheable. If this driver moves to a target
    // with D-cache, clean the range here before the controller reads it.
    asm::dsb();
}

#[inline]
fn dma_invalidate(_ptr: *const u8, _len: usize) {
    // MCXA577 SRAM is currently non-cacheable. If this driver moves to a target
    // with D-cache, invalidate the range here before the CPU reads it.
    asm::dsb();
}

/// Architectural address of the SCB Configuration and Control Register, and its
/// data-cache-enable bit (DC).
const SCB_CCR_ADDR: *const u32 = 0xE000_ED14 as *const u32;
const SCB_CCR_DC: u32 = 1 << 16;

/// Debug-assert that the data cache is disabled. The DMA structures rely on
/// non-cacheable SRAM for coherence with the controller (only `dsb()` barriers,
/// no clean/invalidate); on a future cached port this fires unless `dma_clean` /
/// `dma_invalidate` are taught real cache maintenance.
#[inline]
fn assert_dma_noncacheable() {
    // SAFETY: architectural read-only access to the System Control Block CCR.
    let dcache_enabled = unsafe { core::ptr::read_volatile(SCB_CCR_ADDR) } & SCB_CCR_DC != 0;
    debug_assert!(
        !dcache_enabled,
        "USB DMA assumes non-cacheable SRAM, but the data cache is enabled; implement cache \
         maintenance in dma_clean/dma_invalidate before enabling the D-cache"
    );
}

// =========================================================================
// Interrupt handler
// =========================================================================

type UsbInterrupt = crate::interrupt::typelevel::USB1_HS;

/// Interrupt handler for the USB controller.
///
/// Bind this with [`crate::bind_interrupts!`] to the `USB1_HS` interrupt.
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<UsbInterrupt> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = regs();
        let status = r.usbsts() & r.usbintr();
        if status == 0 {
            return;
        }

        // Acknowledge all handled sources (write-1-to-clear).
        r.clear_usbsts(status);

        if status & USBSTS_URI != 0 {
            RESET_COUNT.fetch_add(1, Ordering::Relaxed);
            SUSPENDED.store(false, Ordering::Relaxed);
            // A reset means the bus is live again: discard any pending suspend/resume
            // so a stale Suspend cannot be delivered after the Reset and leave the
            // stack suspended while the bus is active.
            FLAG_SUSPEND.store(false, Ordering::Relaxed);
            FLAG_RESUME.store(false, Ordering::Relaxed);
            FLAG_RESET.store(true, Ordering::Relaxed);
            abort_endpoint_transfers();
            BUS_WAKER.wake();
        }
        if status & USBSTS_SLI != 0 {
            SUSPEND_COUNT.fetch_add(1, Ordering::Relaxed);
            SUSPENDED.store(true, Ordering::Relaxed);
            FLAG_SUSPEND.store(true, Ordering::Relaxed);
            abort_endpoint_transfers();
            BUS_WAKER.wake();
        }
        if status & USBSTS_PCI != 0 {
            // Port-change also covers connect, enable, and speed changes. Only
            // report Resume when it is an edge out of a known suspended state.
            if SUSPENDED.swap(false, Ordering::Relaxed) {
                RESUME_COUNT.fetch_add(1, Ordering::Relaxed);
                FLAG_RESUME.store(true, Ordering::Relaxed);
                BUS_WAKER.wake();
            }
        }
        if status & USBSTS_UEI != 0 {
            ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
            // A transaction error retires only the offending dTD (its error bits are
            // set and ACTIVE is cleared); it does not invalidate other endpoints.
            // Wake all transfer waiters so the affected endpoint re-checks its
            // descriptor and fails itself, but do NOT bump the global abort epoch:
            // that would also unwind healthy in-flight transfers on other endpoints.
            wake_all_endpoint_wakers();
        }

        if status & USBSTS_UI != 0 {
            // Endpoint transfer(s) and/or setup packet(s) complete.
            // Wake setup/OUT/IN waiters; the futures re-check hardware state.
            let setup = r.endptsetupstat();
            let complete = r.endptcomplete();
            if complete != 0 {
                r.clear_endptcomplete(complete);
            }
            for ep in 0..EP_COUNT {
                let out = EndpointSlot::new(ep, Direction::Out);
                let in_ = EndpointSlot::new(ep, Direction::In);
                if setup & (1 << ep) != 0 {
                    out.waker().wake();
                    in_.waker().wake();
                }
                if complete & out.endpoint_bit() != 0 {
                    out.waker().wake();
                }
                if complete & in_.endpoint_bit() != 0 {
                    in_.waker().wake();
                }
            }
        }
    }
}

// =========================================================================
// Clock gate
// =========================================================================

impl Gate for crate::peripherals::USB1 {
    type MrccPeriphConfig = UsbhsConfig;

    #[inline]
    unsafe fn enable_clock() {
        crate::pac::MRCC0.mrcc_glb_cc2().modify(|w| {
            w.set_usb1(true);
            w.set_usb1_phy(true);
        });
    }

    #[inline]
    unsafe fn disable_clock() {
        crate::pac::MRCC0.mrcc_glb_cc2().modify(|w| {
            w.set_usb1(false);
            w.set_usb1_phy(false);
        });
    }

    #[inline]
    unsafe fn assert_reset() {
        crate::pac::MRCC0.mrcc_glb_rst2().modify(|w| {
            w.set_usb1(false);
            w.set_usb1_phy(false);
        });
        // Wait until BOTH reset bits read de-asserted. `is_reset_released()` is the
        // AND of the two bits, so reusing it here would stop as soon as either one
        // cleared rather than both.
        while {
            let rst2 = crate::pac::MRCC0.mrcc_glb_rst2().read();
            rst2.usb1() || rst2.usb1_phy()
        } {}
    }

    #[inline]
    unsafe fn release_reset() {
        crate::pac::MRCC0.mrcc_glb_rst2().modify(|w| {
            w.set_usb1(true);
            w.set_usb1_phy(true);
        });
        while !Self::is_reset_released() {}
    }

    #[inline]
    fn is_clock_enabled() -> bool {
        let cc2 = crate::pac::MRCC0.mrcc_glb_cc2().read();
        cc2.usb1() && cc2.usb1_phy()
    }

    #[inline]
    fn is_reset_released() -> bool {
        let rst2 = crate::pac::MRCC0.mrcc_glb_rst2().read();
        rst2.usb1() && rst2.usb1_phy()
    }
}

/// Marker for OUT endpoints.
pub enum Out {}
/// Marker for IN endpoints.
pub enum In {}

trait EndpointDir {
    const DIR: Direction;
}

impl EndpointDir for Out {
    const DIR: Direction = Direction::Out;
}

impl EndpointDir for In {
    const DIR: Direction = Direction::In;
}

#[derive(Clone, Copy)]
struct EndpointConfig {
    ep_type: EndpointType,
    max_packet_size: u16,
}

impl EndpointConfig {
    #[inline]
    fn store(self, slot: EndpointSlot) {
        let i = slot.array_index();
        EP_MAX_PACKET[i].store(self.max_packet_size, Ordering::Relaxed);
        EP_TYPE[i].store(self.ep_type as u8, Ordering::Relaxed);
    }

    #[inline]
    fn load(slot: EndpointSlot) -> Option<Self> {
        let i = slot.array_index();
        let ep_type = match EP_TYPE[i].load(Ordering::Relaxed) {
            0 => EndpointType::Control,
            1 => EndpointType::Isochronous,
            2 => EndpointType::Bulk,
            3 => EndpointType::Interrupt,
            _ => return None,
        };
        let max_packet_size = EP_MAX_PACKET[i].load(Ordering::Relaxed);
        (max_packet_size != 0).then_some(Self {
            ep_type,
            max_packet_size,
        })
    }
}

// =========================================================================
// Driver
// =========================================================================

/// USB device driver.
pub struct Driver<'d> {
    _phantom: PhantomData<&'d mut crate::peripherals::USB1>,
    _wake_guard: Option<WakeGuard>,
    alloc_out: u8,
    alloc_in: u8,
}

impl<'d> Driver<'d> {
    /// Create a new USB device driver, forced to full speed.
    ///
    /// This enables the USB clocks, brings up the PHY, and resets the controller
    /// into device mode. The controller starts detached; the `embassy-usb` stack
    /// attaches it via [`embassy_usb_driver::Bus::enable`].
    pub fn new(
        _usb: Peri<'d, crate::peripherals::USB1>,
        _irq: impl Binding<UsbInterrupt, InterruptHandler>,
        config: impl Into<Config>,
    ) -> Result<Self, InitError> {
        let config = config.into();
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "usb: init phy pll_div={=u8} d_cal={=u8} txcal45dp={=u8} txcal45dm={=u8}",
            config.phy.pll_div_sel,
            config.phy.d_cal,
            config.phy.txcal45dp,
            config.phy.txcal45dm
        );
        assert_dma_noncacheable();
        reset_static_state();

        // SAFETY: we own the USB peripheral and bring up its clocks/PHY once.
        let parts = unsafe { clocks::enable_and_reset::<crate::peripherals::USB1>(&config.clock)? };

        // SAFETY: MRCC has enabled and released the USBHS/USBPHY clocks/resets.
        unsafe {
            clock::init_phy(&config.phy).map_err(|err| match err {
                clock::PhyInitError::InvalidConfig => InitError::PhyConfig,
                clock::PhyInitError::PllLock => InitError::PhyPllLock,
            })?;
        }
        reset_controller();

        // EP0 (control) is implicitly allocated.
        Ok(Self {
            _phantom: PhantomData,
            _wake_guard: parts.wake_guard,
            alloc_out: 1 << 0,
            alloc_in: 1 << 0,
        })
    }

    fn alloc_endpoint<D: EndpointDir>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Endpoint<D>, EndpointAllocError> {
        if ep_type == EndpointType::Control
            || ep_type == EndpointType::Isochronous
            || max_packet_size == 0
            || max_packet_size > FS_MAX_PACKET
        {
            return Err(EndpointAllocError);
        }

        let is_in = D::DIR == Direction::In;
        let alloc = if is_in { &mut self.alloc_in } else { &mut self.alloc_out };

        let index = match ep_addr {
            Some(addr) => {
                let i = addr.index();
                if addr.is_in() != is_in || i == 0 || i >= EP_COUNT || (*alloc & (1 << i)) != 0 {
                    return Err(EndpointAllocError);
                }
                i
            }
            None => {
                // Endpoint 0 is reserved for control transfers.
                let mut found = None;
                for i in 1..EP_COUNT {
                    if *alloc & (1 << i) == 0 {
                        found = Some(i);
                        break;
                    }
                }
                found.ok_or(EndpointAllocError)?
            }
        };

        *alloc |= 1 << index;

        let addr = EndpointAddress::from_parts(index, D::DIR);
        let slot = EndpointSlot::from_addr(addr);
        EndpointConfig {
            ep_type,
            max_packet_size,
        }
        .store(slot);

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "usb: alloc ep{=usize} dir={=u8} type={=u8} mps={=u16} interval={=u8}",
            index,
            slot.dir_num(),
            endpoint_type_bits(ep_type) as u8,
            max_packet_size,
            interval_ms
        );

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr,
                ep_type,
                max_packet_size,
                interval_ms,
            },
        })
    }
}

impl<'d> embassy_usb_driver::Driver<'d> for Driver<'d> {
    type EndpointOut = Endpoint<Out>;
    type EndpointIn = Endpoint<In>;
    type ControlPipe = ControlPipe;
    type Bus = Bus;

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.alloc_endpoint::<Out>(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint::<In>(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        // Configure the control endpoint queue heads.
        init_control_qh(control_max_packet_size);

        let bus = Bus {
            power_detected: false,
            control_max_packet_size,
            _wake_guard: self._wake_guard,
        };
        let control = ControlPipe {
            max_packet_size: control_max_packet_size,
        };
        (bus, control)
    }
}

// =========================================================================
// Controller bring-up helpers
// =========================================================================

/// Reset the controller and place it in device mode (detached, full speed).
fn reset_controller() {
    let r = regs();

    // Stop the controller, then issue a controller reset and wait for it.
    r.modify_usbcmd(|v| v & !USBCMD_RS);
    r.modify_usbcmd(|v| v | USBCMD_RST);
    while r.usbcmd() & USBCMD_RST != 0 {}

    // Device mode, setup-lockout off (we use the setup tripwire instead).
    r.set_usbmode(USBMODE_CM_DEVICE | USBMODE_SLOM);

    // Force full speed.
    r.modify_portsc1(|v| v | PORTSC1_PFSC);

    // Program the endpoint list base.
    let qh_addr = QH_LIST.as_ptr() as u32;
    r.set_endptlistaddr(qh_addr);

    #[cfg(feature = "defmt")]
    defmt::trace!("usb: controller reset id=0x{=u32:08x} qh=0x{=u32:08x}", r.id(), qh_addr);

    // Reasonable default burst size.
    r.set_burstsize(DEFAULT_BURST_SIZE);

    // Clear any pending status, leave interrupts disabled until `enable`.
    r.clear_usbsts(ALL_USBSTS);
    r.set_usbintr(0);
}

/// Initialize the control-endpoint (EP0) queue heads. Called once during `start`.
fn init_control_qh(mps: u16) {
    let out = EndpointSlot::new(0, Direction::Out);
    let in_ = EndpointSlot::new(0, Direction::In);
    EndpointConfig {
        ep_type: EndpointType::Control,
        max_packet_size: mps,
    }
    .store(out);
    EndpointConfig {
        ep_type: EndpointType::Control,
        max_packet_size: mps,
    }
    .store(in_);

    // SAFETY: exclusive access during start-up; QH list is owned by the driver.
    unsafe {
        let qh = qh_ptr();
        // Disable automatic ZLP termination. `embassy-usb` requests status/ZLP
        // packets explicitly via the control pipe and endpoint transfers.
        // OUT QH[0]: interrupt-on-setup so SETUP packets notify us.
        (*qh)[0].capabilities = ((mps as u32) << QH_CAP_MAXLEN_SHIFT) | QH_CAP_IOS | QH_CAP_ZLT;
        (*qh)[0].next_dtd = DTD_NEXT_TERMINATE;
        // IN QH[1].
        (*qh)[1].capabilities = ((mps as u32) << QH_CAP_MAXLEN_SHIFT) | QH_CAP_ZLT;
        (*qh)[1].next_dtd = DTD_NEXT_TERMINATE;
    }

    let r = regs();
    r.modify_endptctrl(0, |v| {
        (v & !(EPCTRL_TXS | EPCTRL_RXS)) | EPCTRL_RXE | EPCTRL_TXE | EPCTRL_RXR | EPCTRL_TXR
    });
}

// =========================================================================
// Endpoint
// =========================================================================

/// A USB endpoint.
pub struct Endpoint<D> {
    _phantom: PhantomData<D>,
    info: EndpointInfo,
}

/// Configure an endpoint's queue head and control register.
fn configure_endpoint(addr: EndpointAddress, ep_type: EndpointType, mps: u16) {
    let index = addr.index();
    let slot = EndpointSlot::from_addr(addr);
    let qhi = slot.array_index();

    // SAFETY: queue-head list is owned by the driver; exclusive access here.
    unsafe {
        let qh = qh_ptr();
        // Disable automatic ZLP termination. The USB stack explicitly requests
        // short/ZLP packets when the class transfer needs one.
        (*qh)[qhi].capabilities = ((mps as u32) << QH_CAP_MAXLEN_SHIFT) | QH_CAP_ZLT;
        (*qh)[qhi].next_dtd = DTD_NEXT_TERMINATE;
        (*qh)[qhi].token = 0;
    }

    let r = regs();
    let (type_mask, enable_bits) = endpoint_ctrl_enable_bits(slot, ep_type);
    #[cfg(feature = "defmt")]
    defmt::trace!(
        "usb: configure ep{=usize} dir={=u8} type={=u8} mps={=u16}",
        index,
        slot.dir_num(),
        endpoint_type_bits(ep_type) as u8,
        mps
    );
    r.modify_endptctrl(index, |v| {
        // Reset the data toggle and enable the requested direction.
        (v & !type_mask) | enable_bits
    });
}

impl<D> embassy_usb_driver::Endpoint for Endpoint<D> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        let slot = EndpointSlot::from_addr(self.info.addr);
        poll_fn(|cx| {
            slot.waker().register(cx.waker());
            let r = regs();
            if slot.is_enabled(r) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl embassy_usb_driver::EndpointOut for Endpoint<Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        ep_read(index, buf).await
    }
}

impl embassy_usb_driver::EndpointIn for Endpoint<In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        let index = self.info.addr.index();
        ep_write(index, buf, false).await
    }
}

// =========================================================================
// Low-level transfer primitives (dTD prime + completion wait)
// =========================================================================

fn wait_register_clear(mut read: impl FnMut() -> u32, mask: u32) -> bool {
    for _ in 0..ENDPOINT_WAIT_ITERS {
        if read() & mask == 0 {
            return true;
        }
        asm::nop();
    }
    false
}

fn clear_transfer_overlay(slot: EndpointSlot) {
    let i = slot.array_index();
    // SAFETY: called after the endpoint direction is flushed or inactive.
    unsafe {
        let dtd = dtd_ptr();
        (*dtd)[i] = TransferDescriptor::new();

        let qh = qh_ptr();
        (*qh)[i].next_dtd = DTD_NEXT_TERMINATE;
        (*qh)[i].token = 0;
    }
}

fn flush_endpoint_bit(bit: u32) -> Result<(), EndpointError> {
    if bit == 0 {
        return Ok(());
    }

    let r = regs();
    r.set_endptflush(bit);
    if wait_register_clear(|| r.endptflush(), bit) {
        Ok(())
    } else {
        Err(EndpointError::Disabled)
    }
}

fn flush_endpoint(slot: EndpointSlot) -> Result<(), EndpointError> {
    let bit = slot.endpoint_bit();
    let res = flush_endpoint_bit(bit);
    if res.is_ok() {
        clear_transfer_overlay(slot);
    }
    res
}

struct TransferGuard {
    slot: EndpointSlot,
    armed: bool,
}

impl TransferGuard {
    fn new(slot: EndpointSlot) -> Self {
        Self { slot, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TransferGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = flush_endpoint(self.slot);
        }
    }
}

/// Prime a transfer descriptor on the given endpoint/direction and wait for it
/// to complete, returning the number of bytes transferred.
async fn ep_transfer(
    slot: EndpointSlot,
    buf_ptr: *mut u8,
    len: usize,
    abort_on_setup: bool,
) -> Result<usize, EndpointError> {
    let dtd_i = slot.array_index();
    let qhi = dtd_i;
    let abort_epoch = TRANSFER_ABORT_EPOCH.load(Ordering::Relaxed);

    // SAFETY: the descriptor/queue-head for this endpoint direction is owned by
    // the caller for the duration of the transfer.
    unsafe {
        // Build the transfer descriptor.
        let dtd = dtd_ptr();
        (*dtd)[dtd_i].next = DTD_NEXT_TERMINATE;
        (*dtd)[dtd_i].token = ((len as u32) << DTD_TOKEN_TOTAL_SHIFT) | DTD_TOKEN_IOC | DTD_TOKEN_ACTIVE;
        let base = buf_ptr as u32;
        (*dtd)[dtd_i].buffer[0] = base;
        for page in 1..5 {
            (*dtd)[dtd_i].buffer[page] = (base & !DTD_BUFFER_PAGE_MASK) + (page as u32) * DTD_BUFFER_PAGE_SIZE;
        }

        // Link it into the queue head overlay and clear status.
        let qh = qh_ptr();
        (*qh)[qhi].next_dtd = core::ptr::addr_of!((*dtd)[dtd_i]) as u32;
        (*qh)[qhi].token = 0;
    }

    let r = regs();
    let prime_bit = slot.endpoint_bit();

    dma_clean(buf_ptr, len);
    // SAFETY: raw pointers to the descriptor and queue head just programmed.
    unsafe {
        let dtd = dtd_ptr();
        let qh = qh_ptr();
        dma_clean(
            core::ptr::addr_of!((*dtd)[dtd_i]).cast(),
            core::mem::size_of::<TransferDescriptor>(),
        );
        dma_clean(
            core::ptr::addr_of!((*qh)[qhi]).cast(),
            core::mem::size_of::<QueueHead>(),
        );
    }

    // Prime the endpoint. We allow only one outstanding dTD per endpoint
    // direction, so there is no need to use the add-dTD tripwire (ATDTW).
    r.set_endptprime(prime_bit);
    // Wait until the controller has acknowledged the prime.
    if !wait_register_clear(|| r.endptprime(), prime_bit) {
        let _ = flush_endpoint(slot);
        return Err(EndpointError::Disabled);
    }

    // Wait for completion via the waker, re-checking the descriptor status.
    let mut guard = TransferGuard::new(slot);
    let res = poll_fn(|cx| {
        slot.waker().register(cx.waker());
        if TRANSFER_ABORT_EPOCH.load(Ordering::Relaxed) != abort_epoch {
            let _ = flush_endpoint(slot);
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        if !slot.is_enabled(r) {
            let _ = flush_endpoint(slot);
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        if abort_on_setup && control_setup_pending() {
            let _ = flush_endpoint(slot);
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        dma_invalidate(buf_ptr, len);
        // SAFETY: raw pointer to the hardware-updated transfer descriptor.
        unsafe {
            let dtd = dtd_ptr();
            dma_invalidate(
                core::ptr::addr_of!((*dtd)[dtd_i]).cast(),
                core::mem::size_of::<TransferDescriptor>(),
            );
        }
        // SAFETY: reading the (volatile) hardware-updated descriptor token.
        let token = unsafe {
            let dtd = dtd_ptr();
            core::ptr::read_volatile(core::ptr::addr_of!((*dtd)[dtd_i].token))
        };
        if token & DTD_TOKEN_ACTIVE != 0 {
            return Poll::Pending;
        }
        if token & DTD_TOKEN_ERROR_MASK != 0 {
            #[cfg(feature = "defmt")]
            defmt::warn!(
                "usb: transfer error ep{=usize} dir={=u8} token=0x{=u32:08x}",
                slot.index,
                slot.dir_num(),
                token
            );
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        // Remaining bytes are in token[30:16]; transferred = requested - remaining.
        let remaining = (token >> DTD_TOKEN_TOTAL_SHIFT) & DTD_TOKEN_TOTAL_MASK;
        Poll::Ready(Ok(len.saturating_sub(remaining as usize)))
    })
    .await;

    guard.disarm();
    res
}

/// Static bounce buffers for endpoint transfers (one per endpoint direction),
/// ensuring DMA-visible, suitably-aligned storage.
#[repr(C, align(64))]
struct EpBuffers {
    buf: [[u8; FS_MAX_PACKET as usize]; EP_COUNT * 2],
}
static EP_BUFFERS: DmaCell<EpBuffers> = DmaCell::new(EpBuffers {
    buf: [[0; FS_MAX_PACKET as usize]; EP_COUNT * 2],
});

async fn ep_read(index: usize, buf: &mut [u8]) -> Result<usize, EndpointError> {
    let slot = EndpointSlot::new(index, Direction::Out);
    let bounce = ep_buffer_ptr(slot);
    let cap = endpoint_max_packet(slot).unwrap_or(FS_MAX_PACKET) as usize;
    let n = ep_transfer(slot, bounce, cap, false).await?;
    if n > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }
    // SAFETY: `bounce` is valid for `n` bytes and `buf` for its length.
    unsafe { core::ptr::copy_nonoverlapping(bounce, buf.as_mut_ptr(), n) };
    Ok(n)
}

async fn ep_write(index: usize, buf: &[u8], abort_on_setup: bool) -> Result<(), EndpointError> {
    let slot = EndpointSlot::new(index, Direction::In);
    let cap = endpoint_max_packet(slot).unwrap_or(FS_MAX_PACKET) as usize;
    if buf.len() > cap {
        return Err(EndpointError::BufferOverflow);
    }
    let bounce = ep_buffer_ptr(slot);
    // SAFETY: copying caller data into the owned bounce buffer.
    unsafe { core::ptr::copy_nonoverlapping(buf.as_ptr(), bounce, buf.len()) };
    ep_transfer(slot, bounce, buf.len(), abort_on_setup).await?;
    Ok(())
}

async fn ep_read_control(buf: &mut [u8]) -> Result<usize, EndpointError> {
    let slot = EndpointSlot::new(0, Direction::Out);
    let bounce = ep_buffer_ptr(slot);
    let cap = endpoint_max_packet(slot).unwrap_or(FS_MAX_PACKET) as usize;
    let n = ep_transfer(slot, bounce, cap, true).await?;
    if n > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }
    // SAFETY: `bounce` is valid for `n` bytes and `buf` for its length.
    unsafe { core::ptr::copy_nonoverlapping(bounce, buf.as_mut_ptr(), n) };
    Ok(n)
}

async fn ep_zlp(index: usize, dir: Direction, abort_on_setup: bool) -> Result<(), EndpointError> {
    let slot = EndpointSlot::new(index, dir);
    // EHCI should not dereference buffer pointers for zero-length transfers, but
    // giving it a real aligned address avoids relying on null-pointer behavior.
    let bounce = ep_buffer_ptr(slot);
    ep_transfer(slot, bounce, 0, abort_on_setup).await?;
    Ok(())
}

// =========================================================================
// Control pipe
// =========================================================================

/// Control endpoint (EP0) pipe.
pub struct ControlPipe {
    max_packet_size: u16,
}

impl embassy_usb_driver::ControlPipe for ControlPipe {
    fn max_packet_size(&self) -> usize {
        self.max_packet_size as usize
    }

    async fn setup(&mut self) -> [u8; 8] {
        let r = regs();
        poll_fn(|cx| {
            EndpointSlot::new(0, Direction::Out).waker().register(cx.waker());
            let stat = r.endptsetupstat();
            if stat & 1 == 0 {
                return Poll::Pending;
            }

            // Acknowledge before the SUTW copy loop. If another SETUP arrives
            // during the copy, hardware re-sets ENDPTSETUPSTAT and preserves it
            // for the next `setup()` call.
            r.clear_endptsetupstat(stat);

            // Read the setup packet using the setup tripwire so a back-to-back
            // SETUP cannot corrupt the read. The retry loop is bounded so a wedged
            // controller cannot spin the single-threaded executor forever; on
            // exhaustion we proceed with the last read (a possibly torn packet the
            // host re-issues on failure) and warn, rather than hang.
            let mut iters = 0usize;
            let setup = loop {
                r.modify_usbcmd(|v| v | USBCMD_SUTW);
                // SAFETY: control OUT queue head holds the latest setup bytes.
                let bytes = unsafe {
                    let qh = qh_ptr();
                    let w0 = core::ptr::read_volatile(core::ptr::addr_of!((*qh)[0].setup[0]));
                    let w1 = core::ptr::read_volatile(core::ptr::addr_of!((*qh)[0].setup[1]));
                    let mut b = [0u8; 8];
                    b[0..4].copy_from_slice(&w0.to_le_bytes());
                    b[4..8].copy_from_slice(&w1.to_le_bytes());
                    b
                };
                if r.usbcmd() & USBCMD_SUTW != 0 {
                    break bytes;
                }
                iters += 1;
                if iters >= ENDPOINT_WAIT_ITERS {
                    #[cfg(feature = "defmt")]
                    defmt::warn!("usb: SUTW tripwire did not settle; using last setup read");
                    break bytes;
                }
            };
            r.modify_usbcmd(|v| v & !USBCMD_SUTW);
            r.modify_endptctrl(0, |v| v & !(EPCTRL_TXS | EPCTRL_RXS));

            Poll::Ready(setup)
        })
        .await
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        // EP0 OUT owned by the control pipe.
        // `embassy-usb-driver` chunks multi-packet control OUT transfers by
        // `max_packet_size()`, so the EP0 bounce buffer only has to hold one
        // full-speed control packet.
        ep_read_control(buf).await
    }

    async fn data_in(&mut self, data: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        // EP0 IN owned by the control pipe.
        ep_write(0, data, true).await?;
        if last {
            // Status stage: receive the host's zero-length OUT.
            ep_zlp(0, Direction::Out, true).await?;
        }
        Ok(())
    }

    async fn accept(&mut self) {
        if control_setup_pending() {
            return;
        }
        // Status stage: send a zero-length IN packet.
        // EP0 IN owned by control pipe.
        let _ = ep_zlp(0, Direction::In, true).await;
    }

    async fn reject(&mut self) {
        if control_setup_pending() {
            return;
        }
        // Stall EP0 in both directions to reject the request.
        let r = regs();
        let _ = flush_endpoint(EndpointSlot::new(0, Direction::Out));
        let _ = flush_endpoint(EndpointSlot::new(0, Direction::In));
        r.modify_endptctrl(0, |v| v | EPCTRL_TXS | EPCTRL_RXS);
    }

    async fn accept_set_address(&mut self, addr: u8) {
        if control_setup_pending() {
            return;
        }
        // EHCI: program the address with the "advance" bit so it is applied
        // only after the status stage completes, then send the status IN.
        let r = regs();
        r.set_deviceaddr(((addr as u32) << DEVICEADDR_USBADR_SHIFT) | DEVICEADDR_USBADRA);
        // EP0 IN for status.
        let _ = ep_zlp(0, Direction::In, true).await;
    }
}

// =========================================================================
// Bus
// =========================================================================

/// USB bus control.
pub struct Bus {
    power_detected: bool,
    control_max_packet_size: u16,
    _wake_guard: Option<WakeGuard>,
}

impl Drop for Bus {
    /// Stops the controller and masks its interrupt only.
    ///
    /// Clock and PHY teardown is intentionally NOT performed here. Like the nrf and
    /// stm32 USB drivers, the MRCC gates and the PHY PLL are left to the peripheral
    /// lifetime rather than `Bus::drop`: gating from here would reach back into
    /// `MRCC0` and risk touching controller registers after their clock was removed,
    /// because `embassy-usb` does not guarantee the endpoints are dropped before the
    /// `Bus`. The clocks stay enabled for the borrowed `USB1` lifetime; a subsequent
    /// `Driver::new` re-runs `enable_and_reset`, so re-initialization is clean.
    fn drop(&mut self) {
        let r = regs();
        r.modify_usbcmd(|v| v & !USBCMD_RS);
        r.set_usbintr(0);
        r.clear_usbsts(ALL_USBSTS);
        UsbInterrupt::disable();
    }
}

impl embassy_usb_driver::Bus for Bus {
    async fn enable(&mut self) {
        let r = regs();
        // Enable the interrupt sources we handle.
        r.set_usbintr(USBSTS_UI | USBSTS_UEI | USBSTS_PCI | USBSTS_URI | USBSTS_SLI);
        #[cfg(feature = "defmt")]
        defmt::trace!("usb: bus enable");

        // SAFETY: enabling the controller interrupt.
        unsafe {
            UsbInterrupt::unpend();
            UsbInterrupt::enable();
        }

        // Attach: set Run/Stop.
        r.modify_usbcmd(|v| v | USBCMD_RS);
    }

    async fn disable(&mut self) {
        let r = regs();
        #[cfg(feature = "defmt")]
        defmt::trace!("usb: bus disable");
        r.modify_usbcmd(|v| v & !USBCMD_RS);
        r.set_usbintr(0);
        UsbInterrupt::disable();
    }

    async fn poll(&mut self) -> Event {
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());

            if !self.power_detected {
                // Surface an initial power-detected event so the stack proceeds.
                self.power_detected = true;
                return Poll::Ready(Event::PowerDetected);
            }

            // Bus events are drained in fixed priority: reset, then suspend, then
            // resume. Resume is only ever flagged on an edge out of an observed
            // suspended state (see the interrupt handler), so a suspend->resume pair
            // is emitted in order. Reset clears any pending suspend/resume below, so
            // the bus is never left suspended after a reset.
            if FLAG_RESET.swap(false, Ordering::Relaxed) {
                #[cfg(feature = "defmt")]
                defmt::trace!("usb: bus reset");
                // A reset supersedes any suspend/resume that raced ahead of this poll.
                FLAG_SUSPEND.store(false, Ordering::Relaxed);
                FLAG_RESUME.store(false, Ordering::Relaxed);
                // Re-initialize endpoint 0 and clear setup/complete state.
                let r = regs();
                let setup = r.endptsetupstat();
                r.clear_endptsetupstat(setup);
                let complete = r.endptcomplete();
                r.clear_endptcomplete(complete);
                if !wait_register_clear(|| r.endptprime(), ALL_ENDPOINT_BITS) {
                    #[cfg(feature = "defmt")]
                    defmt::warn!("usb: reset timed out waiting for ENDPTPRIME");
                }
                let _ = flush_endpoint_bit(ALL_ENDPOINT_BITS);
                r.set_deviceaddr(0);
                init_control_qh(self.control_max_packet_size);
                return Poll::Ready(Event::Reset);
            }
            if FLAG_SUSPEND.swap(false, Ordering::Relaxed) {
                #[cfg(feature = "defmt")]
                defmt::trace!("usb: suspend");
                return Poll::Ready(Event::Suspend);
            }
            if FLAG_RESUME.swap(false, Ordering::Relaxed) {
                #[cfg(feature = "defmt")]
                defmt::trace!("usb: resume");
                return Poll::Ready(Event::Resume);
            }
            Poll::Pending
        })
        .await
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        let slot = EndpointSlot::from_addr(ep_addr);
        if enabled {
            if let Some(config) = EndpointConfig::load(slot) {
                configure_endpoint(ep_addr, config.ep_type, config.max_packet_size);
            }
        } else {
            let r = regs();
            #[cfg(feature = "defmt")]
            defmt::trace!("usb: disable ep{=usize} dir={=u8}", slot.index, slot.dir_num());
            r.modify_endptctrl(slot.index, |v| v & !endpoint_enable_bit(slot));
        }
        slot.waker().wake();
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        let slot = EndpointSlot::from_addr(ep_addr);
        let r = regs();
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "usb: stall ep{=usize} dir={=u8} stalled={=u8}",
            slot.index,
            slot.dir_num(),
            stalled as u8
        );
        r.modify_endptctrl(slot.index, |v| {
            let bit = endpoint_stall_bit(slot);
            if stalled {
                v | bit
            } else {
                (v & !bit) | endpoint_reset_bit(slot)
            }
        });
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let slot = EndpointSlot::from_addr(ep_addr);
        regs().endptctrl(slot.index) & endpoint_stall_bit(slot) != 0
    }
}
