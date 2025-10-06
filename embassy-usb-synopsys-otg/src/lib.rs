#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![allow(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This must go FIRST so that all the other modules see its macros.
mod fmt;

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::{
    Bus as _, Direction, EndpointAddress, EndpointAllocError, EndpointError, EndpointIn, EndpointInfo, EndpointOut,
    EndpointType, Event, Unsupported,
};

use crate::fmt::Bytes;

pub mod otg_v1;

use otg_v1::{Otg, regs, vals};

/// Handle interrupts.
pub unsafe fn on_interrupt<const MAX_EP_COUNT: usize>(r: Otg, state: &State<MAX_EP_COUNT>, ep_count: usize) {
    trace!("irq");

    let ints = r.gintsts().read();
    if ints.wkupint() || ints.usbsusp() || ints.usbrst() || ints.enumdne() || ints.otgint() || ints.srqint() {
        // Mask interrupts and notify `Bus` to process them
        r.gintmsk().write(|w| {
            w.set_iepint(true);
            w.set_oepint(true);
            w.set_rxflvlm(true);
        });
        state.bus_waker.wake();
    }

    // Handle RX
    while r.gintsts().read().rxflvl() {
        let status = r.grxstsp().read();
        trace!("=== status {:08x}", status.0);
        let ep_num = status.epnum() as usize;
        let len = status.bcnt() as usize;

        assert!(ep_num < ep_count);

        match status.pktstsd() {
            vals::Pktstsd::SETUP_DATA_RX => {
                trace!("SETUP_DATA_RX");
                assert!(len == 8, "invalid SETUP packet length={}", len);
                assert!(ep_num == 0, "invalid SETUP packet endpoint={}", ep_num);

                // flushing TX if something stuck in control endpoint
                if r.dieptsiz(ep_num).read().pktcnt() != 0 {
                    r.grstctl().modify(|w| {
                        w.set_txfnum(ep_num as _);
                        w.set_txfflsh(true);
                    });
                    while r.grstctl().read().txfflsh() {}
                }

                let data = &state.cp_state.setup_data;
                data[0].store(r.fifo(0).read().data(), Ordering::Relaxed);
                data[1].store(r.fifo(0).read().data(), Ordering::Relaxed);
            }
            vals::Pktstsd::OUT_DATA_RX => {
                trace!("OUT_DATA_RX ep={} len={}", ep_num, len);

                if state.ep_states[ep_num].out_size.load(Ordering::Acquire) == EP_OUT_BUFFER_EMPTY {
                    // SAFETY: Buffer size is allocated to be equal to endpoint's maximum packet size
                    // We trust the peripheral to not exceed its configured MPSIZ
                    let buf =
                        unsafe { core::slice::from_raw_parts_mut(*state.ep_states[ep_num].out_buffer.get(), len) };

                    let mut chunks = buf.chunks_exact_mut(4);
                    for chunk in &mut chunks {
                        // RX FIFO is shared so always read from fifo(0)
                        let data = r.fifo(0).read().0;
                        chunk.copy_from_slice(&data.to_ne_bytes());
                    }
                    let rem = chunks.into_remainder();
                    if !rem.is_empty() {
                        let data = r.fifo(0).read().0;
                        rem.copy_from_slice(&data.to_ne_bytes()[0..rem.len()]);
                    }

                    state.ep_states[ep_num].out_size.store(len as u16, Ordering::Release);
                    state.ep_states[ep_num].out_waker.wake();
                } else {
                    error!("ep_out buffer overflow index={}", ep_num);

                    // discard FIFO data
                    let len_words = (len + 3) / 4;
                    for _ in 0..len_words {
                        r.fifo(0).read().data();
                    }
                }
            }
            vals::Pktstsd::OUT_DATA_DONE => {
                trace!("OUT_DATA_DONE ep={}", ep_num);
            }
            vals::Pktstsd::SETUP_DATA_DONE => {
                trace!("SETUP_DATA_DONE ep={}", ep_num);
            }
            x => trace!("unknown PKTSTS: {}", x.to_bits()),
        }
    }

    // IN endpoint interrupt
    if ints.iepint() {
        let mut ep_mask = r.daint().read().iepint();
        let mut ep_num = 0;

        // Iterate over endpoints while there are non-zero bits in the mask
        while ep_mask != 0 {
            if ep_mask & 1 != 0 {
                let ep_ints = r.diepint(ep_num).read();

                // clear all
                r.diepint(ep_num).write_value(ep_ints);

                // TXFE is cleared in DIEPEMPMSK
                if ep_ints.txfe() {
                    critical_section::with(|_| {
                        r.diepempmsk().modify(|w| {
                            w.set_ineptxfem(w.ineptxfem() & !(1 << ep_num));
                        });
                    });
                }

                state.ep_states[ep_num].in_waker.wake();
                trace!("in ep={} irq val={:08x}", ep_num, ep_ints.0);
            }

            ep_mask >>= 1;
            ep_num += 1;
        }
    }

    // out endpoint interrupt
    if ints.oepint() {
        trace!("oepint");
        let mut ep_mask = r.daint().read().oepint();
        let mut ep_num = 0;

        // Iterate over endpoints while there are non-zero bits in the mask
        while ep_mask != 0 {
            if ep_mask & 1 != 0 {
                let ep_ints = r.doepint(ep_num).read();
                // clear all
                r.doepint(ep_num).write_value(ep_ints);

                if ep_ints.stup() {
                    state.cp_state.setup_ready.store(true, Ordering::Release);
                }
                state.ep_states[ep_num].out_waker.wake();
                trace!("out ep={} irq val={:08x}", ep_num, ep_ints.0);
            }

            ep_mask >>= 1;
            ep_num += 1;
        }
    }
}

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
    /// External ULPI Full-Speed PHY (or High-Speed PHY in Full-Speed mode)
    ExternalFullSpeed,
    /// External ULPI High-Speed PHY
    ExternalHighSpeed,
}

impl PhyType {
    /// Get whether this PHY is any of the internal types.
    pub fn internal(&self) -> bool {
        match self {
            PhyType::InternalFullSpeed | PhyType::InternalHighSpeed => true,
            PhyType::ExternalHighSpeed | PhyType::ExternalFullSpeed => false,
        }
    }

    /// Get whether this PHY is any of the high-speed types.
    pub fn high_speed(&self) -> bool {
        match self {
            PhyType::InternalFullSpeed | PhyType::ExternalFullSpeed => false,
            PhyType::ExternalHighSpeed | PhyType::InternalHighSpeed => true,
        }
    }

    fn to_dspd(&self) -> vals::Dspd {
        match self {
            PhyType::InternalFullSpeed => vals::Dspd::FULL_SPEED_INTERNAL,
            PhyType::InternalHighSpeed => vals::Dspd::HIGH_SPEED,
            PhyType::ExternalFullSpeed => vals::Dspd::FULL_SPEED_EXTERNAL,
            PhyType::ExternalHighSpeed => vals::Dspd::HIGH_SPEED,
        }
    }
}

/// Indicates that [State::ep_out_buffers] is empty.
const EP_OUT_BUFFER_EMPTY: u16 = u16::MAX;

struct EpState {
    in_waker: AtomicWaker,
    out_waker: AtomicWaker,
    /// RX FIFO is shared so extra buffers are needed to dequeue all data without waiting on each endpoint.
    /// Buffers are ready when associated [State::ep_out_size] != [EP_OUT_BUFFER_EMPTY].
    out_buffer: UnsafeCell<*mut u8>,
    out_size: AtomicU16,
}

// SAFETY: The EndpointAllocator ensures that the buffer points to valid memory exclusive for each endpoint and is
// large enough to hold the maximum packet size. Access to the buffer is synchronized between the USB interrupt and the
// EndpointOut impl using the out_size atomic variable.
unsafe impl Send for EpState {}
unsafe impl Sync for EpState {}

struct ControlPipeSetupState {
    /// Holds received SETUP packets. Available if [Ep0State::setup_ready] is true.
    setup_data: [AtomicU32; 2],
    setup_ready: AtomicBool,
}

/// USB OTG driver state.
pub struct State<const EP_COUNT: usize> {
    cp_state: ControlPipeSetupState,
    ep_states: [EpState; EP_COUNT],
    bus_waker: AtomicWaker,
}

unsafe impl<const EP_COUNT: usize> Send for State<EP_COUNT> {}
unsafe impl<const EP_COUNT: usize> Sync for State<EP_COUNT> {}

impl<const EP_COUNT: usize> State<EP_COUNT> {
    /// Create a new State.
    pub const fn new() -> Self {
        Self {
            cp_state: ControlPipeSetupState {
                setup_data: [const { AtomicU32::new(0) }; 2],
                setup_ready: AtomicBool::new(false),
            },
            ep_states: [const {
                EpState {
                    in_waker: AtomicWaker::new(),
                    out_waker: AtomicWaker::new(),
                    out_buffer: UnsafeCell::new(0 as _),
                    out_size: AtomicU16::new(EP_OUT_BUFFER_EMPTY),
                }
            }; EP_COUNT],
            bus_waker: AtomicWaker::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EndpointData {
    ep_type: EndpointType,
    max_packet_size: u16,
    fifo_size_words: u16,
}

/// USB driver config.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    /// Enable VBUS detection.
    ///
    /// The USB spec requires USB devices monitor for USB cable plug/unplug and react accordingly.
    /// This is done by checking whether there is 5V on the VBUS pin or not.
    ///
    /// If your device is bus-powered (powers itself from the USB host via VBUS), then this is optional.
    /// (If there's no power in VBUS your device would be off anyway, so it's fine to always assume
    /// there's power in VBUS, i.e. the USB cable is always plugged in.)
    ///
    /// If your device is self-powered (i.e. it gets power from a source other than the USB cable, and
    /// therefore can stay powered through USB cable plug/unplug) then you MUST set this to true.
    ///
    /// If you set this to true, you must connect VBUS to PA9 for FS, PB13 for HS, possibly with a
    /// voltage divider. See ST application note AN4879 and the reference manual for more details.
    pub vbus_detection: bool,

    /// Enable transceiver delay.
    ///
    /// Some ULPI PHYs like the Microchip USB334x series require a delay between the ULPI register write that initiates
    /// the HS Chirp and the subsequent transmit command, otherwise the HS Chirp does not get executed and the deivce
    /// enumerates in FS mode. Some USB Link IP like those in the STM32H7 series support adding this delay to work with
    /// the affected PHYs.
    pub xcvrdly: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vbus_detection: false,
            xcvrdly: false,
        }
    }
}

/// USB OTG driver.
pub struct Driver<'d, const MAX_EP_COUNT: usize> {
    config: Config,
    ep_in: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out_buffer: &'d mut [u8],
    ep_out_buffer_offset: usize,
    instance: OtgInstance<'d, MAX_EP_COUNT>,
}

impl<'d, const MAX_EP_COUNT: usize> Driver<'d, MAX_EP_COUNT> {
    /// Initializes the USB OTG peripheral.
    ///
    /// # Arguments
    ///
    /// * `ep_out_buffer` - An internal buffer used to temporarily store received packets.
    /// Must be large enough to fit all OUT endpoint max packet sizes.
    /// Endpoint allocation will fail if it is too small.
    /// * `instance` - The USB OTG peripheral instance and its configuration.
    /// * `config` - The USB driver configuration.
    pub fn new(ep_out_buffer: &'d mut [u8], instance: OtgInstance<'d, MAX_EP_COUNT>, config: Config) -> Self {
        Self {
            config,
            ep_in: [None; MAX_EP_COUNT],
            ep_out: [None; MAX_EP_COUNT],
            ep_out_buffer,
            ep_out_buffer_offset: 0,
            instance,
        }
    }

    /// Returns the total amount of words (u32) allocated in dedicated FIFO.
    fn allocated_fifo_words(&self) -> u16 {
        self.instance.extra_rx_fifo_words + ep_fifo_size(&self.ep_out) + ep_fifo_size(&self.ep_in)
    }

    /// Creates an [`Endpoint`] with the given parameters.
    fn alloc_endpoint<D: Dir>(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Endpoint<'d, D>, EndpointAllocError> {
        trace!(
            "allocating type={:?} mps={:?} interval_ms={}, dir={:?}",
            ep_type,
            max_packet_size,
            interval_ms,
            D::dir()
        );

        if D::dir() == Direction::Out {
            if self.ep_out_buffer_offset + max_packet_size as usize > self.ep_out_buffer.len() {
                error!("Not enough endpoint out buffer capacity");
                return Err(EndpointAllocError);
            }
        };

        let fifo_size_words = match D::dir() {
            Direction::Out => (max_packet_size + 3) / 4,
            // INEPTXFD requires minimum size of 16 words
            Direction::In => u16::max((max_packet_size + 3) / 4, 16),
        };

        if fifo_size_words + self.allocated_fifo_words() > self.instance.fifo_depth_words {
            error!("Not enough FIFO capacity");
            return Err(EndpointAllocError);
        }

        let eps = match D::dir() {
            Direction::Out => &mut self.ep_out[..self.instance.endpoint_count],
            Direction::In => &mut self.ep_in[..self.instance.endpoint_count],
        };

        // Find endpoint slot
        let slot = if let Some(addr) = ep_addr {
            // Use the specified endpoint address
            let requested_index = addr.index();
            if requested_index >= self.instance.endpoint_count {
                return Err(EndpointAllocError);
            }
            if requested_index == 0 && ep_type != EndpointType::Control {
                return Err(EndpointAllocError); // EP0 is reserved for control
            }
            if eps[requested_index].is_some() {
                return Err(EndpointAllocError); // Already allocated
            }
            Some((requested_index, &mut eps[requested_index]))
        } else {
            // Find any free endpoint slot
            eps.iter_mut().enumerate().find(|(i, ep)| {
                if *i == 0 && ep_type != EndpointType::Control {
                    // reserved for control pipe
                    false
                } else {
                    ep.is_none()
                }
            })
        };

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

        let state = &self.instance.state.ep_states[index];
        if D::dir() == Direction::Out {
            // Buffer capacity check was done above, now allocation cannot fail
            unsafe {
                *state.out_buffer.get() = self.ep_out_buffer.as_mut_ptr().offset(self.ep_out_buffer_offset as _);
            }
            self.ep_out_buffer_offset += max_packet_size as usize;
        }

        Ok(Endpoint {
            _phantom: PhantomData,
            regs: self.instance.regs,
            state,
            info: EndpointInfo {
                addr: EndpointAddress::from_parts(index, D::dir()),
                ep_type,
                max_packet_size,
                interval_ms,
            },
        })
    }
}

impl<'d, const MAX_EP_COUNT: usize> embassy_usb_driver::Driver<'d> for Driver<'d, MAX_EP_COUNT> {
    type EndpointOut = Endpoint<'d, Out>;
    type EndpointIn = Endpoint<'d, In>;
    type ControlPipe = ControlPipe<'d>;
    type Bus = Bus<'d, MAX_EP_COUNT>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.alloc_endpoint(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn start(mut self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let ep_out = self
            .alloc_endpoint(EndpointType::Control, None, control_max_packet_size, 0)
            .unwrap();
        let ep_in = self
            .alloc_endpoint(EndpointType::Control, None, control_max_packet_size, 0)
            .unwrap();
        assert_eq!(ep_out.info.addr.index(), 0);
        assert_eq!(ep_in.info.addr.index(), 0);

        trace!("start");

        let regs = self.instance.regs;
        let cp_setup_state = &self.instance.state.cp_state;
        (
            Bus {
                config: self.config,
                ep_in: self.ep_in,
                ep_out: self.ep_out,
                inited: false,
                instance: self.instance,
            },
            ControlPipe {
                max_packet_size: control_max_packet_size,
                setup_state: cp_setup_state,
                ep_out,
                ep_in,
                regs,
            },
        )
    }
}

/// USB bus.
pub struct Bus<'d, const MAX_EP_COUNT: usize> {
    config: Config,
    ep_in: [Option<EndpointData>; MAX_EP_COUNT],
    ep_out: [Option<EndpointData>; MAX_EP_COUNT],
    instance: OtgInstance<'d, MAX_EP_COUNT>,
    inited: bool,
}

impl<'d, const MAX_EP_COUNT: usize> Bus<'d, MAX_EP_COUNT> {
    fn restore_irqs(&mut self) {
        self.instance.regs.gintmsk().write(|w| {
            w.set_usbrst(true);
            w.set_enumdnem(true);
            w.set_usbsuspm(true);
            w.set_wuim(true);
            w.set_iepint(true);
            w.set_oepint(true);
            w.set_rxflvlm(true);
            w.set_srqim(true);
            w.set_otgint(true);
        });
    }

    /// Returns the PHY type.
    pub fn phy_type(&self) -> PhyType {
        self.instance.phy_type
    }

    /// Configures the PHY as a device.
    pub fn configure_as_device(&mut self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;
        r.gusbcfg().write(|w| {
            // Force device mode
            w.set_fdmod(true);
            // Enable internal full-speed PHY
            w.set_physel(phy_type.internal() && !phy_type.high_speed());
        });
    }

    /// Applies configuration specific to
    /// Core ID 0x0000_1100 and 0x0000_1200
    pub fn config_v1(&mut self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;
        assert!(phy_type != PhyType::InternalHighSpeed);

        r.gccfg_v1().modify(|w| {
            // Enable internal full-speed PHY, logic is inverted
            w.set_pwrdwn(phy_type.internal());
        });

        // F429-like chips have the GCCFG.NOVBUSSENS bit
        r.gccfg_v1().modify(|w| {
            w.set_novbussens(!self.config.vbus_detection);
            w.set_vbusasen(false);
            w.set_vbusbsen(self.config.vbus_detection);
            w.set_sofouten(false);
        });
    }

    /// Applies configuration specific to
    /// Core ID 0x0000_2000, 0x0000_2100, 0x0000_2300, 0x0000_3000 and 0x0000_3100
    pub fn config_v2v3(&mut self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;

        // F446-like chips have the GCCFG.VBDEN bit with the opposite meaning
        r.gccfg_v2().modify(|w| {
            // Enable internal full-speed PHY, logic is inverted
            w.set_pwrdwn(phy_type.internal() && !phy_type.high_speed());
            w.set_phyhsen(phy_type.internal() && phy_type.high_speed());
        });

        r.gccfg_v2().modify(|w| {
            w.set_vbden(self.config.vbus_detection);
        });

        // Force B-peripheral session
        r.gotgctl().modify(|w| {
            w.set_bvaloen(!self.config.vbus_detection);
            w.set_bvaloval(true);
        });
    }

    /// Applies configuration specific to
    /// Core ID 0x0000_5000
    pub fn config_v5(&mut self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;

        if phy_type == PhyType::InternalHighSpeed {
            r.gccfg_v3().modify(|w| {
                w.set_vbvaloven(!self.config.vbus_detection);
                w.set_vbvaloval(!self.config.vbus_detection);
                w.set_vbden(self.config.vbus_detection);
            });
        } else {
            r.gotgctl().modify(|w| {
                w.set_bvaloen(!self.config.vbus_detection);
                w.set_bvaloval(!self.config.vbus_detection);
            });
            r.gccfg_v3().modify(|w| {
                w.set_vbden(self.config.vbus_detection);
            });
        }
    }

    fn init(&mut self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;

        // Soft disconnect.
        r.dctl().write(|w| w.set_sdis(true));

        // Set speed.
        r.dcfg().write(|w| {
            w.set_pfivl(vals::Pfivl::FRAME_INTERVAL_80);
            w.set_dspd(phy_type.to_dspd());
            if self.config.xcvrdly {
                w.set_xcvrdly(true);
            }
        });

        // Unmask transfer complete EP interrupt
        r.diepmsk().write(|w| {
            w.set_xfrcm(true);
        });

        // Unmask SETUP received EP interrupt
        r.doepmsk().write(|w| {
            w.set_stupm(true);
        });

        // Unmask and clear core interrupts
        self.restore_irqs();
        r.gintsts().write_value(regs::Gintsts(0xFFFF_FFFF));

        // Unmask global interrupt
        r.gahbcfg().write(|w| {
            w.set_gint(true); // unmask global interrupt
        });

        // Connect
        r.dctl().write(|w| w.set_sdis(false));
    }

    fn init_fifo(&mut self) {
        trace!("init_fifo");

        let regs = self.instance.regs;
        // ERRATA NOTE: Don't interrupt FIFOs being written to. The interrupt
        // handler COULD interrupt us here and do FIFO operations, so ensure
        // the interrupt does not occur.
        critical_section::with(|_| {
            // Configure RX fifo size. All endpoints share the same FIFO area.
            let rx_fifo_size_words = self.instance.extra_rx_fifo_words + ep_fifo_size(&self.ep_out);
            trace!("configuring rx fifo size={}", rx_fifo_size_words);

            regs.grxfsiz().modify(|w| w.set_rxfd(rx_fifo_size_words));

            // Configure TX (USB in direction) fifo size for each endpoint
            let mut fifo_top = rx_fifo_size_words;
            for i in 0..self.instance.endpoint_count {
                if let Some(ep) = self.ep_in[i] {
                    trace!(
                        "configuring tx fifo ep={}, offset={}, size={}",
                        i, fifo_top, ep.fifo_size_words
                    );

                    let dieptxf = if i == 0 { regs.dieptxf0() } else { regs.dieptxf(i - 1) };

                    dieptxf.write(|w| {
                        w.set_fd(ep.fifo_size_words);
                        w.set_sa(fifo_top);
                    });

                    fifo_top += ep.fifo_size_words;
                }
            }

            assert!(
                fifo_top <= self.instance.fifo_depth_words,
                "FIFO allocations exceeded maximum capacity"
            );

            // Flush fifos
            regs.grstctl().write(|w| {
                w.set_rxfflsh(true);
                w.set_txfflsh(true);
                w.set_txfnum(0x10);
            });
        });

        loop {
            let x = regs.grstctl().read();
            if !x.rxfflsh() && !x.txfflsh() {
                break;
            }
        }
    }

    fn configure_endpoints(&mut self) {
        trace!("configure_endpoints");

        let regs = self.instance.regs;

        // Configure IN endpoints
        for (index, ep) in self.ep_in.iter().enumerate() {
            if let Some(ep) = ep {
                critical_section::with(|_| {
                    regs.diepctl(index).write(|w| {
                        if index == 0 {
                            w.set_mpsiz(ep0_mpsiz(ep.max_packet_size));
                        } else {
                            w.set_mpsiz(ep.max_packet_size);
                            w.set_eptyp(to_eptyp(ep.ep_type));
                            w.set_sd0pid_sevnfrm(true);
                            w.set_txfnum(index as _);
                            w.set_snak(true);
                        }
                    });
                });
            }
        }

        // Configure OUT endpoints
        for (index, ep) in self.ep_out.iter().enumerate() {
            if let Some(ep) = ep {
                critical_section::with(|_| {
                    regs.doepctl(index).write(|w| {
                        if index == 0 {
                            w.set_mpsiz(ep0_mpsiz(ep.max_packet_size));
                        } else {
                            w.set_mpsiz(ep.max_packet_size);
                            w.set_eptyp(to_eptyp(ep.ep_type));
                            w.set_sd0pid_sevnfrm(true);
                        }
                    });

                    regs.doeptsiz(index).modify(|w| {
                        w.set_xfrsiz(ep.max_packet_size as _);
                        if index == 0 {
                            w.set_rxdpid_stupcnt(3);
                        } else {
                            w.set_pktcnt(1);
                        }
                    });
                });
            }
        }

        // Enable IRQs for allocated endpoints
        regs.daintmsk().modify(|w| {
            w.set_iepm(ep_irq_mask(&self.ep_in));
            w.set_oepm(ep_irq_mask(&self.ep_out));
        });
    }

    fn disable_all_endpoints(&mut self) {
        for i in 0..self.instance.endpoint_count {
            self.endpoint_set_enabled(EndpointAddress::from_parts(i, Direction::In), false);
            self.endpoint_set_enabled(EndpointAddress::from_parts(i, Direction::Out), false);
        }
    }
}

impl<'d, const MAX_EP_COUNT: usize> embassy_usb_driver::Bus for Bus<'d, MAX_EP_COUNT> {
    async fn poll(&mut self) -> Event {
        poll_fn(move |cx| {
            if !self.inited {
                self.init();
                self.inited = true;

                // If no vbus detection, just return a single PowerDetected event at startup.
                if !self.config.vbus_detection {
                    return Poll::Ready(Event::PowerDetected);
                }
            }

            let regs = self.instance.regs;
            self.instance.state.bus_waker.register(cx.waker());

            let ints = regs.gintsts().read();

            if ints.srqint() {
                trace!("vbus detected");

                regs.gintsts().write(|w| w.set_srqint(true)); // clear
                self.restore_irqs();

                if self.config.vbus_detection {
                    return Poll::Ready(Event::PowerDetected);
                }
            }

            if ints.otgint() {
                let otgints = regs.gotgint().read();
                regs.gotgint().write_value(otgints); // clear all
                self.restore_irqs();

                if otgints.sedet() {
                    trace!("vbus removed");
                    if self.config.vbus_detection {
                        self.disable_all_endpoints();
                        return Poll::Ready(Event::PowerRemoved);
                    }
                }
            }

            if ints.usbrst() {
                trace!("reset");

                self.init_fifo();
                self.configure_endpoints();

                // Reset address
                critical_section::with(|_| {
                    regs.dcfg().modify(|w| {
                        w.set_dad(0);
                    });
                });

                regs.gintsts().write(|w| w.set_usbrst(true)); // clear
                self.restore_irqs();
            }

            if ints.enumdne() {
                trace!("enumdne");

                let speed = regs.dsts().read().enumspd();
                let trdt = (self.instance.calculate_trdt_fn)(speed);
                trace!("  speed={} trdt={}", speed.to_bits(), trdt);
                regs.gusbcfg().modify(|w| w.set_trdt(trdt));

                regs.gintsts().write(|w| w.set_enumdne(true)); // clear
                self.restore_irqs();

                return Poll::Ready(Event::Reset);
            }

            if ints.usbsusp() {
                trace!("suspend");
                regs.gintsts().write(|w| w.set_usbsusp(true)); // clear
                self.restore_irqs();
                return Poll::Ready(Event::Suspend);
            }

            if ints.wkupint() {
                trace!("resume");
                regs.gintsts().write(|w| w.set_wkupint(true)); // clear
                self.restore_irqs();
                return Poll::Ready(Event::Resume);
            }

            Poll::Pending
        })
        .await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        trace!("endpoint_set_stalled ep={:?} en={}", ep_addr, stalled);

        assert!(
            ep_addr.index() < self.instance.endpoint_count,
            "endpoint_set_stalled index {} out of range",
            ep_addr.index()
        );

        let regs = self.instance.regs;
        let state = self.instance.state;
        match ep_addr.direction() {
            Direction::Out => {
                critical_section::with(|_| {
                    regs.doepctl(ep_addr.index()).modify(|w| {
                        w.set_stall(stalled);
                    });
                });

                state.ep_states[ep_addr.index()].out_waker.wake();
            }
            Direction::In => {
                critical_section::with(|_| {
                    regs.diepctl(ep_addr.index()).modify(|w| {
                        w.set_stall(stalled);
                    });
                });

                state.ep_states[ep_addr.index()].in_waker.wake();
            }
        }
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        assert!(
            ep_addr.index() < self.instance.endpoint_count,
            "endpoint_is_stalled index {} out of range",
            ep_addr.index()
        );

        let regs = self.instance.regs;
        match ep_addr.direction() {
            Direction::Out => regs.doepctl(ep_addr.index()).read().stall(),
            Direction::In => regs.diepctl(ep_addr.index()).read().stall(),
        }
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        trace!("endpoint_set_enabled ep={:?} en={}", ep_addr, enabled);

        assert!(
            ep_addr.index() < self.instance.endpoint_count,
            "endpoint_set_enabled index {} out of range",
            ep_addr.index()
        );

        let regs = self.instance.regs;
        let state = self.instance.state;
        match ep_addr.direction() {
            Direction::Out => {
                critical_section::with(|_| {
                    // cancel transfer if active
                    if !enabled && regs.doepctl(ep_addr.index()).read().epena() {
                        regs.doepctl(ep_addr.index()).modify(|w| {
                            w.set_snak(true);
                            w.set_epdis(true);
                        })
                    }

                    regs.doepctl(ep_addr.index()).modify(|w| {
                        w.set_usbaep(enabled);
                    });

                    // Flush tx fifo
                    regs.grstctl().write(|w| {
                        w.set_txfflsh(true);
                        w.set_txfnum(ep_addr.index() as _);
                    });
                    loop {
                        let x = regs.grstctl().read();
                        if !x.txfflsh() {
                            break;
                        }
                    }
                });

                // Wake `Endpoint::wait_enabled()`
                state.ep_states[ep_addr.index()].out_waker.wake();
            }
            Direction::In => {
                critical_section::with(|_| {
                    // cancel transfer if active
                    if !enabled && regs.diepctl(ep_addr.index()).read().epena() {
                        regs.diepctl(ep_addr.index()).modify(|w| {
                            w.set_snak(true); // set NAK
                            w.set_epdis(true);
                        })
                    }

                    regs.diepctl(ep_addr.index()).modify(|w| {
                        w.set_usbaep(enabled);
                        w.set_cnak(enabled); // clear NAK that might've been set by SNAK above.
                    })
                });

                // Wake `Endpoint::wait_enabled()`
                state.ep_states[ep_addr.index()].in_waker.wake();
            }
        }
    }

    async fn enable(&mut self) {
        trace!("enable");
        // TODO: enable the peripheral once enable/disable semantics are cleared up in embassy-usb
    }

    async fn disable(&mut self) {
        trace!("disable");

        // TODO: disable the peripheral once enable/disable semantics are cleared up in embassy-usb
        //Bus::disable(self);
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

/// USB endpoint direction.
trait Dir {
    /// Returns the direction value.
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
pub struct Endpoint<'d, D> {
    _phantom: PhantomData<D>,
    regs: Otg,
    info: EndpointInfo,
    state: &'d EpState,
}

impl<'d> embassy_usb_driver::Endpoint for Endpoint<'d, In> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        poll_fn(|cx| {
            let ep_index = self.info.addr.index();

            self.state.in_waker.register(cx.waker());

            if self.regs.diepctl(ep_index).read().usbaep() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d> embassy_usb_driver::Endpoint for Endpoint<'d, Out> {
    fn info(&self) -> &EndpointInfo {
        &self.info
    }

    async fn wait_enabled(&mut self) {
        poll_fn(|cx| {
            let ep_index = self.info.addr.index();

            self.state.out_waker.register(cx.waker());

            if self.regs.doepctl(ep_index).read().usbaep() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d> embassy_usb_driver::EndpointOut for Endpoint<'d, Out> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        trace!("read start len={}", buf.len());

        poll_fn(|cx| {
            let index = self.info.addr.index();
            self.state.out_waker.register(cx.waker());

            let doepctl = self.regs.doepctl(index).read();
            trace!("read ep={:?}: doepctl {:08x}", self.info.addr, doepctl.0,);
            if !doepctl.usbaep() {
                trace!("read ep={:?} error disabled", self.info.addr);
                return Poll::Ready(Err(EndpointError::Disabled));
            }

            let len = self.state.out_size.load(Ordering::Relaxed);
            if len != EP_OUT_BUFFER_EMPTY {
                trace!("read ep={:?} done len={}", self.info.addr, len);

                if len as usize > buf.len() {
                    return Poll::Ready(Err(EndpointError::BufferOverflow));
                }

                // SAFETY: exclusive access ensured by `out_size` atomic variable
                let data = unsafe { core::slice::from_raw_parts(*self.state.out_buffer.get(), len as usize) };
                buf[..len as usize].copy_from_slice(data);

                // Release buffer
                self.state.out_size.store(EP_OUT_BUFFER_EMPTY, Ordering::Release);

                critical_section::with(|_| {
                    // Receive 1 packet
                    self.regs.doeptsiz(index).modify(|w| {
                        w.set_xfrsiz(self.info.max_packet_size as _);
                        w.set_pktcnt(1);
                    });

                    if self.info.ep_type == EndpointType::Isochronous {
                        // Isochronous endpoints must set the correct even/odd frame bit to
                        // correspond with the next frame's number.
                        let frame_number = self.regs.dsts().read().fnsof();
                        let frame_is_odd = frame_number & 0x01 == 1;

                        self.regs.doepctl(index).modify(|r| {
                            if frame_is_odd {
                                r.set_sd0pid_sevnfrm(true);
                            } else {
                                r.set_sd1pid_soddfrm(true);
                            }
                        });
                    }

                    // Clear NAK to indicate we are ready to receive more data
                    self.regs.doepctl(index).modify(|w| {
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

impl<'d> embassy_usb_driver::EndpointIn for Endpoint<'d, In> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        trace!("write ep={:?} data={:?}", self.info.addr, Bytes(buf));

        if buf.len() > self.info.max_packet_size as usize {
            return Err(EndpointError::BufferOverflow);
        }

        let index = self.info.addr.index();
        // Wait for previous transfer to complete and check if endpoint is disabled
        poll_fn(|cx| {
            self.state.in_waker.register(cx.waker());

            let diepctl = self.regs.diepctl(index).read();
            let dtxfsts = self.regs.dtxfsts(index).read();
            trace!(
                "write ep={:?}: diepctl {:08x} ftxfsts {:08x}",
                self.info.addr, diepctl.0, dtxfsts.0
            );
            if !diepctl.usbaep() {
                trace!("write ep={:?} wait for prev: error disabled", self.info.addr);
                Poll::Ready(Err(EndpointError::Disabled))
            } else if !diepctl.epena() {
                trace!("write ep={:?} wait for prev: ready", self.info.addr);
                Poll::Ready(Ok(()))
            } else {
                trace!("write ep={:?} wait for prev: pending", self.info.addr);
                Poll::Pending
            }
        })
        .await?;

        if buf.len() > 0 {
            poll_fn(|cx| {
                self.state.in_waker.register(cx.waker());

                let size_words = (buf.len() + 3) / 4;

                let fifo_space = self.regs.dtxfsts(index).read().ineptfsav() as usize;
                if size_words > fifo_space {
                    // Not enough space in fifo, enable tx fifo empty interrupt
                    critical_section::with(|_| {
                        self.regs.diepempmsk().modify(|w| {
                            w.set_ineptxfem(w.ineptxfem() | (1 << index));
                        });
                    });

                    trace!("tx fifo for ep={} full, waiting for txfe", index);

                    Poll::Pending
                } else {
                    trace!("write ep={:?} wait for fifo: ready", self.info.addr);
                    Poll::Ready(())
                }
            })
            .await
        }

        // ERRATA: Transmit data FIFO is corrupted when a write sequence to the FIFO is interrupted with
        // accesses to certain OTG_FS registers.
        //
        // Prevent the interrupt (which might poke FIFOs) from executing while copying data to FIFOs.
        critical_section::with(|_| {
            // Setup transfer size
            self.regs.dieptsiz(index).write(|w| {
                w.set_mcnt(1);
                w.set_pktcnt(1);
                w.set_xfrsiz(buf.len() as _);
            });

            if self.info.ep_type == EndpointType::Isochronous {
                // Isochronous endpoints must set the correct even/odd frame bit to
                // correspond with the next frame's number.
                let frame_number = self.regs.dsts().read().fnsof();
                let frame_is_odd = frame_number & 0x01 == 1;

                self.regs.diepctl(index).modify(|r| {
                    if frame_is_odd {
                        r.set_sd0pid_sevnfrm(true);
                    } else {
                        r.set_sd1pid_soddfrm(true);
                    }
                });
            }

            // Enable endpoint
            self.regs.diepctl(index).modify(|w| {
                w.set_cnak(true);
                w.set_epena(true);
            });

            // Write data to FIFO
            let fifo = self.regs.fifo(index);
            let mut chunks = buf.chunks_exact(4);
            for chunk in &mut chunks {
                let val = u32::from_ne_bytes(chunk.try_into().unwrap());
                fifo.write_value(regs::Fifo(val));
            }
            // Write any last chunk
            let rem = chunks.remainder();
            if !rem.is_empty() {
                let mut tmp = [0u8; 4];
                tmp[0..rem.len()].copy_from_slice(rem);
                let tmp = u32::from_ne_bytes(tmp);
                fifo.write_value(regs::Fifo(tmp));
            }
        });

        trace!("write done ep={:?}", self.info.addr);

        Ok(())
    }
}

/// USB control pipe.
pub struct ControlPipe<'d> {
    max_packet_size: u16,
    regs: Otg,
    setup_state: &'d ControlPipeSetupState,
    ep_in: Endpoint<'d, In>,
    ep_out: Endpoint<'d, Out>,
}

impl<'d> embassy_usb_driver::ControlPipe for ControlPipe<'d> {
    fn max_packet_size(&self) -> usize {
        usize::from(self.max_packet_size)
    }

    async fn setup(&mut self) -> [u8; 8] {
        poll_fn(|cx| {
            self.ep_out.state.out_waker.register(cx.waker());

            if self.setup_state.setup_ready.load(Ordering::Relaxed) {
                let mut data = [0; 8];
                data[0..4].copy_from_slice(&self.setup_state.setup_data[0].load(Ordering::Relaxed).to_ne_bytes());
                data[4..8].copy_from_slice(&self.setup_state.setup_data[1].load(Ordering::Relaxed).to_ne_bytes());
                self.setup_state.setup_ready.store(false, Ordering::Release);

                // EP0 should not be controlled by `Bus` so this RMW does not need a critical section
                self.regs.doeptsiz(self.ep_out.info.addr.index()).modify(|w| {
                    w.set_rxdpid_stupcnt(3);
                });

                // Clear NAK to indicate we are ready to receive more data
                self.regs
                    .doepctl(self.ep_out.info.addr.index())
                    .modify(|w| w.set_cnak(true));

                trace!("SETUP received: {:?}", Bytes(&data));
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
        trace!("control: data_out read: {:?}", Bytes(&buf[..len]));
        Ok(len)
    }

    async fn data_in(&mut self, data: &[u8], _first: bool, last: bool) -> Result<(), EndpointError> {
        trace!("control: data_in write: {:?}", Bytes(data));
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
        self.regs.diepctl(self.ep_in.info.addr.index()).modify(|w| {
            w.set_stall(true);
        });
        self.regs.doepctl(self.ep_out.info.addr.index()).modify(|w| {
            w.set_stall(true);
        });
    }

    async fn accept_set_address(&mut self, addr: u8) {
        trace!("setting addr: {}", addr);
        critical_section::with(|_| {
            self.regs.dcfg().modify(|w| {
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
            if ep.is_some() { mask | (1 << index) } else { mask }
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

/// Hardware-dependent USB IP configuration.
pub struct OtgInstance<'d, const MAX_EP_COUNT: usize> {
    /// The USB peripheral.
    pub regs: Otg,
    /// The USB state.
    pub state: &'d State<MAX_EP_COUNT>,
    /// FIFO depth in words.
    pub fifo_depth_words: u16,
    /// Number of used endpoints.
    pub endpoint_count: usize,
    /// The PHY type.
    pub phy_type: PhyType,
    /// Extra RX FIFO words needed by some implementations.
    pub extra_rx_fifo_words: u16,
    /// Function to calculate TRDT value based on some internal clock speed.
    pub calculate_trdt_fn: fn(speed: vals::Dspd) -> u8,
}
