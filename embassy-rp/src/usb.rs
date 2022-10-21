use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::Ordering;
use core::task::Poll;

use atomic_polyfill::compiler_fence;
use embassy_hal_common::into_ref;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use crate::interrupt::{Interrupt, InterruptExt};
use crate::{pac, peripherals, Peripheral, RegExt};

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::usb::Usb;
        fn dpram() -> crate::pac::usb_dpram::UsbDpram;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

impl crate::usb::sealed::Instance for peripherals::USB {
    fn regs() -> pac::usb::Usb {
        pac::USBCTRL_REGS
    }
    fn dpram() -> crate::pac::usb_dpram::UsbDpram {
        pac::USBCTRL_DPRAM
    }
}

impl crate::usb::Instance for peripherals::USB {
    type Interrupt = crate::interrupt::USBCTRL_IRQ;
}

const EP_COUNT: usize = 16;
const EP_MEMORY_SIZE: usize = 4096;
const EP_MEMORY: *mut u8 = pac::USBCTRL_DPRAM.0;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];

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

pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_in: [EndpointData; EP_COUNT],
    ep_out: [EndpointData; EP_COUNT],
    ep_mem_free: u16, // first free address in EP mem, in bytes.
}

impl<'d, T: Instance> Driver<'d, T> {
    pub fn new(_usb: impl Peripheral<P = T> + 'd, irq: impl Peripheral<P = T::Interrupt> + 'd) -> Self {
        into_ref!(irq);
        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        let regs = T::regs();
        unsafe {
            // zero fill regs
            let p = regs.0 as *mut u32;
            for i in 0..0x9c / 4 {
                p.add(i).write_volatile(0)
            }

            // zero fill epmem
            let p = EP_MEMORY as *mut u32;
            for i in 0..0x100 / 4 {
                p.add(i).write_volatile(0)
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
        }

        // Initialize the bus so that it signals that power is available
        BUS_WAKER.wake();

        Self {
            phantom: PhantomData,
            ep_in: [EndpointData::new(); EP_COUNT],
            ep_out: [EndpointData::new(); EP_COUNT],
            ep_mem_free: 0x180, // data buffer region
        }
    }

    fn on_interrupt(_: *mut ()) {
        unsafe {
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

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Endpoint<'d, T, D>, driver::EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval,
            D::dir()
        );

        let alloc = match D::dir() {
            Direction::Out => &mut self.ep_out,
            Direction::In => &mut self.ep_in,
        };

        let index = alloc.iter_mut().enumerate().find(|(i, ep)| {
            if *i == 0 {
                return false; // reserved for control pipe
            }
            !ep.used
        });

        let (index, ep) = index.ok_or(EndpointAllocError)?;
        assert!(!ep.used);

        if max_packet_size > 64 {
            warn!("max_packet_size too high: {}", max_packet_size);
            return Err(EndpointAllocError);
        }

        // ep mem addrs must be 64-byte aligned, so there's no point in trying
        // to allocate smaller chunks to save memory.
        let len = 64;

        let addr = self.ep_mem_free;
        if addr + len > EP_MEMORY_SIZE as _ {
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
            EndpointType::Bulk => pac::usb_dpram::vals::EpControlEndpointType::BULK,
            EndpointType::Control => pac::usb_dpram::vals::EpControlEndpointType::CONTROL,
            EndpointType::Interrupt => pac::usb_dpram::vals::EpControlEndpointType::INTERRUPT,
            EndpointType::Isochronous => pac::usb_dpram::vals::EpControlEndpointType::ISOCHRONOUS,
        };

        match D::dir() {
            Direction::Out => unsafe {
                T::dpram().ep_out_control(index - 1).write(|w| {
                    w.set_enable(false);
                    w.set_buffer_address(addr);
                    w.set_interrupt_per_buff(true);
                    w.set_endpoint_type(ep_type_reg);
                })
            },
            Direction::In => unsafe {
                T::dpram().ep_in_control(index - 1).write(|w| {
                    w.set_enable(false);
                    w.set_buffer_address(addr);
                    w.set_interrupt_per_buff(true);
                    w.set_endpoint_type(ep_type_reg);
                })
            },
        }

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval,
            },
            buf,
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
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, max_packet_size, interval)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, max_packet_size, interval)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let regs = T::regs();
        unsafe {
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
            })
        }
        trace!("enabled");

        (
            Bus {
                phantom: PhantomData,
                inited: false,
                ep_out: self.ep_out,
            },
            ControlPipe {
                _phantom: PhantomData,
                max_packet_size: control_max_packet_size,
            },
        )
    }
}

pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_out: [EndpointData; EP_COUNT],
    inited: bool,
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    type PollFuture<'a> = impl Future<Output = Event> + 'a where Self: 'a;

    fn poll<'a>(&'a mut self) -> Self::PollFuture<'a> {
        poll_fn(move |cx| unsafe {
            BUS_WAKER.register(cx.waker());

            if !self.inited {
                self.inited = true;
                return Poll::Ready(Event::PowerDetected);
            }

            let regs = T::regs();
            let siestatus = regs.sie_status().read();

            if siestatus.resume() {
                regs.sie_status().write(|w| w.set_resume(true));
                return Poll::Ready(Event::Resume);
            }

            if siestatus.bus_reset() {
                regs.sie_status().write(|w| {
                    w.set_bus_reset(true);
                    w.set_setup_rec(true);
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

            if siestatus.suspended() {
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
    }

    #[inline]
    fn set_address(&mut self, addr: u8) {
        let regs = T::regs();
        trace!("setting addr: {}", addr);
        unsafe { regs.addr_endp().write(|w| w.set_address(addr)) }
    }

    fn endpoint_set_stalled(&mut self, _ep_addr: EndpointAddress, _stalled: bool) {
        todo!();
    }

    fn endpoint_is_stalled(&mut self, _ep_addr: EndpointAddress) -> bool {
        todo!();
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:?} {}", ep_addr, enabled);
        if ep_addr.index() == 0 {
            return;
        }

        let n = ep_addr.index();
        match ep_addr.direction() {
            Direction::In => unsafe {
                T::dpram().ep_in_control(n - 1).modify(|w| w.set_enable(enabled));
                T::dpram().ep_in_buffer_control(ep_addr.index()).write(|w| {
                    w.set_pid(0, true); // first packet is DATA0, but PID is flipped before
                });
                EP_IN_WAKERS[n].wake();
            },
            Direction::Out => unsafe {
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
            },
        }
    }

    type EnableFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn enable(&mut self) -> Self::EnableFuture<'_> {
        async move {}
    }

    type DisableFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn disable(&mut self) -> Self::DisableFuture<'_> {
        async move {}
    }

    type RemoteWakeupFuture<'a> =  impl Future<Output = Result<(), Unsupported>> + 'a where Self: 'a;

    fn remote_wakeup(&mut self) -> Self::RemoteWakeupFuture<'_> {
        async move { Err(Unsupported) }
    }
}

trait Dir {
    fn dir() -> Direction;
    fn waker(i: usize) -> &'static AtomicWaker;
}

pub enum In {}
impl Dir for In {
    fn dir() -> Direction {
        Direction::In
    }

    #[inline]
    fn waker(i: usize) -> &'static AtomicWaker {
        &EP_IN_WAKERS[i]
    }
}

pub enum Out {}
impl Dir for Out {
    fn dir() -> Direction {
        Direction::Out
    }

    #[inline]
    fn waker(i: usize) -> &'static AtomicWaker {
        &EP_OUT_WAKERS[i]
    }
}

pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    buf: EndpointBuffer<T>,
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, In> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    type WaitEnabledFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn wait_enabled(&mut self) -> Self::WaitEnabledFuture<'_> {
        async move {
            trace!("wait_enabled IN WAITING");
            let index = self.info.addr.index();
            poll_fn(|cx| {
                EP_OUT_WAKERS[index].register(cx.waker());
                let val = unsafe { T::dpram().ep_in_control(self.info.addr.index() - 1).read() };
                if val.enable() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            trace!("wait_enabled IN OK");
        }
    }
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, Out> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    type WaitEnabledFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn wait_enabled(&mut self) -> Self::WaitEnabledFuture<'_> {
        async move {
            trace!("wait_enabled OUT WAITING");
            let index = self.info.addr.index();
            poll_fn(|cx| {
                EP_OUT_WAKERS[index].register(cx.waker());
                let val = unsafe { T::dpram().ep_out_control(self.info.addr.index() - 1).read() };
                if val.enable() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            trace!("wait_enabled OUT OK");
        }
    }
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, EndpointError>> + 'a where Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            trace!("READ WAITING, buf.len() = {}", buf.len());
            let index = self.info.addr.index();
            let val = poll_fn(|cx| unsafe {
                EP_OUT_WAKERS[index].register(cx.waker());
                let val = T::dpram().ep_out_buffer_control(index).read();
                if val.available(0) {
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

            unsafe {
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
            }

            Ok(rx_len)
        }
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    type WriteFuture<'a> = impl Future<Output = Result<(), EndpointError>> + 'a where Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            if buf.len() > self.info.max_packet_size as usize {
                return Err(EndpointError::BufferOverflow);
            }

            trace!("WRITE WAITING");

            let index = self.info.addr.index();
            let val = poll_fn(|cx| unsafe {
                EP_IN_WAKERS[index].register(cx.waker());
                let val = T::dpram().ep_in_buffer_control(index).read();
                if val.available(0) {
                    Poll::Pending
                } else {
                    Poll::Ready(val)
                }
            })
            .await;

            self.buf.write(buf);

            unsafe {
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
            }

            trace!("WRITE OK");

            Ok(())
        }
    }
}

pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    type SetupFuture<'a> = impl Future<Output = [u8;8]> + 'a where Self: 'a;
    type DataOutFuture<'a> = impl Future<Output = Result<usize, EndpointError>> + 'a where Self: 'a;
    type DataInFuture<'a> = impl Future<Output = Result<(), EndpointError>> + 'a where Self: 'a;
    type AcceptFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;
    type RejectFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn max_packet_size(&self) -> usize {
        64
    }

    fn setup<'a>(&'a mut self) -> Self::SetupFuture<'a> {
        async move {
            loop {
                trace!("SETUP read waiting");
                let regs = T::regs();
                unsafe { regs.inte().write_set(|w| w.set_setup_req(true)) };

                poll_fn(|cx| unsafe {
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
                unsafe {
                    regs.sie_status().write(|w| w.set_setup_rec(true));

                    // set PID to 0, so (after toggling) first DATA is PID 1
                    T::dpram().ep_in_buffer_control(0).write(|w| w.set_pid(0, false));
                    T::dpram().ep_out_buffer_control(0).write(|w| w.set_pid(0, false));
                }

                trace!("SETUP read ok");
                return buf;
            }
        }
    }

    fn data_out<'a>(&'a mut self, buf: &'a mut [u8], _first: bool, _last: bool) -> Self::DataOutFuture<'a> {
        async move {
            unsafe {
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
            }

            trace!("control: data_out len={} first={} last={}", buf.len(), _first, _last);
            let val = poll_fn(|cx| unsafe {
                EP_OUT_WAKERS[0].register(cx.waker());
                let val = T::dpram().ep_out_buffer_control(0).read();
                if val.available(0) {
                    Poll::Pending
                } else {
                    Poll::Ready(val)
                }
            })
            .await;

            let rx_len = val.length(0) as _;
            trace!("control data_out DONE, rx_len = {}", rx_len);

            if rx_len > buf.len() {
                return Err(EndpointError::BufferOverflow);
            }
            EndpointBuffer::<T>::new(0x100, 64).read(&mut buf[..rx_len]);

            Ok(rx_len)
        }
    }

    fn data_in<'a>(&'a mut self, buf: &'a [u8], _first: bool, _last: bool) -> Self::DataInFuture<'a> {
        async move {
            trace!("control: data_in len={} first={} last={}", buf.len(), _first, _last);

            if buf.len() > 64 {
                return Err(EndpointError::BufferOverflow);
            }
            EndpointBuffer::<T>::new(0x100, 64).write(buf);

            unsafe {
                let bufcontrol = T::dpram().ep_in_buffer_control(0);
                let pid = !bufcontrol.read().pid(0);
                bufcontrol.write(|w| {
                    w.set_length(0, buf.len() as _);
                    w.set_pid(0, pid);
                    w.set_full(0, true);
                });
                cortex_m::asm::delay(12);
                bufcontrol.write(|w| {
                    w.set_length(0, buf.len() as _);
                    w.set_pid(0, pid);
                    w.set_full(0, true);
                    w.set_available(0, true);
                });
            }

            poll_fn(|cx| unsafe {
                EP_IN_WAKERS[0].register(cx.waker());
                let bufcontrol = T::dpram().ep_in_buffer_control(0);
                if bufcontrol.read().available(0) {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
            trace!("control: data_in DONE");

            if _last {
                // prepare status phase right away.
                unsafe {
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
            }

            Ok(())
        }
    }

    fn accept<'a>(&'a mut self) -> Self::AcceptFuture<'a> {
        async move {
            trace!("control: accept");

            let bufcontrol = T::dpram().ep_in_buffer_control(0);
            unsafe {
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
            }

            // wait for completion before returning, needed so
            // set_address() doesn't happen early.
            poll_fn(|cx| {
                EP_IN_WAKERS[0].register(cx.waker());
                if unsafe { bufcontrol.read().available(0) } {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
        }
    }

    fn reject<'a>(&'a mut self) -> Self::RejectFuture<'a> {
        async move {
            trace!("control: reject");

            let regs = T::regs();
            unsafe {
                regs.ep_stall_arm().write_set(|w| {
                    w.set_ep0_in(true);
                    w.set_ep0_out(true);
                });
                T::dpram().ep_out_buffer_control(0).write(|w| w.set_stall(true));
                T::dpram().ep_in_buffer_control(0).write(|w| w.set_stall(true));
            }
        }
    }
}
