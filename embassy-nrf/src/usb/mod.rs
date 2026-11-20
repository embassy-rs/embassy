//! Universal Serial Bus (USB) driver.

#![macro_use]

pub mod vbus_detect;

use core::cell::RefCell;
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use cortex_m::peripheral::NVIC;
use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::iso::{IsoSchedule, forward};
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointConfig, EndpointDisabledError, EndpointError, EndpointInfo, EndpointType,
    Event, IsoEndpointIn, IsoInSlot, IsoWriteError, Speed, Unsupported,
};

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

// nRF uses dedicated Endpoint 8 for ISO, not the EPIN/EPOUT arrays.
const ISO_EP: usize = 8;
const ISO_MAX_PACKET_SIZE: usize = 1023;
const ISO_FRAME_SPAN: u32 = 1 << 11;
const BULK_EP_COUNT: usize = 7;

static ISO_SOF_SEEN: AtomicBool = AtomicBool::new(false);
static ISO_IN_RESET: AtomicBool = AtomicBool::new(true);

static EP_IN_XFERS: Mutex<RefCell<[Option<InXfer>; BULK_EP_COUNT]>> =
    Mutex::new(RefCell::new([const { None }; BULK_EP_COUNT]));
static EP_OUT_XFERS: Mutex<RefCell<[Option<OutXfer>; BULK_EP_COUNT]>> =
    Mutex::new(RefCell::new([const { None }; BULK_EP_COUNT]));
static EP0_XFER: Mutex<RefCell<Option<Ep0Xfer>>> = Mutex::new(RefCell::new(None));

struct OutXfer {
    buf_ptr: *mut u8,
    buf_len: usize,
    total: usize,
    max_packet_size: usize,
    result: Option<Result<usize, EndpointError>>,
}

// SAFETY: `OutXfer` is only kept active while the owning future holds `&mut [u8]`
// and is cleared by its Drop guard inside a critical section.
unsafe impl Send for OutXfer {}

impl OutXfer {
    fn new(buf: &mut [u8], max_packet_size: usize) -> Self {
        Self {
            buf_ptr: buf.as_mut_ptr(),
            buf_len: buf.len(),
            total: 0,
            max_packet_size,
            result: None,
        }
    }

    fn on_packet(&mut self, cs: CriticalSection, regs: pac::usbd::Usbd, i: usize) -> bool {
        if self.result.is_some() {
            return false;
        }
        let chunk_len = (self.buf_len - self.total).min(self.max_packet_size);
        // SAFETY: `self.buf_ptr..self.buf_ptr + self.buf_len` is borrowed for the lifetime
        // of the active transfer future, and `self.total + chunk_len <= self.buf_len`.
        let dst = unsafe { core::slice::from_raw_parts_mut(self.buf_ptr.add(self.total), chunk_len) };
        match unsafe { read_dma_in_cs(cs, regs, i, dst) } {
            Ok(n) => {
                self.total += n;
                if n < self.max_packet_size || self.total == self.buf_len {
                    self.result = Some(Ok(self.total));
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                if i != 0 {
                    READY_ENDPOINTS.fetch_or(Out::mask(i), Ordering::AcqRel);
                }
                self.result = Some(Err(e));
                true
            }
        }
    }
}

struct InXfer {
    buf_ptr: *const u8,
    buf_len: usize,
    offset: usize,
    send_zlp: bool,
    max_packet_size: usize,
    wait_last_complete: bool,
    done: bool,
    error: Option<EndpointDisabledError>,
}

// SAFETY: `InXfer` is only kept active while the owning future holds `&[u8]`
// and is cleared by its Drop guard inside a critical section.
unsafe impl Send for InXfer {}

impl InXfer {
    fn has_more(&self) -> bool {
        self.offset < self.buf_len || self.send_zlp
    }

    fn on_packet(&mut self, cs: CriticalSection, regs: pac::usbd::Usbd, i: usize) -> bool {
        if self.done || self.error.is_some() {
            return false;
        }

        if self.offset < self.buf_len {
            let chunk_len = (self.buf_len - self.offset).min(self.max_packet_size);
            // SAFETY: `self.buf_ptr..self.buf_ptr + self.buf_len` is borrowed for the lifetime
            // of the active transfer future.
            let chunk = unsafe { core::slice::from_raw_parts(self.buf_ptr.add(self.offset), chunk_len) };
            self.offset += chunk_len;

            if i == 0 {
                let is_last = !self.has_more();
                regs.events_ep0datadone().write_value(0);
                regs.shorts().write(|w| w.set_ep0datadone_ep0status(is_last));
            }
            unsafe { write_dma_in_cs(cs, regs, i, chunk) };

            if !self.wait_last_complete && !self.has_more() {
                self.done = true;
                return true;
            }
            false
        } else if self.send_zlp {
            self.send_zlp = false;
            if i == 0 {
                regs.events_ep0datadone().write_value(0);
                regs.shorts().write(|w| w.set_ep0datadone_ep0status(true));
            }
            unsafe { write_dma_in_cs(cs, regs, i, &[]) };

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

enum Ep0Xfer {
    In(InXfer),
    Out(OutXfer),
}

struct OutXferGuard(usize);

impl Drop for OutXferGuard {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            EP_OUT_XFERS.borrow(cs).borrow_mut()[self.0 - 1] = None;
        });
    }
}

struct InXferGuard(usize);

impl Drop for InXferGuard {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            EP_IN_XFERS.borrow(cs).borrow_mut()[self.0 - 1] = None;
        });
    }
}

struct Ep0XferGuard;

impl Drop for Ep0XferGuard {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            *EP0_XFER.borrow(cs).borrow_mut() = None;
            crate::pac::USBD.intenclr().write(|w| w.set_ep0datadone(true));
        });
    }
}

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
            if regs.events_usbreset().read() != 0 || regs.events_ep0setup().read() != 0 {
                regs.intenclr().write(|w| w.set_ep0datadone(true));
                EP0_WAKER.wake();
            } else {
                critical_section::with(|cs| {
                    let mut ep0 = EP0_XFER.borrow(cs).borrow_mut();
                    match ep0.as_mut() {
                        Some(Ep0Xfer::In(xfer)) => {
                            if xfer.on_packet(cs, regs, 0) {
                                regs.intenclr().write(|w| w.set_ep0datadone(true));
                                EP0_WAKER.wake();
                            }
                        }
                        Some(Ep0Xfer::Out(xfer)) => {
                            if xfer.on_packet(cs, regs, 0) {
                                regs.intenclr().write(|w| w.set_ep0datadone(true));
                                EP0_WAKER.wake();
                            } else {
                                regs.events_ep0datadone().write_value(0);
                                regs.tasks_ep0rcvout().write_value(1);
                            }
                        }
                        None => {
                            regs.intenclr().write(|w| w.set_ep0datadone(true));
                            EP0_WAKER.wake();
                        }
                    }
                });
            }
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

        // Clear the SOF interrupt event independently of ISO IN state.
        // ISO IN uses SOF instead of EPDATASTATUS to pace writes.
        if regs.intenset().read().sof() && regs.events_sof().read() != 0 {
            regs.events_sof().write_value(0);
            ISO_SOF_SEEN.store(true, Ordering::Release);
            if regs.epinen().read().isoin() {
                READY_ENDPOINTS.fetch_or(In::mask(ISO_EP), Ordering::AcqRel);
                In::waker(ISO_EP).wake();
            }
        }

        if regs.events_epdata().read() != 0 {
            regs.events_epdata().write_value(0);

            let r = regs.epdatastatus().read();
            regs.epdatastatus().write_value(r);

            critical_section::with(|cs| {
                let mut in_xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                let mut out_xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();

                for i in 1..=7 {
                    if r.0 & In::mask(i) != 0 {
                        if let Some(xfer) = in_xfers[i - 1].as_mut() {
                            if !xfer.done && xfer.error.is_none() {
                                if !In::is_enabled(regs, i) {
                                    xfer.error = Some(EndpointDisabledError);
                                    In::waker(i).wake();
                                } else if xfer.on_packet(cs, regs, i) {
                                    In::waker(i).wake();
                                }
                            } else {
                                READY_ENDPOINTS.fetch_or(In::mask(i), Ordering::AcqRel);
                                In::waker(i).wake();
                            }
                        } else {
                            READY_ENDPOINTS.fetch_or(In::mask(i), Ordering::AcqRel);
                            In::waker(i).wake();
                        }
                    }

                    if r.0 & Out::mask(i) != 0 {
                        if let Some(xfer) = out_xfers[i - 1].as_mut() {
                            if xfer.result.is_none() {
                                if !Out::is_enabled(regs, i) {
                                    xfer.result = Some(Err(EndpointError::Disabled));
                                    Out::waker(i).wake();
                                } else if xfer.on_packet(cs, regs, i) {
                                    Out::waker(i).wake();
                                }
                            } else {
                                READY_ENDPOINTS.fetch_or(Out::mask(i), Ordering::AcqRel);
                                Out::waker(i).wake();
                            }
                        } else {
                            READY_ENDPOINTS.fetch_or(Out::mask(i), Ordering::AcqRel);
                            Out::waker(i).wake();
                        }
                    }
                }
            });
        }
    }
}

/// USB driver.
pub struct Driver<'d, V: VbusDetect> {
    regs: pac::usbd::Usbd,
    alloc_in: Allocator,
    alloc_out: Allocator,
    vbus_detect: V,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, V: VbusDetect> Driver<'d, V> {
    /// Create a new USB driver.
    pub fn new<T: Instance>(
        _usb: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        vbus_detect: V,
    ) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            regs: crate::pac::USBD,
            alloc_in: Allocator::new(),
            alloc_out: Allocator::new(),
            vbus_detect,
            _phantom: PhantomData,
        }
    }
}

impl<'d, V: VbusDetect + 'd> driver::Driver<'d> for Driver<'d, V> {
    type EndpointOut = Endpoint<'d, Out>;
    type EndpointIn = Endpoint<'d, In>;
    type Bus = Bus<'d, V>;

    fn alloc_endpoint_in(&mut self, config: &EndpointConfig) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        let index = self.alloc_in.allocate(config.ep_type, config.addr)?;
        let ep_addr = EndpointAddress::from_parts(index, Direction::In);
        Ok(Endpoint::new(
            self.regs,
            EndpointInfo::new(ep_addr, config.ep_type, config.max_packet_size, config.interval),
        ))
    }

    fn alloc_endpoint_out(&mut self, config: &EndpointConfig) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        let index = self.alloc_out.allocate(config.ep_type, config.addr)?;
        let ep_addr = EndpointAddress::from_parts(index, Direction::Out);
        Ok(Endpoint::new(
            self.regs,
            EndpointInfo::new(ep_addr, config.ep_type, config.max_packet_size, config.interval),
        ))
    }

    fn start(self, control_max_packet_size: u16) -> Self::Bus {
        Bus {
            regs: self.regs,
            power_available: false,
            enable_sof: self.alloc_in.used & (1 << ISO_EP) != 0,
            vbus_detect: self.vbus_detect,
            control_max_packet_size,
            _phantom: PhantomData,
        }
    }
}

/// Set or clear the STALL handshake of an endpoint.
///
/// Shared by `Bus::endpoint_set_stalled` and `Endpoint::set_stalled`, so that a class
/// driver can halt its own endpoint without access to the bus.
fn set_endpoint_stalled(regs: pac::usbd::Usbd, ep_addr: EndpointAddress, stalled: bool) {
    if ep_addr.index() == 0 {
        if stalled {
            regs.tasks_ep0stall().write_value(1);
        }
    } else if ep_addr.index() != ISO_EP {
        // ISO endpoints cannot stall; EPSTALL only addresses endpoints 0..=7.
        regs.epstall().write(|w| {
            w.set_ep(ep_addr.index() as u8 & 0b111);
            w.set_io(match ep_addr.direction() {
                Direction::In => vals::Io::In,
                Direction::Out => vals::Io::Out,
            });
            w.set_stall(stalled);
        });
    }
}

/// Get whether the STALL handshake is set on an endpoint.
fn is_endpoint_stalled(regs: pac::usbd::Usbd, ep_addr: EndpointAddress) -> bool {
    let i = ep_addr.index();
    if i == ISO_EP {
        return false;
    }
    match ep_addr.direction() {
        Direction::Out => regs.halted().epout(i).read().getstatus() == vals::Getstatus::Halted,
        Direction::In => regs.halted().epin(i).read().getstatus() == vals::Getstatus::Halted,
    }
}

/// USB bus.
pub struct Bus<'d, V: VbusDetect> {
    regs: pac::usbd::Usbd,
    power_available: bool,
    enable_sof: bool,
    vbus_detect: V,
    control_max_packet_size: u16,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, V: VbusDetect> driver::Bus for Bus<'d, V> {
    async fn enable(&mut self) {
        let regs = self.regs;

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

        if self.enable_sof {
            ISO_SOF_SEEN.store(false, Ordering::Release);
            ISO_IN_RESET.store(true, Ordering::Release);
            regs.isoinconfig().write(|w| w.set_response(vals::Response::ZeroData));
            regs.events_sof().write_value(0);
        }

        unsafe { NVIC::unmask(pac::Interrupt::USBD) };

        regs.intenset().write(|w| {
            w.set_usbreset(true);
            w.set_usbevent(true);
            w.set_epdata(true);
            w.set_sof(self.enable_sof);
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
        let regs = self.regs;
        regs.intenclr().write(|w| w.set_sof(true));
        regs.enable().write(|x| x.set_enable(false));
    }

    fn poll(&mut self) -> impl Future<Output = Event> {
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            EP0_WAKER.register(cx.waker());
            let regs = self.regs;

            if regs.events_usbreset().read() != 0 {
                regs.events_usbreset().write_value(0);
                regs.intenset().write(|w| w.set_usbreset(true));

                // Disable all endpoints except EP0
                critical_section::with(|cs| {
                    regs.epinen().write(|w| w.0 = 0x01);
                    regs.epouten().write(|w| w.0 = 0x01);
                    READY_ENDPOINTS.store(In::mask(0), Ordering::Release);
                    ISO_SOF_SEEN.store(false, Ordering::Release);
                    ISO_IN_RESET.store(true, Ordering::Release);
                    for xfer in EP_IN_XFERS.borrow(cs).borrow_mut().iter_mut().flatten() {
                        if !xfer.done && xfer.error.is_none() {
                            xfer.error = Some(EndpointDisabledError);
                        }
                    }
                    for xfer in EP_OUT_XFERS.borrow(cs).borrow_mut().iter_mut().flatten() {
                        if xfer.result.is_none() {
                            xfer.result = Some(Err(EndpointError::Disabled));
                        }
                    }
                });
                for i in 1..=7 {
                    In::waker(i).wake();
                    Out::waker(i).wake();
                }
                In::waker(ISO_EP).wake();

                return Poll::Ready(Event::Reset(Speed::Full));
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
                regs.lowpower().write(|w| w.set_lowpower(vals::Lowpower::LowPower));
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

            // Bus events come first: a reset or suspend makes a SETUP received before it stale.
            if regs.events_ep0setup().read() != 0 {
                regs.events_ep0setup().write_value(0);

                // Reset shorts
                regs.shorts().write(|_| ());

                let mut buf = [0; 8];
                buf[0] = regs.bmrequesttype().read().0 as u8;
                buf[1] = regs.brequest().read().0 as u8;
                buf[2] = regs.wvaluel().read().0 as u8;
                buf[3] = regs.wvalueh().read().0 as u8;
                buf[4] = regs.windexl().read().0 as u8;
                buf[5] = regs.windexh().read().0 as u8;
                buf[6] = regs.wlengthl().read().0 as u8;
                buf[7] = regs.wlengthh().read().0 as u8;

                return Poll::Ready(Event::Setup(buf));
            }

            regs.intenset().write(|w| w.set_ep0setup(true));
            Poll::Pending
        })
    }

    async fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        set_endpoint_stalled(self.regs, ep_addr, stalled);
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        is_endpoint_stalled(self.regs, ep_addr)
    }

    async fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        let regs = self.regs;

        let i = ep_addr.index();
        let mask = 1 << i;

        debug!("endpoint_set_enabled {:?} {}", ep_addr, enabled);

        match ep_addr.direction() {
            Direction::In => {
                critical_section::with(|cs| {
                    let mut was_enabled = false;
                    regs.epinen().modify(|w| {
                        was_enabled = (w.0 & mask) != 0;
                        if enabled { w.0 |= mask } else { w.0 &= !mask }
                    });

                    let ready_mask = In::mask(i);
                    if enabled {
                        if !was_enabled {
                            READY_ENDPOINTS.fetch_or(ready_mask, Ordering::AcqRel);
                        }
                        if i == ISO_EP {
                            ISO_IN_RESET.store(true, Ordering::Release);
                        }
                    } else {
                        READY_ENDPOINTS.fetch_and(!ready_mask, Ordering::AcqRel);
                        if i == ISO_EP {
                            ISO_IN_RESET.store(true, Ordering::Release);
                        }
                        if (1..=BULK_EP_COUNT).contains(&i) {
                            if let Some(xfer) = EP_IN_XFERS.borrow(cs).borrow_mut()[i - 1].as_mut() {
                                if !xfer.done && xfer.error.is_none() {
                                    xfer.error = Some(EndpointDisabledError);
                                }
                            }
                        }
                    }
                });

                In::waker(i).wake();
            }
            Direction::Out => {
                critical_section::with(|cs| {
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
                        if (1..=BULK_EP_COUNT).contains(&i) {
                            if let Some(xfer) = EP_OUT_XFERS.borrow(cs).borrow_mut()[i - 1].as_mut() {
                                if xfer.result.is_none() {
                                    xfer.result = Some(Err(EndpointError::Disabled));
                                }
                            }
                        }
                    }
                });

                Out::waker(i).wake();
            }
        }
    }

    fn endpoint_is_enabled(&mut self, ep_addr: EndpointAddress) -> bool {
        let i = ep_addr.index();
        let mask = 1 << i;
        match ep_addr.direction() {
            Direction::In => (self.regs.epinen().read().0 & mask) != 0,
            Direction::Out => (self.regs.epouten().read().0 & mask) != 0,
        }
    }

    #[inline]
    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        let regs = self.regs;

        if regs.lowpower().read().lowpower() == vals::Lowpower::LowPower {
            errata::pre_wakeup();

            regs.lowpower().write(|w| w.set_lowpower(vals::Lowpower::ForceNormal));

            poll_fn(|cx| {
                BUS_WAKER.register(cx.waker());
                let regs = self.regs;
                let r = regs.eventcause().read();

                if regs.events_usbreset().read() != 0 {
                    Poll::Ready(())
                } else if r.resume() {
                    Poll::Ready(())
                } else if r.usbwuallowed() {
                    regs.eventcause().write(|w| w.set_usbwuallowed(true));
                    regs.dpdmvalue().write(|w| w.set_state(vals::State::Resume));
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

    fn control_max_packet_size(&self) -> usize {
        usize::from(self.control_max_packet_size)
    }

    async fn control_data_out(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let regs = self.regs;
        let max_packet_size = self.control_max_packet_size();

        let aborted = critical_section::with(|cs| {
            if regs.events_usbreset().read() != 0
                || regs.events_ep0setup().read() != 0
                || regs.eventcause().read().suspend()
            {
                return true;
            }
            regs.events_ep0datadone().write_value(0);
            regs.tasks_ep0rcvout().write_value(1);
            *EP0_XFER.borrow(cs).borrow_mut() = Some(Ep0Xfer::Out(OutXfer::new(buf, max_packet_size)));
            regs.intenset().write(|w| {
                w.set_usbreset(true);
                w.set_ep0setup(true);
                w.set_ep0datadone(true);
            });
            false
        });
        if aborted {
            return Err(EndpointError::Disabled);
        }

        let _guard = Ep0XferGuard;
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            BUS_WAKER.register(cx.waker());
            if regs.events_usbreset().read() != 0 {
                trace!("aborted control data_out: usb reset");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if regs.eventcause().read().suspend() {
                trace!("aborted control data_out: suspend");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            if regs.events_ep0setup().read() != 0 {
                trace!("aborted control data_out: received another SETUP");
                return Poll::Ready(Err(EndpointError::Disabled));
            }
            critical_section::with(|cs| {
                let mut ep0 = EP0_XFER.borrow(cs).borrow_mut();
                if let Some(Ep0Xfer::Out(xfer)) = ep0.as_mut() {
                    if let Some(res) = xfer.result {
                        *ep0 = None;
                        return Poll::Ready(res);
                    }
                }
                Poll::Pending
            })
        })
        .await
    }

    async fn control_data_in(&mut self, data: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        let regs = self.regs;
        let max_packet_size = self.control_max_packet_size();
        let first_len = data.len().min(max_packet_size);
        let send_zlp = needs_zlp && !data.is_empty() && data.len().is_multiple_of(max_packet_size);
        let is_last = first_len == data.len() && !send_zlp;

        let aborted = critical_section::with(|cs| {
            if regs.events_usbreset().read() != 0
                || regs.events_ep0setup().read() != 0
                || regs.eventcause().read().suspend()
            {
                return true;
            }
            regs.events_ep0datadone().write_value(0);
            regs.shorts().write(|w| w.set_ep0datadone_ep0status(is_last));
            unsafe { write_dma_in_cs(cs, regs, 0, &data[..first_len]) };
            *EP0_XFER.borrow(cs).borrow_mut() = Some(Ep0Xfer::In(InXfer {
                buf_ptr: data.as_ptr(),
                buf_len: data.len(),
                offset: first_len,
                send_zlp,
                max_packet_size,
                wait_last_complete: true,
                done: false,
                error: None,
            }));
            regs.intenset().write(|w| {
                w.set_usbreset(true);
                w.set_ep0setup(true);
                w.set_ep0datadone(true);
            });
            false
        });
        if aborted {
            return Err(EndpointDisabledError);
        }

        let _guard = Ep0XferGuard;
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            BUS_WAKER.register(cx.waker());
            if regs.events_usbreset().read() != 0 {
                trace!("aborted control data_in: usb reset");
                return Poll::Ready(Err(EndpointDisabledError));
            }
            if regs.eventcause().read().suspend() {
                trace!("aborted control data_in: suspend");
                return Poll::Ready(Err(EndpointDisabledError));
            }
            if regs.events_ep0setup().read() != 0 {
                trace!("aborted control data_in: received another SETUP");
                return Poll::Ready(Err(EndpointDisabledError));
            }
            critical_section::with(|cs| {
                let mut ep0 = EP0_XFER.borrow(cs).borrow_mut();
                if let Some(Ep0Xfer::In(xfer)) = ep0.as_mut() {
                    if let Some(err) = xfer.error {
                        *ep0 = None;
                        return Poll::Ready(Err(err));
                    }
                    if xfer.done {
                        *ep0 = None;
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Pending
            })
        })
        .await
    }

    async fn control_accept(&mut self) {
        let regs = self.regs;
        regs.tasks_ep0status().write_value(1);
    }

    async fn control_reject(&mut self) {
        let regs = self.regs;
        regs.tasks_ep0stall().write_value(1);
    }

    async fn control_accept_set_address(&mut self, _addr: u8) {
        self.control_accept().await;
        // Nothing to do, the peripheral handles this.
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
        if i == ISO_EP {
            regs.epinen().read().isoin()
        } else {
            regs.epinen().read().in_(i)
        }
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
pub struct Endpoint<'d, Dir> {
    regs: pac::usbd::Usbd,
    info: EndpointInfo,
    schedule: IsoSchedule,
    iso_written_frame: Option<u32>,
    _phantom: PhantomData<(&'d (), Dir)>,
}

impl<'d, Dir> Endpoint<'d, Dir> {
    fn new(regs: pac::usbd::Usbd, info: EndpointInfo) -> Self {
        Self {
            regs,
            info,
            schedule: IsoSchedule::new(),
            iso_written_frame: None,
            _phantom: PhantomData,
        }
    }
}

impl<'d, Dir: EndpointDir> driver::Endpoint for Endpoint<'d, Dir> {
    fn info(&self) -> EndpointInfo {
        self.info
    }

    async fn wait_enabled(&mut self) {
        self.wait_enabled_state(true).await
    }

    fn is_enabled(&self) -> bool {
        let i = self.info.addr.index();
        Dir::is_enabled(self.regs, i)
    }

    async fn set_stalled(&mut self, stalled: bool) {
        set_endpoint_stalled(self.regs, self.info.addr, stalled);
    }

    fn is_stalled(&self) -> bool {
        is_endpoint_stalled(self.regs, self.info.addr)
    }
}

#[allow(private_bounds)]
impl<'d, Dir: EndpointDir> Endpoint<'d, Dir> {
    fn wait_enabled_state(&mut self, state: bool) -> impl Future<Output = ()> + use<'_, 'd, Dir> {
        let i = self.info.addr.index();
        assert!(i != 0);

        poll_fn(move |cx| {
            Dir::waker(i).register(cx.waker());
            if Dir::is_enabled(self.regs, i) == state {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }

    /// Wait for the endpoint to be disabled
    pub fn wait_disabled(&mut self) -> impl Future<Output = ()> + use<'_, 'd, Dir> {
        self.wait_enabled_state(false)
    }
}

impl<'d, Dir> Endpoint<'d, Dir> {
    async fn wait_data_ready(&mut self) -> Result<(), ()>
    where
        Dir: EndpointDir,
    {
        let i = self.info.addr.index();
        assert!(i != 0);
        poll_fn(|cx| {
            Dir::waker(i).register(cx.waker());
            let r = READY_ENDPOINTS.load(Ordering::Acquire);
            if !Dir::is_enabled(self.regs, i) {
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

unsafe fn read_dma_in_cs(
    _cs: CriticalSection,
    regs: pac::usbd::Usbd,
    i: usize,
    buf: &mut [u8],
) -> Result<usize, EndpointError> {
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

    // Do NOT write SIZE.EPOUT here. A bulk/interrupt OUT endpoint resumes
    // accepting packets when the EasyDMA transfer completes *or* when
    // SIZE.EPOUT[n] is written — either one on its own re-arms it. ENDEPOUT
    // above has already done so, and writing SIZE here arms it a second time,
    // letting the endpoint accept a further packet on top of one it has not yet
    // handed to software. That packet is overwritten in the endpoint's internal
    // buffer and lost. Initial arming, before any DMA has run, is done in
    // `endpoint_set_enabled`.

    Ok(size)
}

unsafe fn write_dma_in_cs(cs: CriticalSection, regs: pac::usbd::Usbd, i: usize, buf: &[u8]) {
    assert!(buf.len() <= if i == ISO_EP { ISO_MAX_PACKET_SIZE } else { 64 });

    if slice_in_ram(buf) {
        write_dma_inner(cs, regs, i, buf);
    } else if i == ISO_EP {
        write_dma_buffered::<1024>(cs, regs, i, buf);
    } else {
        write_dma_buffered::<64>(cs, regs, i, buf);
    }
}

// Keep Flash staging buffers off the direct DMA path.
#[inline(never)]
unsafe fn write_dma_buffered<const SIZE: usize>(cs: CriticalSection, regs: pac::usbd::Usbd, i: usize, buf: &[u8]) {
    assert!(buf.len() <= SIZE);

    let mut ram_buf: MaybeUninit<[u8; SIZE]> = MaybeUninit::uninit();
    let ptr = ram_buf.as_mut_ptr() as *mut u8;
    core::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, buf.len());
    write_dma_inner(cs, regs, i, core::slice::from_raw_parts(ptr, buf.len()));
}

unsafe fn write_dma_inner(_cs: CriticalSection, regs: pac::usbd::Usbd, i: usize, buf: &[u8]) {
    let ptr = buf.as_ptr();
    if i == ISO_EP {
        regs.isoin().ptr().write_value(ptr as u32);
        regs.isoin().maxcnt().write(|w| w.set_maxcnt(buf.len() as u16));
        regs.events_endisoin().write_value(0);
        dma_start();
        regs.tasks_startisoin().write_value(1);
        while regs.events_endisoin().read() == 0 {}
        dma_end();
        return;
    }

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

impl<'d> Endpoint<'d, Out> {
    async fn read_packet_internal(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let i = self.info.addr.index();
        assert!(i != 0);

        self.wait_data_ready().await.map_err(|_| EndpointError::Disabled)?;

        critical_section::with(|cs| unsafe { read_dma_in_cs(cs, self.regs, i, buf) })
    }
}

impl<'d> driver::EndpointOut for Endpoint<'d, Out> {
    async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let i = self.info.addr.index();
        assert!(i != 0);
        let max_packet_size = usize::from(self.info.max_packet_size);

        if i == ISO_EP {
            let mut total = 0;
            loop {
                let chunk_len = (buf.len() - total).min(max_packet_size);
                if chunk_len == 0 && total > 0 {
                    break;
                }
                let n = self.read_packet_internal(&mut buf[total..total + chunk_len]).await?;
                total += n;
                if n < max_packet_size || total == buf.len() {
                    break;
                }
            }
            return Ok(total);
        }

        let immediate = critical_section::with(|cs| {
            if !Out::is_enabled(self.regs, i) {
                return Some(Err(EndpointError::Disabled));
            }
            let mut xfer = OutXfer::new(buf, max_packet_size);
            if READY_ENDPOINTS.load(Ordering::Acquire) & Out::mask(i) != 0 {
                READY_ENDPOINTS.fetch_and(!Out::mask(i), Ordering::AcqRel);
                if xfer.on_packet(cs, self.regs, i) {
                    return xfer.result;
                }
            }
            EP_OUT_XFERS.borrow(cs).borrow_mut()[i - 1] = Some(xfer);
            None
        });
        if let Some(res) = immediate {
            return res;
        }

        let _guard = OutXferGuard(i);
        poll_fn(|cx| {
            Out::waker(i).register(cx.waker());
            critical_section::with(|cs| {
                let mut xfers = EP_OUT_XFERS.borrow(cs).borrow_mut();
                let xfer = xfers[i - 1].as_mut().unwrap();
                if let Some(res) = xfer.result {
                    xfers[i - 1] = None;
                    return Poll::Ready(res);
                }
                if !Out::is_enabled(self.regs, i) {
                    xfers[i - 1] = None;
                    return Poll::Ready(Err(EndpointError::Disabled));
                }
                Poll::Pending
            })
        })
        .await
    }
}

impl<'d> Endpoint<'d, In> {
    async fn write_packet_internal(&mut self, buf: &[u8]) -> Result<(), EndpointDisabledError> {
        let i = self.info.addr.index();
        assert!(i != 0);

        self.wait_data_ready().await.map_err(|_| EndpointDisabledError)?;

        if i == ISO_EP && !slice_in_ram(buf) {
            let mut ram_buf: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();
            let ptr = ram_buf.as_mut_ptr() as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, buf.len());
                let ram_slice = core::slice::from_raw_parts(ptr, buf.len());
                critical_section::with(|cs| write_dma_inner(cs, self.regs, i, ram_slice));
            }
        } else {
            critical_section::with(|cs| unsafe { write_dma_in_cs(cs, self.regs, i, buf) });
        }

        Ok(())
    }
}

impl<'d> driver::EndpointIn for Endpoint<'d, In> {
    async fn write_transfer(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError> {
        let i = self.info.addr.index();
        assert!(i != 0);
        let max_packet_size = usize::from(self.info.max_packet_size);

        if i == ISO_EP {
            if buf.is_empty() {
                return self.write_packet_internal(&[]).await;
            }
            for chunk in buf.chunks(max_packet_size) {
                self.write_packet_internal(chunk).await?;
            }
            if needs_zlp && buf.len().is_multiple_of(max_packet_size) {
                self.write_packet_internal(&[]).await?;
            }
            return Ok(());
        }

        let needs_wait = poll_fn(|cx| {
            In::waker(i).register(cx.waker());
            critical_section::with(|cs| {
                if !In::is_enabled(self.regs, i) {
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                if READY_ENDPOINTS.load(Ordering::Acquire) & In::mask(i) == 0 {
                    return Poll::Pending;
                }
                READY_ENDPOINTS.fetch_and(!In::mask(i), Ordering::AcqRel);

                let first_len = buf.len().min(max_packet_size);
                unsafe { write_dma_in_cs(cs, self.regs, i, &buf[..first_len]) };

                let send_zlp = needs_zlp && !buf.is_empty() && buf.len().is_multiple_of(max_packet_size);
                if first_len == buf.len() && !send_zlp {
                    return Poll::Ready(Ok(false));
                }

                EP_IN_XFERS.borrow(cs).borrow_mut()[i - 1] = Some(InXfer {
                    buf_ptr: buf.as_ptr(),
                    buf_len: buf.len(),
                    offset: first_len,
                    send_zlp,
                    max_packet_size,
                    wait_last_complete: false,
                    done: false,
                    error: None,
                });
                Poll::Ready(Ok(true))
            })
        })
        .await?;

        if !needs_wait {
            return Ok(());
        }

        let _guard = InXferGuard(i);
        poll_fn(|cx| {
            In::waker(i).register(cx.waker());
            critical_section::with(|cs| {
                let mut xfers = EP_IN_XFERS.borrow(cs).borrow_mut();
                let xfer = xfers[i - 1].as_mut().unwrap();
                if let Some(err) = xfer.error {
                    xfers[i - 1] = None;
                    return Poll::Ready(Err(err));
                }
                if !In::is_enabled(self.regs, i) {
                    xfers[i - 1] = None;
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                if xfer.done {
                    xfers[i - 1] = None;
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending
            })
        })
        .await
    }
}

/// Reserved isochronous IN slot returned by [`IsoEndpointIn::ready`].
pub struct IsoSlot<'a, 'd> {
    ep: &'a mut Endpoint<'d, In>,
    frame: u32,
}

impl IsoInSlot for IsoSlot<'_, '_> {
    fn write(self, data: &[u8]) -> Result<(), IsoWriteError> {
        if data.len() > usize::from(self.ep.info.max_packet_size) {
            return Err(IsoWriteError::BufferOverflow);
        }

        let regs = self.ep.regs;
        let written_frame = critical_section::with(|cs| {
            if !ISO_SOF_SEEN.load(Ordering::Acquire) {
                return Err(IsoWriteError::TooLate);
            }
            let now = u32::from(regs.framecntr().read().framecntr());
            if forward(now, self.frame, ISO_FRAME_SPAN) != 1 {
                return Err(IsoWriteError::TooLate);
            }
            if !In::is_enabled(regs, ISO_EP) {
                return Ok(now);
            }
            // Copies `data` into the peripheral's ISOIN buffer via EasyDMA. Hardware latches it
            // at the next SOF and automatically responds with a ZLP if uncollected on later frames.
            unsafe { write_dma_in_cs(cs, regs, ISO_EP, data) };
            Ok(now)
        })?;

        self.ep.iso_written_frame = Some(written_frame);
        self.ep.schedule.accept();
        Ok(())
    }
}

impl<'d> IsoEndpointIn for Endpoint<'d, In> {
    type Slot<'a>
        = IsoSlot<'a, 'd>
    where
        Self: 'a;

    fn queue_depth(&self) -> u8 {
        if self.info.addr.index() == ISO_EP && self.info.ep_type == EndpointType::Isochronous {
            1
        } else {
            0
        }
    }

    async fn ready(&mut self) -> Result<(Option<NonZeroU32>, Self::Slot<'_>), EndpointDisabledError> {
        assert_eq!(
            self.info.ep_type,
            EndpointType::Isochronous,
            "IsoEndpointIn requires an isochronous endpoint"
        );
        assert_eq!(self.info.addr.index(), ISO_EP);

        if !In::is_enabled(self.regs, ISO_EP) {
            return Err(EndpointDisabledError);
        }
        let reset = critical_section::with(|_| {
            let r = ISO_IN_RESET.load(Ordering::Acquire);
            if r {
                ISO_IN_RESET.store(false, Ordering::Release);
            }
            r
        });
        if reset {
            self.schedule.reset();
            self.iso_written_frame = None;
        }

        let interval = self.info.interval.frames().clamp(1, ISO_FRAME_SPAN / 2);

        // Wait until SOF is live and has advanced past the frame of any previous EasyDMA load
        // so the hardware has latched that packet and freed the EasyDMA staging buffer.
        let now = poll_fn(|cx| {
            In::waker(ISO_EP).register(cx.waker());
            if !In::is_enabled(self.regs, ISO_EP) {
                return Poll::Ready(Err(EndpointDisabledError));
            }
            if !ISO_SOF_SEEN.load(Ordering::Acquire) {
                return Poll::Pending;
            }
            let now = u32::from(self.regs.framecntr().read().framecntr());
            if self.iso_written_frame == Some(now) {
                Poll::Pending
            } else {
                Poll::Ready(Ok(now))
            }
        })
        .await?;

        let mut total_skipped: u32 = 0;
        let mut current = now;
        loop {
            let offer = self.schedule.offer(current, interval, ISO_FRAME_SPAN);
            total_skipped = total_skipped.saturating_add(offer.skipped.map_or(0, NonZeroU32::get));
            let frame = offer.frame;

            let ready = poll_fn(|cx| {
                In::waker(ISO_EP).register(cx.waker());
                if !In::is_enabled(self.regs, ISO_EP) {
                    return Poll::Ready(Err(EndpointDisabledError));
                }
                if !ISO_SOF_SEEN.load(Ordering::Acquire) {
                    return Poll::Pending;
                }
                let now = u32::from(self.regs.framecntr().read().framecntr());
                let ahead = forward(now, frame, ISO_FRAME_SPAN);
                if ahead == 1 {
                    Poll::Ready(Ok(true))
                } else if ahead == 0 || ahead > interval {
                    Poll::Ready(Ok(false))
                } else {
                    Poll::Pending
                }
            })
            .await?;

            if ready {
                return Ok((NonZeroU32::new(total_skipped), IsoSlot { ep: self, frame }));
            }

            current = u32::from(self.regs.framecntr().read().framecntr());
        }
    }
}

fn dma_start() {
    compiler_fence(Ordering::Release);
    errata::dma_start();
}

fn dma_end() {
    errata::dma_end();
    compiler_fence(Ordering::Acquire);
}

struct Allocator {
    used: u16,
}

impl Allocator {
    fn new() -> Self {
        Self { used: 0 }
    }

    fn allocate(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
    ) -> Result<usize, driver::EndpointAllocError> {
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

        let alloc_index = if let Some(addr) = ep_addr {
            // Use the specified endpoint address
            let requested_index = addr.index();
            // Validate the requested index based on endpoint type
            match ep_type {
                EndpointType::Isochronous => {
                    if requested_index != 8 {
                        return Err(driver::EndpointAllocError);
                    }
                }
                EndpointType::Control => return Err(driver::EndpointAllocError),
                EndpointType::Interrupt | EndpointType::Bulk => {
                    if requested_index < 1 || requested_index > 7 {
                        return Err(driver::EndpointAllocError);
                    }
                }
            }
            requested_index
        } else {
            // Allocate any available endpoint
            match ep_type {
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

    /// Works around Erratum 199: "USBD cannot receive tasks during DMA". A USBD
    /// task triggered while an EasyDMA transfer is in progress is silently
    /// dropped instead of being performed.
    ///
    /// Applies to every nRF52840 revision — Nordic's `nrf52_errata_199()`
    /// returns true for all of them, including its default arm — so unlike
    /// errata 187 and 171 above there is no revision to check against.
    ///
    /// Called around every USBD EasyDMA transfer, matching nrfx's
    /// `usbd_dma_pending_set` / `usbd_dma_pending_clear`.
    /// based on https://docs.nordicsemi.com/r/bundle/errata_nrf52840_rev3/page/err/nrf52840/rev3/latest/anomaly_840_199.html
    pub fn dma_start() {
        #[cfg(feature = "nrf52840")]
        unsafe {
            poke(0x40027C1C, 0x00000082);
        }
    }

    /// Counterpart to [`dma_start`]. Works around Erratum 199.
    pub fn dma_end() {
        #[cfg(feature = "nrf52840")]
        unsafe {
            poke(0x40027C1C, 0x00000000);
        }
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
