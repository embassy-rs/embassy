#![macro_use]

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::time::{with_timeout, Duration};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use embassy_usb::control::Request;
use embassy_usb::driver::{self, Event, ReadError, WriteError};
use embassy_usb::types::{EndpointAddress, EndpointInfo, EndpointType, UsbDirection};
use futures::future::poll_fn;
use futures::Future;
use pac::NVIC;

pub use embassy_usb;

use crate::interrupt::Interrupt;
use crate::pac;
use crate::util::slice_in_ram;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP_IN_WAKERS: [AtomicWaker; 9] = [NEW_AW; 9];
static EP_OUT_WAKERS: [AtomicWaker; 9] = [NEW_AW; 9];
static READY_ENDPOINTS: AtomicU32 = AtomicU32::new(0);

pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    alloc_in: Allocator,
    alloc_out: Allocator,
}

impl<'d, T: Instance> Driver<'d, T> {
    pub fn new(
        _usb: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
    ) -> Self {
        unborrow!(irq);
        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
            alloc_in: Allocator::new(),
            alloc_out: Allocator::new(),
        }
    }

    fn on_interrupt(_: *mut ()) {
        let regs = T::regs();

        if regs.events_usbreset.read().bits() != 0 {
            regs.intenclr.write(|w| w.usbreset().clear());
            BUS_WAKER.wake();
        }

        if regs.events_ep0setup.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0setup().clear());
            EP_OUT_WAKERS[0].wake();
        }

        if regs.events_ep0datadone.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0datadone().clear());
            EP_IN_WAKERS[0].wake();
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
            //regs.intenclr.write(|w| w.usbevent().clear());
            BUS_WAKER.wake();
        }

        if regs.events_epdata.read().bits() != 0 {
            regs.events_epdata.reset();

            let r = regs.epdatastatus.read().bits();
            regs.epdatastatus.write(|w| unsafe { w.bits(r) });
            READY_ENDPOINTS.fetch_or(r, Ordering::AcqRel);
            for i in 1..=7 {
                if r & (1 << i) != 0 {
                    EP_IN_WAKERS[i].wake();
                }
                if r & (1 << (i + 16)) != 0 {
                    EP_OUT_WAKERS[i].wake();
                }
            }
        }
    }

    fn set_stalled(ep_addr: EndpointAddress, stalled: bool) {
        let regs = T::regs();

        unsafe {
            if ep_addr.index() == 0 {
                regs.tasks_ep0stall
                    .write(|w| w.tasks_ep0stall().bit(stalled));
            } else {
                regs.epstall.write(|w| {
                    w.ep().bits(ep_addr.index() as u8 & 0b111);
                    w.io().bit(ep_addr.is_in());
                    w.stall().bit(stalled)
                });
            }
        }

        //if stalled {
        //    self.busy_in_endpoints &= !(1 << ep_addr.index());
        //}
    }

    fn is_stalled(ep_addr: EndpointAddress) -> bool {
        let regs = T::regs();

        let i = ep_addr.index();
        match ep_addr.direction() {
            UsbDirection::Out => regs.halted.epout[i].read().getstatus().is_halted(),
            UsbDirection::In => regs.halted.epin[i].read().getstatus().is_halted(),
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
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        let index = self
            .alloc_in
            .allocate(ep_addr, ep_type, max_packet_size, interval)?;
        let ep_addr = EndpointAddress::from_parts(index, UsbDirection::In);
        Ok(Endpoint::new(EndpointInfo {
            addr: ep_addr,
            ep_type,
            max_packet_size,
            interval,
        }))
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        let index = self
            .alloc_out
            .allocate(ep_addr, ep_type, max_packet_size, interval)?;
        let ep_addr = EndpointAddress::from_parts(index, UsbDirection::Out);
        Ok(Endpoint::new(EndpointInfo {
            addr: ep_addr,
            ep_type,
            max_packet_size,
            interval,
        }))
    }

    fn alloc_control_pipe(
        &mut self,
        max_packet_size: u16,
    ) -> Result<Self::ControlPipe, driver::EndpointAllocError> {
        self.alloc_endpoint_out(Some(0x00.into()), EndpointType::Control, max_packet_size, 0)?;
        self.alloc_endpoint_in(Some(0x80.into()), EndpointType::Control, max_packet_size, 0)?;
        Ok(ControlPipe {
            _phantom: PhantomData,
            max_packet_size,
            request: None,
        })
    }

    fn enable(self) -> Self::Bus {
        let regs = T::regs();

        errata::pre_enable();

        regs.enable.write(|w| w.enable().enabled());

        // Wait until the peripheral is ready.
        while !regs.eventcause.read().ready().is_ready() {}
        regs.eventcause.write(|w| w.ready().set_bit()); // Write 1 to clear.

        errata::post_enable();

        unsafe { NVIC::unmask(pac::Interrupt::USBD) };

        regs.intenset.write(|w| {
            w.usbreset().set_bit();
            w.usbevent().set_bit();
            w.epdata().set_bit();
            w
        });
        // Enable the USB pullup, allowing enumeration.
        regs.usbpullup.write(|w| w.connect().enabled());
        info!("enabled");

        Bus {
            phantom: PhantomData,
            alloc_in: self.alloc_in,
            alloc_out: self.alloc_out,
        }
    }
}

pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    alloc_in: Allocator,
    alloc_out: Allocator,
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    type PollFuture<'a> = impl Future<Output = Event> + 'a where Self: 'a;

    fn poll<'a>(&'a mut self) -> Self::PollFuture<'a> {
        poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            let regs = T::regs();

            if regs.events_usbreset.read().bits() != 0 {
                regs.events_usbreset.reset();
                regs.intenset.write(|w| w.usbreset().set());
                return Poll::Ready(Event::Reset);
            }

            let r = regs.eventcause.read();

            if r.isooutcrc().bit() {
                regs.eventcause.write(|w| w.isooutcrc().set_bit());
                info!("USB event: isooutcrc");
            }
            if r.usbwuallowed().bit() {
                regs.eventcause.write(|w| w.usbwuallowed().set_bit());
                info!("USB event: usbwuallowed");
            }
            if r.suspend().bit() {
                regs.eventcause.write(|w| w.suspend().set_bit());
                info!("USB event: suspend");
            }
            if r.resume().bit() {
                regs.eventcause.write(|w| w.resume().set_bit());
                info!("USB event: resume");
            }
            if r.ready().bit() {
                regs.eventcause.write(|w| w.ready().set_bit());
                info!("USB event: ready");
            }

            Poll::Pending
        })
    }

    #[inline]
    fn reset(&mut self) {
        let regs = T::regs();

        // TODO: Initialize ISO buffers

        // XXX this is not spec compliant; the endpoints should only be enabled after the device
        // has been put in the Configured state. However, usb-device provides no hook to do that
        unsafe {
            regs.epinen.write(|w| w.bits(self.alloc_in.used.into()));
            regs.epouten.write(|w| w.bits(self.alloc_out.used.into()));
        }

        for i in 1..8 {
            let out_enabled = self.alloc_out.used & (1 << i) != 0;

            // when first enabled, bulk/interrupt OUT endpoints will *not* receive data (the
            // peripheral will NAK all incoming packets) until we write a zero to the SIZE
            // register (see figure 203 of the 52840 manual). To avoid that we write a 0 to the
            // SIZE register
            if out_enabled {
                regs.size.epout[i].reset();
            }
        }

        // IN endpoints (low bits) default to ready.
        // OUT endpoints (high bits) default to NOT ready, they become ready when data comes in.
        READY_ENDPOINTS.store(0x0000FFFF, Ordering::Release);
    }

    #[inline]
    fn set_device_address(&mut self, _addr: u8) {
        // Nothing to do, the peripheral handles this.
    }

    fn set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        Driver::<T>::set_stalled(ep_addr, stalled)
    }

    fn is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        Driver::<T>::is_stalled(ep_addr)
    }

    #[inline]
    fn suspend(&mut self) {
        let regs = T::regs();
        regs.lowpower.write(|w| w.lowpower().low_power());
    }

    #[inline]
    fn resume(&mut self) {
        let regs = T::regs();

        errata::pre_wakeup();

        regs.lowpower.write(|w| w.lowpower().force_normal());
    }
}

pub enum Out {}
pub enum In {}

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

impl<'d, T: Instance, Dir> driver::Endpoint for Endpoint<'d, T, Dir> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    fn set_stalled(&self, stalled: bool) {
        Driver::<T>::set_stalled(self.info.addr, stalled)
    }

    fn is_stalled(&self) -> bool {
        Driver::<T>::is_stalled(self.info.addr)
    }
}

unsafe fn read_dma<T: Instance>(i: usize, buf: &mut [u8]) -> Result<usize, ReadError> {
    let regs = T::regs();

    // Check that the packet fits into the buffer
    let size = regs.size.epout[i].read().bits() as usize;
    if size > buf.len() {
        return Err(ReadError::BufferOverflow);
    }

    if i == 0 {
        regs.events_ep0datadone.reset();
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
    while regs.events_endepout[i]
        .read()
        .events_endepout()
        .bit_is_clear()
    {}
    regs.events_endepout[i].reset();
    dma_end();

    regs.size.epout[i].reset();

    Ok(size)
}

unsafe fn write_dma<T: Instance>(i: usize, buf: &[u8]) -> Result<(), WriteError> {
    let regs = T::regs();
    if buf.len() > 64 {
        return Err(WriteError::BufferOverflow);
    }

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

    Ok(())
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, ReadError>> + 'a where Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let i = self.info.addr.index();
            assert!(i != 0);

            // Wait until ready
            poll_fn(|cx| {
                EP_OUT_WAKERS[i].register(cx.waker());
                let r = READY_ENDPOINTS.load(Ordering::Acquire);
                if r & (1 << (i + 16)) != 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            // Mark as not ready
            READY_ENDPOINTS.fetch_and(!(1 << (i + 16)), Ordering::AcqRel);

            unsafe { read_dma::<T>(i, buf) }
        }
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    type WriteFuture<'a> = impl Future<Output = Result<(), WriteError>> + 'a where Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            let i = self.info.addr.index();
            assert!(i != 0);

            // Wait until ready.
            poll_fn(|cx| {
                EP_IN_WAKERS[i].register(cx.waker());
                let r = READY_ENDPOINTS.load(Ordering::Acquire);
                if r & (1 << i) != 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            // Mark as not ready
            READY_ENDPOINTS.fetch_and(!(1 << i), Ordering::AcqRel);

            unsafe { write_dma::<T>(i, buf) }
        }
    }
}

pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
    request: Option<Request>,
}

impl<'d, T: Instance> ControlPipe<'d, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
        let regs = T::regs();

        // Wait until ready
        regs.intenset.write(|w| w.ep0datadone().set());
        poll_fn(|cx| {
            EP_OUT_WAKERS[0].register(cx.waker());
            let regs = T::regs();
            if regs
                .events_ep0datadone
                .read()
                .events_ep0datadone()
                .bit_is_set()
            {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        unsafe { read_dma::<T>(0, buf) }
    }

    async fn write(&mut self, buf: &[u8], last_chunk: bool) {
        let regs = T::regs();
        regs.events_ep0datadone.reset();
        unsafe {
            write_dma::<T>(0, buf).unwrap();
        }

        regs.shorts
            .modify(|_, w| w.ep0datadone_ep0status().bit(last_chunk));

        regs.intenset.write(|w| w.ep0datadone().set());
        let res = with_timeout(
            Duration::from_millis(10),
            poll_fn(|cx| {
                EP_IN_WAKERS[0].register(cx.waker());
                let regs = T::regs();
                if regs.events_ep0datadone.read().bits() != 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }),
        )
        .await;

        if res.is_err() {
            error!("ControlPipe::write timed out.");
        }
    }
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    type SetupFuture<'a> = impl Future<Output = Request> + 'a where Self: 'a;
    type DataOutFuture<'a> = impl Future<Output = Result<usize, ReadError>> + 'a where Self: 'a;
    type AcceptInFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn setup<'a>(&'a mut self) -> Self::SetupFuture<'a> {
        async move {
            assert!(self.request.is_none());

            let regs = T::regs();

            // Wait for SETUP packet
            regs.intenset.write(|w| w.ep0setup().set());
            poll_fn(|cx| {
                EP_OUT_WAKERS[0].register(cx.waker());
                let regs = T::regs();
                if regs.events_ep0setup.read().bits() != 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            // Reset shorts
            regs.shorts
                .modify(|_, w| w.ep0datadone_ep0status().clear_bit());
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

            let req = Request::parse(&buf);

            if req.direction == UsbDirection::Out {
                regs.tasks_ep0rcvout
                    .write(|w| w.tasks_ep0rcvout().set_bit());
            }

            self.request = Some(req);
            req
        }
    }

    fn data_out<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::DataOutFuture<'a> {
        async move {
            let req = self.request.unwrap();
            assert_eq!(req.direction, UsbDirection::Out);
            assert!(req.length > 0);

            let req_length = usize::from(req.length);
            let max_packet_size = usize::from(self.max_packet_size);
            let mut total = 0;
            for chunk in buf.chunks_mut(max_packet_size) {
                let size = self.read(chunk).await?;
                total += size;
                if size < max_packet_size || total == req_length {
                    break;
                }
            }

            Ok(total)
        }
    }

    fn accept(&mut self) {
        let regs = T::regs();
        regs.tasks_ep0status
            .write(|w| w.tasks_ep0status().bit(true));
        self.request = None;
    }

    fn accept_in<'a>(&'a mut self, buf: &'a [u8]) -> Self::AcceptInFuture<'a> {
        async move {
            info!("control accept {=[u8]:x}", buf);
            let req = self.request.unwrap();
            assert_eq!(req.direction, UsbDirection::In);

            let req_len = usize::from(req.length);
            let len = buf.len().min(req_len);
            let need_zlp = len != req_len && (len % usize::from(self.max_packet_size)) == 0;
            let mut chunks = buf[0..len]
                .chunks(usize::from(self.max_packet_size))
                .chain(need_zlp.then(|| -> &[u8] { &[] }));
            while let Some(chunk) = chunks.next() {
                self.write(chunk, chunks.size_hint().0 == 0).await;
            }

            self.request = None;
        }
    }

    fn reject(&mut self) {
        let regs = T::regs();
        regs.tasks_ep0stall.write(|w| w.tasks_ep0stall().bit(true));
        self.request = None;
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
    // Buffers can be up to 64 Bytes since this is a Full-Speed implementation.
    lens: [u8; 9],
}

impl Allocator {
    fn new() -> Self {
        Self {
            used: 0,
            lens: [0; 9],
        }
    }

    fn allocate(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        _interval: u8,
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

        let alloc_index = if let Some(ep_addr) = ep_addr {
            match (ep_addr.index(), ep_type) {
                (0, EndpointType::Control) => {}
                (8, EndpointType::Isochronous) => {}
                (n, EndpointType::Bulk) | (n, EndpointType::Interrupt) if n >= 1 && n <= 7 => {}
                _ => return Err(driver::EndpointAllocError),
            }

            ep_addr.index()
        } else {
            match ep_type {
                EndpointType::Isochronous => 8,
                EndpointType::Control => 0,
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
        self.lens[alloc_index] = max_packet_size as u8;

        Ok(alloc_index)
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::usbd::RegisterBlock;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static + Send {
    type Interrupt: Interrupt;
}

macro_rules! impl_usb {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::usb::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::usbd::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
        }
        impl crate::usb::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

mod errata {

    /// Writes `val` to `addr`. Used to apply Errata workarounds.
    unsafe fn poke(addr: u32, val: u32) {
        (addr as *mut u32).write_volatile(val);
    }

    /// Reads 32 bits from `addr`.
    unsafe fn peek(addr: u32) -> u32 {
        (addr as *mut u32).read_volatile()
    }

    pub fn pre_enable() {
        // Works around Erratum 187 on chip revisions 1 and 2.
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
        unsafe {
            poke(0x4006EC00, 0x00009375);
            poke(0x4006ED14, 0x00000000);
            poke(0x4006EC00, 0x00009375);
        }
    }

    pub fn pre_wakeup() {
        // Works around Erratum 171 on chip revisions 1 and 2.

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

        unsafe {
            if peek(0x4006EC00) == 0x00000000 {
                poke(0x4006EC00, 0x00009375);
            }

            poke(0x4006EC14, 0x00000000);
            poke(0x4006EC00, 0x00009375);
        }
    }
}
