use core::future::poll_fn;
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use super::{Dir, In, Instance, Out};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::{Peri, RegExt, interrupt, pac};

const EP_COUNT: usize = 16;
const EP_MEMORY_SIZE: usize = 4096;
const EP_MEMORY: *mut u8 = pac::USB_DPRAM.as_ptr() as *mut u8;

/// Update one 16-bit half of an RP USB buffer-control register.
///
/// The RP2040 USB controller owns the two halves independently. A 32-bit
/// read-modify-write can overwrite the state of the other buffer. The fields
/// in `f` are relative to the selected half, so use buffer index 0 there.
fn update_buffer_half<T: Instance>(
    direction: Direction,
    endpoint: usize,
    buffer: usize,
    f: impl FnOnce(&mut pac::usb_dpram::regs::EpBufferControl),
) {
    let reg = match direction {
        Direction::In => T::dpram().ep_in_buffer_control(endpoint),
        Direction::Out => T::dpram().ep_out_buffer_control(endpoint),
    };
    let ptr = reg.as_ptr() as *mut u16;
    // SAFETY: `endpoint` is valid and `buffer` is 0 or 1; the register has two
    // independently owned, aligned 16-bit halves. Use the PAC's 16-bit accessor.
    let half: pac::common::Reg<u16, pac::common::RW> = unsafe { pac::common::Reg::from_ptr(ptr.add(buffer)) };
    half.modify(|w| {
        let mut value = pac::usb_dpram::regs::EpBufferControl(*w as u32);
        f(&mut value);
        *w = value.0 as u16;
    });
}

/// Reset a double-buffered IN endpoint to buffer 0 and DATA0, dropping any
/// queued packets, and tell its `Endpoint` to reset its buffer selection too.
fn reset_double_buffered_in<T: Instance>(index: usize) {
    critical_section::with(|_| {
        update_buffer_half::<T>(Direction::In, index, 0, |w| {
            w.0 = 0;
            w.set_reset(true);
        });
        update_buffer_half::<T>(Direction::In, index, 1, |w| w.0 = 0);
        // No atomic RMW on thumbv6m; serialize with the endpoint's packet handoff.
        let generation = &EP_IN_RESET_GENERATION[index];
        generation.store(generation.load(Ordering::Relaxed).wrapping_add(1), Ordering::Release);
    });
}

/// Reset OUT to buffer 0/DATA0. Each buffer keeps its PID when re-armed:
/// buffer 0 receives DATA0 and buffer 1 receives DATA1, in alternation.
fn reset_double_buffered_out<T: Instance>(index: usize, max_packet_size: u16, armed: bool) {
    critical_section::with(|_| {
        for buffer in 0..2 {
            update_buffer_half::<T>(Direction::Out, index, buffer, |w| {
                w.0 = 0;
                w.set_reset(buffer == 0);
                w.set_pid(0, buffer == 1);
                w.set_length(0, max_packet_size);
            });
        }
        if armed {
            cortex_m::asm::delay(12);
            for buffer in 0..2 {
                update_buffer_half::<T>(Direction::Out, index, buffer, |w| w.set_available(0, true));
            }
        }
        let generation = &EP_OUT_RESET_GENERATION[index];
        generation.store(generation.load(Ordering::Relaxed).wrapping_add(1), Ordering::Release);
    });
}

static BUS_WAKER: AtomicWaker = AtomicWaker::new();
// Bumped by `reset_double_buffered_{in,out}`, so the endpoint notices the hardware reset.
static EP_IN_RESET_GENERATION: [AtomicU32; EP_COUNT] = [const { AtomicU32::new(0) }; EP_COUNT];
static EP_OUT_RESET_GENERATION: [AtomicU32; EP_COUNT] = [const { AtomicU32::new(0) }; EP_COUNT];
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [const { AtomicWaker::new() }; EP_COUNT];

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

    /// Hardware buffer `n` of a double-buffered endpoint whose first buffer is `self`.
    /// The second buffer is always at +64.
    fn double_buffer_half(&self, n: usize) -> Self {
        Self::new(self.addr + 64 * n as u16, self.len)
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
    double_in: u16,
    double_out: u16,
    ep_mem_free: u16, // first free address in EP mem, in bytes.
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Create a new USB driver.
    pub fn new(_usb: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>>) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let regs = T::regs();
        unsafe {
            // zero fill regs
            let p = regs.as_ptr() as *mut u32;
            for i in 0..0x9c / 4 {
                p.add(i).write_volatile(0)
            }

            // zero fill epmem
            let p = EP_MEMORY as *mut u32;
            for i in 0..0x100 / 4 {
                p.add(i).write_volatile(0)
            }
        }

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
            double_in: 0,
            double_out: 0,
            ep_mem_free: 0x180, // data buffer region
        }
    }

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
        double_buffered: bool,
    ) -> Result<Endpoint<'d, T, D>, driver::EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval_ms={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval_ms,
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

        if double_buffered && (ep_type != EndpointType::Bulk || !matches!(max_packet_size, 8 | 16 | 32 | 64)) {
            return Err(EndpointAllocError);
        }

        // ep mem addrs must be 64-byte aligned, so there's no point in trying
        // to allocate smaller chunks to save memory. The second buffer of a
        // double-buffered endpoint is always at +64.
        let len = max_packet_size.div_ceil(64) * 64;
        let total_len = if double_buffered { 128 } else { len };

        let addr = self.ep_mem_free;
        if addr + total_len > EP_MEMORY_SIZE as u16 {
            warn!("Endpoint memory full");
            return Err(EndpointAllocError);
        }
        self.ep_mem_free += total_len;

        let buf = EndpointBuffer {
            addr,
            len,
            _phantom: PhantomData,
        };

        trace!("  index={} addr={} len={}", index, buf.addr, buf.len);

        ep.ep_type = ep_type;
        ep.used = true;
        ep.max_packet_size = max_packet_size;
        if double_buffered {
            match D::dir() {
                Direction::In => self.double_in |= 1 << index,
                Direction::Out => self.double_out |= 1 << index,
            }
        }

        let ep_type_reg = match ep_type {
            EndpointType::Bulk => pac::usb_dpram::vals::EpControlEndpointType::Bulk,
            EndpointType::Control => pac::usb_dpram::vals::EpControlEndpointType::Control,
            EndpointType::Interrupt => pac::usb_dpram::vals::EpControlEndpointType::Interrupt,
            EndpointType::Isochronous => pac::usb_dpram::vals::EpControlEndpointType::Isochronous,
        };

        match D::dir() {
            Direction::Out => {
                T::dpram().ep_out_control(index - 1).write(|w| {
                    w.set_enable(false);
                    w.set_buffer_address(addr);
                    w.set_interrupt_per_buff(true);
                    w.set_double_buffered(double_buffered);
                    w.set_endpoint_type(ep_type_reg);
                });
                if double_buffered {
                    reset_double_buffered_out::<T>(index, max_packet_size, false);
                }
            }
            Direction::In => {
                T::dpram().ep_in_control(index - 1).write(|w| {
                    w.set_enable(false);
                    w.set_buffer_address(addr);
                    w.set_interrupt_per_buff(true);
                    w.set_double_buffered(double_buffered);
                    w.set_endpoint_type(ep_type_reg);
                });
                if double_buffered {
                    reset_double_buffered_in::<T>(index);
                }
            }
        }

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
            buf,
            double_buffer: double_buffered.then(|| DoubleBuffer {
                next_buf: 0,
                next_pid: false,
                reset_generation: match D::dir() {
                    Direction::In => EP_IN_RESET_GENERATION[index].load(Ordering::Relaxed),
                    Direction::Out => EP_OUT_RESET_GENERATION[index].load(Ordering::Relaxed),
                },
            }),
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

            for i in 0..EP_COUNT {
                if s.ep_in(i) {
                    EP_IN_WAKERS[i].wake();
                }
                if s.ep_out(i) {
                    EP_OUT_WAKERS[i].wake();
                }
            }
        }
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
    ) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms, false)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms, false)
    }

    fn alloc_endpoint_bulk_out_double_buffered(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.alloc_endpoint(EndpointType::Bulk, ep_addr, max_packet_size, 0, true)
    }

    fn alloc_endpoint_bulk_in_double_buffered(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint(EndpointType::Bulk, ep_addr, max_packet_size, 0, true)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
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

        (
            Bus {
                phantom: PhantomData,
                inited: false,
                ep_out: self.ep_out,
                double_in: self.double_in,
                double_out: self.double_out,
            },
            ControlPipe {
                _phantom: PhantomData,
                max_packet_size: control_max_packet_size,
            },
        )
    }
}

/// Type representing the RP USB bus.
pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_out: [EndpointData; EP_COUNT],
    double_in: u16,
    double_out: u16,
    inited: bool,
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            BUS_WAKER.register(cx.waker());

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
                critical_section::with(|_| {
                    // Disable and flush before consuming the reset flag, so a
                    // preempted packet handoff cannot resume in the old epoch.
                    for i in 1..EP_COUNT {
                        if self.double_in & (1 << i) != 0 {
                            T::dpram().ep_in_control(i - 1).modify(|w| w.set_enable(false));
                            reset_double_buffered_in::<T>(i);
                        }
                        if self.double_out & (1 << i) != 0 {
                            T::dpram().ep_out_control(i - 1).modify(|w| w.set_enable(false));
                            reset_double_buffered_out::<T>(i, self.ep_out[i].max_packet_size, false);
                        }
                    }
                });
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

                for w in &EP_IN_WAKERS {
                    w.wake()
                }
                for w in &EP_OUT_WAKERS {
                    w.wake()
                }
                return Poll::Ready(Event::Reset);
            }

            if siestatus.suspended() && intrstatus.dev_suspend() {
                regs.sie_status().write(|w| w.set_suspended(true));
                return Poll::Ready(Event::Suspend);
            }

            // no pending event. Reenable all irqs.
            regs.inte().write_set(|w| {
                w.set_bus_reset(true);
                w.set_dev_resume_from_host(true);
                w.set_dev_suspend(true);
            });
            Poll::Pending
        })
        .await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
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

        let double_buffered = (if ep_addr.is_in() {
            self.double_in
        } else {
            self.double_out
        }) & (1 << n)
            != 0;

        match (stalled, ep_addr.direction()) {
            (true, _) if double_buffered => critical_section::with(|_| ctrl.write(|w| w.set_stall(true))),
            // write, not modify: clears AVAILABLE so an in-flight packet can't complete instead of stalling.
            (true, _) => ctrl.write(|w| w.set_stall(true)),

            // the control pipe resets EP0's toggle on every SETUP, so only drop the stall.
            (false, _) if n == 0 => ctrl.modify(|w| w.set_stall(false)),

            // clearing a halt resets the toggle to DATA0 (USB 2.0 §9.4.5).
            (false, Direction::In) if double_buffered => reset_double_buffered_in::<T>(n),

            // same, but PID is flipped before use.
            (false, Direction::In) => ctrl.write(|w| w.set_pid(0, true)),

            (false, Direction::Out) if double_buffered => {
                reset_double_buffered_out::<T>(n, self.ep_out[n].max_packet_size, true);
            }

            // same, plus re-arm the buffer that stalling un-armed.
            (false, Direction::Out) => {
                ctrl.write(|w| {
                    w.set_pid(0, false);
                    w.set_length(0, self.ep_out[n].max_packet_size);
                });
                cortex_m::asm::delay(12);
                ctrl.write(|w| {
                    w.set_pid(0, false);
                    w.set_length(0, self.ep_out[n].max_packet_size);
                    w.set_available(0, true);
                });
            }
        }

        let wakers = if ep_addr.is_in() { &EP_IN_WAKERS } else { &EP_OUT_WAKERS };
        wakers[n].wake();
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let n = ep_addr.index();

        let ctrl = if ep_addr.is_in() {
            T::dpram().ep_in_buffer_control(n)
        } else {
            T::dpram().ep_out_buffer_control(n)
        };

        ctrl.read().stall()
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:?} {}", ep_addr, enabled);
        if ep_addr.index() == 0 {
            return;
        }

        let n = ep_addr.index();
        match ep_addr.direction() {
            Direction::In if self.double_in & (1 << n) != 0 => {
                critical_section::with(|_| {
                    T::dpram().ep_in_control(n - 1).modify(|w| w.set_enable(false));
                    reset_double_buffered_in::<T>(n);
                    T::dpram().ep_in_control(n - 1).modify(|w| w.set_enable(enabled));
                });
                EP_IN_WAKERS[n].wake();
            }
            Direction::In => {
                T::dpram().ep_in_control(n - 1).modify(|w| w.set_enable(enabled));
                T::dpram().ep_in_buffer_control(ep_addr.index()).write(|w| {
                    w.set_pid(0, true); // first packet is DATA0, but PID is flipped before
                });
                EP_IN_WAKERS[n].wake();
            }
            Direction::Out if self.double_out & (1 << n) != 0 => {
                critical_section::with(|_| {
                    T::dpram().ep_out_control(n - 1).modify(|w| w.set_enable(false));
                    reset_double_buffered_out::<T>(n, self.ep_out[n].max_packet_size, enabled);
                    T::dpram().ep_out_control(n - 1).modify(|w| w.set_enable(enabled));
                });
                EP_OUT_WAKERS[n].wake();
            }
            Direction::Out => {
                T::dpram().ep_out_control(n - 1).modify(|w| w.set_enable(enabled));

                T::dpram().ep_out_buffer_control(ep_addr.index()).write(|w| {
                    w.set_pid(0, false);
                    w.set_length(0, self.ep_out[n].max_packet_size);
                });
                cortex_m::asm::delay(12);
                T::dpram().ep_out_buffer_control(ep_addr.index()).write(|w| {
                    w.set_pid(0, false);
                    w.set_length(0, self.ep_out[n].max_packet_size);
                    w.set_available(0, true);
                });
                EP_OUT_WAKERS[n].wake();
            }
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
}

/// Endpoint for RP USB driver.
pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    buf: EndpointBuffer<T>,
    double_buffer: Option<DoubleBuffer>,
}

/// Software state of a double-buffered bulk endpoint.
struct DoubleBuffer {
    /// Hardware buffer (0 or 1) that holds the next packet.
    next_buf: usize,
    /// DATA PID of the next IN packet.
    next_pid: bool,
    /// Last seen value of the endpoint's reset generation.
    reset_generation: u32,
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, In> {
    fn info(&self) -> &EndpointInfo {
        &self.info
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
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, Out> {
    fn info(&self) -> &EndpointInfo {
        &self.info
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
}

impl<'d, T: Instance> Endpoint<'d, T, Out> {
    async fn read_double_buffered(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            // Serialize readiness, packet copying and re-arming with reset/clear-halt.
            critical_section::with(|_| {
                if T::regs().sie_status().read().bus_reset() || !T::dpram().ep_out_control(index - 1).read().enable() {
                    return Poll::Ready(Err(EndpointError::Disabled));
                }
                let Some(state) = self.double_buffer.as_mut() else {
                    unreachable!()
                };
                let generation = EP_OUT_RESET_GENERATION[index].load(Ordering::Acquire);
                if state.reset_generation != generation {
                    state.reset_generation = generation;
                    state.next_buf = 0;
                }
                let buffer = state.next_buf;
                let val = T::dpram().ep_out_buffer_control(index).read();
                if val.stall() || val.available(buffer) || !val.full(buffer) {
                    return Poll::Pending;
                }
                let len = val.length(buffer) as usize;
                if len > buf.len() {
                    return Poll::Ready(Err(EndpointError::BufferOverflow)); // retain the packet
                }
                self.buf.double_buffer_half(buffer).read(&mut buf[..len]);
                update_buffer_half::<T>(Direction::Out, index, buffer, |w| {
                    w.0 = 0;
                    w.set_pid(0, buffer == 1);
                    w.set_length(0, self.info.max_packet_size);
                });
                cortex_m::asm::delay(12);
                update_buffer_half::<T>(Direction::Out, index, buffer, |w| w.set_available(0, true));
                state.next_buf ^= 1;
                Poll::Ready(Ok(len))
            })
        })
        .await
    }
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        if self.double_buffer.is_some() {
            return self.read_double_buffered(buf).await;
        }

        trace!("READ WAITING, buf.len() = {}", buf.len());
        let index = self.info.addr.index();
        let val = poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            let val = T::dpram().ep_out_buffer_control(index).read();
            // stay parked while stalled, otherwise the re-arm below would clear the stall.
            if val.available(0) || val.stall() {
                Poll::Pending
            } else {
                Poll::Ready(val)
            }
        })
        .await;

        let rx_len = val.length(0) as usize;
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        self.buf.read(&mut buf[..rx_len]);

        trace!("READ OK, rx_len = {}", rx_len);

        let pid = !val.pid(0);
        T::dpram().ep_out_buffer_control(index).write(|w| {
            w.set_pid(0, pid);
            w.set_length(0, self.info.max_packet_size);
        });
        cortex_m::asm::delay(12);
        T::dpram().ep_out_buffer_control(index).write(|w| {
            w.set_pid(0, pid);
            w.set_length(0, self.info.max_packet_size);
            w.set_available(0, true);
        });

        Ok(rx_len)
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        if buf.len() > self.info.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }
        if self.double_buffer.is_some() {
            return self.write_double_buffered(buf).await;
        }

        trace!("WRITE WAITING");

        let index = self.info.addr.index();
        let val = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            let val = T::dpram().ep_in_buffer_control(index).read();
            // stay parked while stalled, otherwise the write below would clear the stall.
            if val.available(0) || val.stall() {
                Poll::Pending
            } else {
                Poll::Ready(val)
            }
        })
        .await;

        self.buf.write(buf);

        let pid = !val.pid(0);
        T::dpram().ep_in_buffer_control(index).write(|w| {
            w.set_pid(0, pid);
            w.set_length(0, buf.len() as _);
            w.set_full(0, true);
        });
        cortex_m::asm::delay(12);
        T::dpram().ep_in_buffer_control(index).write(|w| {
            w.set_pid(0, pid);
            w.set_length(0, buf.len() as _);
            w.set_full(0, true);
            w.set_available(0, true);
        });

        trace!("WRITE OK");

        Ok(())
    }
}

impl<'d, T: Instance> Endpoint<'d, T, In> {
    async fn write_double_buffered(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        let index = self.info.addr.index();
        let buffer_index = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            if T::regs().sie_status().read().bus_reset() || !T::dpram().ep_in_control(index - 1).read().enable() {
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let Some(state) = self.double_buffer.as_mut() else {
                unreachable!()
            };
            let generation = EP_IN_RESET_GENERATION[index].load(Ordering::Acquire);
            if state.reset_generation != generation {
                state.reset_generation = generation;
                state.next_buf = 0;
                state.next_pid = false;
            }
            let buffer = state.next_buf;
            let val = T::dpram().ep_in_buffer_control(index).read();
            // stay parked while stalled, otherwise the write below would clear the stall.
            if val.available(buffer) || val.stall() {
                Poll::Pending
            } else {
                Poll::Ready(Ok(buffer))
            }
        })
        .await?;

        critical_section::with(|_| {
            let Some(state) = self.double_buffer.as_mut() else {
                unreachable!()
            };
            if T::regs().sie_status().read().bus_reset()
                || !T::dpram().ep_in_control(index - 1).read().enable()
                || EP_IN_RESET_GENERATION[index].load(Ordering::Acquire) != state.reset_generation
            {
                return Err(EndpointError::Disabled);
            }
            self.buf.double_buffer_half(buffer_index).write(buf);

            let pid = state.next_pid;
            update_buffer_half::<T>(Direction::In, index, buffer_index, |w| {
                w.set_pid(0, pid);
                w.set_length(0, buf.len() as _);
                w.set_full(0, true);
            });
            cortex_m::asm::delay(12);
            update_buffer_half::<T>(Direction::In, index, buffer_index, |w| {
                w.set_pid(0, pid);
                w.set_length(0, buf.len() as _);
                w.set_full(0, true);
                w.set_available(0, true);
            });

            state.next_buf ^= 1;
            state.next_pid = !state.next_pid;
            trace!("WRITE OK");
            Ok(())
        })
    }
}

/// Control pipe for RP USB driver.
pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
}

/// Tells if the transfer in progress can no longer complete, because the host abandoned it with a
/// new SETUP or reset the bus. Without this the stage waits forever, wedging the whole USB task.
fn control_aborted<T: Instance>() -> bool {
    // on_interrupt masks setup_req when it fires, and only setup() re-arms it.
    T::regs().inte().write_set(|w| w.set_setup_req(true));
    let status = T::regs().sie_status().read();
    // embassy-usb holds the bus across a control transfer, so it can't spot a reset itself.
    // leave both flags set, setup() consumes setup_rec and Bus::poll consumes bus_reset.
    status.setup_rec() || status.bus_reset()
}

/// Wake on the transfer completing, a new SETUP, or a bus reset.
fn register_ep0_wakers(cx: &mut core::task::Context) {
    EP_IN_WAKERS[0].register(cx.waker());
    EP_OUT_WAKERS[0].register(cx.waker());
    BUS_WAKER.register(cx.waker());
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        64
    }

    async fn setup(&mut self) -> [u8; 8] {
        trace!("SETUP read waiting");
        let regs = T::regs();
        regs.inte().write_set(|w| w.set_setup_req(true));

        poll_fn(|cx| {
            EP_OUT_WAKERS[0].register(cx.waker());
            let regs = T::regs();
            if regs.sie_status().read().setup_rec() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        let mut buf = [0; 8];
        EndpointBuffer::<T>::new(0, 8).read(&mut buf);

        let regs = T::regs();
        regs.sie_status().write(|w| w.set_setup_rec(true));

        // set PID to 0, so (after toggling) first DATA is PID 1
        T::dpram().ep_in_buffer_control(0).write(|w| w.set_pid(0, false));
        T::dpram().ep_out_buffer_control(0).write(|w| w.set_pid(0, false));

        trace!("SETUP read ok");
        buf
    }

    async fn data_out(&mut self, buf: &mut [u8], first: bool, last: bool) -> Result<usize, EndpointError> {
        let bufcontrol = T::dpram().ep_out_buffer_control(0);
        let pid = !bufcontrol.read().pid(0);
        bufcontrol.write(|w| {
            w.set_length(0, self.max_packet_size);
            w.set_pid(0, pid);
        });
        cortex_m::asm::delay(12);
        bufcontrol.write(|w| {
            w.set_length(0, self.max_packet_size);
            w.set_pid(0, pid);
            w.set_available(0, true);
        });

        trace!("control: data_out len={} first={} last={}", buf.len(), first, last);
        let val = poll_fn(|cx| {
            register_ep0_wakers(cx);
            if control_aborted::<T>() {
                trace!("control: data_out aborted");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let val = T::dpram().ep_out_buffer_control(0).read();
            if val.available(0) {
                Poll::Pending
            } else {
                Poll::Ready(Ok(val))
            }
        })
        .await?;

        let rx_len = val.length(0) as _;
        trace!("control data_out DONE, rx_len = {}", rx_len);

        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        EndpointBuffer::<T>::new(0x100, 64).read(&mut buf[..rx_len]);

        Ok(rx_len)
    }

    async fn data_in(&mut self, data: &[u8], first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in len={} first={} last={}", data.len(), first, last);

        if data.len() > 64 {
            return Err(EndpointError::BufferOverflow);
        }
        EndpointBuffer::<T>::new(0x100, 64).write(data);

        let bufcontrol = T::dpram().ep_in_buffer_control(0);
        let pid = !bufcontrol.read().pid(0);
        bufcontrol.write(|w| {
            w.set_length(0, data.len() as _);
            w.set_pid(0, pid);
            w.set_full(0, true);
        });
        cortex_m::asm::delay(12);
        bufcontrol.write(|w| {
            w.set_length(0, data.len() as _);
            w.set_pid(0, pid);
            w.set_full(0, true);
            w.set_available(0, true);
        });

        poll_fn(|cx| {
            register_ep0_wakers(cx);
            if control_aborted::<T>() {
                trace!("control: data_in aborted");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            let bufcontrol = T::dpram().ep_in_buffer_control(0);
            if bufcontrol.read().available(0) {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
        .await?;
        trace!("control: data_in DONE");

        if last {
            // prepare status phase right away.
            let bufcontrol = T::dpram().ep_out_buffer_control(0);
            bufcontrol.write(|w| {
                w.set_length(0, 0);
                w.set_pid(0, true);
            });
            cortex_m::asm::delay(12);
            bufcontrol.write(|w| {
                w.set_length(0, 0);
                w.set_pid(0, true);
                w.set_available(0, true);
            });
        }

        Ok(())
    }

    async fn accept(&mut self) {
        trace!("control: accept");

        let bufcontrol = T::dpram().ep_in_buffer_control(0);
        bufcontrol.write(|w| {
            w.set_length(0, 0);
            w.set_pid(0, true);
            w.set_full(0, true);
        });
        cortex_m::asm::delay(12);
        bufcontrol.write(|w| {
            w.set_length(0, 0);
            w.set_pid(0, true);
            w.set_full(0, true);
            w.set_available(0, true);
        });

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

    async fn reject(&mut self) {
        trace!("control: reject");

        let regs = T::regs();
        regs.ep_stall_arm().write_set(|w| {
            w.set_ep0_in(true);
            w.set_ep0_out(true);
        });
        T::dpram().ep_out_buffer_control(0).write(|w| w.set_stall(true));
        T::dpram().ep_in_buffer_control(0).write(|w| w.set_stall(true));
    }

    async fn accept_set_address(&mut self, addr: u8) {
        self.accept().await;

        let regs = T::regs();
        trace!("setting addr: {}", addr);
        regs.addr_endp().write(|w| w.set_address(addr))
    }
}
