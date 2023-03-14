use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::task::Poll;

use atomic_polyfill::{AtomicBool, AtomicU16, Ordering};
use embassy_cortex_m::interrupt::InterruptExt;
use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::{
    self, Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointIn, EndpointInfo, EndpointOut,
    EndpointType, Event, Unsupported,
};
use futures::future::poll_fn;

use super::*;
use crate::gpio::sealed::AFType;
use crate::pac::otg::{regs, vals};
use crate::rcc::sealed::RccPeripheral;
use crate::time::Hertz;

macro_rules! config_ulpi_pins {
    ($($pin:ident),*) => {
        into_ref!($($pin),*);
        // NOTE(unsafe) Exclusive access to the registers
        critical_section::with(|_| unsafe {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                #[cfg(gpio_v2)]
                $pin.set_speed(crate::gpio::Speed::VeryHigh);
            )*
        })
    };
}

// From `synopsys-usb-otg` crate:
// This calculation doesn't correspond to one in a Reference Manual.
// In fact, the required number of words is higher than indicated in RM.
// The following numbers are pessimistic and were figured out empirically.
const RX_FIFO_EXTRA_SIZE_WORDS: u16 = 30;

/// USB PHY type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PhyType {
    /// Internal Full-Speed PHY
    ///
    /// Available on most High-Speed peripherals.
    InternalFullSpeed,
    /// Internal High-Speed PHY
    ///
    /// Available on a few STM32 chips.
    InternalHighSpeed,
    /// External ULPI High-Speed PHY
    ExternalHighSpeed,
}

impl PhyType {
    pub fn internal(&self) -> bool {
        match self {
            PhyType::InternalFullSpeed | PhyType::InternalHighSpeed => true,
            PhyType::ExternalHighSpeed => false,
        }
    }

    pub fn high_speed(&self) -> bool {
        match self {
            PhyType::InternalFullSpeed => false,
            PhyType::ExternalHighSpeed | PhyType::InternalHighSpeed => true,
        }
    }

    pub fn to_dspd(&self) -> vals::Dspd {
        match self {
            PhyType::InternalFullSpeed => vals::Dspd::FULL_SPEED_INTERNAL,
            PhyType::InternalHighSpeed => vals::Dspd::HIGH_SPEED,
            PhyType::ExternalHighSpeed => vals::Dspd::HIGH_SPEED,
        }
    }
}

/// Indicates that [State::ep_out_buffers] is empty.
const EP_OUT_BUFFER_EMPTY: u16 = u16::MAX;

pub struct State<const EP_COUNT: usize> {
    /// Holds received SETUP packets. Available if [State::ep0_setup_ready] is true.
    ep0_setup_data: UnsafeCell<[u8; 8]>,
    ep0_setup_ready: AtomicBool,
    ep_in_wakers: [AtomicWaker; EP_COUNT],
    ep_out_wakers: [AtomicWaker; EP_COUNT],
    /// RX FIFO is shared so extra buffers are needed to dequeue all data without waiting on each endpoint.
    /// Buffers are ready when associated [State::ep_out_size] != [EP_OUT_BUFFER_EMPTY].
    ep_out_buffers: [UnsafeCell<*mut u8>; EP_COUNT],
    ep_out_size: [AtomicU16; EP_COUNT],
    bus_waker: AtomicWaker,
}

unsafe impl<const EP_COUNT: usize> Send for State<EP_COUNT> {}
unsafe impl<const EP_COUNT: usize> Sync for State<EP_COUNT> {}

impl<const EP_COUNT: usize> State<EP_COUNT> {
    pub const fn new() -> Self {
        const NEW_AW: AtomicWaker = AtomicWaker::new();
        const NEW_BUF: UnsafeCell<*mut u8> = UnsafeCell::new(0 as _);
        const NEW_SIZE: AtomicU16 = AtomicU16::new(EP_OUT_BUFFER_EMPTY);

        Self {
            ep0_setup_data: UnsafeCell::new([0u8; 8]),
            ep0_setup_ready: AtomicBool::new(false),
            ep_in_wakers: [NEW_AW; EP_COUNT],
            ep_out_wakers: [NEW_AW; EP_COUNT],
            ep_out_buffers: [NEW_BUF; EP_COUNT],
            ep_out_size: [NEW_SIZE; EP_COUNT],
            bus_waker: NEW_AW,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EndpointData {
    ep_type: EndpointType,
    max_packet_size: u16,
    fifo_size_words: u16,
}

pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    irq: PeripheralRef<'d, T::Interrupt>,
    ep_in: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out_buffer: &'d mut [u8],
    ep_out_buffer_offset: usize,
    phy_type: PhyType,
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Initializes USB OTG peripheral with internal Full-Speed PHY.
    ///
    /// # Arguments
    ///
    /// * `ep_out_buffer` - An internal buffer used to temporarily store recevied packets.
    /// Must be large enough to fit all OUT endpoint max packet sizes.
    /// Endpoint allocation will fail if it is too small.
    pub fn new_fs(
        _peri: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        dp: impl Peripheral<P = impl DpPin<T>> + 'd,
        dm: impl Peripheral<P = impl DmPin<T>> + 'd,
        ep_out_buffer: &'d mut [u8],
    ) -> Self {
        into_ref!(dp, dm, irq);

        unsafe {
            dp.set_as_af(dp.af_num(), AFType::OutputPushPull);
            dm.set_as_af(dm.af_num(), AFType::OutputPushPull);
        }

        Self {
            phantom: PhantomData,
            irq,
            ep_in: [None; MAX_EP_COUNT],
            ep_out: [None; MAX_EP_COUNT],
            ep_out_buffer,
            ep_out_buffer_offset: 0,
            phy_type: PhyType::InternalFullSpeed,
        }
    }

    /// Initializes USB OTG peripheral with external High-Speed PHY.
    ///
    /// # Arguments
    ///
    /// * `ep_out_buffer` - An internal buffer used to temporarily store recevied packets.
    /// Must be large enough to fit all OUT endpoint max packet sizes.
    /// Endpoint allocation will fail if it is too small.
    pub fn new_hs_ulpi(
        _peri: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        ulpi_clk: impl Peripheral<P = impl UlpiClkPin<T>> + 'd,
        ulpi_dir: impl Peripheral<P = impl UlpiDirPin<T>> + 'd,
        ulpi_nxt: impl Peripheral<P = impl UlpiNxtPin<T>> + 'd,
        ulpi_stp: impl Peripheral<P = impl UlpiStpPin<T>> + 'd,
        ulpi_d0: impl Peripheral<P = impl UlpiD0Pin<T>> + 'd,
        ulpi_d1: impl Peripheral<P = impl UlpiD1Pin<T>> + 'd,
        ulpi_d2: impl Peripheral<P = impl UlpiD2Pin<T>> + 'd,
        ulpi_d3: impl Peripheral<P = impl UlpiD3Pin<T>> + 'd,
        ulpi_d4: impl Peripheral<P = impl UlpiD4Pin<T>> + 'd,
        ulpi_d5: impl Peripheral<P = impl UlpiD5Pin<T>> + 'd,
        ulpi_d6: impl Peripheral<P = impl UlpiD6Pin<T>> + 'd,
        ulpi_d7: impl Peripheral<P = impl UlpiD7Pin<T>> + 'd,
        ep_out_buffer: &'d mut [u8],
    ) -> Self {
        assert!(T::HIGH_SPEED == true, "Peripheral is not capable of high-speed USB");

        config_ulpi_pins!(
            ulpi_clk, ulpi_dir, ulpi_nxt, ulpi_stp, ulpi_d0, ulpi_d1, ulpi_d2, ulpi_d3, ulpi_d4, ulpi_d5, ulpi_d6,
            ulpi_d7
        );

        into_ref!(irq);

        Self {
            phantom: PhantomData,
            irq,
            ep_in: [None; MAX_EP_COUNT],
            ep_out: [None; MAX_EP_COUNT],
            ep_out_buffer,
            ep_out_buffer_offset: 0,
            phy_type: PhyType::ExternalHighSpeed,
        }
    }

    // Returns total amount of words (u32) allocated in dedicated FIFO
    fn allocated_fifo_words(&self) -> u16 {
        RX_FIFO_EXTRA_SIZE_WORDS + ep_fifo_size(&self.ep_out) + ep_fifo_size(&self.ep_in)
    }

    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Endpoint<'d, T, D>, EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval_ms={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval_ms,
            D::dir()
        );

        if D::dir() == Direction::Out {
            if self.ep_out_buffer_offset + max_packet_size as usize >= self.ep_out_buffer.len() {
                error!("Not enough endpoint out buffer capacity");
                return Err(EndpointAllocError);
            }
        };

        let fifo_size_words = match D::dir() {
            Direction::Out => (max_packet_size + 3) / 4,
            // INEPTXFD requires minimum size of 16 words
            Direction::In => u16::max((max_packet_size + 3) / 4, 16),
        };

        if fifo_size_words + self.allocated_fifo_words() > T::FIFO_DEPTH_WORDS {
            error!("Not enough FIFO capacity");
            return Err(EndpointAllocError);
        }

        let eps = match D::dir() {
            Direction::Out => &mut self.ep_out,
            Direction::In => &mut self.ep_in,
        };

        // Find free endpoint slot
        let slot = eps.iter_mut().enumerate().find(|(i, ep)| {
            if *i == 0 && ep_type != EndpointType::Control {
                // reserved for control pipe
                false
            } else {
                ep.is_none()
            }
        });

        let index = match slot {
            Some((index, ep)) => {
                *ep = Some(EndpointData {
                    ep_type,
                    max_packet_size,
                    fifo_size_words,
                });
                index
            }
            None => {
                error!("No free endpoints available");
                return Err(EndpointAllocError);
            }
        };

        trace!("  index={}", index);

        if D::dir() == Direction::Out {
            // Buffer capacity check was done above, now allocation cannot fail
            unsafe {
                *T::state().ep_out_buffers[index].get() =
                    self.ep_out_buffer.as_mut_ptr().offset(self.ep_out_buffer_offset as _);
            }
            self.ep_out_buffer_offset += max_packet_size as usize;
        }

        Ok(Endpoint {
            _phantom: PhantomData,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
        })
    }
}

impl<'d, T: Instance> embassy_usb_driver::Driver<'d> for Driver<'d, T> {
    type EndpointOut = Endpoint<'d, T, Out>;
    type EndpointIn = Endpoint<'d, T, In>;
    type ControlPipe = ControlPipe<'d, T>;
    type Bus = Bus<'d, T>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint(ep_type, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
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

        trace!("start");

        (
            Bus {
                phantom: PhantomData,
                irq: self.irq,
                ep_in: self.ep_in,
                ep_out: self.ep_out,
                phy_type: self.phy_type,
                enabled: false,
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
    irq: PeripheralRef<'d, T::Interrupt>,
    ep_in: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out: [Option<EndpointData>; MAX_EP_COUNT],
    phy_type: PhyType,
    enabled: bool,
}

impl<'d, T: Instance> Bus<'d, T> {
    fn restore_irqs() {
        // SAFETY: atomic write
        unsafe {
            T::regs().gintmsk().write(|w| {
                w.set_usbrst(true);
                w.set_enumdnem(true);
                w.set_usbsuspm(true);
                w.set_wuim(true);
                w.set_iepint(true);
                w.set_oepint(true);
                w.set_rxflvlm(true);
            });
        }
    }
}

impl<'d, T: Instance> Bus<'d, T> {
    fn init_fifo(&mut self) {
        trace!("init_fifo");

        let r = T::regs();

        // Configure RX fifo size. All endpoints share the same FIFO area.
        let rx_fifo_size_words = RX_FIFO_EXTRA_SIZE_WORDS + ep_fifo_size(&self.ep_out);
        trace!("configuring rx fifo size={}", rx_fifo_size_words);

        // SAFETY: register is exclusive to `Bus` with `&mut self`
        unsafe { r.grxfsiz().modify(|w| w.set_rxfd(rx_fifo_size_words)) };

        // Configure TX (USB in direction) fifo size for each endpoint
        let mut fifo_top = rx_fifo_size_words;
        for i in 0..T::ENDPOINT_COUNT {
            if let Some(ep) = self.ep_in[i] {
                trace!(
                    "configuring tx fifo ep={}, offset={}, size={}",
                    i,
                    fifo_top,
                    ep.fifo_size_words
                );

                let dieptxf = if i == 0 { r.dieptxf0() } else { r.dieptxf(i - 1) };

                // SAFETY: register is exclusive to `Bus` with `&mut self`
                unsafe {
                    dieptxf.write(|w| {
                        w.set_fd(ep.fifo_size_words);
                        w.set_sa(fifo_top);
                    });
                }

                fifo_top += ep.fifo_size_words;
            }
        }

        assert!(
            fifo_top <= T::FIFO_DEPTH_WORDS,
            "FIFO allocations exceeded maximum capacity"
        );
    }

    fn configure_endpoints(&mut self) {
        trace!("configure_endpoints");

        let r = T::regs();

        // Configure IN endpoints
        for (index, ep) in self.ep_in.iter().enumerate() {
            if let Some(ep) = ep {
                // SAFETY: DIEPCTL is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    r.diepctl(index).write(|w| {
                        if index == 0 {
                            w.set_mpsiz(ep0_mpsiz(ep.max_packet_size));
                        } else {
                            w.set_mpsiz(ep.max_packet_size);
                            w.set_eptyp(to_eptyp(ep.ep_type));
                            w.set_sd0pid_sevnfrm(true);
                        }
                    });
                });
            }
        }

        // Configure OUT endpoints
        for (index, ep) in self.ep_out.iter().enumerate() {
            if let Some(ep) = ep {
                // SAFETY: DOEPCTL/DOEPTSIZ is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    r.doepctl(index).write(|w| {
                        if index == 0 {
                            w.set_mpsiz(ep0_mpsiz(ep.max_packet_size));
                        } else {
                            w.set_mpsiz(ep.max_packet_size);
                            w.set_eptyp(to_eptyp(ep.ep_type));
                            w.set_sd0pid_sevnfrm(true);
                        }
                    });

                    r.doeptsiz(index).modify(|w| {
                        w.set_xfrsiz(ep.max_packet_size as _);
                        if index == 0 {
                            w.set_rxdpid_stupcnt(1);
                        } else {
                            w.set_pktcnt(1);
                        }
                    });
                });
            }
        }

        // Enable IRQs for allocated endpoints
        // SAFETY: register is exclusive to `Bus` with `&mut self`
        unsafe {
            r.daintmsk().modify(|w| {
                w.set_iepm(ep_irq_mask(&self.ep_in));
                // OUT interrupts not used, handled in RXFLVL
                // w.set_oepm(ep_irq_mask(&self.ep_out));
            });
        }
    }

    fn disable(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();

        <T as RccPeripheral>::disable();

        #[cfg(stm32l4)]
        unsafe {
            crate::pac::PWR.cr2().modify(|w| w.set_usv(false));
            // Cannot disable PWR, because other peripherals might be using it
        }
    }

    fn on_interrupt(_: *mut ()) {
        trace!("irq");
        let r = T::regs();
        let state = T::state();

        // SAFETY: atomic read/write
        let ints = unsafe { r.gintsts().read() };
        if ints.wkupint() || ints.usbsusp() || ints.usbrst() || ints.enumdne() {
            // Mask interrupts and notify `Bus` to process them
            unsafe { r.gintmsk().write(|_| {}) };
            T::state().bus_waker.wake();
        }

        // Handle RX
        // SAFETY: atomic read with no side effects
        while unsafe { r.gintsts().read().rxflvl() } {
            // SAFETY: atomic "pop" register
            let status = unsafe { r.grxstsp().read() };
            let ep_num = status.epnum() as usize;
            let len = status.bcnt() as usize;

            assert!(ep_num < T::ENDPOINT_COUNT);

            match status.pktstsd() {
                vals::Pktstsd::SETUP_DATA_RX => {
                    trace!("SETUP_DATA_RX");
                    assert!(len == 8, "invalid SETUP packet length={}", len);
                    assert!(ep_num == 0, "invalid SETUP packet endpoint={}", ep_num);

                    if state.ep0_setup_ready.load(Ordering::Relaxed) == false {
                        // SAFETY: exclusive access ensured by atomic bool
                        let data = unsafe { &mut *state.ep0_setup_data.get() };
                        // SAFETY: FIFO reads are exclusive to this IRQ
                        unsafe {
                            data[0..4].copy_from_slice(&r.fifo(0).read().0.to_ne_bytes());
                            data[4..8].copy_from_slice(&r.fifo(0).read().0.to_ne_bytes());
                        }
                        state.ep0_setup_ready.store(true, Ordering::Release);
                        state.ep_out_wakers[0].wake();
                    } else {
                        error!("received SETUP before previous finished processing");
                        // discard FIFO
                        // SAFETY: FIFO reads are exclusive to IRQ
                        unsafe {
                            r.fifo(0).read();
                            r.fifo(0).read();
                        }
                    }
                }
                vals::Pktstsd::OUT_DATA_RX => {
                    trace!("OUT_DATA_RX ep={} len={}", ep_num, len);

                    if state.ep_out_size[ep_num].load(Ordering::Acquire) == EP_OUT_BUFFER_EMPTY {
                        // SAFETY: Buffer size is allocated to be equal to endpoint's maximum packet size
                        // We trust the peripheral to not exceed its configured MPSIZ
                        let buf = unsafe { core::slice::from_raw_parts_mut(*state.ep_out_buffers[ep_num].get(), len) };

                        for chunk in buf.chunks_mut(4) {
                            // RX FIFO is shared so always read from fifo(0)
                            // SAFETY: FIFO reads are exclusive to IRQ
                            let data = unsafe { r.fifo(0).read().0 };
                            chunk.copy_from_slice(&data.to_ne_bytes()[0..chunk.len()]);
                        }

                        state.ep_out_size[ep_num].store(len as u16, Ordering::Release);
                        state.ep_out_wakers[ep_num].wake();
                    } else {
                        error!("ep_out buffer overflow index={}", ep_num);

                        // discard FIFO data
                        let len_words = (len + 3) / 4;
                        for _ in 0..len_words {
                            // SAFETY: FIFO reads are exclusive to IRQ
                            unsafe { r.fifo(0).read().data() };
                        }
                    }
                }
                vals::Pktstsd::OUT_DATA_DONE => {
                    trace!("OUT_DATA_DONE ep={}", ep_num);
                }
                vals::Pktstsd::SETUP_DATA_DONE => {
                    trace!("SETUP_DATA_DONE ep={}", ep_num);
                }
                x => trace!("unknown PKTSTS: {}", x.0),
            }
        }

        // IN endpoint interrupt
        if ints.iepint() {
            // SAFETY: atomic read with no side effects
            let mut ep_mask = unsafe { r.daint().read().iepint() };
            let mut ep_num = 0;

            // Iterate over endpoints while there are non-zero bits in the mask
            while ep_mask != 0 {
                if ep_mask & 1 != 0 {
                    // SAFETY: atomic read with no side effects
                    let ep_ints = unsafe { r.diepint(ep_num).read() };

                    // clear all
                    // SAFETY: DIEPINT is exclusive to IRQ
                    unsafe { r.diepint(ep_num).write_value(ep_ints) };

                    // TXFE is cleared in DIEPEMPMSK
                    if ep_ints.txfe() {
                        // SAFETY: DIEPEMPMSK is shared with `Endpoint` so critical section is needed for RMW
                        critical_section::with(|_| unsafe {
                            r.diepempmsk().modify(|w| {
                                w.set_ineptxfem(w.ineptxfem() & !(1 << ep_num));
                            });
                        });
                    }

                    state.ep_in_wakers[ep_num].wake();
                    trace!("in ep={} irq val={:b}", ep_num, ep_ints.0);
                }

                ep_mask >>= 1;
                ep_num += 1;
            }
        }

        // not needed? reception handled in rxflvl
        // OUT endpoint interrupt
        // if ints.oepint() {
        //     let mut ep_mask = r.daint().read().oepint();
        //     let mut ep_num = 0;

        //     while ep_mask != 0 {
        //         if ep_mask & 1 != 0 {
        //             let ep_ints = r.doepint(ep_num).read();
        //             // clear all
        //             r.doepint(ep_num).write_value(ep_ints);
        //             state.ep_out_wakers[ep_num].wake();
        //             trace!("out ep={} irq val={=u32:b}", ep_num, ep_ints.0);
        //         }

        //         ep_mask >>= 1;
        //         ep_num += 1;
        //     }
        // }
    }
}

impl<'d, T: Instance> embassy_usb_driver::Bus for Bus<'d, T> {
    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            // TODO: implement VBUS detection
            if !self.enabled {
                return Poll::Ready(Event::PowerDetected);
            }

            let r = T::regs();

            T::state().bus_waker.register(cx.waker());

            let ints = unsafe { r.gintsts().read() };
            if ints.usbrst() {
                trace!("reset");

                self.init_fifo();
                self.configure_endpoints();

                // Reset address
                // SAFETY: DCFG is shared with `ControlPipe` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    r.dcfg().modify(|w| {
                        w.set_dad(0);
                    });
                });

                // SAFETY: atomic clear on rc_w1 register
                unsafe { r.gintsts().write(|w| w.set_usbrst(true)) }; // clear
                Self::restore_irqs();
            }

            if ints.enumdne() {
                trace!("enumdne");

                // SAFETY: atomic read with no side effects
                let speed = unsafe { r.dsts().read().enumspd() };
                trace!("  speed={}", speed.0);

                // SAFETY: register is only accessed by `Bus` under `&mut self`
                unsafe {
                    r.gusbcfg().modify(|w| {
                        w.set_trdt(calculate_trdt(speed, T::frequency()));
                    })
                };

                // SAFETY: atomic clear on rc_w1 register
                unsafe { r.gintsts().write(|w| w.set_enumdne(true)) }; // clear
                Self::restore_irqs();

                return Poll::Ready(Event::Reset);
            }

            if ints.usbsusp() {
                trace!("suspend");
                // SAFETY: atomic clear on rc_w1 register
                unsafe { r.gintsts().write(|w| w.set_usbsusp(true)) }; // clear
                Self::restore_irqs();
                return Poll::Ready(Event::Suspend);
            }

            if ints.wkupint() {
                trace!("resume");
                // SAFETY: atomic clear on rc_w1 register
                unsafe { r.gintsts().write(|w| w.set_wkupint(true)) }; // clear
                Self::restore_irqs();
                return Poll::Ready(Event::Resume);
            }

            Poll::Pending
        })
        .await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        trace!("endpoint_set_stalled ep={:?} en={}", ep_addr, stalled);

        assert!(
            ep_addr.index() < T::ENDPOINT_COUNT,
            "endpoint_set_stalled index {} out of range",
            ep_addr.index()
        );

        let regs = T::regs();
        match ep_addr.direction() {
            Direction::Out => {
                // SAFETY: DOEPCTL is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    regs.doepctl(ep_addr.index()).modify(|w| {
                        w.set_stall(stalled);
                    });
                });

                T::state().ep_out_wakers[ep_addr.index()].wake();
            }
            Direction::In => {
                // SAFETY: DIEPCTL is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    regs.diepctl(ep_addr.index()).modify(|w| {
                        w.set_stall(stalled);
                    });
                });

                T::state().ep_in_wakers[ep_addr.index()].wake();
            }
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        assert!(
            ep_addr.index() < T::ENDPOINT_COUNT,
            "endpoint_is_stalled index {} out of range",
            ep_addr.index()
        );

        let regs = T::regs();

        // SAFETY: atomic read with no side effects
        match ep_addr.direction() {
            Direction::Out => unsafe { regs.doepctl(ep_addr.index()).read().stall() },
            Direction::In => unsafe { regs.diepctl(ep_addr.index()).read().stall() },
        }
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("endpoint_set_enabled ep={:?} en={}", ep_addr, enabled);

        assert!(
            ep_addr.index() < T::ENDPOINT_COUNT,
            "endpoint_set_enabled index {} out of range",
            ep_addr.index()
        );

        let r = T::regs();
        match ep_addr.direction() {
            Direction::Out => {
                // SAFETY: DOEPCTL is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    // cancel transfer if active
                    if !enabled && r.doepctl(ep_addr.index()).read().epena() {
                        r.doepctl(ep_addr.index()).modify(|w| {
                            w.set_snak(true);
                            w.set_epdis(true);
                        })
                    }

                    r.doepctl(ep_addr.index()).modify(|w| {
                        w.set_usbaep(enabled);
                    })
                });

                // Wake `Endpoint::wait_enabled()`
                T::state().ep_out_wakers[ep_addr.index()].wake();
            }
            Direction::In => {
                // SAFETY: DIEPCTL is shared with `Endpoint` so critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    // cancel transfer if active
                    if !enabled && r.diepctl(ep_addr.index()).read().epena() {
                        r.diepctl(ep_addr.index()).modify(|w| {
                            w.set_snak(true);
                            w.set_epdis(true);
                        })
                    }

                    r.diepctl(ep_addr.index()).modify(|w| {
                        w.set_usbaep(enabled);
                    })
                });

                // Wake `Endpoint::wait_enabled()`
                T::state().ep_in_wakers[ep_addr.index()].wake();
            }
        }
    }

    async fn enable(&mut self) {
        trace!("enable");

        // SAFETY: registers are only accessed by `Bus` under `&mut self`
        unsafe {
            #[cfg(stm32l4)]
            {
                crate::peripherals::PWR::enable();
                critical_section::with(|_| crate::pac::PWR.cr2().modify(|w| w.set_usv(true)));
            }

            #[cfg(stm32h7)]
            {
                // If true, VDD33USB is generated by internal regulator from VDD50USB
                // If false, VDD33USB and VDD50USB must be suplied directly with 3.3V (default on nucleo)
                // TODO: unhardcode
                let internal_regulator = false;

                // Enable USB power
                critical_section::with(|_| {
                    crate::pac::PWR.cr3().modify(|w| {
                        w.set_usb33den(true);
                        w.set_usbregen(internal_regulator);
                    })
                });

                // Wait for USB power to stabilize
                while !crate::pac::PWR.cr3().read().usb33rdy() {}

                // Use internal 48MHz HSI clock. Should be enabled in RCC by default.
                critical_section::with(|_| {
                    crate::pac::RCC
                        .d2ccip2r()
                        .modify(|w| w.set_usbsel(crate::pac::rcc::vals::Usbsel::HSI48))
                });

                // Enable ULPI clock if external PHY is used
                let ulpien = !self.phy_type.internal();
                critical_section::with(|_| {
                    crate::pac::RCC.ahb1enr().modify(|w| {
                        if T::HIGH_SPEED {
                            w.set_usb_otg_hs_ulpien(ulpien);
                        } else {
                            w.set_usb_otg_fs_ulpien(ulpien);
                        }
                    });
                    crate::pac::RCC.ahb1lpenr().modify(|w| {
                        if T::HIGH_SPEED {
                            w.set_usb_otg_hs_ulpilpen(ulpien);
                        } else {
                            w.set_usb_otg_fs_ulpilpen(ulpien);
                        }
                    });
                });
            }

            #[cfg(stm32u5)]
            {
                // Enable USB power
                critical_section::with(|_| {
                    crate::pac::RCC.ahb3enr().modify(|w| {
                        w.set_pwren(true);
                    });
                    cortex_m::asm::delay(2);

                    crate::pac::PWR.svmcr().modify(|w| {
                        w.set_usv(true);
                        w.set_uvmen(true);
                    });
                });

                // Wait for USB power to stabilize
                while !crate::pac::PWR.svmsr().read().vddusbrdy() {}

                // Select HSI48 as USB clock source.
                critical_section::with(|_| {
                    crate::pac::RCC.ccipr1().modify(|w| {
                        w.set_iclksel(crate::pac::rcc::vals::Iclksel::HSI48);
                    })
                });
            }

            <T as RccPeripheral>::enable();
            <T as RccPeripheral>::reset();

            self.irq.set_handler(Self::on_interrupt);
            self.irq.unpend();
            self.irq.enable();

            let r = T::regs();
            let core_id = r.cid().read().0;
            info!("Core id {:08x}", core_id);

            // Wait for AHB ready.
            while !r.grstctl().read().ahbidl() {}

            // Configure as device.
            r.gusbcfg().write(|w| {
                // Force device mode
                w.set_fdmod(true);
                // Enable internal full-speed PHY
                w.set_physel(self.phy_type.internal() && !self.phy_type.high_speed());
            });

            // Configuring Vbus sense and SOF output
            match core_id {
                0x0000_1200 | 0x0000_1100 => {
                    assert!(self.phy_type != PhyType::InternalHighSpeed);

                    r.gccfg_v1().modify(|w| {
                        // Enable internal full-speed PHY, logic is inverted
                        w.set_pwrdwn(self.phy_type.internal());
                    });

                    // F429-like chips have the GCCFG.NOVBUSSENS bit
                    r.gccfg_v1().modify(|w| {
                        w.set_novbussens(true);
                        w.set_vbusasen(false);
                        w.set_vbusbsen(false);
                        w.set_sofouten(false);
                    });
                }
                0x0000_2000 | 0x0000_2100 | 0x0000_2300 | 0x0000_3000 | 0x0000_3100 => {
                    // F446-like chips have the GCCFG.VBDEN bit with the opposite meaning
                    r.gccfg_v2().modify(|w| {
                        // Enable internal full-speed PHY, logic is inverted
                        w.set_pwrdwn(self.phy_type.internal() && !self.phy_type.high_speed());
                        w.set_phyhsen(self.phy_type.internal() && self.phy_type.high_speed());
                    });

                    r.gccfg_v2().modify(|w| {
                        w.set_vbden(false);
                    });

                    // Force B-peripheral session
                    r.gotgctl().modify(|w| {
                        w.set_bvaloen(true);
                        w.set_bvaloval(true);
                    });
                }
                _ => unimplemented!("Unknown USB core id {:X}", core_id),
            }

            // Soft disconnect.
            r.dctl().write(|w| w.set_sdis(true));

            // Set speed.
            r.dcfg().write(|w| {
                w.set_pfivl(vals::Pfivl::FRAME_INTERVAL_80);
                w.set_dspd(self.phy_type.to_dspd());
            });

            // Unmask transfer complete EP interrupt
            r.diepmsk().write(|w| {
                w.set_xfrcm(true);
            });

            // Unmask and clear core interrupts
            Bus::<T>::restore_irqs();
            r.gintsts().write_value(regs::Gintsts(0xFFFF_FFFF));

            // Unmask global interrupt
            r.gahbcfg().write(|w| {
                w.set_gint(true); // unmask global interrupt
            });

            // Connect
            r.dctl().write(|w| w.set_sdis(false));
        }

        self.enabled = true;
    }

    async fn disable(&mut self) {
        trace!("disable");

        Bus::disable(self);

        self.enabled = false;
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

impl<'d, T: Instance> Drop for Bus<'d, T> {
    fn drop(&mut self) {
        Bus::disable(self);
    }
}

trait Dir {
    fn dir() -> Direction;
}

pub enum In {}
impl Dir for In {
    fn dir() -> Direction {
        Direction::In
    }
}

pub enum Out {}
impl Dir for Out {
    fn dir() -> Direction {
        Direction::Out
    }
}

pub struct Endpoint<'d, T: Instance, D> {
    _phantom: PhantomData<(&'d mut T, D)>,
    info: EndpointInfo,
}

impl<'d, T: Instance> embassy_usb_driver::Endpoint for Endpoint<'d, T, In> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        poll_fn(|cx| {
            let ep_index = self.info.addr.index();

            T::state().ep_in_wakers[ep_index].register(cx.waker());

            // SAFETY: atomic read without side effects
            if unsafe { T::regs().diepctl(ep_index).read().usbaep() } {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d, T: Instance> embassy_usb_driver::Endpoint for Endpoint<'d, T, Out> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        poll_fn(|cx| {
            let ep_index = self.info.addr.index();

            T::state().ep_out_wakers[ep_index].register(cx.waker());

            // SAFETY: atomic read without side effects
            if unsafe { T::regs().doepctl(ep_index).read().usbaep() } {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d, T: Instance> embassy_usb_driver::EndpointOut for Endpoint<'d, T, Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        trace!("read start len={}", buf.len());

        poll_fn(|cx| {
            let index = self.info.addr.index();
            let state = T::state();

            state.ep_out_wakers[index].register(cx.waker());

            let len = state.ep_out_size[index].load(Ordering::Relaxed);
            if len != EP_OUT_BUFFER_EMPTY {
                trace!("read done len={}", len);

                if len as usize > buf.len() {
                    return Poll::Ready(Err(EndpointError::BufferOverflow));
                }

                // SAFETY: exclusive access ensured by `ep_out_size` atomic variable
                let data = unsafe { core::slice::from_raw_parts(*state.ep_out_buffers[index].get(), len as usize) };
                buf[..len as usize].copy_from_slice(data);

                // Release buffer
                state.ep_out_size[index].store(EP_OUT_BUFFER_EMPTY, Ordering::Release);

                // SAFETY: DOEPCTL/DOEPTSIZ is shared with `Bus` so a critical section is needed for RMW
                critical_section::with(|_| unsafe {
                    // Receive 1 packet
                    T::regs().doeptsiz(index).modify(|w| {
                        w.set_xfrsiz(self.info.max_packet_size as _);
                        w.set_pktcnt(1);
                    });

                    // Clear NAK to indicate we are ready to receive more data
                    T::regs().doepctl(index).modify(|w| {
                        w.set_cnak(true);
                    });
                });

                Poll::Ready(Ok(len as usize))
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d, T: Instance> embassy_usb_driver::EndpointIn for Endpoint<'d, T, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        trace!("write ep={:?} data={:?}", self.info.addr, buf);
        // info!("w:{}", self.info.addr.index());

        if buf.len() > self.info.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let r = T::regs();
        let index = self.info.addr.index();
        let state = T::state();

        // Wait for previous transfer to complete and check if endpoint is disabled
        poll_fn(|cx| {
            state.ep_in_wakers[index].register(cx.waker());

            // SAFETY: atomic read with no side effects
            let diepctl = unsafe { r.diepctl(index).read() };
            if !diepctl.usbaep() {
                Poll::Ready(Err(EndpointError::Disabled))
            } else if !diepctl.epena() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await?;

        info!("ww");

        if buf.len() > 0 {
            poll_fn(|cx| {
                state.ep_in_wakers[index].register(cx.waker());

                let size_words = (buf.len() + 3) / 4;

                // SAFETY: atomic read with no side effects
                let fifo_space = unsafe { r.dtxfsts(index).read().ineptfsav() as usize };
                if size_words > fifo_space {
                    // Not enough space in fifo, enable tx fifo empty interrupt
                    // SAFETY: DIEPEMPMSK is shared with IRQ so critical section is needed for RMW
                    critical_section::with(|_| unsafe {
                        r.diepempmsk().modify(|w| {
                            w.set_ineptxfem(w.ineptxfem() | (1 << index));
                        });
                    });

                    trace!("tx fifo for ep={} full, waiting for txfe", index);

                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await
        }

        // SAFETY: DIEPTSIZ is exclusive to this endpoint under `&mut self`
        unsafe {
            // Setup transfer size
            r.dieptsiz(index).write(|w| {
                w.set_mcnt(1);
                w.set_pktcnt(1);
                w.set_xfrsiz(buf.len() as _);
            });
        }

        // SAFETY: DIEPCTL is shared with `Bus` so a critical section is needed for RMW
        critical_section::with(|_| unsafe {
            // Enable endpoint
            r.diepctl(index).modify(|w| {
                w.set_cnak(true);
                w.set_epena(true);
            });
        });

        // Write data to FIFO
        for chunk in buf.chunks(4) {
            let mut tmp = [0u8; 4];
            tmp[0..chunk.len()].copy_from_slice(chunk);
            // SAFETY: FIFO is exclusive to this endpoint under `&mut self`
            unsafe { r.fifo(index).write_value(regs::Fifo(u32::from_ne_bytes(tmp))) };
        }

        info!("wd");
        trace!("write done ep={:?}", self.info.addr);

        Ok(())
    }
}

pub struct ControlPipe<'d, T: Instance> {
    _phantom: PhantomData<&'d mut T>,
    max_packet_size: u16,
    ep_in: Endpoint<'d, T, In>,
    ep_out: Endpoint<'d, T, Out>,
}

impl<'d, T: Instance> embassy_usb_driver::ControlPipe for ControlPipe<'d, T> {
    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    async fn setup(&mut self) -> [u8; 8] {
        poll_fn(|cx| {
            let state = T::state();

            state.ep_out_wakers[0].register(cx.waker());

            if state.ep0_setup_ready.load(Ordering::Relaxed) {
                let data = unsafe { *state.ep0_setup_data.get() };
                state.ep0_setup_ready.store(false, Ordering::Release);

                // EP0 should not be controlled by `Bus` so this RMW does not need a critical section
                unsafe {
                    // Receive 1 SETUP packet
                    T::regs().doeptsiz(self.ep_out.info.addr.index()).modify(|w| {
                        w.set_rxdpid_stupcnt(1);
                    });

                    // Clear NAK to indicate we are ready to receive more data
                    T::regs().doepctl(self.ep_out.info.addr.index()).modify(|w| {
                        w.set_cnak(true);
                    });
                }

                trace!("SETUP received: {:?}", data);
                Poll::Ready(data)
            } else {
                trace!("SETUP waiting");
                Poll::Pending
            }
        })
        .await
    }

    async fn data_out(&mut self, buf: &mut [u8], _first: bool, _last: bool) -> Result<usize, EndpointError> {
        trace!("control: data_out");
        let len = self.ep_out.read(buf).await?;
        trace!("control: data_out read: {:?}", &buf[..len]);
        Ok(len)
    }

    async fn data_in(&mut self, data: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in write: {:?}", data);
        self.ep_in.write(data).await?;

        // wait for status response from host after sending the last packet
        if last {
            trace!("control: data_in waiting for status");
            self.ep_out.read(&mut []).await?;
            trace!("control: complete");
        }

        Ok(())
    }

    async fn accept(&mut self) {
        trace!("control: accept");

        self.ep_in.write(&[]).await.ok();

        trace!("control: accept OK");
    }

    async fn reject(&mut self) {
        trace!("control: reject");

        // EP0 should not be controlled by `Bus` so this RMW does not need a critical section
        unsafe {
            let regs = T::regs();
            regs.diepctl(self.ep_in.info.addr.index()).modify(|w| {
                w.set_stall(true);
            });
            regs.doepctl(self.ep_out.info.addr.index()).modify(|w| {
                w.set_stall(true);
            });
        }
    }

    async fn accept_set_address(&mut self, addr: u8) {
        trace!("setting addr: {}", addr);
        critical_section::with(|_| unsafe {
            T::regs().dcfg().modify(|w| {
                w.set_dad(addr);
            });
        });

        // synopsys driver requires accept to be sent after changing address
        self.accept().await
    }
}

/// Translates HAL [EndpointType] into PAC [vals::Eptyp]
fn to_eptyp(ep_type: EndpointType) -> vals::Eptyp {
    match ep_type {
        EndpointType::Control => vals::Eptyp::CONTROL,
        EndpointType::Isochronous => vals::Eptyp::ISOCHRONOUS,
        EndpointType::Bulk => vals::Eptyp::BULK,
        EndpointType::Interrupt => vals::Eptyp::INTERRUPT,
    }
}

/// Calculates total allocated FIFO size in words
fn ep_fifo_size(eps: &[Option<EndpointData>]) -> u16 {
    eps.iter().map(|ep| ep.map(|ep| ep.fifo_size_words).unwrap_or(0)).sum()
}

/// Generates IRQ mask for enabled endpoints
fn ep_irq_mask(eps: &[Option<EndpointData>]) -> u16 {
    eps.iter().enumerate().fold(
        0,
        |mask, (index, ep)| {
            if ep.is_some() {
                mask | (1 << index)
            } else {
                mask
            }
        },
    )
}

/// Calculates MPSIZ value for EP0, which uses special values.
fn ep0_mpsiz(max_packet_size: u16) -> u16 {
    match max_packet_size {
        8 => 0b11,
        16 => 0b10,
        32 => 0b01,
        64 => 0b00,
        other => panic!("Unsupported EP0 size: {}", other),
    }
}

fn calculate_trdt(speed: vals::Dspd, ahb_freq: Hertz) -> u8 {
    match speed {
        vals::Dspd::HIGH_SPEED => {
            // From RM0431 (F72xx), RM0090 (F429), RM0390 (F446)
            if ahb_freq.0 >= 30_000_000 {
                0x9
            } else {
                panic!("AHB frequency is too low")
            }
        }
        vals::Dspd::FULL_SPEED_EXTERNAL | vals::Dspd::FULL_SPEED_INTERNAL => {
            // From RM0431 (F72xx), RM0090 (F429)
            match ahb_freq.0 {
                0..=14_199_999 => panic!("AHB frequency is too low"),
                14_200_000..=14_999_999 => 0xF,
                15_000_000..=15_999_999 => 0xE,
                16_000_000..=17_199_999 => 0xD,
                17_200_000..=18_499_999 => 0xC,
                18_500_000..=19_999_999 => 0xB,
                20_000_000..=21_799_999 => 0xA,
                21_800_000..=23_999_999 => 0x9,
                24_000_000..=27_499_999 => 0x8,
                27_500_000..=31_999_999 => 0x7, // 27.7..32 in code from CubeIDE
                32_000_000..=u32::MAX => 0x6,
            }
        }
        _ => unimplemented!(),
    }
}
