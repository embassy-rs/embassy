use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::num::NonZeroU32;
use core::slice;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use critical_section::Mutex;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::iso::{IsoSchedule, forward};
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointConfig, EndpointDisabledError, EndpointError, EndpointInfo,
    EndpointInterval, EndpointType, Event, IsoEndpointIn, IsoInSlot, IsoWriteError, Speed, Unsupported,
};

use super::{Dir, In, Instance, Out};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::{Peri, RegExt, interrupt, pac};

const EP_COUNT: usize = 16;
const EP_MEMORY_SIZE: usize = 4096;
const EP_MEMORY: *mut u8 = pac::USB_DPRAM.as_ptr() as *mut u8;
const ISO_FRAME_SPAN: u32 = 1 << 11;

static BUS_WAKER: AtomicWaker = AtomicWaker::new();
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];
static EP_IN_XFERS: Mutex<RefCell<[Option<InXfer>; EP_COUNT]>> = Mutex::new(RefCell::new([const { None }; EP_COUNT]));
static EP_OUT_XFERS: Mutex<RefCell<[Option<OutXfer>; EP_COUNT]>> = Mutex::new(RefCell::new([const { None }; EP_COUNT]));
static ISO_IN_STATE: Mutex<RefCell<[IsoInState; EP_COUNT]>> = Mutex::new(RefCell::new([IsoInState::new(); EP_COUNT]));
static ISO_SOF: Mutex<RefCell<IsoSofState>> = Mutex::new(RefCell::new(IsoSofState {
    last_sof: 0,
    seen: false,
}));

#[derive(Clone, Copy)]
struct IsoInState {
    /// True while this endpoint is enabled as an isochronous IN endpoint.
    enabled: bool,
    /// True while an isochronous IN packet is armed in DPRAM.
    armed: bool,
    /// Frame counter value in `0..ISO_FRAME_SPAN` at which the armed packet expires if uncollected.
    expiry_frame: u16,
    /// Locked host polling phase in `0..interval`, or `None` until the first completion.
    phase: Option<u16>,
    /// Service interval in frames.
    interval: u16,
    /// True while [`IsoEndpointIn::ready`] is waiting on SOF.
    arm_wait: bool,
    /// Set on endpoint disable or bus reset so [`IsoEndpointIn::ready`] resets its [`IsoSchedule`].
    reset: bool,
}

impl IsoInState {
    const fn new() -> Self {
        Self {
            enabled: false,
            armed: false,
            expiry_frame: 0,
            phase: None,
            interval: 1,
            arm_wait: false,
            reset: true,
        }
    }

    fn reset_state(&mut self) {
        self.armed = false;
        self.phase = None;
        self.arm_wait = false;
        self.reset = true;
    }
}

struct IsoSofState {
    last_sof: u16,
    seen: bool,
}

fn update_dev_sof_irq<T: Instance>(cs: critical_section::CriticalSection<'_>) {
    let active = ISO_IN_STATE
        .borrow(cs)
        .borrow()
        .iter()
        .any(|s| s.enabled || s.armed || s.arm_wait);
    if active {
        T::regs().inte().write_set(|w| w.set_dev_sof(true));
    } else {
        T::regs().inte().write_clear(|w| w.set_dev_sof(true));
        ISO_SOF.borrow(cs).borrow_mut().seen = false;
    }
}

struct OutXfer {
    buf_ptr: *mut u8,
    buf_len: usize,
    total: usize,
    max_packet_size: u16,
    dpram_addr: u16,
    dpram_len: u16,
    rearm_always: bool,
    result: Option<Result<usize, EndpointError>>,
}

// SAFETY: `OutXfer` is only kept active while the owning future holds `&mut [u8]`
// and is cleared by `OutXferGuard` on drop inside a critical section.
unsafe impl Send for OutXfer {}

impl OutXfer {
    fn new(buf: &mut [u8], max_packet_size: u16, dpram_addr: u16, dpram_len: u16, rearm_always: bool) -> Self {
        Self {
            buf_ptr: buf.as_mut_ptr(),
            buf_len: buf.len(),
            total: 0,
            max_packet_size,
            dpram_addr,
            dpram_len,
            rearm_always,
            result: None,
        }
    }

    fn on_packet<T: Instance>(&mut self, index: usize, val: pac::usb_dpram::regs::EpBufferControl) -> bool {
        if self.result.is_some() {
            return false;
        }
        let rx_len = val.length(0) as usize;
        let chunk_len = (self.buf_len - self.total).min(self.max_packet_size as usize);
        if rx_len > chunk_len {
            self.result = Some(Err(EndpointError::BufferOverflow));
            return true;
        }

        // SAFETY: `self.buf_ptr..self.buf_ptr + self.buf_len` is borrowed for the lifetime
        // of the active transfer future, and `self.total + rx_len <= self.buf_len`.
        let dst = unsafe { slice::from_raw_parts_mut(self.buf_ptr.add(self.total), rx_len) };
        EndpointBuffer::<T>::new(self.dpram_addr, self.dpram_len).read(dst);
        self.total += rx_len;

        let ended = rx_len < self.max_packet_size as usize || self.total == self.buf_len;
        if self.rearm_always || !ended {
            arm_ep_out_buffer::<T>(index, !val.pid(0), self.max_packet_size);
        }

        if ended {
            self.result = Some(Ok(self.total));
            true
        } else {
            false
        }
    }
}

struct OutXferGuard(usize);

impl Drop for OutXferGuard {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            EP_OUT_XFERS.borrow(cs).borrow_mut()[self.0] = None;
        });
    }
}

struct InXfer {
    buf_ptr: *const u8,
    buf_len: usize,
    offset: usize,
    send_zlp: bool,
    max_packet_size: u16,
    dpram_addr: u16,
    dpram_len: u16,
    wait_last_complete: bool,
    done: bool,
    error: Option<EndpointDisabledError>,
}

// SAFETY: `InXfer` is only kept active while the owning future holds `&[u8]`
// and is cleared by `InXferGuard` on drop inside a critical section.
unsafe impl Send for InXfer {}

impl InXfer {
    fn has_more(&self) -> bool {
        self.offset < self.buf_len || self.send_zlp
    }

    fn on_packet<T: Instance>(&mut self, index: usize, val: pac::usb_dpram::regs::EpBufferControl) -> bool {
        if self.done || self.error.is_some() {
            return false;
        }

        if self.offset < self.buf_len {
            let chunk_len = (self.buf_len - self.offset).min(self.max_packet_size as usize);
            // SAFETY: `self.buf_ptr..self.buf_ptr + self.buf_len` is borrowed for the lifetime
            // of the active transfer future.
            let chunk = unsafe { slice::from_raw_parts(self.buf_ptr.add(self.offset), chunk_len) };
            EndpointBuffer::<T>::new(self.dpram_addr, self.dpram_len).write(chunk);
            self.offset += chunk_len;
            arm_ep_in_buffer::<T>(index, !val.pid(0), chunk_len as u16);

            if !self.wait_last_complete && !self.has_more() {
                self.done = true;
                return true;
            }
            false
        } else if self.send_zlp {
            self.send_zlp = false;
            arm_ep_in_buffer::<T>(index, !val.pid(0), 0);

            if !self.wait_last_complete {
                self.done = true;
                return true;
            }
            false
        } else {
            self.done = true;
            true
        }
    }
}

struct InXferGuard(usize);

impl Drop for InXferGuard {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            EP_IN_XFERS.borrow(cs).borrow_mut()[self.0] = None;
        });
    }
}

fn arm_ep_out_buffer<T: Instance>(index: usize, pid: bool, len: u16) {
    let bufcontrol = T::dpram().ep_out_buffer_control(index);
    bufcontrol.write(|w| {
        w.set_length(0, len);
        w.set_pid(0, pid);
    });
    cortex_m::asm::delay(12);
    bufcontrol.write(|w| {
        w.set_length(0, len);
        w.set_pid(0, pid);
        w.set_available(0, true);
    });
}

fn arm_ep_in_buffer<T: Instance>(index: usize, pid: bool, len: u16) {
    let bufcontrol = T::dpram().ep_in_buffer_control(index);
    bufcontrol.write(|w| {
        w.set_length(0, len);
        w.set_pid(0, pid);
        w.set_full(0, true);
    });
    cortex_m::asm::delay(12);
    bufcontrol.write(|w| {
        w.set_length(0, len);
        w.set_pid(0, pid);
        w.set_full(0, true);
        w.set_available(0, true);
    });
}

struct EndpointBuffer<T: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<T>,
}

impl<T: Instance> EndpointBuffer<T> {
    const fn new(addr: u16, len: u16) -> Self {
        Self {
            addr,
            len,
            _phantom: PhantomData,
        }
    }

    fn read(&mut self, buf: &mut [u8]) {
        assert!(buf.len() <= self.len as usize);
        compiler_fence(Ordering::SeqCst);
        let mem = unsafe { slice::from_raw_parts(EP_MEMORY.add(self.addr as _), buf.len()) };
        buf.copy_from_slice(mem);
        compiler_fence(Ordering::SeqCst);
    }

    fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= self.len as usize);
        compiler_fence(Ordering::SeqCst);
        let mem = unsafe { slice::from_raw_parts_mut(EP_MEMORY.add(self.addr as _), buf.len()) };
        mem.copy_from_slice(buf);
        compiler_fence(Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct EndpointData {
    ep_type: EndpointType, // only valid if used
    max_packet_size: u16,
    used: bool,
}

impl EndpointData {
    const fn new() -> Self {
        Self {
            ep_type: EndpointType::Bulk,
            max_packet_size: 0,
            used: false,
        }
    }
}

/// RP2040 USB driver handle.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_in: [EndpointData; EP_COUNT],
    ep_out: [EndpointData; EP_COUNT],
    ep_mem_free: u16, // first free address in EP mem, in bytes.
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Create a new USB driver.
    pub fn new(_usb: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>>) -> Self {
        let regs = T::regs();

        // Reset the peripheral: zeroing its registers by hand would clear RP235x's `LINESTATE_TUNING`,
        // whose reset value enables controller fixes (RP2350 datasheet 12.7.2). Keep the interrupt off
        // until the reset is done, so the handler cannot touch the block while it is held in reset.
        T::Interrupt::disable();
        pac::RESETS.reset().write_set(|w| w.set_usbctrl(true));
        pac::RESETS.reset().write_clear(|w| w.set_usbctrl(true));
        while !pac::RESETS.reset_done().read().usbctrl() {}

        // DPRAM is outside the peripheral reset, so clear the control area by hand.
        unsafe {
            let p = EP_MEMORY as *mut u32;
            for i in 0..0x100 / 4 {
                p.add(i).write_volatile(0)
            }
        }

        critical_section::with(|cs| {
            let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
            for s in iso.iter_mut() {
                *s = IsoInState::new();
            }
            let mut sof = ISO_SOF.borrow(cs).borrow_mut();
            sof.last_sof = 0;
            sof.seen = false;
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        regs.usb_muxing().write(|w| {
            w.set_to_phy(true);
            w.set_softcon(true);
        });
        regs.usb_pwr().write(|w| {
            w.set_vbus_detect(true);
            w.set_vbus_detect_override_en(true);
        });
        regs.main_ctrl().write(|w| {
            w.set_controller_en(true);
        });

        // Initialize the bus so that it signals that power is available
        BUS_WAKER.wake();

        Self {
            phantom: PhantomData,
            ep_in: [EndpointData::new(); EP_COUNT],
            ep_out: [EndpointData::new(); EP_COUNT],
            ep_mem_free: 0x180, // data buffer region
        }
    }

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval: EndpointInterval,
    ) -> Result<Endpoint<'d, T, D>, driver::EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval={:?}, dir={:?}",
            ep_type,
            max_packet_size,
            interval,
            D::dir()
        );

        let alloc = match D::dir() {
            Direction::Out => &mut self.ep_out,
            Direction::In => &mut self.ep_in,
        };

        let index = if let Some(addr) = ep_addr {
            // Use the specified endpoint address
            let requested_index = addr.index();
            if requested_index == 0 || requested_index >= EP_COUNT {
                return Err(EndpointAllocError);
            }
            if alloc[requested_index].used {
                return Err(EndpointAllocError);
            }
            Some((requested_index, &mut alloc[requested_index]))
        } else {
            // Find any available endpoint
            alloc.iter_mut().enumerate().find(|(i, ep)| {
                if *i == 0 {
                    return false; // reserved for control pipe
                }
                !ep.used
            })
        };

        let (index, ep) = index.ok_or(EndpointAllocError)?;
        assert!(!ep.used);

        // as per datasheet, the maximum buffer size is 64, except for isochronous
        // endpoints, which are allowed to be up to 1023 bytes.
        if (ep_type != EndpointType::Isochronous && max_packet_size > 64) || max_packet_size > 1023 {
            warn!("max_packet_size too high: {}", max_packet_size);
            return Err(EndpointAllocError);
        }

        // ep mem addrs must be 64-byte aligned, so there's no point in trying
        // to allocate smaller chunks to save memory.
        let len = max_packet_size.div_ceil(64) * 64;

        let addr = self.ep_mem_free;
        if addr + len > EP_MEMORY_SIZE as u16 {
            warn!("Endpoint memory full");
            return Err(EndpointAllocError);
        }
        self.ep_mem_free += len;

        let buf = EndpointBuffer {
            addr,
            len,
            _phantom: PhantomData,
        };

        trace!("  index={} addr={} len={}", index, buf.addr, buf.len);

        ep.ep_type = ep_type;
        ep.used = true;
        ep.max_packet_size = max_packet_size;

        let ep_type_reg = match ep_type {
            EndpointType::Bulk => pac::usb_dpram::vals::EpControlEndpointType::Bulk,
            EndpointType::Control => pac::usb_dpram::vals::EpControlEndpointType::Control,
            EndpointType::Interrupt => pac::usb_dpram::vals::EpControlEndpointType::Interrupt,
            EndpointType::Isochronous => pac::usb_dpram::vals::EpControlEndpointType::Isochronous,
        };

        match D::dir() {
            Direction::Out => T::dpram().ep_out_control(index - 1).write(|w| {
                w.set_enable(false);
                w.set_buffer_address(addr);
                w.set_interrupt_per_buff(true);
                w.set_endpoint_type(ep_type_reg);
            }),
            Direction::In => T::dpram().ep_in_control(index - 1).write(|w| {
                w.set_enable(false);
                w.set_buffer_address(addr);
                w.set_interrupt_per_buff(true);
                w.set_endpoint_type(ep_type_reg);
            }),
        }

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo::new(
                EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval,
            ),
            buf,
            schedule: IsoSchedule::new(),
        })
    }
}

/// USB interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        //let x = regs.istr().read().0;
        //trace!("USB IRQ: {:08x}", x);

        let ints = regs.ints().read();

        if ints.bus_reset() {
            regs.inte().write_clear(|w| w.set_bus_reset(true));
            critical_section::with(|cs| {
                let mut in_xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                for (i, slot) in in_xfers.iter_mut().enumerate() {
                    if let Some(xfer) = slot.as_mut() {
                        if !xfer.done && xfer.error.is_none() {
                            xfer.error = Some(EndpointDisabledError);
                        }
                        EP_IN_WAKERS[i].wake();
                    }
                }
                let mut out_xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();
                for (i, slot) in out_xfers.iter_mut().enumerate() {
                    if let Some(xfer) = slot.as_mut() {
                        if xfer.result.is_none() {
                            xfer.result = Some(Err(EndpointError::Disabled));
                        }
                        EP_OUT_WAKERS[i].wake();
                    }
                }
                let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
                for (i, state) in iso.iter_mut().enumerate() {
                    if state.enabled || state.armed || state.arm_wait {
                        state.enabled = false;
                        state.reset_state();
                        EP_IN_WAKERS[i].wake();
                    }
                }
                drop(iso);
                update_dev_sof_irq::<T>(cs);
            });
            BUS_WAKER.wake();
        }
        if ints.dev_resume_from_host() {
            regs.inte().write_clear(|w| w.set_dev_resume_from_host(true));
            BUS_WAKER.wake();
        }
        if ints.dev_suspend() {
            regs.inte().write_clear(|w| w.set_dev_suspend(true));
            BUS_WAKER.wake();
        }
        if ints.setup_req() {
            regs.inte().write_clear(|w| w.set_setup_req(true));
            EP_OUT_WAKERS[0].wake();
        }

        if ints.buff_status() {
            let s = regs.buff_status().read();
            regs.buff_status().write_value(s);

            let sie_status = regs.sie_status().read();
            let ep0_aborted = sie_status.setup_rec() || sie_status.bus_reset();

            critical_section::with(|cs| {
                let mut in_xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                let mut out_xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();

                for i in 0..EP_COUNT {
                    if s.ep_in(i) {
                        if i == 0 && ep0_aborted {
                            if let Some(xfer) = in_xfers[0].as_mut() {
                                if !xfer.done && xfer.error.is_none() {
                                    xfer.error = Some(EndpointDisabledError);
                                }
                            }
                            EP_IN_WAKERS[0].wake();
                        } else if let Some(xfer) = in_xfers[i].as_mut() {
                            if i > 0 && !T::dpram().ep_in_control(i - 1).read().enable() {
                                xfer.error = Some(EndpointDisabledError);
                                EP_IN_WAKERS[i].wake();
                            } else {
                                let val = T::dpram().ep_in_buffer_control(i).read();
                                if !val.available(0) && !val.stall() && xfer.on_packet::<T>(i, val) {
                                    EP_IN_WAKERS[i].wake();
                                }
                            }
                        } else {
                            let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
                            if iso[i].armed {
                                iso[i].armed = false;
                                if iso[i].phase.is_none() {
                                    let now = ISO_SOF.borrow(cs).borrow().last_sof;
                                    let interval = iso[i].interval.max(1);
                                    iso[i].phase = Some(now % interval);
                                }
                            }
                            EP_IN_WAKERS[i].wake();
                        }
                    }

                    if s.ep_out(i) {
                        if i == 0 && ep0_aborted {
                            if let Some(xfer) = out_xfers[0].as_mut() {
                                if xfer.result.is_none() {
                                    xfer.result = Some(Err(EndpointError::Disabled));
                                }
                            }
                            EP_OUT_WAKERS[0].wake();
                        } else if let Some(xfer) = out_xfers[i].as_mut() {
                            if i > 0 && !T::dpram().ep_out_control(i - 1).read().enable() {
                                xfer.result = Some(Err(EndpointError::Disabled));
                                EP_OUT_WAKERS[i].wake();
                            } else {
                                let val = T::dpram().ep_out_buffer_control(i).read();
                                if !val.available(0) && !val.stall() && xfer.on_packet::<T>(i, val) {
                                    EP_OUT_WAKERS[i].wake();
                                }
                            }
                        } else {
                            EP_OUT_WAKERS[i].wake();
                        }
                    }
                }
            });
        }

        if ints.dev_sof() {
            // Reading SOF_RD acknowledges DEV_SOF on RP2040/RP235x.
            let now = regs.sof_rd().read().count();
            critical_section::with(|cs| {
                {
                    let mut sof = ISO_SOF.borrow(cs).borrow_mut();
                    sof.last_sof = now;
                    sof.seen = true;
                }
                let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
                for i in 1..EP_COUNT {
                    let state = &mut iso[i];
                    let mut wake = state.arm_wait;
                    if state.armed
                        && forward(u32::from(state.expiry_frame), u32::from(now), ISO_FRAME_SPAN) < ISO_FRAME_SPAN / 2
                    {
                        // Drop an uncollected isochronous packet so it is never sent late.
                        T::dpram().ep_in_buffer_control(i).write(|_| {});
                        state.armed = false;
                        wake = true;
                    }
                    if wake {
                        EP_IN_WAKERS[i].wake();
                    }
                }
            });
        }
    }
}

impl<'d, T: Instance> driver::Driver<'d> for Driver<'d, T> {
    type EndpointOut = Endpoint<'d, T, Out>;
    type EndpointIn = Endpoint<'d, T, In>;
    type Bus = Bus<'d, T>;

    fn alloc_endpoint_in(&mut self, config: &EndpointConfig) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        self.alloc_endpoint(config.ep_type, config.addr, config.max_packet_size, config.interval)
    }

    fn alloc_endpoint_out(&mut self, config: &EndpointConfig) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        self.alloc_endpoint(config.ep_type, config.addr, config.max_packet_size, config.interval)
    }

    fn start(self, control_max_packet_size: u16) -> Self::Bus {
        let regs = T::regs();
        regs.inte().write(|w| {
            w.set_bus_reset(true);
            w.set_buff_status(true);
            w.set_dev_resume_from_host(true);
            w.set_dev_suspend(true);
            w.set_setup_req(true);
        });
        regs.int_ep_ctrl().write(|w| {
            w.set_int_ep_active(0xFFFE); // all EPs
        });
        regs.sie_ctrl().write(|w| {
            w.set_ep0_int_1buf(true);
            w.set_pullup_en(true);
        });

        trace!("enabled");

        Bus {
            phantom: PhantomData,
            inited: false,
            ep_out: self.ep_out,
            control_max_packet_size,
        }
    }
}

/// Enables the bus event interrupts that `on_interrupt` masks when they fire, and the SETUP one.
fn rearm_bus_irqs<T: Instance>() {
    T::regs().inte().write_set(|w| {
        w.set_bus_reset(true);
        w.set_dev_resume_from_host(true);
        w.set_dev_suspend(true);
        w.set_setup_req(true);
    });
}

/// Set or clear the STALL handshake of an endpoint.
///
/// `max_packet_size` is the max packet size of the endpoint, used to re-arm the buffer
/// of an OUT endpoint when the stall is cleared. It is ignored for IN endpoints.
///
/// Shared by `Bus::endpoint_set_stalled` and `Endpoint::set_stalled`, so that a class
/// driver can halt its own endpoint without access to the bus.
fn set_endpoint_stalled<T: Instance>(ep_addr: EndpointAddress, max_packet_size: u16, stalled: bool) {
    let n = ep_addr.index();

    if n == 0 {
        T::regs().ep_stall_arm().modify(|w| {
            if ep_addr.is_in() {
                w.set_ep0_in(stalled);
            } else {
                w.set_ep0_out(stalled);
            }
        });
    }

    let ctrl = if ep_addr.is_in() {
        T::dpram().ep_in_buffer_control(n)
    } else {
        T::dpram().ep_out_buffer_control(n)
    };

    match (stalled, ep_addr.direction()) {
        // write, not modify: clears AVAILABLE so an in-flight packet can't complete instead of stalling.
        (true, _) => ctrl.write(|w| w.set_stall(true)),

        // the control pipe resets EP0's toggle on every SETUP, so only drop the stall.
        (false, _) if n == 0 => ctrl.modify(|w| w.set_stall(false)),

        // clearing a halt resets the toggle to DATA0 (USB 2.0 §9.4.5), but PID is flipped before use.
        (false, Direction::In) => ctrl.write(|w| w.set_pid(0, true)),

        // same, plus re-arm the buffer that stalling un-armed.
        (false, Direction::Out) => arm_ep_out_buffer::<T>(n, false, max_packet_size),
    }

    let wakers = if ep_addr.is_in() { &EP_IN_WAKERS } else { &EP_OUT_WAKERS };
    wakers[n].wake();
}

/// Get whether the STALL handshake is set on an endpoint.
fn is_endpoint_stalled<T: Instance>(ep_addr: EndpointAddress) -> bool {
    let n = ep_addr.index();

    let ctrl = if ep_addr.is_in() {
        T::dpram().ep_in_buffer_control(n)
    } else {
        T::dpram().ep_out_buffer_control(n)
    };

    ctrl.read().stall()
}

/// Type representing the RP USB bus.
pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_out: [EndpointData; EP_COUNT],
    inited: bool,
    control_max_packet_size: u16,
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            BUS_WAKER.register(cx.waker());
            EP_OUT_WAKERS[0].register(cx.waker());

            // TODO: implement VBUS detection.
            if !self.inited {
                self.inited = true;
                return Poll::Ready(Event::PowerDetected);
            }

            let regs = T::regs();
            let siestatus = regs.sie_status().read();
            let intrstatus = regs.intr().read();

            if siestatus.resume() || intrstatus.dev_resume_from_host() {
                regs.sie_status().write(|w| w.set_resume(true));
                return Poll::Ready(Event::Resume);
            }

            if siestatus.bus_reset() {
                regs.sie_status().write(|w| {
                    w.set_bus_reset(true);
                    w.set_setup_rec(true);
                    // clear a suspend latched before the reset, else the next poll reports a
                    // spurious Suspend and embassy-usb waits for a resume that never comes.
                    w.set_suspended(true);
                });
                regs.buff_status().write(|w| w.0 = 0xFFFF_FFFF);
                regs.addr_endp().write(|w| w.set_address(0));

                for i in 1..EP_COUNT {
                    T::dpram().ep_in_control(i - 1).modify(|w| w.set_enable(false));
                    T::dpram().ep_out_control(i - 1).modify(|w| w.set_enable(false));
                }

                critical_section::with(|cs| {
                    let mut in_xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                    for xfer in in_xfers.iter_mut().skip(1).flatten() {
                        if !xfer.done && xfer.error.is_none() {
                            xfer.error = Some(EndpointDisabledError);
                        }
                    }
                    let mut out_xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();
                    for xfer in out_xfers.iter_mut().skip(1).flatten() {
                        if xfer.result.is_none() {
                            xfer.result = Some(Err(EndpointError::Disabled));
                        }
                    }
                });

                for w in &EP_IN_WAKERS {
                    w.wake()
                }
                for w in &EP_OUT_WAKERS {
                    w.wake()
                }
                return Poll::Ready(Event::Reset(Speed::Full));
            }

            if siestatus.suspended() && intrstatus.dev_suspend() {
                regs.sie_status().write(|w| w.set_suspended(true));
                return Poll::Ready(Event::Suspend);
            }

            // Bus events come first: a reset or suspend makes a SETUP received before it stale.
            if siestatus.setup_rec() {
                let mut buf = [0; 8];
                EndpointBuffer::<T>::new(0, 8).read(&mut buf);
                regs.sie_status().write(|w| w.set_setup_rec(true));

                // set PID to 0, so (after toggling) first DATA is PID 1
                T::dpram().ep_in_buffer_control(0).write(|w| w.set_pid(0, false));
                T::dpram().ep_out_buffer_control(0).write(|w| w.set_pid(0, false));

                trace!("SETUP read ok");
                // The control request that follows does not poll the bus, so a reset or suspend during
                // it must wake the stage through its interrupt. Arm them here too: only the path below
                // used to, because the control pipe was polled after the bus.
                rearm_bus_irqs::<T>();
                return Poll::Ready(Event::Setup(buf));
            }

            // no pending event. Reenable all irqs, including the one for SETUP.
            rearm_bus_irqs::<T>();
            Poll::Pending
        })
        .await
    }

    async fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        set_endpoint_stalled::<T>(ep_addr, self.ep_out[ep_addr.index()].max_packet_size, stalled);
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        is_endpoint_stalled::<T>(ep_addr)
    }

    async fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:?} {}", ep_addr, enabled);
        if ep_addr.index() == 0 {
            return;
        }

        let n = ep_addr.index();
        match ep_addr.direction() {
            Direction::In => {
                critical_section::with(|cs| {
                    T::dpram().ep_in_control(n - 1).modify(|w| w.set_enable(enabled));
                    T::dpram().ep_in_buffer_control(n).write(|w| {
                        w.set_pid(0, true); // first packet is DATA0, but PID is flipped before
                    });
                    if let Some(xfer) = EP_IN_XFERS.borrow(cs).borrow_mut()[n].as_mut() {
                        if !xfer.done && xfer.error.is_none() {
                            xfer.error = Some(EndpointDisabledError);
                        }
                    }
                    let is_iso = T::dpram().ep_in_control(n - 1).read().endpoint_type()
                        == pac::usb_dpram::vals::EpControlEndpointType::Isochronous;
                    {
                        let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
                        iso[n].reset_state();
                        iso[n].enabled = enabled && is_iso;
                    }
                    update_dev_sof_irq::<T>(cs);
                });
                EP_IN_WAKERS[n].wake();
            }
            Direction::Out => {
                critical_section::with(|cs| {
                    T::dpram().ep_out_control(n - 1).modify(|w| w.set_enable(enabled));
                    arm_ep_out_buffer::<T>(n, false, self.ep_out[n].max_packet_size);
                    if let Some(xfer) = EP_OUT_XFERS.borrow(cs).borrow_mut()[n].as_mut() {
                        if xfer.result.is_none() {
                            xfer.result = Some(Err(EndpointError::Disabled));
                        }
                    }
                });
                EP_OUT_WAKERS[n].wake();
            }
        }
    }

    fn endpoint_is_enabled(&mut self, ep_addr: EndpointAddress) -> bool {
        let n = ep_addr.index();
        if n == 0 {
            return true;
        }
        match ep_addr.direction() {
            Direction::In => T::dpram().ep_in_control(n - 1).read().enable(),
            Direction::Out => T::dpram().ep_out_control(n - 1).read().enable(),
        }
    }

    async fn enable(&mut self) {}

    async fn disable(&mut self) {}

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        // SIE_CTRL.RESUME ("Device: Remote wakeup. Device can initiate its own
        // resume after suspend") is self-clearing per the RP2040 datasheet
        // (pico-sdk: USB_SIE_CTRL_RESUME access type "SC"): the controller
        // drives the resume signaling on the bus by itself, so a single write
        // is sufficient.
        //
        // This returns once signaling is initiated rather than waiting for it
        // to finish. That is safe because a full-speed device only transmits
        // in response to host tokens: endpoints armed while the bus is still
        // resuming simply wait until the host restarts polling. Callers must
        // only invoke this while suspended with remote wakeup enabled by the
        // host, which `embassy-usb`'s `UsbDevice::remote_wakeup` guarantees;
        // calling it in any other bus state is unsupported.
        T::regs().sie_ctrl().modify(|w| w.set_resume(true));
        Ok(())
    }

    fn control_max_packet_size(&self) -> usize {
        64
    }

    async fn control_data_out(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        trace!("control: data_out len={}", buf.len());
        let max_packet_size = self.control_max_packet_size.min(64);
        let immediate = critical_section::with(|cs| {
            if control_aborted::<T>() {
                trace!("control: data_out aborted");
                return Some(Err(EndpointError::Disabled));
            }
            let pid = !T::dpram().ep_out_buffer_control(0).read().pid(0);
            arm_ep_out_buffer::<T>(0, pid, max_packet_size);
            EP_OUT_XFERS.borrow(cs).borrow_mut()[0] = Some(OutXfer::new(buf, max_packet_size, 0x100, 64, false));
            None
        });
        if let Some(res) = immediate {
            return res;
        }

        let _guard = OutXferGuard(0);
        poll_fn(|cx| {
            register_ep0_wakers(cx);
            if control_aborted::<T>() {
                trace!("control: data_out aborted");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            critical_section::with(|cs| {
                let mut xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();
                let xfer = xfers[0].as_mut().unwrap();
                if let Some(res) = xfer.result {
                    xfers[0] = None;
                    return Poll::Ready(res);
                }
                Poll::Pending
            })
        })
        .await
    }

    async fn control_data_in(&mut self, data: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        trace!("control: data_in len={} zlp={}", data.len(), needs_zlp);

        let max_packet_size = (self.control_max_packet_size as usize).min(64);
        let first_len = data.len().min(max_packet_size);
        let send_zlp = needs_zlp && !data.is_empty() && data.len().is_multiple_of(max_packet_size);

        let aborted = critical_section::with(|cs| {
            if control_aborted::<T>() {
                trace!("control: data_in aborted");
                return true;
            }
            EndpointBuffer::<T>::new(0x100, 64).write(&data[..first_len]);
            let pid = !T::dpram().ep_in_buffer_control(0).read().pid(0);
            arm_ep_in_buffer::<T>(0, pid, first_len as u16);

            EP_IN_XFERS.borrow(cs).borrow_mut()[0] = Some(InXfer {
                buf_ptr: data.as_ptr(),
                buf_len: data.len(),
                offset: first_len,
                send_zlp,
                max_packet_size: max_packet_size as u16,
                dpram_addr: 0x100,
                dpram_len: 64,
                wait_last_complete: true,
                done: false,
                error: None,
            });
            false
        });
        if aborted {
            return Err(EndpointDisabledError);
        }

        {
            let _guard = InXferGuard(0);
            poll_fn(|cx| {
                register_ep0_wakers(cx);
                if control_aborted::<T>() {
                    trace!("control: data_in aborted");
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                critical_section::with(|cs| {
                    let mut xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                    let xfer = xfers[0].as_mut().unwrap();
                    if let Some(err) = xfer.error {
                        xfers[0] = None;
                        return Poll::Ready(Err(err));
                    }
                    if xfer.done {
                        xfers[0] = None;
                        return Poll::Ready(Ok(()));
                    }
                    Poll::Pending
                })
            })
            .await?;
        }

        // prepare status phase right away.
        arm_ep_out_buffer::<T>(0, true, 0);

        Ok(())
    }

    async fn control_accept(&mut self) {
        trace!("control: accept");

        arm_ep_in_buffer::<T>(0, true, 0);
        let bufcontrol = T::dpram().ep_in_buffer_control(0);

        // wait for completion before returning, needed so
        // set_address() doesn't happen early.
        poll_fn(|cx| {
            register_ep0_wakers(cx);
            // accept has no error channel, returning early is enough.
            if control_aborted::<T>() {
                trace!("control: accept aborted");
                return Poll::Ready(());
            }
            if bufcontrol.read().available(0) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    async fn control_reject(&mut self) {
        trace!("control: reject");

        let regs = T::regs();
        regs.ep_stall_arm().write_set(|w| {
            w.set_ep0_in(true);
            w.set_ep0_out(true);
        });
        T::dpram().ep_out_buffer_control(0).write(|w| w.set_stall(true));
        T::dpram().ep_in_buffer_control(0).write(|w| w.set_stall(true));
    }

    async fn control_accept_set_address(&mut self, addr: u8) {
        self.control_accept().await;

        let regs = T::regs();
        trace!("setting addr: {}", addr);
        regs.addr_endp().write(|w| w.set_address(addr))
    }
}

/// Endpoint for RP USB driver.
pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    buf: EndpointBuffer<T>,
    schedule: IsoSchedule,
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, In> {
    fn info(&self) -> EndpointInfo {
        self.info
    }

    async fn wait_enabled(&mut self) {
        trace!("wait_enabled IN WAITING");
        let index = self.info.addr.index();
        poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            let val = T::dpram().ep_in_control(self.info.addr.index() - 1).read();
            if val.enable() { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;
        trace!("wait_enabled IN OK");
    }

    fn is_enabled(&self) -> bool {
        let index = self.info.addr.index();
        if index == 0 {
            return true;
        }
        T::dpram().ep_in_control(index - 1).read().enable()
    }

    async fn set_stalled(&mut self, stalled: bool) {
        set_endpoint_stalled::<T>(self.info.addr, self.info.max_packet_size, stalled);
    }

    fn is_stalled(&self) -> bool {
        is_endpoint_stalled::<T>(self.info.addr)
    }
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, Out> {
    fn info(&self) -> EndpointInfo {
        self.info
    }

    async fn wait_enabled(&mut self) {
        trace!("wait_enabled OUT WAITING");
        let index = self.info.addr.index();
        poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            let val = T::dpram().ep_out_control(self.info.addr.index() - 1).read();
            if val.enable() { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;
        trace!("wait_enabled OUT OK");
    }

    fn is_enabled(&self) -> bool {
        let index = self.info.addr.index();
        if index == 0 {
            return true;
        }
        T::dpram().ep_out_control(index - 1).read().enable()
    }

    async fn set_stalled(&mut self, stalled: bool) {
        set_endpoint_stalled::<T>(self.info.addr, self.info.max_packet_size, stalled);
    }

    fn is_stalled(&self) -> bool {
        is_endpoint_stalled::<T>(self.info.addr)
    }
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        trace!("READ WAITING, buf.len() = {}", buf.len());
        let index = self.info.addr.index();
        let immediate = critical_section::with(|cs| {
            if !T::dpram().ep_out_control(index - 1).read().enable() {
                return Some(Err(EndpointError::Disabled));
            }
            let mut xfer = OutXfer::new(buf, self.info.max_packet_size, self.buf.addr, self.buf.len, true);
            let val = T::dpram().ep_out_buffer_control(index).read();
            // stay parked while stalled, otherwise the re-arm would clear the stall.
            if !val.available(0) && !val.stall() {
                xfer.on_packet::<T>(index, val);
                if let Some(res) = xfer.result {
                    return Some(res);
                }
            }
            EP_OUT_XFERS.borrow(cs).borrow_mut()[index] = Some(xfer);
            None
        });
        if let Some(res) = immediate {
            return res;
        }

        let _guard = OutXferGuard(index);
        poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            critical_section::with(|cs| {
                let mut xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();
                let xfer = xfers[index].as_mut().unwrap();
                if let Some(res) = xfer.result {
                    xfers[index] = None;
                    return Poll::Ready(res);
                }
                if !T::dpram().ep_out_control(index - 1).read().enable() {
                    xfers[index] = None;
                    return Poll::Ready(Err(EndpointError::Disabled));
                }
                Poll::Pending
            })
        })
        .await
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write_transfer(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        trace!("WRITE WAITING, buf.len() = {}, zlp = {}", buf.len(), needs_zlp);
        let index = self.info.addr.index();
        let max_packet_size = self.info.max_packet_size as usize;

        let needs_wait = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            critical_section::with(|cs| {
                if !T::dpram().ep_in_control(index - 1).read().enable() {
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                let val = T::dpram().ep_in_buffer_control(index).read();
                // stay parked while stalled, otherwise the write below would clear the stall.
                if val.available(0) || val.stall() {
                    return Poll::Pending;
                }

                let first_len = buf.len().min(max_packet_size);
                self.buf.write(&buf[..first_len]);
                arm_ep_in_buffer::<T>(index, !val.pid(0), first_len as u16);

                let send_zlp = needs_zlp && !buf.is_empty() && buf.len().is_multiple_of(max_packet_size);
                if first_len == buf.len() && !send_zlp {
                    return Poll::Ready(Ok(false));
                }

                EP_IN_XFERS.borrow(cs).borrow_mut()[index] = Some(InXfer {
                    buf_ptr: buf.as_ptr(),
                    buf_len: buf.len(),
                    offset: first_len,
                    send_zlp,
                    max_packet_size: self.info.max_packet_size,
                    dpram_addr: self.buf.addr,
                    dpram_len: self.buf.len,
                    wait_last_complete: false,
                    done: false,
                    error: None,
                });
                Poll::Ready(Ok(true))
            })
        })
        .await?;

        if !needs_wait {
            trace!("WRITE OK");
            return Ok(());
        }

        let _guard = InXferGuard(index);
        poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            critical_section::with(|cs| {
                let mut xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                let xfer = xfers[index].as_mut().unwrap();
                if let Some(err) = xfer.error {
                    xfers[index] = None;
                    return Poll::Ready(Err(err));
                }
                if !T::dpram().ep_in_control(index - 1).read().enable() {
                    xfers[index] = None;
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                if xfer.done {
                    xfers[index] = None;
                    trace!("WRITE OK");
                    return Poll::Ready(Ok(()));
                }
                let val = T::dpram().ep_in_buffer_control(index).read();
                if !val.available(0) && !val.stall() && xfer.on_packet::<T>(index, val) {
                    xfers[index] = None;
                    trace!("WRITE OK");
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending
            })
        })
        .await
    }
}

/// Reserved isochronous IN slot returned by [`IsoEndpointIn::ready`].
pub struct IsoSlot<'a, 'd, T: Instance> {
    ep: &'a mut Endpoint<'d, T, In>,
    frame: u32,
    skipped: Option<NonZeroU32>,
}

impl<T: Instance> IsoInSlot for IsoSlot<'_, '_, T> {
    fn skipped(&self) -> Option<NonZeroU32> {
        self.skipped
    }

    fn write(self, data: &[u8]) -> Result<(), IsoWriteError> {
        let index = self.ep.info.addr.index();
        if data.len() > usize::from(self.ep.info.max_packet_size) {
            return Err(IsoWriteError::BufferOverflow);
        }

        let interval = self.ep.iso_interval();
        let res = critical_section::with(|cs| {
            if !T::dpram().ep_in_control(index - 1).read().enable() {
                return Err(IsoWriteError::Disabled);
            }
            let sof = ISO_SOF.borrow(cs).borrow();
            if !sof.seen || forward(u32::from(sof.last_sof), self.frame, ISO_FRAME_SPAN) != 1 {
                return Err(IsoWriteError::TooLate);
            }
            drop(sof);

            self.ep.buf.write(data);
            // Full-speed isochronous IN packets always use DATA0 (USB 2.0 §5.6.4).
            arm_ep_in_buffer::<T>(index, false, data.len() as u16);

            let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
            let state = &mut iso[index];
            state.armed = true;
            state.interval = interval as u16;
            // Before phase lock for interval > 1, keep the packet armed across the whole interval
            // so the host's first IN token locks the phase; afterward drop at the end of the slot.
            let window = if state.phase.is_some() || interval == 1 {
                1
            } else {
                interval
            };
            state.expiry_frame = ((self.frame + window) % ISO_FRAME_SPAN) as u16;
            drop(iso);
            update_dev_sof_irq::<T>(cs);
            Ok(())
        });

        res?;
        self.ep.schedule.accept();
        Ok(())
    }
}

struct IsoArmWaitGuard<T: Instance> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T: Instance> Drop for IsoArmWaitGuard<T> {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            ISO_IN_STATE.borrow(cs).borrow_mut()[self.index].arm_wait = false;
            update_dev_sof_irq::<T>(cs);
        });
    }
}

impl<'d, T: Instance> Endpoint<'d, T, In> {
    fn iso_interval(&self) -> u32 {
        self.info.interval.frames().clamp(1, ISO_FRAME_SPAN / 2)
    }
}

impl<'d, T: Instance> IsoEndpointIn for Endpoint<'d, T, In> {
    type Slot<'a>
        = IsoSlot<'a, 'd, T>
    where
        Self: 'a;

    fn queue_depth(&self) -> u8 {
        if self.info.ep_type == EndpointType::Isochronous {
            1
        } else {
            0
        }
    }

    async fn ready(&mut self) -> Result<Self::Slot<'_>, EndpointDisabledError> {
        assert_eq!(
            self.info.ep_type,
            EndpointType::Isochronous,
            "IsoEndpointIn requires an isochronous endpoint"
        );
        let index = self.info.addr.index();
        assert!(index != 0);

        let interval = self.iso_interval();
        let _guard = IsoArmWaitGuard::<T> {
            index,
            _phantom: PhantomData,
        };

        // Wait until any previous armed packet has been sent or dropped and SOF is live.
        let now = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            critical_section::with(|cs| {
                if !T::dpram().ep_in_control(index - 1).read().enable() {
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
                let sof = ISO_SOF.borrow(cs).borrow();
                if !iso[index].armed && sof.seen {
                    iso[index].arm_wait = false;
                    Poll::Ready(Ok(u32::from(sof.last_sof)))
                } else {
                    iso[index].arm_wait = true;
                    drop(sof);
                    drop(iso);
                    update_dev_sof_irq::<T>(cs);
                    Poll::Pending
                }
            })
        })
        .await?;

        critical_section::with(|cs| {
            let mut iso = ISO_IN_STATE.borrow(cs).borrow_mut();
            if iso[index].reset {
                iso[index].reset = false;
                self.schedule.reset();
            }
            if let Some(phase) = iso[index].phase {
                self.schedule.lock_phase(u32::from(phase), interval);
            }
        });

        let mut total_skipped: u32 = 0;
        let mut current = now;
        loop {
            let offer = self.schedule.offer(current, interval, ISO_FRAME_SPAN);
            total_skipped = total_skipped.saturating_add(offer.skipped.map_or(0, NonZeroU32::get));
            let frame = offer.frame;

            let ready = poll_fn(|cx| {
                EP_IN_WAKERS[index].register(cx.waker());
                critical_section::with(|cs| {
                    if !T::dpram().ep_in_control(index - 1).read().enable() {
                        return Poll::Ready(Err(EndpointDisabledError));
                    }
                    let sof = ISO_SOF.borrow(cs).borrow();
                    if !sof.seen {
                        ISO_IN_STATE.borrow(cs).borrow_mut()[index].arm_wait = true;
                        drop(sof);
                        update_dev_sof_irq::<T>(cs);
                        return Poll::Pending;
                    }
                    let now = u32::from(sof.last_sof);
                    let ahead = forward(now, frame, ISO_FRAME_SPAN);
                    if ahead == 1 {
                        ISO_IN_STATE.borrow(cs).borrow_mut()[index].arm_wait = false;
                        Poll::Ready(Ok(Some(now)))
                    } else if ahead == 0 || ahead > interval {
                        ISO_IN_STATE.borrow(cs).borrow_mut()[index].arm_wait = false;
                        Poll::Ready(Ok(None))
                    } else {
                        ISO_IN_STATE.borrow(cs).borrow_mut()[index].arm_wait = true;
                        drop(sof);
                        update_dev_sof_irq::<T>(cs);
                        Poll::Pending
                    }
                })
            })
            .await?;

            if ready.is_some() {
                return Ok(IsoSlot {
                    ep: self,
                    frame,
                    skipped: NonZeroU32::new(total_skipped),
                });
            }

            current = critical_section::with(|cs| u32::from(ISO_SOF.borrow(cs).borrow().last_sof));
        }
    }
}

/// Tells if the transfer in progress can no longer complete, because the host abandoned it with a
/// new SETUP, reset the bus, or the bus got suspended. Without this the stage waits forever, wedging
/// the whole USB task.
fn control_aborted<T: Instance>() -> bool {
    // on_interrupt masks setup_req when it fires, and only Bus::poll re-arms it.
    T::regs().inte().write_set(|w| w.set_setup_req(true));
    let status = T::regs().sie_status().read();
    let intr = T::regs().intr().read();
    // embassy-usb holds the bus across a control transfer, so it can't spot these itself.
    // Leave the flags set, Bus::poll consumes all of them.
    status.setup_rec() || status.bus_reset() || (status.suspended() && intr.dev_suspend())
}

/// Wake on the transfer completing, a new SETUP, or a bus reset.
fn register_ep0_wakers(cx: &mut core::task::Context) {
    EP_IN_WAKERS[0].register(cx.waker());
    EP_OUT_WAKERS[0].register(cx.waker());
    BUS_WAKER.register(cx.waker());
}
