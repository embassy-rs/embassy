//! USB1 high-speed device driver for LPC55 (ip3511-HS).
//!
//! Device mode only. Supports control, bulk and interrupt endpoints;
//! isochronous endpoint allocation returns [`EndpointAllocError`].
//!
//! The system (main) clock must run at 96 MHz or higher, see
//! [`crate::config::MainClock::FroHf96`].

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac;

/// Number of logical endpoints (each has an IN and an OUT side).
const EP_COUNT: usize = 6;
/// Dedicated USB1 SRAM.
const USB_SRAM_ADDR: u32 = 0x4010_0000;
const USB_SRAM_SIZE: u32 = 0x4000;
/// The endpoint command/status list occupies `EP_COUNT * 4` words at the
/// start of the USB SRAM (256-byte aligned by construction). Word layout per
/// logical EP i: `[out_buf0, out_buf1, in_buf0, in_buf1]`. EP0's `out_buf1`
/// slot holds the SETUP buffer pointer.
///
/// Data buffers are bump-allocated 64-byte aligned from here on; the first
/// allocation (in `Driver::new`) is the 8-byte SETUP buffer.
const DATA_BUFFERS_START: u32 = USB_SRAM_ADDR + 0x80;

// HS endpoint command/status word encoding. NOTE: differs from the USB0-FS
// controller (NBytes is 15 bits at bit 11, AddressOffset is 11 bits).
const CMD_A: u32 = 1 << 31; // Active (HW clears on completion)
const CMD_D: u32 = 1 << 30; // Disabled
const CMD_S: u32 = 1 << 29; // Stall
const CMD_TR: u32 = 1 << 28; // Toggle Reset (write-1)
const CMD_TV: u32 = 1 << 27; // RF/TV: data toggle value for bulk/interrupt
const NBYTES_SHIFT: u32 = 11;
const NBYTES_MASK: u32 = 0x7FFF;

/// Per-slot buffer capacity for HS bulk endpoints, in bytes.
///
/// The ip3511-HS packetizes a single command/status entry: NBytes (15 bits)
/// may span multiple max-packet-size packets and hardware streams them all
/// without CPU involvement, so larger slots amortize the ISR + executor +
/// re-arm turnaround over many packets.
///
/// OUT slots are a power of two (8 x 512) because an OUT window only
/// completes when the buffer fills or a short packet arrives: host-side
/// writes are power-of-two-sized, so windows tile them exactly. IN has no
/// such constraint; 3584 (7 x 512) makes both directions plus EP0 and one
/// interrupt endpoint fit the 16 KiB USB SRAM (2x4096 + 2x3584 + overhead).
const BULK_OUT_SLOT_LEN: u16 = 4096;
const BULK_IN_SLOT_LEN: u16 = 3584;

static BUS_WAKER: AtomicWaker = AtomicWaker::new();
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];

// Double-buffer bookkeeping for data endpoints, shared between `Bus`
// (reset/enable/stall paths) and the endpoint futures. Hardware consumes the
// two command/status entries of a double-buffered endpoint in strict EPINUSE
// ping-pong order, so a 1-bit cursor kept in lockstep (resynced to slot 0
// whenever EPINUSE is cleared) tracks the hardware exactly.
const SLOT: u8 = 1 << 0; // next slot to arm (IN) / consume (OUT)
const PRIMED0: u8 = 1 << 1; // OUT slot armed or holding an unread packet (`PRIMED0 << slot`)
const TR_PENDING: u8 = 1 << 3; // reset the data toggle on the next arm
static EP_IN_STATE: [AtomicU8; EP_COUNT] = [const { AtomicU8::new(0) }; EP_COUNT];
static EP_OUT_STATE: [AtomicU8; EP_COUNT] = [const { AtomicU8::new(0) }; EP_COUNT];

fn nbytes(n: u32) -> u32 {
    (n & NBYTES_MASK) << NBYTES_SHIFT
}

fn remaining_bytes(word: u32) -> u32 {
    (word >> NBYTES_SHIFT) & NBYTES_MASK
}

/// AddressOffset field: bits [10:0] = buffer address >> 6. The upper address
/// bits come from DATABUFSTART.
fn addroff(buf_addr: u32) -> u32 {
    debug_assert_eq!(buf_addr & 0x3F, 0);
    (buf_addr >> 6) & 0x7FF
}

/// Pointer to the command/status word of a physical endpoint buffer slot.
fn ep_cmd_ptr(index: usize, dir: Direction, buf1: bool) -> *mut u32 {
    let slot = match dir {
        Direction::Out => 0,
        Direction::In => 2,
    } + buf1 as usize;
    (USB_SRAM_ADDR as usize + index * 16 + slot * 4) as *mut u32
}

fn ep_cmd_read(index: usize, dir: Direction, slot: usize) -> u32 {
    unsafe { ep_cmd_ptr(index, dir, slot != 0).read_volatile() }
}

fn ep_cmd_write(index: usize, dir: Direction, slot: usize, word: u32) {
    unsafe { ep_cmd_ptr(index, dir, slot != 0).write_volatile(word) }
}

fn ep_cmd_modify(index: usize, dir: Direction, slot: usize, f: impl FnOnce(u32) -> u32) {
    critical_section::with(|_| {
        let p = ep_cmd_ptr(index, dir, slot != 0);
        unsafe { p.write_volatile(f(p.read_volatile())) }
    });
}

/// Arm one buffer slot of a data endpoint.
///
/// The command word is written whole: the HS controller tracks the
/// bulk/interrupt data toggle internally per endpoint (read-only EPTOGGLE
/// register), not in the entry's TV bit, which is only consumed together
/// with TR. `tr` requests that toggle reset; it is set exactly once for the
/// first transfer after enable/unstall (the NXP SDK's deferred toggle-reset
/// discipline).
fn ep_arm_slot(index: usize, dir: Direction, slot: usize, len: u32, buf_addr: u32, tr: bool) {
    let tr = if tr { CMD_TR } else { 0 };
    ep_cmd_write(index, dir, slot, CMD_A | nbytes(len) | addroff(buf_addr) | tr);
}

/// Copy into the dedicated USB SRAM using 32-bit accesses.
///
/// The SRAM sits behind the AHB matrix, so each access is a full bus
/// transaction: byte-wise copies are 4x the transactions and dominate the
/// per-packet cost at high speed. Endpoint buffers are 64-byte aligned and
/// rounded up to 64-byte multiples (see `alloc_buffer`), so word-aligned
/// stores plus a full-word tail never leave the allocation.
fn copy_to_usb_sram(buf_addr: u32, data: &[u8]) {
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

/// Copy out of the dedicated USB SRAM using 32-bit accesses; see
/// [`copy_to_usb_sram`] for the alignment/over-read reasoning.
fn copy_from_usb_sram(buf_addr: u32, data: &mut [u8]) {
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

trait SealedInstance {
    fn regs() -> pac::usbhsd::Usbhsd;
}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

impl SealedInstance for crate::peripherals::USBHSD {
    fn regs() -> pac::usbhsd::Usbhsd {
        pac::USBHSD
    }
}

impl Instance for crate::peripherals::USBHSD {
    type Interrupt = crate::interrupt::typelevel::USB1;
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
        let stat = regs.intstat().read();
        // W1C exactly the bits we saw. Later-arriving bits stay pending and
        // re-fire the interrupt. Losing an edge is safe by design: the futures
        // re-check level state (eplist Active bit / DEVCMDSTAT.SETUP).
        regs.intstat().write_value(stat);

        for i in 0..EP_COUNT {
            if stat.0 & (1 << (2 * i)) != 0 {
                EP_OUT_WAKERS[i].wake();
            }
            if stat.0 & (1 << (2 * i + 1)) != 0 {
                EP_IN_WAKERS[i].wake();
            }
        }

        // A pending SETUP must abort in-flight control transfers in either
        // direction, so wake both EP0 futures.
        if regs.devcmdstat().read().setup() {
            EP_OUT_WAKERS[0].wake();
            EP_IN_WAKERS[0].wake();
        }

        // dev_int: DEVCMDSTAT change bits are consumed by `Bus::poll`.
        if stat.0 & (1 << 31) != 0 {
            BUS_WAKER.wake();
        }
    }
}

/// LPC55 USB1 high-speed device driver.
pub struct Driver<'d, T: Instance> {
    _usb: Peri<'d, T>,
    ep_in_used: [bool; EP_COUNT],
    ep_out_used: [bool; EP_COUNT],
    /// Bump allocator over the dedicated USB SRAM.
    alloc_offset: u32,
    /// Address of the 8-byte (64-byte aligned) SETUP buffer.
    setup_addr: u32,
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Create a new USB driver.
    ///
    /// This powers up and configures the USB1 PHY (expects a 32 MHz crystal)
    /// and the device controller, but does not connect to the bus yet; the
    /// soft-connect happens in [`driver::Bus::enable`].
    pub fn new(usb: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd) -> Self {
        use pac::syscon::vals::{Usb1DevRst, Usb1HostRst, Usb1PhyRst, Usb1RamRst};

        let regs = T::regs();

        // Reset the USB1 blocks sequentially, per block, with their clocks
        // still off — the exact order lpc55-hal's `enabled_as_device` uses
        // (host, then device + RAM, then PHY). Clocks are enabled per block
        // further down, right before each block is first used.
        critical_section::with(|_| {
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::ASSERTED));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_host_rst(Usb1HostRst::RELEASED));
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::ASSERTED);
                w.set_usb1_ram_rst(Usb1RamRst::ASSERTED);
            });
            pac::SYSCON.presetctrl2().modify(|w| {
                w.set_usb1_dev_rst(Usb1DevRst::RELEASED);
                w.set_usb1_ram_rst(Usb1RamRst::RELEASED);
            });
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::ASSERTED));
            pac::SYSCON
                .presetctrl2()
                .modify(|w| w.set_usb1_phy_rst(Usb1PhyRst::RELEASED));

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

        // PDRUNCFG bit positions (1 = powered down; writing 1 to the CLR
        // register clears the bit, i.e. powers the block ON).
        const PDEN_XTAL32M: u32 = 1 << 8;
        const PDEN_USBHSPHY: u32 = 1 << 12;
        const PDEN_LDOUSBHS: u32 = 1 << 18;
        const PDEN_LDOXO32M: u32 = 1 << 20;

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
            w.set_pll_div_sel(pac::usbphy::vals::PllSicPllDivSel::VALUE6);
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
            if phy.pll_sic().read().pll_lock() == pac::usbphy::vals::PllSicPllLock::VALUE1 {
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

        // Zero the endpoint command/status list and reserve the SETUP buffer.
        for i in 0..EP_COUNT * 4 {
            unsafe { ((USB_SRAM_ADDR as usize + i * 4) as *mut u32).write_volatile(0) };
        }
        let setup_addr = DATA_BUFFERS_START;
        unsafe {
            (setup_addr as *mut u32).write_volatile(0);
            (setup_addr as *mut u32).add(1).write_volatile(0);
            // EP0 out_buf1 slot holds the SETUP buffer pointer.
            ep_cmd_ptr(0, Direction::Out, true).write_volatile(addroff(setup_addr));
        }

        regs.databufstart().write(|w| w.set_da_buf(USB_SRAM_ADDR));
        regs.epliststart()
            .write_value(pac::usbhsd::regs::Epliststart(USB_SRAM_ADDR));
        // Non-control endpoints run double-buffered: EPBUFCFG has one bit
        // per physical endpoint (EP0 excluded), and hardware ping-pongs the
        // two command/status entries via EPINUSE.
        regs.epinuse().write(|w| w.set_buf(0));
        regs.epbufcfg().write(|w| w.set_buf_sb(0x3FF));
        regs.epskip().write_value(pac::usbhsd::regs::Epskip(0));

        // Enable the device controller, but do not connect yet. Preserve the
        // reset defaults of the other fields (notably LPM_SUP).
        devcmdstat_modify(regs, |w| w.set_dev_en(true));

        regs.intstat().write_value(pac::usbhsd::regs::Intstat(!0));
        regs.inten().write(|w| {
            w.set_ep_int_en(0xFFF);
            w.set_dev_int_en(true);
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _usb: usb,
            ep_in_used: [false; EP_COUNT],
            ep_out_used: [false; EP_COUNT],
            alloc_offset: setup_addr + 64 - USB_SRAM_ADDR,
            setup_addr,
        }
    }

    /// Carve a 64-byte-aligned buffer out of the USB SRAM.
    fn alloc_buffer(&mut self, len: u16) -> Result<u32, EndpointAllocError> {
        let len = (len as u32 + 63) / 64 * 64;
        if self.alloc_offset + len > USB_SRAM_SIZE {
            warn!("USB SRAM full");
            return Err(EndpointAllocError);
        }
        let addr = USB_SRAM_ADDR + self.alloc_offset;
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
            // Isochronous unsupported (scope decision). Control is only used
            // internally for EP0; embassy-usb never allocates extra control
            // endpoints.
            EndpointType::Isochronous | EndpointType::Control => return Err(EndpointAllocError),
            EndpointType::Bulk => 512,
            EndpointType::Interrupt => 1024,
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
                if i == 0 || i >= EP_COUNT || used[i] {
                    return Err(EndpointAllocError);
                }
                i
            }
            None => (1..EP_COUNT).find(|&i| !used[i]).ok_or(EndpointAllocError)?,
        };
        used[index] = true;

        // Two buffer slots per endpoint (double buffering): software fills or
        // drains one while the other is on the wire. HS bulk endpoints get
        // multi-packet slots (see BULK_*_SLOT_LEN); fall back to single-packet
        // slots if the USB SRAM cannot fit the large ones.
        let want_len = match (ep_type, D::dir()) {
            (EndpointType::Bulk, Direction::Out) => BULK_OUT_SLOT_LEN.max(max_packet_size),
            (EndpointType::Bulk, Direction::In) => BULK_IN_SLOT_LEN.max(max_packet_size),
            _ => max_packet_size,
        };
        let mark = self.alloc_offset;
        let (buf_addrs, slot_len) = match (self.alloc_buffer(want_len), self.alloc_buffer(want_len)) {
            (Ok(a), Ok(b)) => ([a, b], want_len),
            _ if want_len > max_packet_size => {
                self.alloc_offset = mark;
                (
                    [self.alloc_buffer(max_packet_size)?, self.alloc_buffer(max_packet_size)?],
                    max_packet_size,
                )
            }
            _ => return Err(EndpointAllocError),
        };

        // Disabled until `endpoint_set_enabled`.
        ep_cmd_write(index, D::dir(), 0, CMD_D);
        ep_cmd_write(index, D::dir(), 1, CMD_D);

        trace!("  index={} addr={:08x}", index, buf_addrs[0]);

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
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
        let ep0_out_addr = self.alloc_buffer(control_max_packet_size).unwrap();
        let ep0_in_addr = self.alloc_buffer(control_max_packet_size).unwrap();

        // EP0 OUT stays armed at all times (lpc55-hal discipline): the
        // ip3511-HS needs an active EP0 OUT entry to accept SETUP and OUT
        // packets on the control pipe. `setup()`/`data_out()` re-arm it after
        // every consumed packet.
        ep_cmd_write(
            0,
            Direction::Out,
            0,
            CMD_A | nbytes(control_max_packet_size as u32) | addroff(ep0_out_addr),
        );
        ep_cmd_write(0, Direction::In, 0, addroff(ep0_in_addr));

        trace!("started");

        (
            Bus {
                _phantom: PhantomData,
                inited: false,
                setup_addr: self.setup_addr,
                ep0_out_addr,
                ep0_mps: control_max_packet_size,
            },
            ControlPipe {
                _phantom: PhantomData,
                max_packet_size: control_max_packet_size,
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
    inited: bool,
    setup_addr: u32,
    ep0_out_addr: u32,
    ep0_mps: u16,
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    async fn enable(&mut self) {
        // Soft-connect.
        devcmdstat_modify(T::regs(), |w| w.set_dcon(true));
    }

    async fn disable(&mut self) {
        devcmdstat_modify(T::regs(), |w| w.set_dcon(false));
    }

    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            BUS_WAKER.register(cx.waker());

            // No VBUS monitoring: report power immediately, once.
            if !self.inited {
                self.inited = true;
                return Poll::Ready(Event::PowerDetected);
            }

            let regs = T::regs();
            let dcs = regs.devcmdstat().read();

            if dcs.dres_c() {
                devcmdstat_modify(regs, |w| {
                    w.set_dres_c(true);
                    w.set_dev_addr(0);
                });

                // Disable all non-control endpoints (both buffer slots) and
                // reset the double-buffer bookkeeping; re-arm EP0 (EP0 OUT
                // stays armed at all times) and keep the SETUP pointer.
                for i in 1..EP_COUNT {
                    for slot in 0..2 {
                        ep_cmd_write(i, Direction::Out, slot, CMD_D);
                        ep_cmd_write(i, Direction::In, slot, CMD_D);
                    }
                    EP_OUT_STATE[i].store(0, Ordering::Relaxed);
                    EP_IN_STATE[i].store(0, Ordering::Relaxed);
                }
                regs.epinuse().write(|w| w.set_buf(0));
                regs.epbufcfg().write(|w| w.set_buf_sb(0x3FF));
                ep_cmd_write(
                    0,
                    Direction::Out,
                    0,
                    CMD_A | nbytes(self.ep0_mps as u32) | addroff(self.ep0_out_addr),
                );
                ep_cmd_write(0, Direction::In, 0, 0);
                unsafe { ep_cmd_ptr(0, Direction::Out, true).write_volatile(addroff(self.setup_addr)) };

                // Unstick any pending endpoint futures; they will observe the
                // disabled/reset state.
                for w in &EP_IN_WAKERS {
                    w.wake();
                }
                for w in &EP_OUT_WAKERS {
                    w.wake();
                }

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
                // No VBUS wired-status semantics needed; just acknowledge.
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
        let state = match dir {
            Direction::Out => &EP_OUT_STATE[index],
            Direction::In => &EP_IN_STATE[index],
        };

        if enabled {
            // Both slots idle (NAK until armed). Hardware must restart on
            // slot 0 to match the software cursor, and the data toggle is
            // reset on the first armed transfer (TR_PENDING).
            ep_cmd_write(index, dir, 0, 0);
            ep_cmd_write(index, dir, 1, 0);
            let phy_ep = 2 * index + (dir == Direction::In) as usize;
            T::regs().epinuse().modify(|w| w.set_buf(w.buf() & !(1 << (phy_ep - 2))));
            state.store(TR_PENDING, Ordering::Relaxed);
        } else {
            ep_cmd_write(index, dir, 0, CMD_D);
            ep_cmd_write(index, dir, 1, CMD_D);
            state.store(0, Ordering::Relaxed);
        }

        match dir {
            Direction::Out => EP_OUT_WAKERS[index].wake(),
            Direction::In => EP_IN_WAKERS[index].wake(),
        }
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        trace!("set_stalled {:?} {}", ep_addr, stalled);
        let index = ep_addr.index();
        let dir = ep_addr.direction();
        let regs = T::regs();
        let phy_ep = 2 * index + (dir == Direction::In) as usize;
        // EP0's second OUT slot is the SETUP pointer: never touch it.
        let slots = if index == 0 { 0..1 } else { 0..2 };

        if stalled {
            for slot in slots.clone() {
                if ep_cmd_read(index, dir, slot) & CMD_A != 0 {
                    // Reclaim the in-flight transfer via EPSKIP (skips the
                    // slot EPINUSE points at).
                    regs.epskip().write_value(pac::usbhsd::regs::Epskip(1 << phy_ep));
                    while regs.epskip().read().0 & (1 << phy_ep) != 0 {}
                    regs.intstat().write_value(pac::usbhsd::regs::Intstat(1 << phy_ep));
                }
            }
            for slot in slots {
                ep_cmd_modify(index, dir, slot, |w| (w & !CMD_A) | CMD_S);
            }
        } else if index == 0 {
            // Clearing a stall resets the data toggle to DATA0.
            ep_cmd_modify(index, dir, 0, |w| (w & !CMD_S) | CMD_TR);
        } else {
            // Drop any buffered packets, resync the hardware slot cursor and
            // defer the toggle reset to the next armed transfer.
            for slot in slots {
                ep_cmd_modify(index, dir, slot, |w| w & !(CMD_S | CMD_A));
            }
            regs.epinuse().modify(|w| w.set_buf(w.buf() & !(1 << (phy_ep - 2))));
            let state = match dir {
                Direction::Out => &EP_OUT_STATE[index],
                Direction::In => &EP_IN_STATE[index],
            };
            state.store(TR_PENDING, Ordering::Relaxed);
        }

        match dir {
            Direction::Out => EP_OUT_WAKERS[index].wake(),
            Direction::In => EP_IN_WAKERS[index].wake(),
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        ep_cmd_read(ep_addr.index(), ep_addr.direction(), 0) & CMD_S != 0
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        devcmdstat_modify(T::regs(), |w| w.set_dsus(false));
        Ok(())
    }
}

/// USB endpoint.
pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    buf_addrs: [u32; 2],
    /// Per-slot buffer capacity (multi-packet for HS bulk endpoints).
    slot_len: u16,
    /// Length each OUT slot was last armed with; needed to compute the
    /// received count from the NBytes-remaining field on completion.
    armed_len: [u16; 2],
}

impl<'d, T: Instance, D: Dir> driver::Endpoint for Endpoint<'d, T, D> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        let index = self.info.addr.index();
        let wakers = match D::dir() {
            Direction::Out => &EP_OUT_WAKERS,
            Direction::In => &EP_IN_WAKERS,
        };
        poll_fn(|cx| {
            wakers[index].register(cx.waker());
            if ep_cmd_read(index, D::dir(), 0) & CMD_D == 0 {
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
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        let mps = self.info.max_packet_size as u32;
        let state = &EP_OUT_STATE[index];

        // Receive window per slot: a single packet by default, or a
        // multi-packet window (hardware packetizes one command entry) when
        // the caller's buffer spans several packets. A window completes when
        // it fills or a short packet arrives, so callers using large buffers
        // must expect coalescing without intermediate packet boundaries.
        let arm_len = if buf.len() as u32 > mps {
            (buf.len() as u32).min(self.slot_len as u32) / mps * mps
        } else {
            mps
        };

        // Keep both slots armed so the endpoint does not NAK between packets
        // while the host streams. A slot with PRIMED set and Active clear
        // holds a received, not-yet-consumed packet.
        let mut s = state.load(Ordering::Relaxed);
        for slot in 0..2 {
            if s & (PRIMED0 << slot) == 0 {
                if ep_cmd_read(index, Direction::Out, slot) & CMD_D != 0 {
                    return Err(EndpointError::Disabled);
                }
                ep_arm_slot(index, Direction::Out, slot, arm_len, self.buf_addrs[slot], s & TR_PENDING != 0);
                self.armed_len[slot] = arm_len as u16;
                s = (s | (PRIMED0 << slot)) & !TR_PENDING;
                state.store(s, Ordering::Relaxed);
            }
        }

        // Consume in hardware ping-pong order.
        let slot = (s & SLOT) as usize;
        let word = poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            let word = ep_cmd_read(index, Direction::Out, slot);
            if word & CMD_D != 0 {
                // Disabled by a bus reset while waiting.
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

        let rx_len = (self.armed_len[slot] as u32 - remaining_bytes(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_usb_sram(self.buf_addrs[slot], &mut buf[..rx_len]);

        // Hand the drained slot straight back to hardware and advance the
        // consume cursor.
        ep_arm_slot(index, Direction::Out, slot, arm_len, self.buf_addrs[slot], false);
        self.armed_len[slot] = arm_len as u16;
        state.store((s ^ SLOT) | (PRIMED0 << slot), Ordering::Relaxed);

        trace!("READ {:?} rx_len = {}", self.info.addr, rx_len);
        Ok(rx_len)
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        // Multi-packet: one command entry sends ceil(len / mps) packets
        // without CPU involvement; the slot capacity is the only limit.
        if buf.len() > self.slot_len as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let index = self.info.addr.index();
        let state = &EP_IN_STATE[index];

        // Fill slots in hardware ping-pong order, returning as soon as the
        // packet is armed: the wire transmits from one slot while software
        // fills the other, so a saturating writer never lets the endpoint
        // NAK an IN token.
        loop {
            let s = state.load(Ordering::Relaxed);
            let slot = (s & SLOT) as usize;

            poll_fn(|cx| {
                EP_IN_WAKERS[index].register(cx.waker());
                let word = ep_cmd_read(index, Direction::In, slot);
                if word & CMD_D != 0 {
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

            copy_to_usb_sram(self.buf_addrs[slot], buf);
            ep_arm_slot(index, Direction::In, slot, buf.len() as u32, self.buf_addrs[slot], s & TR_PENDING != 0);
            state.store((s ^ SLOT) & !TR_PENDING, Ordering::Relaxed);

            trace!("WRITE {:?} len = {}", self.info.addr, buf.len());
            return Ok(());
        }
    }
}

/// USB control pipe (endpoint 0, both directions).
pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
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
            EP_OUT_WAKERS[0].register(cx.waker());
            if regs.devcmdstat().read().setup() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // UM procedure: clear Active and Stall on both EP0 directions BEFORE
        // acknowledging the SETUP bit.
        ep_cmd_modify(0, Direction::Out, 0, |w| w & !(CMD_A | CMD_S));
        ep_cmd_modify(0, Direction::In, 0, |w| w & !(CMD_A | CMD_S));

        let mut buf = [0; 8];
        copy_from_usb_sram(self.setup_addr, &mut buf);

        devcmdstat_modify(regs, |w| w.set_setup(true));

        // Hardware may overwrite the SETUP slot's address field: re-write it,
        // and re-arm EP0 OUT for the next packet.
        unsafe { ep_cmd_ptr(0, Direction::Out, true).write_volatile(addroff(self.setup_addr)) };
        ep_cmd_write(
            0,
            Direction::Out,
            0,
            CMD_A | nbytes(self.max_packet_size as u32) | addroff(self.ep0_out_addr),
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
            EP_OUT_WAKERS[0].register(cx.waker());
            // A new SETUP aborts the transfer (trait contract).
            if regs.devcmdstat().read().setup() {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let word = ep_cmd_read(0, Direction::Out, 0);
            if word & CMD_A == 0 {
                Poll::Ready(Ok(word))
            } else {
                Poll::Pending
            }
        })
        .await?;

        let rx_len = (mps - remaining_bytes(word)) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        copy_from_usb_sram(self.ep0_out_addr, &mut buf[..rx_len]);

        // Re-arm for the next OUT packet (or the status stage).
        ep_cmd_write(0, Direction::Out, 0, CMD_A | nbytes(mps) | addroff(self.ep0_out_addr));

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

        copy_to_usb_sram(self.ep0_in_addr, data);
        ep_cmd_write(
            0,
            Direction::In,
            0,
            CMD_A | nbytes(data.len() as u32) | addroff(self.ep0_in_addr),
        );

        poll_fn(|cx| {
            EP_IN_WAKERS[0].register(cx.waker());
            if regs.devcmdstat().read().setup() {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if ep_cmd_read(0, Direction::In, 0) & CMD_A == 0 {
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
        ep_cmd_write(0, Direction::In, 0, CMD_A | addroff(self.ep0_in_addr));

        poll_fn(|cx| {
            EP_IN_WAKERS[0].register(cx.waker());
            // Bail out if the host abandoned the request with a new SETUP.
            if T::regs().devcmdstat().read().setup() {
                return Poll::Ready(());
            }
            if ep_cmd_read(0, Direction::In, 0) & CMD_A == 0 {
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
        ep_cmd_modify(0, Direction::Out, 0, |w| w | CMD_S);
        ep_cmd_modify(0, Direction::In, 0, |w| w | CMD_S);
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
