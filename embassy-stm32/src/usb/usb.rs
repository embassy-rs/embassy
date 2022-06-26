#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::Ordering;
use core::task::Poll;

use atomic_polyfill::{AtomicBool, AtomicU8};
use embassy::time::{block_for, Duration};
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use embassy_usb::driver::{self, EndpointAllocError, EndpointError, Event, Unsupported};
use embassy_usb::types::{EndpointAddress, EndpointInfo, EndpointType, UsbDirection};
use futures::future::poll_fn;
use futures::Future;
use pac::common::{Reg, RW};
use pac::usb::vals::{EpType, Stat};

use super::{DmPin, DpPin, Instance};
use crate::gpio::sealed::AFType;
use crate::interrupt::InterruptExt;
use crate::pac::usb::regs;
use crate::rcc::sealed::RccPeripheral;
use crate::{pac, Unborrow};

const EP_COUNT: usize = 8;

#[cfg(any(usb_v1_x1, usb_v1_x2))]
const EP_MEMORY_SIZE: usize = 512;
#[cfg(not(any(usb_v1_x1, usb_v1_x2)))]
const EP_MEMORY_SIZE: usize = 1024;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP0_SETUP: AtomicBool = AtomicBool::new(false);
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static IRQ_FLAGS: AtomicU8 = AtomicU8::new(0);
const IRQ_FLAG_RESET: u8 = 0x01;
const IRQ_FLAG_SUSPEND: u8 = 0x02;
const IRQ_FLAG_RESUME: u8 = 0x04;

fn convert_type(t: EndpointType) -> EpType {
    match t {
        EndpointType::Bulk => EpType::BULK,
        EndpointType::Control => EpType::CONTROL,
        EndpointType::Interrupt => EpType::INTERRUPT,
        EndpointType::Isochronous => EpType::ISO,
    }
}

fn invariant(mut r: regs::Epr) -> regs::Epr {
    r.set_ctr_rx(true); // don't clear
    r.set_ctr_tx(true); // don't clear
    r.set_dtog_rx(false); // don't toggle
    r.set_dtog_tx(false); // don't toggle
    r.set_stat_rx(Stat(0));
    r.set_stat_tx(Stat(0));
    r
}

// Returns (actual_len, len_bits)
fn calc_out_len(len: u16) -> (u16, u16) {
    match len {
        2..=62 => ((len + 1) / 2 * 2, ((len + 1) / 2) << 10),
        63..=480 => ((len + 31) / 32 * 32, (((len + 31) / 32 - 1) << 10) | 0x8000),
        _ => panic!("invalid OUT length {}", len),
    }
}
fn ep_in_addr<T: Instance>(index: usize) -> Reg<u16, RW> {
    T::regs().ep_mem(index * 4 + 0)
}
fn ep_in_len<T: Instance>(index: usize) -> Reg<u16, RW> {
    T::regs().ep_mem(index * 4 + 1)
}
fn ep_out_addr<T: Instance>(index: usize) -> Reg<u16, RW> {
    T::regs().ep_mem(index * 4 + 2)
}
fn ep_out_len<T: Instance>(index: usize) -> Reg<u16, RW> {
    T::regs().ep_mem(index * 4 + 3)
}

struct EndpointBuffer<T: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<T>,
}

impl<T: Instance> EndpointBuffer<T> {
    fn read(&mut self, buf: &mut [u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..((buf.len() + 1) / 2) {
            let val = unsafe { T::regs().ep_mem(self.addr as usize / 2 + i).read() };
            buf[i * 2] = val as u8;
            if i * 2 + 1 < buf.len() {
                buf[i * 2 + 1] = (val >> 8) as u8;
            }
        }
    }

    fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..((buf.len() + 1) / 2) {
            let mut val = buf[i * 2] as u16;
            if i * 2 + 1 < buf.len() {
                val |= (buf[i * 2 + 1] as u16) << 8;
            }
            unsafe { T::regs().ep_mem(self.addr as usize / 2 + i).write_value(val) };
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct EndpointData {
    ep_type: EndpointType, // only valid if used_in || used_out
    used_in: bool,
    used_out: bool,
}

pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    alloc: [EndpointData; EP_COUNT],
    ep_mem_free: u16, // first free address in EP mem, in bytes.
}

impl<'d, T: Instance> Driver<'d, T> {
    pub fn new(
        _usb: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        dp: impl Unborrow<Target = impl DpPin<T>> + 'd,
        dm: impl Unborrow<Target = impl DmPin<T>> + 'd,
    ) -> Self {
        unborrow!(irq, dp, dm);
        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        let regs = T::regs();

        #[cfg(stm32l5)]
        unsafe {
            crate::peripherals::PWR::enable();

            pac::PWR.cr2().modify(|w| w.set_usv(true));
        }

        unsafe {
            <T as RccPeripheral>::enable();
            <T as RccPeripheral>::reset();

            regs.cntr().write(|w| {
                w.set_pdwn(false);
                w.set_fres(true);
            });

            block_for(Duration::from_millis(100));

            regs.btable().write(|w| w.set_btable(0));

            dp.set_as_af(dp.af_num(), AFType::OutputPushPull);
            dm.set_as_af(dm.af_num(), AFType::OutputPushPull);
        }

        Self {
            phantom: PhantomData,
            alloc: [EndpointData {
                ep_type: EndpointType::Bulk,
                used_in: false,
                used_out: false,
            }; EP_COUNT],
            ep_mem_free: EP_COUNT as u16 * 8, // for each EP, 4 regs, so 8 bytes
        }
    }

    fn on_interrupt(_: *mut ()) {
        unsafe {
            let regs = T::regs();
            //let x = regs.istr().read().0;
            //trace!("USB IRQ: {:08x}", x);

            let istr = regs.istr().read();

            let mut flags: u8 = 0;

            if istr.susp() {
                //trace!("USB IRQ: susp");
                flags |= IRQ_FLAG_SUSPEND;
                regs.cntr().modify(|w| {
                    w.set_fsusp(true);
                    w.set_lpmode(true);
                })
            }

            if istr.wkup() {
                //trace!("USB IRQ: wkup");
                flags |= IRQ_FLAG_RESUME;
                regs.cntr().modify(|w| {
                    w.set_fsusp(false);
                    w.set_lpmode(false);
                })
            }

            if istr.reset() {
                //trace!("USB IRQ: reset");
                flags |= IRQ_FLAG_RESET;

                // Write 0 to clear.
                let mut clear = regs::Istr(!0);
                clear.set_reset(false);
                regs.istr().write_value(clear);
            }

            if flags != 0 {
                // Send irqs to main thread.
                IRQ_FLAGS.fetch_or(flags, Ordering::AcqRel);
                BUS_WAKER.wake();

                // Clear them
                let mut mask = regs::Istr(0);
                mask.set_wkup(true);
                mask.set_susp(true);
                mask.set_reset(true);
                regs.istr().write_value(regs::Istr(!(istr.0 & mask.0)));
            }

            if istr.ctr() {
                let index = istr.ep_id() as usize;
                let mut epr = regs.epr(index).read();
                if epr.ctr_rx() {
                    if index == 0 && epr.setup() {
                        EP0_SETUP.store(true, Ordering::Relaxed);
                    }
                    //trace!("EP {} RX, setup={}", index, epr.setup());
                    EP_OUT_WAKERS[index].wake();
                }
                if epr.ctr_tx() {
                    //trace!("EP {} TX", index);
                    EP_IN_WAKERS[index].wake();
                }
                epr.set_dtog_rx(false);
                epr.set_dtog_tx(false);
                epr.set_stat_rx(Stat(0));
                epr.set_stat_tx(Stat(0));
                epr.set_ctr_rx(!epr.ctr_rx());
                epr.set_ctr_tx(!epr.ctr_tx());
                regs.epr(index).write_value(epr);
            }
        }
    }

    fn alloc_ep_mem(&mut self, len: u16) -> u16 {
        let addr = self.ep_mem_free;
        if addr + len > EP_MEMORY_SIZE as _ {
            panic!("Endpoint memory full");
        }
        self.ep_mem_free += len;
        addr
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

        let index = self.alloc.iter_mut().enumerate().find(|(i, ep)| {
            if *i == 0 && ep_type != EndpointType::Control {
                return false; // reserved for control pipe
            }
            let used = ep.used_out || ep.used_in;
            let used_dir = match D::dir() {
                UsbDirection::Out => ep.used_out,
                UsbDirection::In => ep.used_in,
            };
            !used || (ep.ep_type == ep_type && !used_dir)
        });

        let (index, ep) = match index {
            Some(x) => x,
            None => return Err(EndpointAllocError),
        };

        ep.ep_type = ep_type;

        let buf = match D::dir() {
            UsbDirection::Out => {
                assert!(!ep.used_out);
                ep.used_out = true;

                let (len, len_bits) = calc_out_len(max_packet_size);
                let addr = self.alloc_ep_mem(len);

                trace!("  len_bits = {:04x}", len_bits);
                unsafe {
                    ep_out_addr::<T>(index).write_value(addr);
                    ep_out_len::<T>(index).write_value(len_bits);
                }

                EndpointBuffer {
                    addr,
                    len,
                    _phantom: PhantomData,
                }
            }
            UsbDirection::In => {
                assert!(!ep.used_in);
                ep.used_in = true;

                let len = (max_packet_size + 1) / 2 * 2;
                let addr = self.alloc_ep_mem(len);

                unsafe {
                    ep_in_addr::<T>(index).write_value(addr);
                    // ep_in_len is written when actually TXing packets.
                }

                EndpointBuffer {
                    addr,
                    len,
                    _phantom: PhantomData,
                }
            }
        };

        trace!("  index={} addr={} len={}", index, buf.addr, buf.len);

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

    fn start(mut self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let ep_out = self
            .alloc_endpoint(EndpointType::Control, control_max_packet_size, 0)
            .unwrap();
        let ep_in = self
            .alloc_endpoint(EndpointType::Control, control_max_packet_size, 0)
            .unwrap();
        assert_eq!(ep_out.info.addr.index(), 0);
        assert_eq!(ep_in.info.addr.index(), 0);

        let regs = T::regs();

        unsafe {
            regs.cntr().write(|w| {
                w.set_pdwn(false);
                w.set_fres(false);
                w.set_resetm(true);
                w.set_suspm(true);
                w.set_wkupm(true);
                w.set_ctrm(true);
            });

            #[cfg(usb_v3)]
            regs.bcdr().write(|w| w.set_dppu(true))
        }

        trace!("enabled");

        let mut ep_types = [EpType::BULK; EP_COUNT - 1];
        for i in 1..EP_COUNT {
            ep_types[i - 1] = convert_type(self.alloc[i].ep_type);
        }

        (
            Bus {
                phantom: PhantomData,
                ep_types,
            },
            ControlPipe {
                _phantom: PhantomData,
                max_packet_size: control_max_packet_size,
                ep_out,
                ep_in,
            },
        )
    }
}

pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_types: [EpType; EP_COUNT - 1],
}

impl<'d, T: Instance> driver::Bus for Bus<'d, T> {
    type PollFuture<'a> = impl Future<Output = Event> + 'a where Self: 'a;

    fn poll<'a>(&'a mut self) -> Self::PollFuture<'a> {
        poll_fn(move |cx| unsafe {
            BUS_WAKER.register(cx.waker());
            let regs = T::regs();

            let flags = IRQ_FLAGS.load(Ordering::Acquire);

            if flags & IRQ_FLAG_RESUME != 0 {
                IRQ_FLAGS.fetch_and(!IRQ_FLAG_RESUME, Ordering::AcqRel);
                return Poll::Ready(Event::Resume);
            }

            if flags & IRQ_FLAG_RESET != 0 {
                IRQ_FLAGS.fetch_and(!IRQ_FLAG_RESET, Ordering::AcqRel);

                trace!("RESET REGS WRITINGINGING");
                regs.daddr().write(|w| {
                    w.set_ef(true);
                    w.set_add(0);
                });

                regs.epr(0).write(|w| {
                    w.set_ep_type(EpType::CONTROL);
                    w.set_stat_rx(Stat::NAK);
                    w.set_stat_tx(Stat::NAK);
                });

                for i in 1..EP_COUNT {
                    regs.epr(i).write(|w| {
                        w.set_ea(i as _);
                        w.set_ep_type(self.ep_types[i - 1]);
                    })
                }

                for w in &EP_IN_WAKERS {
                    w.wake()
                }
                for w in &EP_OUT_WAKERS {
                    w.wake()
                }

                return Poll::Ready(Event::Reset);
            }

            if flags & IRQ_FLAG_SUSPEND != 0 {
                IRQ_FLAGS.fetch_and(!IRQ_FLAG_SUSPEND, Ordering::AcqRel);
                return Poll::Ready(Event::Suspend);
            }

            Poll::Pending
        })
    }

    #[inline]
    fn set_address(&mut self, addr: u8) {
        let regs = T::regs();
        trace!("setting addr: {}", addr);
        unsafe {
            regs.daddr().write(|w| {
                w.set_ef(true);
                w.set_add(addr);
            })
        }
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        // This can race, so do a retry loop.
        let reg = T::regs().epr(ep_addr.index() as _);
        match ep_addr.direction() {
            UsbDirection::In => {
                loop {
                    let r = unsafe { reg.read() };
                    match r.stat_tx() {
                        Stat::DISABLED => break, // if disabled, stall does nothing.
                        Stat::STALL => break,    // done!
                        _ => {
                            let want_stat = match stalled {
                                false => Stat::NAK,
                                true => Stat::STALL,
                            };
                            let mut w = invariant(r);
                            w.set_stat_tx(Stat(r.stat_tx().0 ^ want_stat.0));
                            unsafe { reg.write_value(w) };
                        }
                    }
                }
                EP_IN_WAKERS[ep_addr.index()].wake();
            }
            UsbDirection::Out => {
                loop {
                    let r = unsafe { reg.read() };
                    match r.stat_rx() {
                        Stat::DISABLED => break, // if disabled, stall does nothing.
                        Stat::STALL => break,    // done!
                        _ => {
                            let want_stat = match stalled {
                                false => Stat::VALID,
                                true => Stat::STALL,
                            };
                            let mut w = invariant(r);
                            w.set_stat_rx(Stat(r.stat_rx().0 ^ want_stat.0));
                            unsafe { reg.write_value(w) };
                        }
                    }
                }
                EP_OUT_WAKERS[ep_addr.index()].wake();
            }
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let regs = T::regs();
        let epr = unsafe { regs.epr(ep_addr.index() as _).read() };
        match ep_addr.direction() {
            UsbDirection::In => epr.stat_tx() == Stat::STALL,
            UsbDirection::Out => epr.stat_rx() == Stat::STALL,
        }
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:x} {}", ep_addr, enabled);
        // This can race, so do a retry loop.
        let reg = T::regs().epr(ep_addr.index() as _);
        trace!("EPR before: {:04x}", unsafe { reg.read() }.0);
        match ep_addr.direction() {
            UsbDirection::In => {
                loop {
                    let want_stat = match enabled {
                        false => Stat::DISABLED,
                        true => Stat::NAK,
                    };
                    let r = unsafe { reg.read() };
                    if r.stat_tx() == want_stat {
                        break;
                    }
                    let mut w = invariant(r);
                    w.set_stat_tx(Stat(r.stat_tx().0 ^ want_stat.0));
                    unsafe { reg.write_value(w) };
                }
                EP_IN_WAKERS[ep_addr.index()].wake();
            }
            UsbDirection::Out => {
                loop {
                    let want_stat = match enabled {
                        false => Stat::DISABLED,
                        true => Stat::VALID,
                    };
                    let r = unsafe { reg.read() };
                    if r.stat_rx() == want_stat {
                        break;
                    }
                    let mut w = invariant(r);
                    w.set_stat_rx(Stat(r.stat_rx().0 ^ want_stat.0));
                    unsafe { reg.write_value(w) };
                }
                EP_OUT_WAKERS[ep_addr.index()].wake();
            }
        }
        trace!("EPR after: {:04x}", unsafe { reg.read() }.0);
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
    fn dir() -> UsbDirection;
    fn waker(i: usize) -> &'static AtomicWaker;
}

pub enum In {}
impl Dir for In {
    fn dir() -> UsbDirection {
        UsbDirection::In
    }

    #[inline]
    fn waker(i: usize) -> &'static AtomicWaker {
        &EP_IN_WAKERS[i]
    }
}

pub enum Out {}
impl Dir for Out {
    fn dir() -> UsbDirection {
        UsbDirection::Out
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

impl<'d, T: Instance, D> Endpoint<'d, T, D> {
    fn write_data(&mut self, buf: &[u8]) {
        let index = self.info.addr.index();
        self.buf.write(buf);
        unsafe { ep_in_len::<T>(index).write_value(buf.len() as _) };
    }

    fn read_data(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        let rx_len = unsafe { ep_out_len::<T>(index).read() as usize } & 0x3FF;
        trace!("READ DONE, rx_len = {}", rx_len);
        if rx_len > buf.len() {
            return Err(EndpointError::BufferOverflow);
        }
        self.buf.read(&mut buf[..rx_len]);
        Ok(rx_len)
    }
}

impl<'d, T: Instance> driver::Endpoint for Endpoint<'d, T, In> {
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
                let regs = T::regs();
                if unsafe { regs.epr(index).read() }.stat_tx() == Stat::DISABLED {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
            trace!("wait_enabled OUT OK");
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
                let regs = T::regs();
                if unsafe { regs.epr(index).read() }.stat_rx() == Stat::DISABLED {
                    Poll::Pending
                } else {
                    Poll::Ready(())
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
            let stat = poll_fn(|cx| {
                EP_OUT_WAKERS[index].register(cx.waker());
                let regs = T::regs();
                let stat = unsafe { regs.epr(index).read() }.stat_rx();
                if matches!(stat, Stat::NAK | Stat::DISABLED) {
                    Poll::Ready(stat)
                } else {
                    Poll::Pending
                }
            })
            .await;

            if stat == Stat::DISABLED {
                return Err(EndpointError::Disabled);
            }

            let rx_len = self.read_data(buf)?;

            let regs = T::regs();
            unsafe {
                regs.epr(index).write(|w| {
                    w.set_ep_type(convert_type(self.info.ep_type));
                    w.set_ea(self.info.addr.index() as _);
                    w.set_stat_rx(Stat(Stat::NAK.0 ^ Stat::VALID.0));
                    w.set_stat_tx(Stat(0));
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                })
            };
            trace!("READ OK, rx_len = {}", rx_len);

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

            let index = self.info.addr.index();

            trace!("WRITE WAITING");
            let stat = poll_fn(|cx| {
                EP_IN_WAKERS[index].register(cx.waker());
                let regs = T::regs();
                let stat = unsafe { regs.epr(index).read() }.stat_tx();
                if matches!(stat, Stat::NAK | Stat::DISABLED) {
                    Poll::Ready(stat)
                } else {
                    Poll::Pending
                }
            })
            .await;

            if stat == Stat::DISABLED {
                return Err(EndpointError::Disabled);
            }

            self.write_data(buf);

            let regs = T::regs();
            unsafe {
                regs.epr(index).write(|w| {
                    w.set_ep_type(convert_type(self.info.ep_type));
                    w.set_ea(self.info.addr.index() as _);
                    w.set_stat_tx(Stat(Stat::NAK.0 ^ Stat::VALID.0));
                    w.set_stat_rx(Stat(0));
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                })
            };

            trace!("WRITE OK");

            Ok(())
        }
    }
}

pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
    ep_in: Endpoint<'d, T, In>,
    ep_out: Endpoint<'d, T, Out>,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    type SetupFuture<'a> = impl Future<Output = [u8;8]> + 'a where Self: 'a;
    type DataOutFuture<'a> = impl Future<Output = Result<usize, EndpointError>> + 'a where Self: 'a;
    type DataInFuture<'a> = impl Future<Output = Result<(), EndpointError>> + 'a where Self: 'a;
    type AcceptFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;
    type RejectFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    fn setup<'a>(&'a mut self) -> Self::SetupFuture<'a> {
        async move {
            loop {
                trace!("SETUP read waiting");
                poll_fn(|cx| {
                    EP_OUT_WAKERS[0].register(cx.waker());
                    if EP0_SETUP.load(Ordering::Relaxed) {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                let mut buf = [0; 8];
                let rx_len = self.ep_out.read_data(&mut buf);
                if rx_len != Ok(8) {
                    trace!("SETUP read failed: {:?}", rx_len);
                    continue;
                }

                EP0_SETUP.store(false, Ordering::Relaxed);

                trace!("SETUP read ok");
                return buf;
            }
        }
    }

    fn data_out<'a>(&'a mut self, buf: &'a mut [u8], first: bool, last: bool) -> Self::DataOutFuture<'a> {
        async move {
            let regs = T::regs();

            // When a SETUP is received, Stat/Stat is set to NAK.
            // On first transfer, we must set Stat=VALID, to get the OUT data stage.
            // We want Stat=STALL so that the host gets a STALL if it switches to the status
            // stage too soon, except in the last transfer we set Stat=NAK so that it waits
            // for the status stage, which we will ACK or STALL later.
            if first || last {
                let mut stat_rx = 0;
                let mut stat_tx = 0;
                if first {
                    // change NAK -> VALID
                    stat_rx ^= Stat::NAK.0 ^ Stat::VALID.0;
                    stat_tx ^= Stat::NAK.0 ^ Stat::STALL.0;
                }
                if last {
                    // change STALL -> VALID
                    stat_tx ^= Stat::STALL.0 ^ Stat::NAK.0;
                }
                // Note: if this is the first AND last transfer, the above effectively
                // changes stat_tx like NAK -> NAK, so noop.
                unsafe {
                    regs.epr(0).write(|w| {
                        w.set_ep_type(EpType::CONTROL);
                        w.set_stat_rx(Stat(stat_rx));
                        w.set_stat_tx(Stat(stat_tx));
                        w.set_ctr_rx(true); // don't clear
                        w.set_ctr_tx(true); // don't clear
                    })
                }
            }

            trace!("data_out WAITING, buf.len() = {}", buf.len());
            poll_fn(|cx| {
                EP_OUT_WAKERS[0].register(cx.waker());
                let regs = T::regs();
                if unsafe { regs.epr(0).read() }.stat_rx() == Stat::NAK {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            if EP0_SETUP.load(Ordering::Relaxed) {
                trace!("received another SETUP, aborting data_out.");
                return Err(EndpointError::Disabled);
            }

            let rx_len = self.ep_out.read_data(buf)?;

            unsafe {
                regs.epr(0).write(|w| {
                    w.set_ep_type(EpType::CONTROL);
                    w.set_stat_rx(Stat(match last {
                        // If last, set STAT_RX=STALL.
                        true => Stat::NAK.0 ^ Stat::STALL.0,
                        // Otherwise, set STAT_RX=VALID, to allow the host to send the next packet.
                        false => Stat::NAK.0 ^ Stat::VALID.0,
                    }));
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                })
            };

            Ok(rx_len)
        }
    }

    fn data_in<'a>(&'a mut self, buf: &'a [u8], first: bool, last: bool) -> Self::DataInFuture<'a> {
        async move {
            trace!("control: data_in");

            if buf.len() > self.ep_in.info.max_packet_size as usize {
                return Err(EndpointError::BufferOverflow);
            }

            let regs = T::regs();

            // When a SETUP is received, Stat is set to NAK.
            // We want it to be STALL in non-last transfers.
            // We want it to be VALID in last transfer, so the HW does the status stage.
            if first || last {
                let mut stat_rx = 0;
                if first {
                    // change NAK -> STALL
                    stat_rx ^= Stat::NAK.0 ^ Stat::STALL.0;
                }
                if last {
                    // change STALL -> VALID
                    stat_rx ^= Stat::STALL.0 ^ Stat::VALID.0;
                }
                // Note: if this is the first AND last transfer, the above effectively
                // does a change of NAK -> VALID.
                unsafe {
                    regs.epr(0).write(|w| {
                        w.set_ep_type(EpType::CONTROL);
                        w.set_stat_rx(Stat(stat_rx));
                        w.set_ep_kind(last); // set OUT_STATUS if last.
                        w.set_ctr_rx(true); // don't clear
                        w.set_ctr_tx(true); // don't clear
                    })
                }
            }

            trace!("WRITE WAITING");
            poll_fn(|cx| {
                EP_IN_WAKERS[0].register(cx.waker());
                EP_OUT_WAKERS[0].register(cx.waker());
                let regs = T::regs();
                if unsafe { regs.epr(0).read() }.stat_tx() == Stat::NAK {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            if EP0_SETUP.load(Ordering::Relaxed) {
                trace!("received another SETUP, aborting data_in.");
                return Err(EndpointError::Disabled);
            }

            self.ep_in.write_data(buf);

            let regs = T::regs();
            unsafe {
                regs.epr(0).write(|w| {
                    w.set_ep_type(EpType::CONTROL);
                    w.set_stat_tx(Stat(Stat::NAK.0 ^ Stat::VALID.0));
                    w.set_ep_kind(last); // set OUT_STATUS if last.
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                })
            };

            trace!("WRITE OK");

            Ok(())
        }
    }

    fn accept<'a>(&'a mut self) -> Self::AcceptFuture<'a> {
        async move {
            let regs = T::regs();
            trace!("control: accept");

            self.ep_in.write_data(&[]);

            // Set OUT=stall, IN=accept
            unsafe {
                let epr = regs.epr(0).read();
                regs.epr(0).write(|w| {
                    w.set_ep_type(EpType::CONTROL);
                    w.set_stat_rx(Stat(epr.stat_rx().0 ^ Stat::STALL.0));
                    w.set_stat_tx(Stat(epr.stat_tx().0 ^ Stat::VALID.0));
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                });
            }
            trace!("control: accept WAITING");

            // Wait is needed, so that we don't set the address too soon, breaking the status stage.
            // (embassy-usb sets the address after accept() returns)
            poll_fn(|cx| {
                EP_IN_WAKERS[0].register(cx.waker());
                let regs = T::regs();
                if unsafe { regs.epr(0).read() }.stat_tx() == Stat::NAK {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            trace!("control: accept OK");
        }
    }

    fn reject<'a>(&'a mut self) -> Self::RejectFuture<'a> {
        async move {
            let regs = T::regs();
            trace!("control: reject");

            // Set IN+OUT to stall
            unsafe {
                let epr = regs.epr(0).read();
                regs.epr(0).write(|w| {
                    w.set_ep_type(EpType::CONTROL);
                    w.set_stat_rx(Stat(epr.stat_rx().0 ^ Stat::STALL.0));
                    w.set_stat_tx(Stat(epr.stat_tx().0 ^ Stat::STALL.0));
                    w.set_ctr_rx(true); // don't clear
                    w.set_ctr_tx(true); // don't clear
                });
            }
        }
    }
}
