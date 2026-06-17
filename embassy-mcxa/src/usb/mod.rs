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
//! # Limitations
//! - Device mode only (no host / OTG role switching).
//! - Forced full speed; high-speed operation is not exposed.
//! - One transfer descriptor in flight per endpoint direction at a time.
//!
//! # Register access
//! Until `nxp-pac` gains a register block for this peripheral on MCXA577, the
//! controller registers and DMA structures are defined locally in
//! [`registers`]. The driver logic does not otherwise depend on that choice.

mod clock;
mod registers;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, Ordering};
use core::task::Poll;

use cortex_m::asm;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

pub use clock::PhyConfig;

use crate::interrupt;
use crate::interrupt::typelevel::{Binding, Interrupt};

use registers::*;

/// Number of bidirectional endpoint pairs supported (EP0..EP{N-1}).
///
/// The MCXA577 USBHS controller exposes eight bidirectional endpoint pairs.
const EP_COUNT: usize = 8;

/// Maximum packet size for a full-speed endpoint.
const FS_MAX_PACKET: u16 = 64;

// ---- DMA-visible controller structures (statically allocated) ----
//
// MCXA577 SRAM is treated as non-cacheable by the current MCX-A target setup.
// If this driver is reused on a target with data-cacheable SRAM, these
// descriptors and bounce buffers must either be placed in a non-cacheable
// section or wrapped with explicit cache clean/invalidate operations.

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

static mut QH_LIST: QhList = QhList {
    qh: [QueueHead::new(); EP_COUNT * 2],
};
static mut DTD_LIST: DtdList = DtdList {
    dtd: [TransferDescriptor::new(); EP_COUNT * 2],
};

/// Per-endpoint-direction transfer-complete wakers (index = `2*ep + dir`).
static EP_WAKERS: [AtomicWaker; EP_COUNT * 2] = [const { AtomicWaker::new() }; EP_COUNT * 2];
/// Waker for bus events (reset/suspend/resume) handled in [`Bus::poll`].
static BUS_WAKER: AtomicWaker = AtomicWaker::new();

/// Bus event flags set by the interrupt handler and consumed by [`Bus::poll`].
static FLAG_RESET: AtomicBool = AtomicBool::new(false);
static FLAG_SUSPEND: AtomicBool = AtomicBool::new(false);
static FLAG_RESUME: AtomicBool = AtomicBool::new(false);
static FLAG_SETUP: AtomicBool = AtomicBool::new(false);

/// Per-endpoint-direction configuration captured at allocation time and applied
/// by [`Bus::endpoint_set_enabled`] (index = `2*ep + dir`).
static EP_MAX_PACKET: [AtomicU16; EP_COUNT * 2] = [const { AtomicU16::new(0) }; EP_COUNT * 2];
static EP_TYPE: [AtomicU8; EP_COUNT * 2] = [const { AtomicU8::new(0) }; EP_COUNT * 2];

#[inline]
fn waker_index(addr: EndpointAddress) -> usize {
    let dir = if addr.is_in() { 1 } else { 0 };
    addr.index() * 2 + dir
}

#[inline]
fn qh_index(index: usize, dir: Direction) -> usize {
    let d = if dir == Direction::In { 1 } else { 0 };
    index * 2 + d
}

#[inline]
fn endpoint_bit(index: usize, dir: Direction) -> u32 {
    match dir {
        Direction::Out => 1 << index,
        Direction::In => 1 << (index + 16),
    }
}

/// Controller handle.
#[inline]
fn regs() -> UsbHs {
    // SAFETY: single controller instance, address fixed for this chip.
    unsafe { UsbHs::new(USBHS1_BASE) }
}

#[inline]
fn control_setup_pending() -> bool {
    FLAG_SETUP.load(Ordering::Relaxed) || regs().endptsetupstat() & 1 != 0
}

// =========================================================================
// Interrupt handler
// =========================================================================

/// Interrupt handler for the USB controller.
///
/// Bind this with [`crate::bind_interrupts!`] to the `USB1_HS` interrupt.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = regs();
        let status = r.usbsts() & r.usbintr();
        // Acknowledge all handled sources (write-1-to-clear).
        r.clear_usbsts(status);

        if status & USBSTS_URI != 0 {
            FLAG_RESET.store(true, Ordering::Relaxed);
            BUS_WAKER.wake();
        }
        if status & USBSTS_SLI != 0 {
            FLAG_SUSPEND.store(true, Ordering::Relaxed);
            BUS_WAKER.wake();
        }
        if status & USBSTS_PCI != 0 {
            // Port change: treat as resume notification.
            FLAG_RESUME.store(true, Ordering::Relaxed);
            BUS_WAKER.wake();
        }

        if status & USBSTS_UI != 0 {
            // Endpoint transfer(s) and/or setup packet(s) complete.
            // Wake setup/OUT/IN waiters; the futures re-check hardware state.
            let setup = r.endptsetupstat();
            let complete = r.endptcomplete();
            if setup != 0 {
                FLAG_SETUP.store(true, Ordering::Relaxed);
            }
            if complete != 0 {
                r.clear_endptcomplete(complete);
            }
            for ep in 0..EP_COUNT {
                // OUT (RX) bit `ep`, IN (TX) bit `ep+16`.
                if setup & (1 << ep) != 0 || complete & (1 << ep) != 0 {
                    EP_WAKERS[ep * 2].wake();
                }
                if complete & (1 << (ep + 16)) != 0 {
                    EP_WAKERS[ep * 2 + 1].wake();
                }
            }
        }

        // Mask further interrupts are not needed: sources are level and cleared above.
        T::Interrupt::unpend();
    }
}

// =========================================================================
// Instance trait
// =========================================================================

trait SealedInstance {}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this USB instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl SealedInstance for crate::peripherals::USB1 {}
impl Instance for crate::peripherals::USB1 {
    type Interrupt = crate::interrupt::typelevel::USB1_HS;
}

// =========================================================================
// Driver
// =========================================================================

/// USB device driver.
pub struct Driver<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    alloc_out: u8,
    alloc_in: u8,
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Create a new USB device driver, forced to full speed.
    ///
    /// This enables the USB clocks, brings up the PHY, and resets the controller
    /// into device mode. The controller starts detached; the `embassy-usb` stack
    /// attaches it via [`embassy_usb_driver::Bus::enable`].
    pub fn new(
        _usb: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        phy_config: PhyConfig,
    ) -> Self {
        // SAFETY: we own the USB peripheral and bring up its clocks/PHY once.
        unsafe {
            clock::init_clocks_and_phy(&phy_config);
        }
        reset_controller();

        // EP0 (control) is implicitly allocated.
        Self {
            _phantom: PhantomData,
            alloc_out: 1 << 0,
            alloc_in: 1 << 0,
        }
    }

    fn alloc_endpoint<D>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
        is_in: bool,
    ) -> Result<Endpoint<D>, EndpointAllocError> {
        let alloc = if is_in { &mut self.alloc_in } else { &mut self.alloc_out };

        let index = match ep_addr {
            Some(addr) => {
                let i = addr.index();
                if i >= EP_COUNT || (*alloc & (1 << i)) != 0 {
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

        let dir = if is_in { Direction::In } else { Direction::Out };
        let addr = EndpointAddress::from_parts(index, dir);
        let mps = max_packet_size.min(FS_MAX_PACKET);

        // Record the configuration so the bus can program the queue head and
        // endpoint control register when the endpoint is enabled.
        let wi = waker_index(addr);
        EP_MAX_PACKET[wi].store(mps, Ordering::Relaxed);
        EP_TYPE[wi].store(ep_type as u8, Ordering::Relaxed);

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr,
                ep_type,
                max_packet_size: mps,
                interval_ms,
            },
        })
    }
}

impl<'d, T: Instance> embassy_usb_driver::Driver<'d> for Driver<'d, T> {
    type EndpointOut = Endpoint<Out>;
    type EndpointIn = Endpoint<In>;
    type ControlPipe = ControlPipe;
    type Bus = Bus<T>;

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms, false)
    }

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms, true)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        // Configure the control endpoint queue heads.
        init_control_qh(control_max_packet_size);

        let bus = Bus {
            _phantom: PhantomData,
            inited: false,
            control_max_packet_size,
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
    let qh_addr = core::ptr::addr_of!(QH_LIST) as u32;
    r.set_endptlistaddr(qh_addr);

    // Reasonable default burst size.
    r.set_burstsize((0x10 << 8) | 0x10);

    // Clear any pending status, leave interrupts disabled until `enable`.
    r.clear_usbsts(0xFFFF_FFFF);
    r.set_usbintr(0);
}

/// Initialize the control-endpoint (EP0) queue heads. Called once during `start`.
fn init_control_qh(mps: u16) {
    // SAFETY: exclusive access during start-up; QH list is owned by the driver.
    unsafe {
        let qh = core::ptr::addr_of_mut!(QH_LIST.qh);
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

/// Marker for OUT endpoints.
pub enum Out {}
/// Marker for IN endpoints.
pub enum In {}

/// A USB endpoint.
pub struct Endpoint<D> {
    _phantom: PhantomData<D>,
    info: EndpointInfo,
}

/// Configure an endpoint's queue head and control register.
fn configure_endpoint(addr: EndpointAddress, ep_type: EndpointType, mps: u16) {
    let index = addr.index();
    let is_in = addr.is_in();
    let qhi = qh_index(index, if is_in { Direction::In } else { Direction::Out });

    // SAFETY: queue-head list is owned by the driver; exclusive access here.
    unsafe {
        let qh = core::ptr::addr_of_mut!(QH_LIST.qh);
        (*qh)[qhi].capabilities = ((mps as u32) << QH_CAP_MAXLEN_SHIFT) | QH_CAP_ZLT;
        (*qh)[qhi].next_dtd = DTD_NEXT_TERMINATE;
        (*qh)[qhi].token = 0;
    }

    let r = regs();
    let ty = ep_type as u32;
    r.modify_endptctrl(index, |v| {
        if is_in {
            // Reset the data toggle and enable the TX side.
            let cleared = v & !(0b11 << EPCTRL_TXT_SHIFT);
            cleared | (ty << EPCTRL_TXT_SHIFT) | EPCTRL_TXE | EPCTRL_TXR
        } else {
            let cleared = v & !(0b11 << EPCTRL_RXT_SHIFT);
            cleared | (ty << EPCTRL_RXT_SHIFT) | EPCTRL_RXE | EPCTRL_RXR
        }
    });
}

impl<D> embassy_usb_driver::Endpoint for Endpoint<D> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        let index = self.info.addr.index();
        let is_in = self.info.addr.is_in();
        poll_fn(|cx| {
            EP_WAKERS[waker_index(self.info.addr)].register(cx.waker());
            let r = regs();
            let en = if is_in {
                r.endptctrl(index) & EPCTRL_TXE != 0
            } else {
                r.endptctrl(index) & EPCTRL_RXE != 0
            };
            if en { Poll::Ready(()) } else { Poll::Pending }
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

/// Prime a transfer descriptor on the given endpoint/direction and wait for it
/// to complete, returning the number of bytes transferred.
async fn ep_transfer(
    index: usize,
    dir: Direction,
    buf_ptr: *mut u8,
    len: usize,
    abort_on_setup: bool,
) -> Result<usize, EndpointError> {
    let dtd_i = qh_index(index, dir);
    let qhi = dtd_i;

    // SAFETY: the descriptor/queue-head for this endpoint direction is owned by
    // the caller for the duration of the transfer.
    unsafe {
        // Build the transfer descriptor.
        let dtd = core::ptr::addr_of_mut!(DTD_LIST.dtd);
        (*dtd)[dtd_i].next = DTD_NEXT_TERMINATE;
        (*dtd)[dtd_i].token = ((len as u32) << DTD_TOKEN_TOTAL_SHIFT) | DTD_TOKEN_IOC | DTD_TOKEN_ACTIVE;
        let base = buf_ptr as u32;
        (*dtd)[dtd_i].buffer[0] = base;
        for page in 1..5 {
            (*dtd)[dtd_i].buffer[page] = (base & !0xFFF) + (page as u32) * 0x1000;
        }

        // Link it into the queue head overlay and clear status.
        let qh = core::ptr::addr_of_mut!(QH_LIST.qh);
        (*qh)[qhi].next_dtd = core::ptr::addr_of!((*dtd)[dtd_i]) as u32;
        (*qh)[qhi].token = 0;
    }

    let r = regs();
    let prime_bit = endpoint_bit(index, dir);

    // Ensure descriptor and buffer writes complete before the controller fetches
    // the dTD. This mirrors the MCUX EHCI driver's barrier before EPPRIME.
    asm::dsb();

    // Prime the endpoint.
    r.set_endptprime(prime_bit);
    // Wait until the controller has acknowledged the prime.
    while r.endptprime() & prime_bit != 0 {}

    // Wait for completion via the waker, re-checking the descriptor status.
    let n = poll_fn(|cx| {
        EP_WAKERS[dtd_i].register(cx.waker());
        if abort_on_setup && (FLAG_SETUP.load(Ordering::Relaxed) || r.endptsetupstat() & 1 != 0) {
            r.set_endptflush(prime_bit);
            while r.endptflush() & prime_bit != 0 {}
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        // Ensure CPU loads below see descriptor/buffer writes completed by the controller.
        asm::dsb();
        // SAFETY: reading the (volatile) hardware-updated descriptor token.
        let token = unsafe {
            let dtd = core::ptr::addr_of!(DTD_LIST.dtd);
            core::ptr::read_volatile(core::ptr::addr_of!((*dtd)[dtd_i].token))
        };
        if token & DTD_TOKEN_ACTIVE != 0 {
            return Poll::Pending;
        }
        if token & DTD_TOKEN_ERROR_MASK != 0 {
            return Poll::Ready(Err(EndpointError::Disabled));
        }
        // Remaining bytes are in token[30:16]; transferred = requested - remaining.
        let remaining = (token >> DTD_TOKEN_TOTAL_SHIFT) & 0x7FFF;
        Poll::Ready(Ok(len - remaining as usize))
    })
    .await?;

    Ok(n)
}

/// Static bounce buffers for endpoint transfers (one per endpoint direction),
/// ensuring DMA-visible, suitably-aligned storage.
#[repr(C, align(64))]
struct EpBuffers {
    buf: [[u8; FS_MAX_PACKET as usize]; EP_COUNT * 2],
}
static mut EP_BUFFERS: EpBuffers = EpBuffers {
    buf: [[0; FS_MAX_PACKET as usize]; EP_COUNT * 2],
};

async fn ep_read(index: usize, buf: &mut [u8]) -> Result<usize, EndpointError> {
    let dtd_i = qh_index(index, Direction::Out);
    // SAFETY: each endpoint direction owns its dedicated bounce buffer.
    let bounce = unsafe { core::ptr::addr_of_mut!(EP_BUFFERS.buf[dtd_i]) as *mut u8 };
    let cap = FS_MAX_PACKET as usize;
    let n = ep_transfer(index, Direction::Out, bounce, cap, false).await?;
    if n > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }
    // SAFETY: `bounce` is valid for `n` bytes and `buf` for its length.
    unsafe { core::ptr::copy_nonoverlapping(bounce, buf.as_mut_ptr(), n) };
    Ok(n)
}

async fn ep_write(index: usize, buf: &[u8], abort_on_setup: bool) -> Result<(), EndpointError> {
    if buf.len() > FS_MAX_PACKET as usize {
        return Err(EndpointError::BufferOverflow);
    }
    let dtd_i = qh_index(index, Direction::In);
    // SAFETY: each endpoint direction owns its dedicated bounce buffer.
    let bounce = unsafe { core::ptr::addr_of_mut!(EP_BUFFERS.buf[dtd_i]) as *mut u8 };
    // SAFETY: copying caller data into the owned bounce buffer.
    unsafe { core::ptr::copy_nonoverlapping(buf.as_ptr(), bounce, buf.len()) };
    ep_transfer(index, Direction::In, bounce, buf.len(), abort_on_setup).await?;
    Ok(())
}

async fn ep_read_control(buf: &mut [u8]) -> Result<usize, EndpointError> {
    let dtd_i = qh_index(0, Direction::Out);
    // SAFETY: EP0 OUT owns its dedicated bounce buffer while the control pipe is active.
    let bounce = unsafe { core::ptr::addr_of_mut!(EP_BUFFERS.buf[dtd_i]) as *mut u8 };
    let cap = FS_MAX_PACKET as usize;
    let n = ep_transfer(0, Direction::Out, bounce, cap, true).await?;
    if n > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }
    // SAFETY: `bounce` is valid for `n` bytes and `buf` for its length.
    unsafe { core::ptr::copy_nonoverlapping(bounce, buf.as_mut_ptr(), n) };
    Ok(n)
}

async fn ep_zlp(index: usize, dir: Direction, abort_on_setup: bool) -> Result<(), EndpointError> {
    let dtd_i = qh_index(index, dir);
    // EHCI should not dereference buffer pointers for zero-length transfers, but
    // giving it a real aligned address avoids relying on null-pointer behavior.
    let bounce = unsafe { core::ptr::addr_of_mut!(EP_BUFFERS.buf[dtd_i]) as *mut u8 };
    ep_transfer(index, dir, bounce, 0, abort_on_setup).await?;
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
            EP_WAKERS[0].register(cx.waker());
            let stat = r.endptsetupstat();
            if stat & 1 == 0 {
                return Poll::Pending;
            }

            // Read the setup packet using the setup tripwire so a back-to-back
            // SETUP cannot corrupt the read.
            let setup = loop {
                r.modify_usbcmd(|v| v | USBCMD_SUTW);
                // SAFETY: control OUT queue head holds the latest setup bytes.
                let bytes = unsafe {
                    let qh = core::ptr::addr_of!(QH_LIST.qh[0]);
                    let w0 = core::ptr::read_volatile(core::ptr::addr_of!((*qh).setup[0]));
                    let w1 = core::ptr::read_volatile(core::ptr::addr_of!((*qh).setup[1]));
                    let mut b = [0u8; 8];
                    b[0..4].copy_from_slice(&w0.to_le_bytes());
                    b[4..8].copy_from_slice(&w1.to_le_bytes());
                    b
                };
                if r.usbcmd() & USBCMD_SUTW != 0 {
                    break bytes;
                }
            };
            r.modify_usbcmd(|v| v & !USBCMD_SUTW);
            // Acknowledge the setup status.
            r.clear_endptsetupstat(stat);
            r.modify_endptctrl(0, |v| v & !(EPCTRL_TXS | EPCTRL_RXS));
            FLAG_SETUP.store(false, Ordering::Relaxed);

            Poll::Ready(setup)
        })
        .await
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        // EP0 OUT owned by the control pipe.
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
pub struct Bus<T: Instance> {
    _phantom: PhantomData<T>,
    inited: bool,
    control_max_packet_size: u16,
}

impl<T: Instance> embassy_usb_driver::Bus for Bus<T> {
    async fn enable(&mut self) {
        let r = regs();
        // Enable the interrupt sources we handle.
        r.set_usbintr(USBSTS_UI | USBSTS_UEI | USBSTS_PCI | USBSTS_URI | USBSTS_SLI);

        // SAFETY: enabling the controller interrupt.
        unsafe {
            T::Interrupt::unpend();
            T::Interrupt::enable();
        }

        // Attach: set Run/Stop.
        r.modify_usbcmd(|v| v | USBCMD_RS);
        self.inited = true;
    }

    async fn disable(&mut self) {
        let r = regs();
        r.modify_usbcmd(|v| v & !USBCMD_RS);
        r.set_usbintr(0);
        T::Interrupt::disable();
    }

    async fn poll(&mut self) -> Event {
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());

            if !self.inited {
                // Surface an initial power-detected event so the stack proceeds.
                self.inited = true;
                return Poll::Ready(Event::PowerDetected);
            }

            if FLAG_RESET.swap(false, Ordering::Relaxed) {
                // Re-initialize endpoint 0 and clear setup/complete state.
                let r = regs();
                let setup = r.endptsetupstat();
                r.clear_endptsetupstat(setup);
                FLAG_SETUP.store(false, Ordering::Relaxed);
                let complete = r.endptcomplete();
                r.clear_endptcomplete(complete);
                while r.endptprime() != 0 {}
                r.set_endptflush(0xFFFF_FFFF);
                r.set_deviceaddr(0);
                init_control_qh(self.control_max_packet_size);
                return Poll::Ready(Event::Reset);
            }
            if FLAG_RESUME.swap(false, Ordering::Relaxed) {
                return Poll::Ready(Event::Resume);
            }
            if FLAG_SUSPEND.swap(false, Ordering::Relaxed) {
                return Poll::Ready(Event::Suspend);
            }
            Poll::Pending
        })
        .await
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        let index = ep_addr.index();
        let is_in = ep_addr.is_in();
        if enabled {
            let wi = waker_index(ep_addr);
            let mps = EP_MAX_PACKET[wi].load(Ordering::Relaxed);
            let ty = match EP_TYPE[wi].load(Ordering::Relaxed) {
                0 => EndpointType::Control,
                1 => EndpointType::Isochronous,
                2 => EndpointType::Bulk,
                _ => EndpointType::Interrupt,
            };
            configure_endpoint(ep_addr, ty, mps);
        } else {
            let r = regs();
            r.modify_endptctrl(index, |v| if is_in { v & !EPCTRL_TXE } else { v & !EPCTRL_RXE });
        }
        EP_WAKERS[waker_index(ep_addr)].wake();
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        let index = ep_addr.index();
        let is_in = ep_addr.is_in();
        let r = regs();
        r.modify_endptctrl(index, |v| {
            let bit = if is_in { EPCTRL_TXS } else { EPCTRL_RXS };
            if stalled { v | bit } else { v & !bit }
        });
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let index = ep_addr.index();
        let is_in = ep_addr.is_in();
        let v = regs().endptctrl(index);
        if is_in {
            v & EPCTRL_TXS != 0
        } else {
            v & EPCTRL_RXS != 0
        }
    }
}
