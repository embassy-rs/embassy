//! Universal Serial Bus (USB) driver.

#![macro_use]

pub mod vbus_detect;

use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};
use core::task::Poll;

use cortex_m::peripheral::NVIC;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{Direction, EndpointAddress, EndpointError, EndpointInfo, EndpointType, Event, Unsupported};

use self::vbus_detect::VbusDetect;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::usbd::vals;
use crate::util::slice_in_ram;
use crate::{interrupt, pac};

static BUS_WAKER: AtomicWaker = AtomicWaker::new();
static EP0_WAKER: AtomicWaker = AtomicWaker::new();
static EP_IN_WAKERS: [AtomicWaker; 8] = [const { AtomicWaker::new() }; 8];
static EP_OUT_WAKERS: [AtomicWaker; 8] = [const { AtomicWaker::new() }; 8];
static READY_ENDPOINTS: AtomicU32 = AtomicU32::new(0);

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        if regs.events_usbreset().read() != 0 {
            regs.intenclr().write(|w| w.set_usbreset(true));
            BUS_WAKER.wake();
            EP0_WAKER.wake();
        }

        if regs.events_ep0setup().read() != 0 {
            regs.intenclr().write(|w| w.set_ep0setup(true));
            EP0_WAKER.wake();
        }

        if regs.events_ep0datadone().read() != 0 {
            regs.intenclr().write(|w| w.set_ep0datadone(true));
            EP0_WAKER.wake();
        }

        // USBEVENT and EPDATA events are weird. They're the "aggregate"
        // of individual bits in EVENTCAUSE and EPDATASTATUS. We handle them
        // differently than events normally.
        //
        // They seem to be edge-triggered, not level-triggered: when an
        // individual bit goes 0->1, the event fires *just once*.
        // Therefore, it's fine to clear just the event, and let main thread
        // check the individual bits in EVENTCAUSE and EPDATASTATUS. It
        // doesn't cause an infinite irq loop.
        if regs.events_usbevent().read() != 0 {
            regs.events_usbevent().write_value(0);
            BUS_WAKER.wake();
        }

        if regs.events_epdata().read() != 0 {
            regs.events_epdata().write_value(0);

            let r = regs.epdatastatus().read();
            regs.epdatastatus().write_value(r);
            READY_ENDPOINTS.fetch_or(r.0, Ordering::AcqRel);
            for i in 1..=7 {
                if r.0 & In::mask(i) != 0 {
                    In::waker(i).wake();
                }
                if r.0 & Out::mask(i) != 0 {
                    Out::waker(i).wake();
                }
            }
        }
    }
}

/// USB driver.
pub struct Driver<'d, T: Instance, V: VbusDetect> {
    _p: Peri<'d, T>,
    alloc_in: Allocator,
    alloc_out: Allocator,
    vbus_detect: V,
}

impl<'d, T: Instance, V: VbusDetect> Driver<'d, T, V> {
    /// Create a new USB driver.
    pub fn new(
        usb: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        vbus_detect: V,
    ) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _p: usb,
            alloc_in: Allocator::new(),
            alloc_out: Allocator::new(),
            vbus_detect,
        }
    }
}

impl<'d, T: Instance, V: VbusDetect + 'd> driver::Driver<'d> for Driver<'d, T, V> {
    type EndpointOut = Endpoint<'d, T, Out>;
    type EndpointIn = Endpoint<'d, T, In>;
    type ControlPipe = ControlPipe<'d, T>;
    type Bus = Bus<'d, T, V>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        let index = self.alloc_in.allocate(ep_type)?;
        let ep_addr = EndpointAddress::from_parts(index, Direction::In);
        Ok(Endpoint::new(EndpointInfo {
            addr: ep_addr,
            ep_type,
            max_packet_size: packet_size,
            interval_ms,
        }))
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        let index = self.alloc_out.allocate(ep_type)?;
        let ep_addr = EndpointAddress::from_parts(index, Direction::Out);
        Ok(Endpoint::new(EndpointInfo {
            addr: ep_addr,
            ep_type,
            max_packet_size: packet_size,
            interval_ms,
        }))
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        (
            Bus {
                _p: unsafe { self._p.clone_unchecked() },
                power_available: false,
                vbus_detect: self.vbus_detect,
            },
            ControlPipe {
                _p: self._p,
                max_packet_size: control_max_packet_size,
            },
        )
    }
}

/// USB bus.
pub struct Bus<'d, T: Instance, V: VbusDetect> {
    _p: Peri<'d, T>,
    power_available: bool,
    vbus_detect: V,
}

impl<'d, T: Instance, V: VbusDetect> driver::Bus for Bus<'d, T, V> {
    async fn enable(&mut self) {
        let regs = T::regs();

        errata::pre_enable();

        regs.enable().write(|w| w.set_enable(true));

        // Wait until the peripheral is ready.
        regs.intenset().write(|w| w.set_usbevent(true));
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            if regs.eventcause().read().ready() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        regs.eventcause().write(|w| w.set_ready(true));

        errata::post_enable();

        unsafe { NVIC::unmask(pac::Interrupt::USBD) };

        regs.intenset().write(|w| {
            w.set_usbreset(true);
            w.set_usbevent(true);
            w.set_epdata(true);
        });

        if self.vbus_detect.wait_power_ready().await.is_ok() {
            // Enable the USB pullup, allowing enumeration.
            regs.usbpullup().write(|w| w.set_connect(true));
            trace!("enabled");
        } else {
            trace!("usb power not ready due to usb removal");
        }
    }

    async fn disable(&mut self) {
        let regs = T::regs();
        regs.enable().write(|x| x.set_enable(false));
    }

    fn poll(&mut self) -> impl Future<Output = Event> {
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            let regs = T::regs();

            if regs.events_usbreset().read() != 0 {
                regs.events_usbreset().write_value(0);
                regs.intenset().write(|w| w.set_usbreset(true));

                // Disable all endpoints except EP0
                regs.epinen().write(|w| w.0 = 0x01);
                regs.epouten().write(|w| w.0 = 0x01);
                READY_ENDPOINTS.store(In::mask(0), Ordering::Release);
                for i in 1..=7 {
                    In::waker(i).wake();
                    Out::waker(i).wake();
                }

                return Poll::Ready(Event::Reset);
            }

            let r = regs.eventcause().read();

            if r.isooutcrc() {
                regs.eventcause().write(|w| w.set_isooutcrc(true));
                trace!("USB event: isooutcrc");
            }
            if r.usbwuallowed() {
                regs.eventcause().write(|w| w.set_usbwuallowed(true));
                trace!("USB event: usbwuallowed");
            }
            if r.suspend() {
                regs.eventcause().write(|w| w.set_suspend(true));
                regs.lowpower().write(|w| w.set_lowpower(vals::Lowpower::LOW_POWER));
                return Poll::Ready(Event::Suspend);
            }
            if r.resume() {
                regs.eventcause().write(|w| w.set_resume(true));
                return Poll::Ready(Event::Resume);
            }
            if r.ready() {
                regs.eventcause().write(|w| w.set_ready(true));
                trace!("USB event: ready");
            }

            if self.vbus_detect.is_usb_detected() != self.power_available {
                self.power_available = !self.power_available;
                if self.power_available {
                    trace!("Power event: available");
                    return Poll::Ready(Event::PowerDetected);
                } else {
                    trace!("Power event: removed");
                    return Poll::Ready(Event::PowerRemoved);
                }
            }

            Poll::Pending
        })
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        let regs = T::regs();
        if ep_addr.index() == 0 {
            if stalled {
                regs.tasks_ep0stall().write_value(1);
            }
        } else {
            regs.epstall().write(|w| {
                w.set_ep(ep_addr.index() as u8 & 0b111);
                w.set_io(match ep_addr.direction() {
                    Direction::In => vals::Io::IN,
                    Direction::Out => vals::Io::OUT,
                });
                w.set_stall(stalled);
            });
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let regs = T::regs();
        let i = ep_addr.index();
        match ep_addr.direction() {
            Direction::Out => regs.halted().epout(i).read().getstatus() == vals::Getstatus::HALTED,
            Direction::In => regs.halted().epin(i).read().getstatus() == vals::Getstatus::HALTED,
        }
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        let regs = T::regs();

        let i = ep_addr.index();
        let mask = 1 << i;

        debug!("endpoint_set_enabled {:?} {}", ep_addr, enabled);

        match ep_addr.direction() {
            Direction::In => {
                let mut was_enabled = false;
                regs.epinen().modify(|w| {
                    was_enabled = (w.0 & mask) != 0;
                    if enabled {
                        w.0 |= mask
                    } else {
                        w.0 &= !mask
                    }
                });

                let ready_mask = In::mask(i);
                if enabled {
                    if !was_enabled {
                        READY_ENDPOINTS.fetch_or(ready_mask, Ordering::AcqRel);
                    }
                } else {
                    READY_ENDPOINTS.fetch_and(!ready_mask, Ordering::AcqRel);
                }

                In::waker(i).wake();
            }
            Direction::Out => {
                regs.epouten()
                    .modify(|w| if enabled { w.0 |= mask } else { w.0 &= !mask });

                let ready_mask = Out::mask(i);
                if enabled {
                    // when first enabled, bulk/interrupt OUT endpoints will *not* receive data (the
                    // peripheral will NAK all incoming packets) until we write a zero to the SIZE
                    // register (see figure 203 of the 52840 manual). To avoid that we write a 0 to the
                    // SIZE register
                    regs.size().epout(i).write(|_| ());
                } else {
                    READY_ENDPOINTS.fetch_and(!ready_mask, Ordering::AcqRel);
                }

                Out::waker(i).wake();
            }
        }
    }

    #[inline]
    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        let regs = T::regs();

        if regs.lowpower().read().lowpower() == vals::Lowpower::LOW_POWER {
            errata::pre_wakeup();

            regs.lowpower().write(|w| w.set_lowpower(vals::Lowpower::FORCE_NORMAL));

            poll_fn(|cx| {
                BUS_WAKER.register(cx.waker());
                let regs = T::regs();
                let r = regs.eventcause().read();

                if regs.events_usbreset().read() != 0 {
                    Poll::Ready(())
                } else if r.resume() {
                    Poll::Ready(())
                } else if r.usbwuallowed() {
                    regs.eventcause().write(|w| w.set_usbwuallowed(true));
                    regs.dpdmvalue().write(|w| w.set_state(vals::State::RESUME));
                    regs.tasks_dpdmdrive().write_value(1);

                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            errata::post_wakeup();
        }

        Ok(())
    }
}

/// Type-level marker for OUT endpoints.
pub enum Out {}

/// Type-level marker for IN endpoints.
pub enum In {}

trait EndpointDir {
    fn waker(i: usize) -> &'static AtomicWaker;
    fn mask(i: usize) -> u32;
    fn is_enabled(regs: pac::usbd::Usbd, i: usize) -> bool;
}

impl EndpointDir for In {
    #[inline]
    fn waker(i: usize) -> &'static AtomicWaker {
        &EP_IN_WAKERS[i - 1]
    }

    #[inline]
    fn mask(i: usize) -> u32 {
        1 << i
    }

    #[inline]
    fn is_enabled(regs: pac::usbd::Usbd, i: usize) -> bool {
        regs.epinen().read().in_(i)
    }
}

impl EndpointDir for Out {
    #[inline]
    fn waker(i: usize) -> &'static AtomicWaker {
        &EP_OUT_WAKERS[i - 1]
    }

    #[inline]
    fn mask(i: usize) -> u32 {
        1 << (i + 16)
    }

    #[inline]
    fn is_enabled(regs: pac::usbd::Usbd, i: usize) -> bool {
        regs.epouten().read().out(i)
    }
}

/// USB endpoint.
pub struct Endpoint<'d, T: Instance, Dir> {
    _phantom: PhantomData<(&'d mut T, Dir)>,
    info: EndpointInfo,
}

impl<'d, T: Instance, Dir> Endpoint<'d, T, Dir> {
    fn new(info: EndpointInfo) -> Self {
        Self {
            info,
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance, Dir: EndpointDir> driver::Endpoint for Endpoint<'d, T, Dir> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        self.wait_enabled_state(true).await
    }
}

#[allow(private_bounds)]
impl<'d, T: Instance, Dir: EndpointDir> Endpoint<'d, T, Dir> {
    fn wait_enabled_state(&mut self, state: bool) -> impl Future<Output = ()> {
        let i = self.info.addr.index();
        assert!(i != 0);

        poll_fn(move |cx| {
            Dir::waker(i).register(cx.waker());
            if Dir::is_enabled(T::regs(), i) == state {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }

    /// Wait for the endpoint to be disabled
    pub fn wait_disabled(&mut self) -> impl Future<Output = ()> {
        self.wait_enabled_state(false)
    }
}

impl<'d, T: Instance, Dir> Endpoint<'d, T, Dir> {
    async fn wait_data_ready(&mut self) -> Result<(), ()>
    where
        Dir: EndpointDir,
    {
        let i = self.info.addr.index();
        assert!(i != 0);
        poll_fn(|cx| {
            Dir::waker(i).register(cx.waker());
            let r = READY_ENDPOINTS.load(Ordering::Acquire);
            if !Dir::is_enabled(T::regs(), i) {
                Poll::Ready(Err(()))
            } else if r & Dir::mask(i) != 0 {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await?;

        // Mark as not ready
        READY_ENDPOINTS.fetch_and(!Dir::mask(i), Ordering::AcqRel);

        Ok(())
    }
}

unsafe fn read_dma<T: Instance>(i: usize, buf: &mut [u8]) -> Result<usize, EndpointError> {
    let regs = T::regs();

    // Check that the packet fits into the buffer
    let size = regs.size().epout(i).read().0 as usize;
    if size > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }

    regs.epout(i).ptr().write_value(buf.as_ptr() as u32);
    // MAXCNT must match SIZE
    regs.epout(i).maxcnt().write(|w| w.set_maxcnt(size as _));

    dma_start();
    regs.events_endepout(i).write_value(0);
    regs.tasks_startepout(i).write_value(1);
    while regs.events_endepout(i).read() == 0 {}
    regs.events_endepout(i).write_value(0);
    dma_end();

    regs.size().epout(i).write(|_| ());

    Ok(size)
}

unsafe fn write_dma<T: Instance>(i: usize, buf: &[u8]) {
    let regs = T::regs();
    assert!(buf.len() <= 64);

    let mut ram_buf: MaybeUninit<[u8; 64]> = MaybeUninit::uninit();
    let ptr = if !slice_in_ram(buf) {
        // EasyDMA can't read FLASH, so we copy through RAM
        let ptr = ram_buf.as_mut_ptr() as *mut u8;
        core::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, buf.len());
        ptr
    } else {
        buf.as_ptr()
    };

    // Set the buffer length so the right number of bytes are transmitted.
    // Safety: `buf.len()` has been checked to be <= the max buffer length.
    regs.epin(i).ptr().write_value(ptr as u32);
    regs.epin(i).maxcnt().write(|w| w.set_maxcnt(buf.len() as u8));

    regs.events_endepin(i).write_value(0);

    dma_start();
    regs.tasks_startepin(i).write_value(1);
    while regs.events_endepin(i).read() == 0 {}
    dma_end();
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let i = self.info.addr.index();
        assert!(i != 0);

        self.wait_data_ready().await.map_err(|_| EndpointError::Disabled)?;

        unsafe { read_dma::<T>(i, buf) }
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        let i = self.info.addr.index();
        assert!(i != 0);

        self.wait_data_ready().await.map_err(|_| EndpointError::Disabled)?;

        unsafe { write_dma::<T>(i, buf) }

        Ok(())
    }
}

/// USB control pipe.
pub struct ControlPipe<'d, T: Instance> {
    _p: Peri<'d, T>,
    max_packet_size: u16,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    async fn setup(&mut self) -> [u8; 8] {
        let regs = T::regs();

        // Reset shorts
        regs.shorts().write(|_| ());

        // Wait for SETUP packet
        regs.intenset().write(|w| w.set_ep0setup(true));
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0setup().read() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        regs.events_ep0setup().write_value(0);

        let mut buf = [0; 8];
        buf[0] = regs.bmrequesttype().read().0 as u8;
        buf[1] = regs.brequest().read().0 as u8;
        buf[2] = regs.wvaluel().read().0 as u8;
        buf[3] = regs.wvalueh().read().0 as u8;
        buf[4] = regs.windexl().read().0 as u8;
        buf[5] = regs.windexh().read().0 as u8;
        buf[6] = regs.wlengthl().read().0 as u8;
        buf[7] = regs.wlengthh().read().0 as u8;

        buf
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        let regs = T::regs();

        regs.events_ep0datadone().write_value(0);

        // This starts a RX on EP0. events_ep0datadone notifies when done.
        regs.tasks_ep0rcvout().write_value(1);

        // Wait until ready
        regs.intenset().write(|w| {
            w.set_usbreset(true);
            w.set_ep0setup(true);
            w.set_ep0datadone(true);
        });
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0datadone().read() != 0 {
                Poll::Ready(Ok(()))
            } else if regs.events_usbreset().read() != 0 {
                trace!("aborted control data_out: usb reset");
                Poll::Ready(Err(EndpointError::Disabled))
            } else if regs.events_ep0setup().read() != 0 {
                trace!("aborted control data_out: received another SETUP");
                Poll::Ready(Err(EndpointError::Disabled))
            } else {
                Poll::Pending
            }
        })
        .await?;

        unsafe { read_dma::<T>(0, buf) }
    }

    async fn data_in(&mut self, buf: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        let regs = T::regs();
        regs.events_ep0datadone().write_value(0);

        regs.shorts().write(|w| w.set_ep0datadone_ep0status(last));

        // This starts a TX on EP0. events_ep0datadone notifies when done.
        unsafe { write_dma::<T>(0, buf) }

        regs.intenset().write(|w| {
            w.set_usbreset(true);
            w.set_ep0setup(true);
            w.set_ep0datadone(true);
        });

        poll_fn(|cx| {
            cx.waker().wake_by_ref();
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0datadone().read() != 0 {
                Poll::Ready(Ok(()))
            } else if regs.events_usbreset().read() != 0 {
                trace!("aborted control data_in: usb reset");
                Poll::Ready(Err(EndpointError::Disabled))
            } else if regs.events_ep0setup().read() != 0 {
                trace!("aborted control data_in: received another SETUP");
                Poll::Ready(Err(EndpointError::Disabled))
            } else {
                Poll::Pending
            }
        })
        .await
    }

    async fn accept(&mut self) {
        let regs = T::regs();
        regs.tasks_ep0status().write_value(1);
    }

    async fn reject(&mut self) {
        let regs = T::regs();
        regs.tasks_ep0stall().write_value(1);
    }

    async fn accept_set_address(&mut self, _addr: u8) {
        self.accept().await;
        // Nothing to do, the peripheral handles this.
    }
}

fn dma_start() {
    compiler_fence(Ordering::Release);
}

fn dma_end() {
    compiler_fence(Ordering::Acquire);
}

struct Allocator {
    used: u16,
}

impl Allocator {
    fn new() -> Self {
        Self { used: 0 }
    }

    fn allocate(&mut self, ep_type: EndpointType) -> Result<usize, driver::EndpointAllocError> {
        // Endpoint addresses are fixed in hardware:
        // - 0x80 / 0x00 - Control        EP0
        // - 0x81 / 0x01 - Bulk/Interrupt EP1
        // - 0x82 / 0x02 - Bulk/Interrupt EP2
        // - 0x83 / 0x03 - Bulk/Interrupt EP3
        // - 0x84 / 0x04 - Bulk/Interrupt EP4
        // - 0x85 / 0x05 - Bulk/Interrupt EP5
        // - 0x86 / 0x06 - Bulk/Interrupt EP6
        // - 0x87 / 0x07 - Bulk/Interrupt EP7
        // - 0x88 / 0x08 - Isochronous

        // Endpoint directions are allocated individually.

        let alloc_index = match ep_type {
            EndpointType::Isochronous => 8,
            EndpointType::Control => return Err(driver::EndpointAllocError),
            EndpointType::Interrupt | EndpointType::Bulk => {
                // Find rightmost zero bit in 1..=7
                let ones = (self.used >> 1).trailing_ones() as usize;
                if ones >= 7 {
                    return Err(driver::EndpointAllocError);
                }
                ones + 1
            }
        };

        if self.used & (1 << alloc_index) != 0 {
            return Err(driver::EndpointAllocError);
        }

        self.used |= 1 << alloc_index;

        Ok(alloc_index)
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::usbd::Usbd;
}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_usb {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::usb::SealedInstance for peripherals::$type {
            fn regs() -> pac::usbd::Usbd {
                pac::$pac_type
            }
        }
        impl crate::usb::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

mod errata {

    /// Writes `val` to `addr`. Used to apply Errata workarounds.
    #[cfg(any(feature = "nrf52840", feature = "nrf52833", feature = "nrf52820"))]
    unsafe fn poke(addr: u32, val: u32) {
        (addr as *mut u32).write_volatile(val);
    }

    /// Reads 32 bits from `addr`.
    #[cfg(feature = "nrf52840")]
    unsafe fn peek(addr: u32) -> u32 {
        (addr as *mut u32).read_volatile()
    }

    pub fn pre_enable() {
        // Works around Erratum 187 on chip revisions 1 and 2.
        #[cfg(any(feature = "nrf52840", feature = "nrf52833", feature = "nrf52820"))]
        unsafe {
            poke(0x4006EC00, 0x00009375);
            poke(0x4006ED14, 0x00000003);
            poke(0x4006EC00, 0x00009375);
        }

        pre_wakeup();
    }

    pub fn post_enable() {
        post_wakeup();

        // Works around Erratum 187 on chip revisions 1 and 2.
        #[cfg(any(feature = "nrf52840", feature = "nrf52833", feature = "nrf52820"))]
        unsafe {
            poke(0x4006EC00, 0x00009375);
            poke(0x4006ED14, 0x00000000);
            poke(0x4006EC00, 0x00009375);
        }
    }

    pub fn pre_wakeup() {
        // Works around Erratum 171 on chip revisions 1 and 2.

        #[cfg(feature = "nrf52840")]
        unsafe {
            if peek(0x4006EC00) == 0x00000000 {
                poke(0x4006EC00, 0x00009375);
            }

            poke(0x4006EC14, 0x000000C0);
            poke(0x4006EC00, 0x00009375);
        }
    }

    pub fn post_wakeup() {
        // Works around Erratum 171 on chip revisions 1 and 2.

        #[cfg(feature = "nrf52840")]
        unsafe {
            if peek(0x4006EC00) == 0x00000000 {
                poke(0x4006EC00, 0x00009375);
            }

            poke(0x4006EC14, 0x00000000);
            poke(0x4006EC00, 0x00009375);
        }
    }
}
