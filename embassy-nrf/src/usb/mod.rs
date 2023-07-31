//! Universal Serial Bus (USB) driver.

#![macro_use]

pub mod vbus_detect;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};
use core::task::Poll;

use cortex_m::peripheral::NVIC;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{Direction, EndpointAddress, EndpointError, EndpointInfo, EndpointType, Event, Unsupported};
use pac::usbd::RegisterBlock;

use self::vbus_detect::VbusDetect;
use crate::interrupt::typelevel::Interrupt;
use crate::util::slice_in_ram;
use crate::{interrupt, pac, Peripheral};

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP0_WAKER: AtomicWaker = NEW_AW;
static EP_IN_WAKERS: [AtomicWaker; 8] = [NEW_AW; 8];
static EP_OUT_WAKERS: [AtomicWaker; 8] = [NEW_AW; 8];
static READY_ENDPOINTS: AtomicU32 = AtomicU32::new(0);

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        if regs.events_usbreset.read().bits() != 0 {
            regs.intenclr.write(|w| w.usbreset().clear());
            BUS_WAKER.wake();
            EP0_WAKER.wake();
        }

        if regs.events_ep0setup.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0setup().clear());
            EP0_WAKER.wake();
        }

        if regs.events_ep0datadone.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0datadone().clear());
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
        if regs.events_usbevent.read().bits() != 0 {
            regs.events_usbevent.reset();
            BUS_WAKER.wake();
        }

        if regs.events_epdata.read().bits() != 0 {
            regs.events_epdata.reset();

            let r = regs.epdatastatus.read().bits();
            regs.epdatastatus.write(|w| unsafe { w.bits(r) });
            READY_ENDPOINTS.fetch_or(r, Ordering::AcqRel);
            for i in 1..=7 {
                if r & In::mask(i) != 0 {
                    In::waker(i).wake();
                }
                if r & Out::mask(i) != 0 {
                    Out::waker(i).wake();
                }
            }
        }
    }
}

/// USB driver.
pub struct Driver<'d, T: Instance, V: VbusDetect> {
    _p: PeripheralRef<'d, T>,
    alloc_in: Allocator,
    alloc_out: Allocator,
    vbus_detect: V,
}

impl<'d, T: Instance, V: VbusDetect> Driver<'d, T, V> {
    /// Create a new USB driver.
    pub fn new(
        usb: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        vbus_detect: V,
    ) -> Self {
        into_ref!(usb);

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
    _p: PeripheralRef<'d, T>,
    power_available: bool,
    vbus_detect: V,
}

impl<'d, T: Instance, V: VbusDetect> driver::Bus for Bus<'d, T, V> {
    async fn enable(&mut self) {
        let regs = T::regs();

        errata::pre_enable();

        regs.enable.write(|w| w.enable().enabled());

        // Wait until the peripheral is ready.
        regs.intenset.write(|w| w.usbevent().set_bit());
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            if regs.eventcause.read().ready().is_ready() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        regs.eventcause.write(|w| w.ready().clear_bit_by_one());

        errata::post_enable();

        unsafe { NVIC::unmask(pac::Interrupt::USBD) };

        regs.intenset.write(|w| {
            w.usbreset().set_bit();
            w.usbevent().set_bit();
            w.epdata().set_bit();
            w
        });

        if self.vbus_detect.wait_power_ready().await.is_ok() {
            // Enable the USB pullup, allowing enumeration.
            regs.usbpullup.write(|w| w.connect().enabled());
            trace!("enabled");
        } else {
            trace!("usb power not ready due to usb removal");
        }
    }

    async fn disable(&mut self) {
        let regs = T::regs();
        regs.enable.write(|x| x.enable().disabled());
    }

    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            BUS_WAKER.register(cx.waker());
            let regs = T::regs();

            if regs.events_usbreset.read().bits() != 0 {
                regs.events_usbreset.reset();
                regs.intenset.write(|w| w.usbreset().set());

                // Disable all endpoints except EP0
                regs.epinen.write(|w| unsafe { w.bits(0x01) });
                regs.epouten.write(|w| unsafe { w.bits(0x01) });
                READY_ENDPOINTS.store(In::mask(0), Ordering::Release);
                for i in 1..=7 {
                    In::waker(i).wake();
                    Out::waker(i).wake();
                }

                return Poll::Ready(Event::Reset);
            }

            let r = regs.eventcause.read();

            if r.isooutcrc().bit() {
                regs.eventcause.write(|w| w.isooutcrc().detected());
                trace!("USB event: isooutcrc");
            }
            if r.usbwuallowed().bit() {
                regs.eventcause.write(|w| w.usbwuallowed().allowed());
                trace!("USB event: usbwuallowed");
            }
            if r.suspend().bit() {
                regs.eventcause.write(|w| w.suspend().detected());
                regs.lowpower.write(|w| w.lowpower().low_power());
                return Poll::Ready(Event::Suspend);
            }
            if r.resume().bit() {
                regs.eventcause.write(|w| w.resume().detected());
                return Poll::Ready(Event::Resume);
            }
            if r.ready().bit() {
                regs.eventcause.write(|w| w.ready().ready());
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
        .await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        let regs = T::regs();
        unsafe {
            if ep_addr.index() == 0 {
                regs.tasks_ep0stall.write(|w| w.tasks_ep0stall().bit(stalled));
            } else {
                regs.epstall.write(|w| {
                    w.ep().bits(ep_addr.index() as u8 & 0b111);
                    w.io().bit(ep_addr.is_in());
                    w.stall().bit(stalled)
                });
            }
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let regs = T::regs();
        let i = ep_addr.index();
        match ep_addr.direction() {
            Direction::Out => regs.halted.epout[i].read().getstatus().is_halted(),
            Direction::In => regs.halted.epin[i].read().getstatus().is_halted(),
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
                regs.epinen.modify(|r, w| {
                    let mut bits = r.bits();
                    was_enabled = (bits & mask) != 0;
                    if enabled {
                        bits |= mask
                    } else {
                        bits &= !mask
                    }
                    unsafe { w.bits(bits) }
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
                regs.epouten.modify(|r, w| {
                    let mut bits = r.bits();
                    if enabled {
                        bits |= mask
                    } else {
                        bits &= !mask
                    }
                    unsafe { w.bits(bits) }
                });

                let ready_mask = Out::mask(i);
                if enabled {
                    // when first enabled, bulk/interrupt OUT endpoints will *not* receive data (the
                    // peripheral will NAK all incoming packets) until we write a zero to the SIZE
                    // register (see figure 203 of the 52840 manual). To avoid that we write a 0 to the
                    // SIZE register
                    regs.size.epout[i].reset();
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

        if regs.lowpower.read().lowpower().is_low_power() {
            errata::pre_wakeup();

            regs.lowpower.write(|w| w.lowpower().force_normal());

            poll_fn(|cx| {
                BUS_WAKER.register(cx.waker());
                let regs = T::regs();
                let r = regs.eventcause.read();

                if regs.events_usbreset.read().bits() != 0 {
                    Poll::Ready(())
                } else if r.resume().bit() {
                    Poll::Ready(())
                } else if r.usbwuallowed().bit() {
                    regs.eventcause.write(|w| w.usbwuallowed().allowed());

                    regs.dpdmvalue.write(|w| w.state().resume());
                    regs.tasks_dpdmdrive.write(|w| w.tasks_dpdmdrive().set_bit());

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
    fn is_enabled(regs: &RegisterBlock, i: usize) -> bool;
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
    fn is_enabled(regs: &RegisterBlock, i: usize) -> bool {
        (regs.epinen.read().bits() & (1 << i)) != 0
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
    fn is_enabled(regs: &RegisterBlock, i: usize) -> bool {
        (regs.epouten.read().bits() & (1 << i)) != 0
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
        let i = self.info.addr.index();
        assert!(i != 0);

        poll_fn(move |cx| {
            Dir::waker(i).register(cx.waker());
            if Dir::is_enabled(T::regs(), i) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
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
    let size = regs.size.epout[i].read().bits() as usize;
    if size > buf.len() {
        return Err(EndpointError::BufferOverflow);
    }

    let epout = [
        &regs.epout0,
        &regs.epout1,
        &regs.epout2,
        &regs.epout3,
        &regs.epout4,
        &regs.epout5,
        &regs.epout6,
        &regs.epout7,
    ];
    epout[i].ptr.write(|w| w.bits(buf.as_ptr() as u32));
    // MAXCNT must match SIZE
    epout[i].maxcnt.write(|w| w.bits(size as u32));

    dma_start();
    regs.events_endepout[i].reset();
    regs.tasks_startepout[i].write(|w| w.tasks_startepout().set_bit());
    while regs.events_endepout[i].read().events_endepout().bit_is_clear() {}
    regs.events_endepout[i].reset();
    dma_end();

    regs.size.epout[i].reset();

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

    let epin = [
        &regs.epin0,
        &regs.epin1,
        &regs.epin2,
        &regs.epin3,
        &regs.epin4,
        &regs.epin5,
        &regs.epin6,
        &regs.epin7,
    ];

    // Set the buffer length so the right number of bytes are transmitted.
    // Safety: `buf.len()` has been checked to be <= the max buffer length.
    epin[i].ptr.write(|w| w.bits(ptr as u32));
    epin[i].maxcnt.write(|w| w.maxcnt().bits(buf.len() as u8));

    regs.events_endepin[i].reset();

    dma_start();
    regs.tasks_startepin[i].write(|w| w.bits(1));
    while regs.events_endepin[i].read().bits() == 0 {}
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
    _p: PeripheralRef<'d, T>,
    max_packet_size: u16,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    async fn setup(&mut self) -> [u8; 8] {
        let regs = T::regs();

        // Reset shorts
        regs.shorts.write(|w| w);

        // Wait for SETUP packet
        regs.intenset.write(|w| w.ep0setup().set());
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0setup.read().bits() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        regs.events_ep0setup.reset();

        let mut buf = [0; 8];
        buf[0] = regs.bmrequesttype.read().bits() as u8;
        buf[1] = regs.brequest.read().brequest().bits();
        buf[2] = regs.wvaluel.read().wvaluel().bits();
        buf[3] = regs.wvalueh.read().wvalueh().bits();
        buf[4] = regs.windexl.read().windexl().bits();
        buf[5] = regs.windexh.read().windexh().bits();
        buf[6] = regs.wlengthl.read().wlengthl().bits();
        buf[7] = regs.wlengthh.read().wlengthh().bits();

        buf
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        let regs = T::regs();

        regs.events_ep0datadone.reset();

        // This starts a RX on EP0. events_ep0datadone notifies when done.
        regs.tasks_ep0rcvout.write(|w| w.tasks_ep0rcvout().set_bit());

        // Wait until ready
        regs.intenset.write(|w| {
            w.usbreset().set();
            w.ep0setup().set();
            w.ep0datadone().set()
        });
        poll_fn(|cx| {
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0datadone.read().bits() != 0 {
                Poll::Ready(Ok(()))
            } else if regs.events_usbreset.read().bits() != 0 {
                trace!("aborted control data_out: usb reset");
                Poll::Ready(Err(EndpointError::Disabled))
            } else if regs.events_ep0setup.read().bits() != 0 {
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
        regs.events_ep0datadone.reset();

        regs.shorts.write(|w| w.ep0datadone_ep0status().bit(last));

        // This starts a TX on EP0. events_ep0datadone notifies when done.
        unsafe { write_dma::<T>(0, buf) }

        regs.intenset.write(|w| {
            w.usbreset().set();
            w.ep0setup().set();
            w.ep0datadone().set()
        });

        poll_fn(|cx| {
            cx.waker().wake_by_ref();
            EP0_WAKER.register(cx.waker());
            let regs = T::regs();
            if regs.events_ep0datadone.read().bits() != 0 {
                Poll::Ready(Ok(()))
            } else if regs.events_usbreset.read().bits() != 0 {
                trace!("aborted control data_in: usb reset");
                Poll::Ready(Err(EndpointError::Disabled))
            } else if regs.events_ep0setup.read().bits() != 0 {
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
        regs.tasks_ep0status.write(|w| w.tasks_ep0status().bit(true));
    }

    async fn reject(&mut self) {
        let regs = T::regs();
        regs.tasks_ep0stall.write(|w| w.tasks_ep0stall().bit(true));
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

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::usbd::RegisterBlock;
    }
}

/// USB peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_usb {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::usb::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::usbd::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
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
