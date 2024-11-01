use core::sync::atomic::AtomicBool;
use core::{future::poll_fn, sync::atomic::AtomicU32, task::Poll};

use embassy_sync::waitqueue::AtomicWaker;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, lazy_lock::LazyLock, mutex::Mutex};
use embassy_time::Timer;
use embassy_usb_driver::host::channel::IsControl;
use embassy_usb_driver::{
    host::{
        channel::{self, Direction, Type},
        ChannelError, DeviceEvent, HostError, SetupPacket, UsbChannel, UsbHostDriver,
    },
    EndpointInfo, EndpointType,
};

use crate::otg_v1::{
    vals::{Dpid, Eptyp},
    Otg,
};

extern crate alloc;

const HPRT_W1C_MASK: u32 = 0x2E; // Mask of interrupts inside HPRT; used to avoid eating interrupts (e.g. SOF)
const GINTST_RES_MASK: u32 = 0x08010300;

const OTG_MAX_PIPES: usize = 8;

/// First bit is used to indicate control pipes
static ALLOCATED_PIPES: AtomicU32 = AtomicU32::new(0);
const NEW_AW: AtomicWaker = AtomicWaker::new();
static EP_IN_WAKERS: [AtomicWaker; OTG_MAX_PIPES] = [NEW_AW; OTG_MAX_PIPES];

#[must_use = "need to hold until finished"]
#[clippy::has_significant_drop]
struct SharedChannelGuard {
    channel_idx: u8,
}

impl SharedChannelGuard {
    async fn try_claim(channel_idx: u8) -> SharedChannelGuard {
        loop {
            let current_state = ALLOCATED_PIPES.load(core::sync::atomic::Ordering::Acquire);
            if ALLOCATED_PIPES
                .compare_exchange_weak(
                    current_state,
                    current_state | 1,
                    core::sync::atomic::Ordering::Acquire,
                    core::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }

            // Claim failed; defer
            embassy_time::Timer::after_micros(1).await;
        }

        SharedChannelGuard { channel_idx }
    }
}

impl Drop for SharedChannelGuard {
    fn drop(&mut self) {
        ALLOCATED_PIPES.fetch_and(!(1 << self.channel_idx), core::sync::atomic::Ordering::AcqRel);
    }
}

/// Buffer-DMA implementation of USBOTG host driver
pub struct UsbHostBus {
    regs: Otg,
    dev_conn: AtomicBool,
}

fn dma_alloc_buffer<T>(length: usize, align: usize) -> &'static mut [T] {
    let size = core::mem::size_of::<T>();
    let layout = core::alloc::Layout::from_size_align(size * length, align).unwrap();
    unsafe {
        let ptr = alloc::alloc::alloc(layout);
        if ptr.is_null() {
            error!("make_buffers: alloc failed");
            alloc::alloc::handle_alloc_error(layout);
        }
        core::slice::from_raw_parts_mut(ptr as *mut T, length)
    }
}

unsafe fn dma_dealloc_buffer<T>(buf: &mut [T], align: usize) {
    alloc::alloc::dealloc(
        buf.as_mut_ptr() as *mut u8,
        core::alloc::Layout::from_size_align(core::mem::size_of_val(buf), align).unwrap(),
    );
}

/// A software-interrupt-pipe internval handler that doesn't require async
// In order to implement interrupt pipes in fifo or buffer-dma, we'll need to keep track of intervals ourselves
//  luckily we have a handy frame reference in hfnum. So we can store the interval & calculated next hfnum for trigger (w/ wrapping)
struct HfnumInterruptInterval {
    interval: u16,
    next_hfnum: u16,
    paused: bool,
}

impl HfnumInterruptInterval {
    pub fn new() -> Self {
        HfnumInterruptInterval {
            interval: 0,
            next_hfnum: 0,
            paused: false,
        }
    }

    pub fn set_interval(&mut self, frame_interval: u16) {
        self.interval = frame_interval
    }

    fn check_and_reset_interval(&mut self, hfnum: u16) -> bool {
        if self.interval == 0 || self.paused {
            return false; // No interval set
        }

        if self.next_hfnum.wrapping_sub(hfnum) & 0x3fff > self.interval {
            self.next_hfnum = hfnum;
            return true;
        }
        false
    }
}

pub struct OtgChannel<T: Type, D: Direction> {
    regs: Otg,
    channel_idx: u8,
    interrupt_interval: HfnumInterruptInterval,
    buffer: &'static mut [u8],
    pid: Dpid,

    device_addr: u8,
    endpoint: EndpointInfo,
    ls_pre: bool,

    phantom_type: core::marker::PhantomData<T>,
    phantom_dir: core::marker::PhantomData<D>,
}

impl<T: Type, D: Direction> OtgChannel<T, D> {
    #[must_use = "Expects to be further intialized using `retarget_channel`"]
    fn new_alloc(otg: Otg, channel_idx: u8, buffer_size: usize) -> Self {
        OtgChannel {
            regs: otg,
            channel_idx,

            interrupt_interval: HfnumInterruptInterval::new(),
            pid: Dpid::DATA0,

            device_addr: 0,
            // NOTE: this will be overwritten with retarget_channel
            endpoint: EndpointInfo::new(0.into(), T::ep_type(), 8),
            ls_pre: false,

            buffer: dma_alloc_buffer(buffer_size, 4),
            phantom_type: core::marker::PhantomData,
            phantom_dir: core::marker::PhantomData,
        }
    }

    fn flip_pid(&mut self) {
        self.pid = match self.pid {
            Dpid::DATA0 => Dpid::DATA1,
            Dpid::DATA1 => Dpid::DATA0,
            _ => todo!("Weird state"),
        }
    }

    fn configure_for_endpoint(&mut self, direction_override: Option<embassy_usb_driver::Direction>) {
        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_dad(self.device_addr);
            w.set_lsdev(self.ls_pre);

            w.set_epnum(self.endpoint.addr.into());
            w.set_eptyp(Eptyp::from_bits(self.endpoint.ep_type as u8));
            w.set_mpsiz(self.endpoint.max_packet_size);

            w.set_epdir(
                match direction_override.unwrap_or_else(|| self.endpoint.addr.direction()) {
                    embassy_usb_driver::Direction::In => true,
                    embassy_usb_driver::Direction::Out => false,
                },
            );

            w.set_chena(false);
            w.set_chdis(false);
        });
    }

    fn write_setup(&mut self, setup: &SetupPacket) -> Result<(), ChannelError> {
        let setup_data = setup.as_bytes();

        self.configure_for_endpoint(Some(embassy_usb_driver::Direction::Out));

        let txs = setup_data.len() as u32;
        self.regs.hctsiz(self.channel_idx as usize).modify(|w| {
            w.set_pktcnt(1);
            w.set_xfrsiz(txs);
            w.set_dpid(Dpid::MDATA.into());
            w.set_doping(false);
        });

        self.buffer[..setup_data.len()].copy_from_slice(setup_data);
        // HCDMA gets auto-incremented so we need to set it before each tx
        self.regs
            .hcdma(self.channel_idx as usize)
            .write(|w| w.0 = self.buffer.as_ptr() as u32);

        self.regs.gintmsk().modify(|w| {
            w.set_hcim(true);
        });

        self.regs.hcintmsk(self.channel_idx as usize).modify(|w| {
            w.set_xfrcm(true);
            w.set_chhm(true);
            w.set_stallm(true);
            w.set_txerrm(true);
            w.set_bberrm(true);
            w.set_frmorm(true);
            w.set_dterrm(true);
        });

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_chena(true);
            w.set_chdis(false);
        });

        self.pid = Dpid::DATA1;
        Ok(())
    }

    async fn wait_for_txresult(&mut self) -> Result<(), ChannelError> {
        poll_fn(|cx| {
            // FIXME: add timeout
            let hcintr = self.regs.hcint(self.channel_idx as usize).read();

            trace!(
                "Polling wait_for_txresult: chintr={}, hcintr={}",
                self.channel_idx,
                hcintr.0
            );

            if hcintr.stall() {
                self.regs.hcint(self.channel_idx as usize).write(|w| w.set_stall(true));
                return Poll::Ready(Err(ChannelError::Stall));
            }

            if hcintr.txerr() || hcintr.bberr() {
                self.regs.hcint(self.channel_idx as usize).write(|w| {
                    w.set_txerr(true);
                    w.set_bberr(true);
                });
                self.regs.hcchar(self.channel_idx as usize).modify(|w| {
                    w.set_chena(false);
                    w.set_chdis(true);
                });
                return Poll::Ready(Err(ChannelError::BadResponse));
            }

            // NOTE: these are not needed but useful to log
            if hcintr.frmor() {
                debug!("Framme overrun");
                //     self.interrupt_interval.2 = false; // Pause interrupt channel
                //     self.regs.hcint(self.channel_idx as usize).write(|w| w.set_frmor(true));
            }

            if hcintr.dterr() {
                debug!("Data toggle error");
                //     self.interrupt_interval.2 = false; // Pause interrupt channel
                //     self.regs.hcint(self.channel_idx as usize).write(|w| w.set_dterr(true));
            }

            if hcintr.xfrc() {
                // Transfer was completed
                assert!(hcintr.ack(), "Didn't get ACK, but transfer was complete");

                self.regs.hcchar(self.channel_idx as usize).modify(|w| {
                    // Disable channel for next trx
                    w.set_chena(false);
                    w.set_chdis(false);
                });

                self.regs.hcint(self.channel_idx as usize).write(|w| {
                    w.set_xfrc(true);
                    w.set_ack(true);
                });

                return Poll::Ready(Ok(()));
            }

            // Need to check this after xfrc, since xfrc can cause a halt
            if hcintr.chh() {
                //     // Channel halted, transaction canceled
                //     // TODO[CherryUSB]: apparently Control endpoints do something when at INDATA state?
                trace!("Halted");
                self.regs.hcint(self.channel_idx as usize).write(|w| w.set_chh(true));
                //     Err(ChannelError::Canceled)?
            }

            EP_IN_WAKERS[self.channel_idx as usize].register(cx.waker());

            // Re-enable the interrupt this handled
            self.regs.haintmsk().modify(|w| {
                w.0 |= 1 << self.channel_idx as u16;
            });

            Poll::Pending
        })
        .await
    }
}

impl<T: Type, D: Direction> Drop for OtgChannel<T, D> {
    fn drop(&mut self) {
        if self.channel_idx != 0 {
            ALLOCATED_PIPES.fetch_nand(!(1 << self.channel_idx), core::sync::atomic::Ordering::AcqRel);
        }
        // Cancel any active txs & disable interrupts
        self.regs.hcchar(self.channel_idx as usize).write(|w| w.set_chdis(true));
        self.regs.hcint(self.channel_idx as usize).write(|w| w.0 = 0);
        unsafe {
            dma_dealloc_buffer(self.buffer, 512);
        }
    }
}

impl<T: Type, D: Direction> UsbChannel<T, D> for OtgChannel<T, D> {
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsIn,
    {
        let _ = SharedChannelGuard::try_claim(0).await;

        trace!("trying CTRL_IN setup={}, rt={}", setup, setup.request_type.bits());
        self.write_setup(setup)?;
        trace!("Wating for setup ack");
        self.wait_for_txresult().await?;

        self.configure_for_endpoint(Some(embassy_usb_driver::Direction::In));

        let transfer_size: u32 = setup.length as u32;
        trace!(
            "Finished setup; trying CTRL_IN transfer pid={}, xfrsize={}, mps={}, ep_num={}, dad={}",
            self.pid as u8,
            transfer_size,
            self.endpoint.max_packet_size,
            u8::from(self.endpoint.addr),
            self.device_addr
        );

        self.regs.hctsiz(self.channel_idx as usize).modify(|w| {
            w.set_pktcnt(transfer_size.div_ceil(self.endpoint.max_packet_size as u32).max(1) as u16);
            w.set_xfrsiz(w.pktcnt() as u32 * self.endpoint.max_packet_size as u32);
            w.set_dpid(self.pid.into()); // Control always DATA1
            w.set_doping(false);
        });

        self.regs
            .hcdma(self.channel_idx as usize)
            .write(|w| w.0 = self.buffer.as_ptr() as u32);

        self.regs.gintmsk().modify(|w| {
            w.set_hcim(true);
        });

        self.regs.hcintmsk(self.channel_idx as usize).modify(|w| {
            w.set_xfrcm(true);
            w.set_chhm(true);
            w.set_stallm(true);
            w.set_txerrm(true);
            w.set_bberrm(true);
            w.set_frmorm(true);
            w.set_dterrm(true);
        });

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_chena(true);
            w.set_chdis(false);
        });

        trace!("Wating for CNTRL_IN Ack");
        self.wait_for_txresult().await?;
        buf[..transfer_size as usize].copy_from_slice(&self.buffer[..transfer_size as usize]);

        // TODO: this is kind of useless since we already defined in our setup input
        Ok(setup.length as usize)
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsOut,
    {
        let _ = SharedChannelGuard::try_claim(0).await;

        trace!("trying CTRL_OUT setup={}", setup);
        self.write_setup(setup)?;
        self.wait_for_txresult().await?;

        let transfer_size: u32 = setup.length as u32;

        trace!(
            "Finished setup; trying CTRL_OUT transfer pid={}, xfrsize={}, mps={}, ep_num={}, dad={}",
            self.pid as u8,
            transfer_size,
            self.endpoint.max_packet_size,
            u8::from(self.endpoint.addr),
            self.device_addr
        );

        self.configure_for_endpoint(Some(embassy_usb_driver::Direction::Out));
        self.buffer[..buf.len()].copy_from_slice(buf);
        self.regs
            .hcdma(self.channel_idx as usize)
            .write(|w| w.0 = self.buffer.as_ptr() as u32);

        self.regs.hctsiz(self.channel_idx as usize).modify(|w| {
            w.set_pktcnt(transfer_size.div_ceil(self.endpoint.max_packet_size as u32).max(1) as u16);
            w.set_xfrsiz(transfer_size);
            w.set_dpid(self.pid.into()); // Control always DATA1
            w.set_doping(false);
        });

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_chena(true);
            w.set_chdis(false);
        });

        self.wait_for_txresult().await?;

        Ok(transfer_size as usize)
    }

    fn retarget_channel(&mut self, addr: u8, endpoint: &EndpointInfo, pre: bool) -> Result<(), HostError> {
        self.device_addr = addr;
        self.endpoint = *endpoint;
        self.ls_pre = pre;

        if self.endpoint.max_packet_size as usize > self.buffer.len() {
            todo!("retargeting increased buffer size; should re-alloc")
        }

        // We only have a single hardware control channel, it's multiplexed using a lock
        //  we shouldn't change any of the registers in case a transmission is still in progress elsewhere
        if endpoint.ep_type == EndpointType::Control {
            return Ok(());
        }

        if endpoint.ep_type != T::ep_type() {
            // TODO: add context
            Err(HostError::InvalidDescriptor)?
        }

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_dad(addr);
            w.set_lsdev(pre);

            w.set_epnum(endpoint.addr.into());
            w.set_eptyp(Eptyp::from_bits(endpoint.ep_type as u8));
            w.set_mpsiz(endpoint.max_packet_size);

            w.set_chena(false);
            w.set_chdis(true);
        });

        Ok(())
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsIn,
    {
        // TODO: find a good way to do sofware interruptpipes (maybe in Driver `run_forever()`)
        // Interrupt pipes should be able to resolve instantly assuming the first poll has been resolved

        let intx = self.endpoint.addr.is_in();
        let transfer_size: u32 = buf.len() as u32;
        self.regs.hctsiz(self.channel_idx as usize).modify(|w| {
            w.set_pktcnt(transfer_size.div_ceil(self.endpoint.max_packet_size as u32).max(1) as u16);
            w.set_xfrsiz(if !intx {
                w.pktcnt() as u32 * self.endpoint.max_packet_size as u32
            } else {
                transfer_size
            });
            w.set_dpid(self.pid.into());
            w.set_doping(false);
        });

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_epdir(false); // ENDPOINT_TYPE; IN
        });

        self.regs
            .hcdma(self.channel_idx as usize)
            .write(|w| w.0 = self.buffer.as_ptr() as u32);

        self.regs.gintmsk().modify(|w| {
            w.set_hcim(false);
        });

        self.regs.hcchar(self.channel_idx as usize).modify(|w| {
            w.set_chena(true);
            w.set_chdis(false);
        });

        self.flip_pid();
        self.wait_for_txresult().await?;
        Ok(buf.len())
    }

    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsOut,
    {
        todo!()
    }
}

#[cfg(not(feature = "otg-fifo-1024"))]
const OTG_FIFO_DEPTH: usize = 256;
#[cfg(feature = "otg-fifo-1024")]
const OTG_FIFO_DEPTH: usize = 1024;

const TX_FIFO_WORDS: usize = OTG_FIFO_DEPTH / 4;
const PTX_FIFO_WORDS: usize = OTG_FIFO_DEPTH / 8;
const RX_FIFO_WORDS: usize = OTG_FIFO_DEPTH - PTX_FIFO_WORDS - TX_FIFO_WORDS;

const RX_FIFO_SIZE: usize = RX_FIFO_WORDS * 4;
const TX_FIFO_SIZE: usize = TX_FIFO_WORDS * 4;

static DEVICE_WAKER: AtomicWaker = AtomicWaker::new();

impl UsbHostBus {
    /// Initializes and configures the Synopsys OTG core for Host-mode operation
    pub fn new(otg: Otg) -> Self {
        debug_assert!(otg.snpsid().read() == 0x4F54400A, "Bad synopsys id, peripheral dead?");

        // Wait for AHB ready.
        while !otg.grstctl().read().ahbidl() {}

        // Register which are not cleared by a soft-reset:
        otg.gusbcfg().modify(|w| {
            w.set_fhmod(true); // Force host mode
            w.set_fdmod(false); // Deassert device mode
            w.set_srpcap(false);
            w.set_hnpcap(false);
            w.set_physel(true);
            w.set_trdt(5); // Maximum
            w.set_tocal(7); // Maximum timeout calibration
        });

        // Perform core soft-reset
        otg.grstctl().modify(|w| w.set_csrst(true));
        while otg.grstctl().read().csrst() {}
        while !otg.grstctl().read().ahbidl() {}

        let bus = UsbHostBus {
            regs: otg,
            dev_conn: AtomicBool::new(false),
        };

        bus.init_fifo();

        trace!("Post fifo-init: {}", otg.gintsts().read().0);

        // F429-like chips have the GCCFG.NOVBUSSENS bit
        otg.gccfg_v1().modify(|w| {
            // Enable internal full-speed PHY, logic is inverted
            w.set_pwrdwn(true);
            w.set_novbussens(true);
            w.set_vbusasen(false);
            w.set_vbusbsen(false);
            w.set_sofouten(true); // SOF host frames
        });

        otg.pcgcctl().modify(|w| {
            // Disable power down
            w.set_stppclk(false);
        });

        // Setup core interrupts
        otg.gintmsk().modify(|w| {
            w.set_discint(true);
            w.set_prtim(true);
            w.set_hcim(true);
            // w.set_usbrst(true);
        });

        otg.gahbcfg().modify(|w| {
            w.set_gint(true); // unmask global interrupt
            w.set_dmaen(true);
            w.set_hbstlen(0x7);
        });

        otg.hprt().modify(|w| {
            w.0 &= !HPRT_W1C_MASK;
            w.set_ppwr(true);
        });

        trace!("Post init: {}", otg.gintsts().read().0);
        // Clear all interrupts
        // otg.gintsts().modify(|w| w.0 &= !(GINTST_RES_MASK));

        bus
    }

    /// To be called whenever the UsbHost got an interrupt or is polled
    ///
    /// This will check which interrupts are hit, wake correspoding tasks and mask those interrupts to prevent
    ///  a busy-loop of interrupts; the interrupts are expected to be re-enabled by the task if needed.
    pub fn on_interrupt_or_poll(regs: Otg) {
        let intr = regs.gintsts().read();

        trace!("[usbhostbus]: intr/polling: {}", intr.0);
        if intr.discint() || intr.hprtint() {
            trace!("Prt change, waking DEVICE_WAKER");
            DEVICE_WAKER.wake();

            regs.gintmsk().modify(|w| {
                w.set_prtim(false);
                w.set_discint(false);
            });
        }

        let mut chintr = regs.haint().read().haint();
        while chintr != 0 {
            let idx = chintr.trailing_zeros() as usize;
            trace!("Waking CH = {}", idx);
            EP_IN_WAKERS[idx].wake();
            chintr ^= 1 << idx as u16;

            // Don't trigger an interrupt for this until CH has handled the wake (or re-initialized)
            regs.haintmsk().modify(|w| {
                w.0 ^= 1 << idx as u16;
            })
        }

        // todo!("Interrupt pipe polling initiation");

        // Clear gintsts
        regs.gintsts().write(|_| {});
    }

    fn init_fifo(&self) {
        debug!("init_fifo");
        debug!("configuring rx fifo size={}", RX_FIFO_WORDS);
        self.regs.grxfsiz().modify(|w| w.set_rxfd(RX_FIFO_WORDS as u16));
        // Configure TX (USB in direction) fifo size for each endpoint
        let mut fifo_top = RX_FIFO_WORDS;

        debug!("configuring tx fifo, offset={}, size={}", fifo_top, TX_FIFO_WORDS);
        // Non-periodic tx fifo
        self.regs.hnptxfsiz().write(|w| {
            w.set_fd(TX_FIFO_WORDS as u16);
            w.set_sa(fifo_top as u16);
        });
        fifo_top += TX_FIFO_WORDS;

        // Periodic tx fifo
        self.regs.hptxfsiz().write(|w| {
            w.set_fd(PTX_FIFO_WORDS as u16);
            w.set_sa(fifo_top as u16);
        });
        fifo_top += PTX_FIFO_WORDS;

        debug_assert!(fifo_top <= OTG_FIFO_DEPTH, "Exceeds maximum fifo allocation");

        // Flush fifos (TX & PTX need to be done separately since txfnum is an indicator of which)
        self.regs.grstctl().write(|w| {
            w.set_rxfflsh(true);
            w.set_txfflsh(true);
            w.set_txfnum(0b10000); // Flush all tx [RM0390]
        });
        loop {
            let x = self.regs.grstctl().read();
            if !x.rxfflsh() && !x.txfflsh() {
                break;
            }
        }
    }

    fn set_port_defaults(&self) {
        // Not using descriptor DMA mode
        self.regs.hcfg().modify(|w| {
            w.set_descdma(false);
            w.set_perschedena(false);
        });
        let hprt = self.regs.hprt().read();
        self.regs.hfir().modify(|w| {
            w.set_rldctrl(true);
            w.set_frivl(match hprt.pspd() {
                1 => 48000,
                2 => 6000,
                _ => unreachable!(),
            })
        });
        let hcfg = self.regs.hcfg().read();
        if hcfg.fslspcs() != hprt.pspd() {
            self.regs.hcfg().modify(|w| {
                // [CherryUSB] Align clock for Full-speed/Low-speed
                w.set_fslspcs(hprt.pspd());
            });
            // FIXME: Required after fslspcs change [RM0390]
            // self.bus_reset().await;
        }

        self.init_fifo();
    }
}

impl UsbHostDriver for UsbHostBus {
    type Channel<T: channel::Type, D: channel::Direction> = OtgChannel<T, D>;

    fn alloc_channel<T: embassy_usb_driver::host::channel::Type, D: embassy_usb_driver::host::channel::Direction>(
        &self,
        addr: u8,
        endpoint: &embassy_usb_driver::EndpointInfo,
        pre: bool,
    ) -> Result<Self::Channel<T, D>, embassy_usb_driver::host::HostError> {
        trace!("Attempting to alloc channel {}, {}, {}", addr, pre, endpoint);

        let new_index = if T::ep_type() == EndpointType::Control {
            // Only a single control channel is available
            0
        } else {
            // Atomic read-modify-write to acquire a pipe
            loop {
                let pipes = ALLOCATED_PIPES.load(core::sync::atomic::Ordering::Acquire);
                let new_index = pipes.trailing_ones();
                if new_index as usize >= OTG_MAX_PIPES {
                    Err(HostError::OutOfChannels)?;
                }

                if ALLOCATED_PIPES
                    .compare_exchange_weak(
                        pipes,
                        pipes | 1 << new_index,
                        core::sync::atomic::Ordering::Acquire,
                        core::sync::atomic::Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    break new_index;
                }
            }
        };

        let mut channel = OtgChannel::<T, D>::new_alloc(self.regs, new_index as u8, endpoint.max_packet_size.into());
        channel.retarget_channel(addr, endpoint, pre)?;
        Ok(channel)
    }

    async fn wait_for_device_event(&self) -> embassy_usb_driver::host::DeviceEvent {
        poll_fn(move |cx| {
            trace!("Polling device event");

            let hprt = self.regs.hprt().read();

            if hprt.pcsts() && !self.dev_conn.load(core::sync::atomic::Ordering::Relaxed) {
                // NOTE: de-bounce skipped here; could be done interrupt poll
                // crate::rom::ets_delay_us(30_000);
                // let hprt = self.regs.hprt().read();
                if hprt.pcsts() {
                    let speed = match hprt.pspd() {
                        0 => embassy_usb_driver::Speed::High,
                        1 => embassy_usb_driver::Speed::Full,
                        2 => embassy_usb_driver::Speed::Low,
                        _ => unreachable!(),
                    };
                    self.set_port_defaults();
                    debug!("Got device attached speed={}", speed);
                    self.dev_conn.store(true, core::sync::atomic::Ordering::Relaxed);
                    self.regs.gccfg_v1().modify(|w| w.set_sofouten(true));
                    return Poll::Ready(DeviceEvent::Connected(speed));
                } else {
                    self.dev_conn.store(false, core::sync::atomic::Ordering::Relaxed);
                    return Poll::Ready(DeviceEvent::Disconnected);
                }
            }

            DEVICE_WAKER.register(cx.waker());
            self.regs.gintmsk().modify(|w| {
                w.set_prtim(true);
                w.set_discint(true);
            });

            Poll::Pending
        })
        .await
    }

    async fn bus_reset(&self) {
        debug!("Resetting HostBus");
        self.regs.hprt().modify(|w| {
            // Port reset
            w.0 &= !HPRT_W1C_MASK;
            w.set_prst(true);
        });

        Timer::after_micros(15_000).await;
        self.regs.hprt().modify(|w| {
            w.0 &= !HPRT_W1C_MASK;
            w.set_prst(false);
        });

        Timer::after_micros(15_000).await;
        let hprt = self.regs.hprt().read();
        if !hprt.pena() && !hprt.pcdet() {
            debug!(
                "Reset doesn't seem sucessful pena: {}, pcdet: {}",
                hprt.pena(),
                hprt.pcdet()
            );
        }
    }
}
