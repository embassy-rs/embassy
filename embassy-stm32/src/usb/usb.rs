#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::into_ref;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver as driver;
use embassy_usb_driver::{
    Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointInfo, EndpointType, Event, Unsupported,
};

use crate::pac::usb::regs;
use crate::pac::usb::vals::{EpType, Stat};
use crate::pac::USBRAM;
use crate::rcc::RccPeripheral;
use crate::{interrupt, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        //let x = regs.istr().read().0;
        //trace!("USB IRQ: {:08x}", x);

        let istr = regs.istr().read();

        if istr.susp() {
            //trace!("USB IRQ: susp");
            IRQ_SUSPEND.store(true, Ordering::Relaxed);
            regs.cntr().modify(|w| {
                w.set_fsusp(true);
                w.set_lpmode(true);
            });

            // Write 0 to clear.
            let mut clear = regs::Istr(!0);
            clear.set_susp(false);
            regs.istr().write_value(clear);

            // Wake main thread.
            BUS_WAKER.wake();
        }

        if istr.wkup() {
            //trace!("USB IRQ: wkup");
            IRQ_RESUME.store(true, Ordering::Relaxed);
            regs.cntr().modify(|w| {
                w.set_fsusp(false);
                w.set_lpmode(false);
            });

            // Write 0 to clear.
            let mut clear = regs::Istr(!0);
            clear.set_wkup(false);
            regs.istr().write_value(clear);

            // Wake main thread.
            BUS_WAKER.wake();
        }

        if istr.reset() {
            //trace!("USB IRQ: reset");
            IRQ_RESET.store(true, Ordering::Relaxed);

            // Write 0 to clear.
            let mut clear = regs::Istr(!0);
            clear.set_reset(false);
            regs.istr().write_value(clear);

            // Wake main thread.
            BUS_WAKER.wake();
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
            epr.set_stat_rx(Stat::from_bits(0));
            epr.set_stat_tx(Stat::from_bits(0));
            epr.set_ctr_rx(!epr.ctr_rx());
            epr.set_ctr_tx(!epr.ctr_tx());
            regs.epr(index).write_value(epr);
        }
    }
}

const EP_COUNT: usize = 8;

#[cfg(any(usbram_16x1_512, usbram_16x2_512))]
const USBRAM_SIZE: usize = 512;
#[cfg(any(usbram_16x2_1024, usbram_32_1024))]
const USBRAM_SIZE: usize = 1024;
#[cfg(usbram_32_2048)]
const USBRAM_SIZE: usize = 2048;

#[cfg(not(any(usbram_32_2048, usbram_32_1024)))]
const USBRAM_ALIGN: usize = 2;
#[cfg(any(usbram_32_2048, usbram_32_1024))]
const USBRAM_ALIGN: usize = 4;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP0_SETUP: AtomicBool = AtomicBool::new(false);
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static IRQ_RESET: AtomicBool = AtomicBool::new(false);
static IRQ_SUSPEND: AtomicBool = AtomicBool::new(false);
static IRQ_RESUME: AtomicBool = AtomicBool::new(false);

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
    r.set_stat_rx(Stat::from_bits(0));
    r.set_stat_tx(Stat::from_bits(0));
    r
}

fn align_len_up(len: u16) -> u16 {
    ((len as usize + USBRAM_ALIGN - 1) / USBRAM_ALIGN * USBRAM_ALIGN) as u16
}

// Returns (actual_len, len_bits)
fn calc_out_len(len: u16) -> (u16, u16) {
    match len {
        // NOTE: this could be 2..=62 with 16bit USBRAM, but not with 32bit. Limit it to 60 for simplicity.
        2..=60 => (align_len_up(len), align_len_up(len) / 2 << 10),
        61..=1024 => ((len + 31) / 32 * 32, (((len + 31) / 32 - 1) << 10) | 0x8000),
        _ => panic!("invalid OUT length {}", len),
    }
}

#[cfg(not(any(usbram_32_2048, usbram_32_1024)))]
mod btable {
    use super::*;

    pub(super) fn write_in<T: Instance>(index: usize, addr: u16) {
        USBRAM.mem(index * 4 + 0).write_value(addr);
    }

    pub(super) fn write_in_len<T: Instance>(index: usize, _addr: u16, len: u16) {
        USBRAM.mem(index * 4 + 1).write_value(len);
    }

    pub(super) fn write_out<T: Instance>(index: usize, addr: u16, max_len_bits: u16) {
        USBRAM.mem(index * 4 + 2).write_value(addr);
        USBRAM.mem(index * 4 + 3).write_value(max_len_bits);
    }

    pub(super) fn read_out_len<T: Instance>(index: usize) -> u16 {
        USBRAM.mem(index * 4 + 3).read()
    }
}
#[cfg(any(usbram_32_2048, usbram_32_1024))]
mod btable {
    use super::*;

    pub(super) fn write_in<T: Instance>(_index: usize, _addr: u16) {}

    pub(super) fn write_in_len<T: Instance>(index: usize, addr: u16, len: u16) {
        USBRAM.mem(index * 2).write_value((addr as u32) | ((len as u32) << 16));
    }

    pub(super) fn write_out<T: Instance>(index: usize, addr: u16, max_len_bits: u16) {
        USBRAM
            .mem(index * 2 + 1)
            .write_value((addr as u32) | ((max_len_bits as u32) << 16));
    }

    pub(super) fn read_out_len<T: Instance>(index: usize) -> u16 {
        (USBRAM.mem(index * 2 + 1).read() >> 16) as u16
    }
}

struct EndpointBuffer<T: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<T>,
}

impl<T: Instance> EndpointBuffer<T> {
    fn read(&mut self, buf: &mut [u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..(buf.len() + USBRAM_ALIGN - 1) / USBRAM_ALIGN {
            let val = USBRAM.mem(self.addr as usize / USBRAM_ALIGN + i).read();
            let n = USBRAM_ALIGN.min(buf.len() - i * USBRAM_ALIGN);
            buf[i * USBRAM_ALIGN..][..n].copy_from_slice(&val.to_le_bytes()[..n]);
        }
    }

    fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= self.len as usize);
        for i in 0..(buf.len() + USBRAM_ALIGN - 1) / USBRAM_ALIGN {
            let mut val = [0u8; USBRAM_ALIGN];
            let n = USBRAM_ALIGN.min(buf.len() - i * USBRAM_ALIGN);
            val[..n].copy_from_slice(&buf[i * USBRAM_ALIGN..][..n]);

            #[cfg(not(any(usbram_32_2048, usbram_32_1024)))]
            let val = u16::from_le_bytes(val);
            #[cfg(any(usbram_32_2048, usbram_32_1024))]
            let val = u32::from_le_bytes(val);
            USBRAM.mem(self.addr as usize / USBRAM_ALIGN + i).write_value(val);
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

/// USB driver.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    alloc: [EndpointData; EP_COUNT],
    ep_mem_free: u16, // first free address in EP mem, in bytes.
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Create a new USB driver.
    pub fn new(
        _usb: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dp: impl Peripheral<P = impl DpPin<T>> + 'd,
        dm: impl Peripheral<P = impl DmPin<T>> + 'd,
    ) -> Self {
        into_ref!(dp, dm);

        super::common_init::<T>();

        let regs = T::regs();

        regs.cntr().write(|w| {
            w.set_pdwn(false);
            w.set_fres(true);
        });

        #[cfg(feature = "time")]
        embassy_time::block_for(embassy_time::Duration::from_millis(100));
        #[cfg(not(feature = "time"))]
        cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 10);

        #[cfg(not(usb_v4))]
        regs.btable().write(|w| w.set_btable(0));

        #[cfg(not(stm32l1))]
        {
            dp.set_as_af(dp.af_num(), crate::gpio::AFType::OutputPushPull);
            dm.set_as_af(dm.af_num(), crate::gpio::AFType::OutputPushPull);
        }
        #[cfg(stm32l1)]
        let _ = (dp, dm); // suppress "unused" warnings.

        // Initialize the bus so that it signals that power is available
        BUS_WAKER.wake();

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

    fn alloc_ep_mem(&mut self, len: u16) -> u16 {
        assert!(len as usize % USBRAM_ALIGN == 0);
        let addr = self.ep_mem_free;
        if addr + len > USBRAM_SIZE as _ {
            panic!("Endpoint memory full");
        }
        self.ep_mem_free += len;
        addr
    }

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Endpoint<'d, T, D>, driver::EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval_ms={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval_ms,
            D::dir()
        );

        let index = self.alloc.iter_mut().enumerate().find(|(i, ep)| {
            if *i == 0 && ep_type != EndpointType::Control {
                return false; // reserved for control pipe
            }
            let used = ep.used_out || ep.used_in;
            let used_dir = match D::dir() {
                Direction::Out => ep.used_out,
                Direction::In => ep.used_in,
            };
            !used || (ep.ep_type == ep_type && !used_dir)
        });

        let (index, ep) = match index {
            Some(x) => x,
            None => return Err(EndpointAllocError),
        };

        ep.ep_type = ep_type;

        let buf = match D::dir() {
            Direction::Out => {
                assert!(!ep.used_out);
                ep.used_out = true;

                let (len, len_bits) = calc_out_len(max_packet_size);
                let addr = self.alloc_ep_mem(len);

                trace!("  len_bits = {:04x}", len_bits);
                btable::write_out::<T>(index, addr, len_bits);

                EndpointBuffer {
                    addr,
                    len,
                    _phantom: PhantomData,
                }
            }
            Direction::In => {
                assert!(!ep.used_in);
                ep.used_in = true;

                let len = align_len_up(max_packet_size);
                let addr = self.alloc_ep_mem(len);

                // ep_in_len is written when actually TXing packets.
                btable::write_in::<T>(index, addr);

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
                interval_ms,
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
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, driver::EndpointAllocError> {
        self.alloc_endpoint(ep_type, max_packet_size, interval_ms)
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

        regs.cntr().write(|w| {
            w.set_pdwn(false);
            w.set_fres(false);
            w.set_resetm(true);
            w.set_suspm(true);
            w.set_wkupm(true);
            w.set_ctrm(true);
        });

        #[cfg(any(usb_v3, usb_v4))]
        regs.bcdr().write(|w| w.set_dppu(true));

        #[cfg(stm32l1)]
        crate::pac::SYSCFG.pmc().modify(|w| w.set_usb_pu(true));

        trace!("enabled");

        let mut ep_types = [EpType::BULK; EP_COUNT - 1];
        for i in 1..EP_COUNT {
            ep_types[i - 1] = convert_type(self.alloc[i].ep_type);
        }

        (
            Bus {
                phantom: PhantomData,
                ep_types,
                inited: false,
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

/// USB bus.
pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ep_types: [EpType; EP_COUNT - 1],
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

            if IRQ_RESUME.load(Ordering::Acquire) {
                IRQ_RESUME.store(false, Ordering::Relaxed);
                return Poll::Ready(Event::Resume);
            }

            if IRQ_RESET.load(Ordering::Acquire) {
                IRQ_RESET.store(false, Ordering::Relaxed);

                trace!("RESET");
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

            if IRQ_SUSPEND.load(Ordering::Acquire) {
                IRQ_SUSPEND.store(false, Ordering::Relaxed);
                return Poll::Ready(Event::Suspend);
            }

            Poll::Pending
        })
        .await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        // This can race, so do a retry loop.
        let reg = T::regs().epr(ep_addr.index() as _);
        match ep_addr.direction() {
            Direction::In => {
                loop {
                    let r = reg.read();
                    match r.stat_tx() {
                        Stat::DISABLED => break, // if disabled, stall does nothing.
                        Stat::STALL => break,    // done!
                        _ => {
                            let want_stat = match stalled {
                                false => Stat::NAK,
                                true => Stat::STALL,
                            };
                            let mut w = invariant(r);
                            w.set_stat_tx(Stat::from_bits(r.stat_tx().to_bits() ^ want_stat.to_bits()));
                            reg.write_value(w);
                        }
                    }
                }
                EP_IN_WAKERS[ep_addr.index()].wake();
            }
            Direction::Out => {
                loop {
                    let r = reg.read();
                    match r.stat_rx() {
                        Stat::DISABLED => break, // if disabled, stall does nothing.
                        Stat::STALL => break,    // done!
                        _ => {
                            let want_stat = match stalled {
                                false => Stat::VALID,
                                true => Stat::STALL,
                            };
                            let mut w = invariant(r);
                            w.set_stat_rx(Stat::from_bits(r.stat_rx().to_bits() ^ want_stat.to_bits()));
                            reg.write_value(w);
                        }
                    }
                }
                EP_OUT_WAKERS[ep_addr.index()].wake();
            }
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        let regs = T::regs();
        let epr = regs.epr(ep_addr.index() as _).read();
        match ep_addr.direction() {
            Direction::In => epr.stat_tx() == Stat::STALL,
            Direction::Out => epr.stat_rx() == Stat::STALL,
        }
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("set_enabled {:x} {}", ep_addr, enabled);
        // This can race, so do a retry loop.
        let reg = T::regs().epr(ep_addr.index() as _);
        trace!("EPR before: {:04x}", reg.read().0);
        match ep_addr.direction() {
            Direction::In => {
                loop {
                    let want_stat = match enabled {
                        false => Stat::DISABLED,
                        true => Stat::NAK,
                    };
                    let r = reg.read();
                    if r.stat_tx() == want_stat {
                        break;
                    }
                    let mut w = invariant(r);
                    w.set_stat_tx(Stat::from_bits(r.stat_tx().to_bits() ^ want_stat.to_bits()));
                    reg.write_value(w);
                }
                EP_IN_WAKERS[ep_addr.index()].wake();
            }
            Direction::Out => {
                loop {
                    let want_stat = match enabled {
                        false => Stat::DISABLED,
                        true => Stat::VALID,
                    };
                    let r = reg.read();
                    if r.stat_rx() == want_stat {
                        break;
                    }
                    let mut w = invariant(r);
                    w.set_stat_rx(Stat::from_bits(r.stat_rx().to_bits() ^ want_stat.to_bits()));
                    reg.write_value(w);
                }
                EP_OUT_WAKERS[ep_addr.index()].wake();
            }
        }
        trace!("EPR after: {:04x}", reg.read().0);
    }

    async fn enable(&mut self) {}
    async fn disable(&mut self) {}

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

trait Dir {
    fn dir() -> Direction;
}

/// Marker type for the "IN" direction.
pub enum In {}
impl Dir for In {
    fn dir() -> Direction {
        Direction::In
    }
}

/// Marker type for the "OUT" direction.
pub enum Out {}
impl Dir for Out {
    fn dir() -> Direction {
        Direction::Out
    }
}

/// USB endpoint.
pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
    buf: EndpointBuffer<T>,
}

impl<'d, T: Instance, D> Endpoint<'d, T, D> {
    fn write_data(&mut self, buf: &[u8]) {
        let index = self.info.addr.index();
        self.buf.write(buf);
        btable::write_in_len::<T>(index, self.buf.addr, buf.len() as _);
    }

    fn read_data(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        let index = self.info.addr.index();
        let rx_len = btable::read_out_len::<T>(index) as usize & 0x3FF;
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

    async fn wait_enabled(&mut self) {
        trace!("wait_enabled IN WAITING");
        let index = self.info.addr.index();
        poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            let regs = T::regs();
            if regs.epr(index).read().stat_tx() == Stat::DISABLED {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
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
            let regs = T::regs();
            if regs.epr(index).read().stat_rx() == Stat::DISABLED {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
        trace!("wait_enabled OUT OK");
    }
}

impl<'d, T: Instance> driver::EndpointOut for Endpoint<'d, T, Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        trace!("READ WAITING, buf.len() = {}", buf.len());
        let index = self.info.addr.index();
        let stat = poll_fn(|cx| {
            EP_OUT_WAKERS[index].register(cx.waker());
            let regs = T::regs();
            let stat = regs.epr(index).read().stat_rx();
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
        regs.epr(index).write(|w| {
            w.set_ep_type(convert_type(self.info.ep_type));
            w.set_ea(self.info.addr.index() as _);
            w.set_stat_rx(Stat::from_bits(Stat::NAK.to_bits() ^ Stat::VALID.to_bits()));
            w.set_stat_tx(Stat::from_bits(0));
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });
        trace!("READ OK, rx_len = {}", rx_len);

        Ok(rx_len)
    }
}

impl<'d, T: Instance> driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        if buf.len() > self.info.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let index = self.info.addr.index();

        trace!("WRITE WAITING");
        let stat = poll_fn(|cx| {
            EP_IN_WAKERS[index].register(cx.waker());
            let regs = T::regs();
            let stat = regs.epr(index).read().stat_tx();
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
        regs.epr(index).write(|w| {
            w.set_ep_type(convert_type(self.info.ep_type));
            w.set_ea(self.info.addr.index() as _);
            w.set_stat_tx(Stat::from_bits(Stat::NAK.to_bits() ^ Stat::VALID.to_bits()));
            w.set_stat_rx(Stat::from_bits(0));
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });

        trace!("WRITE OK");

        Ok(())
    }
}

/// USB control pipe.
pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
    ep_in: Endpoint<'d, T, In>,
    ep_out: Endpoint<'d, T, Out>,
}

impl<'d, T: Instance> driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    async fn setup(&mut self) -> [u8; 8] {
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

    async fn data_out(&mut self, buf: &mut [u8], first: bool, last: bool) -> Result<usize, EndpointError> {
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
                stat_rx ^= Stat::NAK.to_bits() ^ Stat::VALID.to_bits();
                stat_tx ^= Stat::NAK.to_bits() ^ Stat::STALL.to_bits();
            }
            if last {
                // change STALL -> VALID
                stat_tx ^= Stat::STALL.to_bits() ^ Stat::NAK.to_bits();
            }
            // Note: if this is the first AND last transfer, the above effectively
            // changes stat_tx like NAK -> NAK, so noop.
            regs.epr(0).write(|w| {
                w.set_ep_type(EpType::CONTROL);
                w.set_stat_rx(Stat::from_bits(stat_rx));
                w.set_stat_tx(Stat::from_bits(stat_tx));
                w.set_ctr_rx(true); // don't clear
                w.set_ctr_tx(true); // don't clear
            });
        }

        trace!("data_out WAITING, buf.len() = {}", buf.len());
        poll_fn(|cx| {
            EP_OUT_WAKERS[0].register(cx.waker());
            let regs = T::regs();
            if regs.epr(0).read().stat_rx() == Stat::NAK {
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

        regs.epr(0).write(|w| {
            w.set_ep_type(EpType::CONTROL);
            w.set_stat_rx(Stat::from_bits(match last {
                // If last, set STAT_RX=STALL.
                true => Stat::NAK.to_bits() ^ Stat::STALL.to_bits(),
                // Otherwise, set STAT_RX=VALID, to allow the host to send the next packet.
                false => Stat::NAK.to_bits() ^ Stat::VALID.to_bits(),
            }));
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });

        Ok(rx_len)
    }

    async fn data_in(&mut self, data: &[u8], first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in");

        if data.len() > self.ep_in.info.max_packet_size as usize {
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
                stat_rx ^= Stat::NAK.to_bits() ^ Stat::STALL.to_bits();
            }
            if last {
                // change STALL -> VALID
                stat_rx ^= Stat::STALL.to_bits() ^ Stat::VALID.to_bits();
            }
            // Note: if this is the first AND last transfer, the above effectively
            // does a change of NAK -> VALID.
            regs.epr(0).write(|w| {
                w.set_ep_type(EpType::CONTROL);
                w.set_stat_rx(Stat::from_bits(stat_rx));
                w.set_ep_kind(last); // set OUT_STATUS if last.
                w.set_ctr_rx(true); // don't clear
                w.set_ctr_tx(true); // don't clear
            });
        }

        trace!("WRITE WAITING");
        poll_fn(|cx| {
            EP_IN_WAKERS[0].register(cx.waker());
            EP_OUT_WAKERS[0].register(cx.waker());
            let regs = T::regs();
            if regs.epr(0).read().stat_tx() == Stat::NAK {
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

        self.ep_in.write_data(data);

        let regs = T::regs();
        regs.epr(0).write(|w| {
            w.set_ep_type(EpType::CONTROL);
            w.set_stat_tx(Stat::from_bits(Stat::NAK.to_bits() ^ Stat::VALID.to_bits()));
            w.set_ep_kind(last); // set OUT_STATUS if last.
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });

        trace!("WRITE OK");

        Ok(())
    }

    async fn accept(&mut self) {
        let regs = T::regs();
        trace!("control: accept");

        self.ep_in.write_data(&[]);

        // Set OUT=stall, IN=accept
        let epr = regs.epr(0).read();
        regs.epr(0).write(|w| {
            w.set_ep_type(EpType::CONTROL);
            w.set_stat_rx(Stat::from_bits(epr.stat_rx().to_bits() ^ Stat::STALL.to_bits()));
            w.set_stat_tx(Stat::from_bits(epr.stat_tx().to_bits() ^ Stat::VALID.to_bits()));
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });
        trace!("control: accept WAITING");

        // Wait is needed, so that we don't set the address too soon, breaking the status stage.
        // (embassy-usb sets the address after accept() returns)
        poll_fn(|cx| {
            EP_IN_WAKERS[0].register(cx.waker());
            let regs = T::regs();
            if regs.epr(0).read().stat_tx() == Stat::NAK {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        trace!("control: accept OK");
    }

    async fn reject(&mut self) {
        let regs = T::regs();
        trace!("control: reject");

        // Set IN+OUT to stall
        let epr = regs.epr(0).read();
        regs.epr(0).write(|w| {
            w.set_ep_type(EpType::CONTROL);
            w.set_stat_rx(Stat::from_bits(epr.stat_rx().to_bits() ^ Stat::STALL.to_bits()));
            w.set_stat_tx(Stat::from_bits(epr.stat_tx().to_bits() ^ Stat::STALL.to_bits()));
            w.set_ctr_rx(true); // don't clear
            w.set_ctr_tx(true); // don't clear
        });
    }

    async fn accept_set_address(&mut self, addr: u8) {
        self.accept().await;

        let regs = T::regs();
        trace!("setting addr: {}", addr);
        regs.daddr().write(|w| {
            w.set_ef(true);
            w.set_add(addr);
        });
    }
}

trait SealedInstance {
    fn regs() -> crate::pac::usb::Usb;
}

/// USB instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// Interrupt for this USB instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Internal PHY pins
pin_trait!(DpPin, Instance);
pin_trait!(DmPin, Instance);

foreach_interrupt!(
    ($inst:ident, usb, $block:ident, LP, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::usb::Usb {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);
