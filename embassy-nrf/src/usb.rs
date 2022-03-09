#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::time::{with_timeout, Duration};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use embassy_usb::driver::{self, ReadError, WriteError};
use embassy_usb::types::{EndpointAddress, EndpointInfo, EndpointType, UsbDirection};
use futures::future::poll_fn;
use futures::Future;
use pac::NVIC;

pub use embassy_usb;

use crate::interrupt::Interrupt;
use crate::pac;

static EP0_WAKER: AtomicWaker = AtomicWaker::new();

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

        if regs.events_ep0setup.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0setup().clear());
            EP0_WAKER.wake();
        }
        if regs.events_ep0datadone.read().bits() != 0 {
            regs.intenclr.write(|w| w.ep0datadone().clear());
            EP0_WAKER.wake();
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
        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: ep_addr,
                ep_type,
                max_packet_size,
                interval,
            },
        })
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
        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: ep_addr,
                ep_type,
                max_packet_size,
                interval,
            },
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

        //self.busy_in_endpoints = 0;
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

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    type ReadFuture<'a>
    where
        Self: 'a,
    = impl Future<Output = Result<usize, ReadError>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let regs = T::regs();

            if buf.len() == 0 {
                regs.tasks_ep0status.write(|w| unsafe { w.bits(1) });
                return Ok(0);
            }

            // Wait for SETUP packet
            regs.events_ep0setup.reset();
            regs.intenset.write(|w| w.ep0setup().set());
            poll_fn(|cx| {
                EP0_WAKER.register(cx.waker());
                if regs.events_ep0setup.read().bits() != 0 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            info!("got SETUP");

            if buf.len() < 8 {
                return Err(ReadError::BufferOverflow);
            }

            buf[0] = regs.bmrequesttype.read().bits() as u8;
            buf[1] = regs.brequest.read().brequest().bits();
            buf[2] = regs.wvaluel.read().wvaluel().bits();
            buf[3] = regs.wvalueh.read().wvalueh().bits();
            buf[4] = regs.windexl.read().windexl().bits();
            buf[5] = regs.windexh.read().windexh().bits();
            buf[6] = regs.wlengthl.read().wlengthl().bits();
            buf[7] = regs.wlengthh.read().wlengthh().bits();

            Ok(8)
        }
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    type WriteFuture<'a>
    where
        Self: 'a,
    = impl Future<Output = Result<(), WriteError>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            info!("write: {:x}", buf);

            let regs = T::regs();

            let ptr = buf.as_ptr() as u32;
            let len = buf.len() as u32;
            regs.epin0.ptr.write(|w| unsafe { w.bits(ptr) });
            regs.epin0.maxcnt.write(|w| unsafe { w.bits(len) });

            regs.events_ep0datadone.reset();
            regs.events_endepin[0].reset();

            dma_start();

            regs.tasks_startepin[0].write(|w| unsafe { w.bits(1) });
            info!("write: waiting for endepin...");
            while regs.events_endepin[0].read().bits() == 0 {}

            dma_end();

            info!("write: waiting for ep0datadone...");
            regs.intenset.write(|w| w.ep0datadone().set());
            let res = with_timeout(
                Duration::from_millis(10),
                poll_fn(|cx| {
                    EP0_WAKER.register(cx.waker());
                    if regs.events_ep0datadone.read().bits() != 0 {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }),
            )
            .await;

            if res.is_err() {
                // todo wrong error
                return Err(driver::WriteError::BufferOverflow);
            }

            info!("write done");

            Ok(())
        }
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

        let alloc_index = match ep_type {
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
