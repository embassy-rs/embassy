//! USB Host mode driver for Synopsys DWC2 OTG controllers.

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::host::{
    ChannelError, DeviceEvent, HostError, SetupPacket, UsbChannel, UsbHostDriver, channel,
};
use embassy_usb_driver::{EndpointInfo, EndpointType, Speed};

use crate::PhyType;
use crate::otg_v1::{Otg, vals};

// Channel transfer result codes stored atomically.
const CH_RESULT_NONE: u8 = 0;
const CH_RESULT_COMPLETE: u8 = 1;
const CH_RESULT_STALL: u8 = 2;
const CH_RESULT_NAK: u8 = 3;
const CH_RESULT_TXERR: u8 = 4;
const CH_RESULT_BBERR: u8 = 5;
const CH_RESULT_FRMOR: u8 = 6;
const CH_RESULT_DTERR: u8 = 7;
const CH_RESULT_HALTED: u8 = 8;

// Port event flags.
const PORT_EVENT_CONNECTED: u8 = 1;
const PORT_EVENT_DISCONNECTED: u8 = 2;
const PORT_EVENT_ENABLED: u8 = 3;
const PORT_EVENT_OVERCURRENT: u8 = 4;

/// Per-channel state for interrupt communication.
struct ChannelState {
    waker: AtomicWaker,
    result: AtomicU8,
    /// Buffer pointer for RX FIFO reads.
    rx_buffer: UnsafeCell<*mut u8>,
    /// Number of bytes received into the buffer.
    rx_count: UnsafeCell<usize>,
    /// Buffer capacity.
    rx_capacity: UnsafeCell<usize>,
    allocated: AtomicBool,
}

// SAFETY: Buffer access is synchronized between ISR and poll_fn via result atomic.
unsafe impl Send for ChannelState {}
unsafe impl Sync for ChannelState {}

/// USB host driver state. Create one per OTG instance.
pub struct HostState<const CH_COUNT: usize> {
    channels: [ChannelState; CH_COUNT],
    port_waker: AtomicWaker,
    port_event: AtomicU8,
    port_speed: AtomicU8,
    inited: AtomicBool,
}

unsafe impl<const CH_COUNT: usize> Send for HostState<CH_COUNT> {}
unsafe impl<const CH_COUNT: usize> Sync for HostState<CH_COUNT> {}

impl<const CH_COUNT: usize> HostState<CH_COUNT> {
    /// Create a new host state.
    pub const fn new() -> Self {
        Self {
            channels: [const {
                ChannelState {
                    waker: AtomicWaker::new(),
                    result: AtomicU8::new(CH_RESULT_NONE),
                    rx_buffer: UnsafeCell::new(core::ptr::null_mut()),
                    rx_count: UnsafeCell::new(0),
                    rx_capacity: UnsafeCell::new(0),
                    allocated: AtomicBool::new(false),
                }
            }; CH_COUNT],
            port_waker: AtomicWaker::new(),
            port_event: AtomicU8::new(0),
            port_speed: AtomicU8::new(0),
            inited: AtomicBool::new(false),
        }
    }
}

/// Hardware-dependent host configuration.
pub struct OtgHostInstance<'d, const CH_COUNT: usize> {
    /// The USB peripheral registers.
    pub regs: Otg,
    /// The host state.
    pub state: &'d HostState<CH_COUNT>,
    /// FIFO depth in words.
    pub fifo_depth_words: u16,
    /// Number of host channels available.
    pub channel_count: usize,
    /// The PHY type.
    pub phy_type: PhyType,
}

/// Handle host-mode interrupts.
///
/// # Safety
/// Must be called from the USB OTG interrupt handler when the controller is in host mode.
pub unsafe fn on_host_interrupt<const CH_COUNT: usize>(r: Otg, state: &HostState<CH_COUNT>, ch_count: usize) {
    let gintsts = r.gintsts().read();

    // Host port interrupt
    if gintsts.hprtint() {
        let hprt = r.hprt().read();

        if hprt.pcdet() {
            // Port connect detected
            state.port_event.store(PORT_EVENT_CONNECTED, Ordering::Release);
            state.port_waker.wake();
        }

        if hprt.penchng() {
            if hprt.pena() {
                // Port enabled - read speed
                let speed = match hprt.pspd() {
                    0b00 => 2, // High speed
                    0b01 => 0, // Full speed
                    0b10 => 1, // Low speed
                    _ => 0,    // Default to full speed
                };
                state.port_speed.store(speed, Ordering::Release);
                state.port_event.store(PORT_EVENT_ENABLED, Ordering::Release);
            } else {
                // Port disabled
                state.port_event.store(PORT_EVENT_DISCONNECTED, Ordering::Release);
            }
            state.port_waker.wake();
        }

        if hprt.pocchng() {
            if hprt.poca() {
                state.port_event.store(PORT_EVENT_OVERCURRENT, Ordering::Release);
                state.port_waker.wake();
            }
        }

        // Clear W1C bits by writing back only the status bits, preserving non-W1C fields.
        // HPRT W1C safety: we must NOT write 1 to pena (would disable port).
        hprt_clear_interrupts(r);
    }

    // Disconnect interrupt
    if gintsts.discint() {
        r.gintsts().write(|w| w.set_discint(true)); // clear
        state.port_event.store(PORT_EVENT_DISCONNECTED, Ordering::Release);
        state.port_waker.wake();
    }

    // RX FIFO non-empty (IN data received)
    while r.gintsts().read().rxflvl() {
        let status = r.grxstsp().read();
        let ch_num = status.epnum() as usize; // In host mode, epnum field is channel number
        let len = status.bcnt() as usize;

        if ch_num >= ch_count {
            // Discard unexpected data
            let words = (len + 3) / 4;
            for _ in 0..words {
                r.fifo(0).read();
            }
            continue;
        }

        match status.pktstsh() {
            vals::Pktstsh::IN_DATA_RX => {
                // Read data from FIFO into channel buffer
                let ch_state = &state.channels[ch_num];
                let buf_ptr = unsafe { *ch_state.rx_buffer.get() };
                let count = unsafe { *ch_state.rx_count.get() };
                let capacity = unsafe { *ch_state.rx_capacity.get() };

                let available = capacity.saturating_sub(count);
                let to_copy = len.min(available);

                if !buf_ptr.is_null() && to_copy > 0 {
                    let dst = unsafe { buf_ptr.add(count) };
                    let mut remaining = to_copy;
                    let mut offset = 0;
                    while remaining >= 4 {
                        let word = r.fifo(0).read().0;
                        unsafe {
                            core::ptr::copy_nonoverlapping(word.to_ne_bytes().as_ptr(), dst.add(offset), 4);
                        }
                        offset += 4;
                        remaining -= 4;
                    }
                    if remaining > 0 {
                        let word = r.fifo(0).read().0;
                        unsafe {
                            core::ptr::copy_nonoverlapping(word.to_ne_bytes().as_ptr(), dst.add(offset), remaining);
                        }
                    }
                    unsafe {
                        *ch_state.rx_count.get() = count + to_copy;
                    }
                } else {
                    // Discard data if buffer full or null
                    let words = (len + 3) / 4;
                    for _ in 0..words {
                        r.fifo(0).read();
                    }
                }

                // Handle any remaining bytes we couldn't store
                if to_copy < len {
                    let discard_words = ((len - to_copy) + 3) / 4;
                    for _ in 0..discard_words {
                        r.fifo(0).read();
                    }
                }
            }
            vals::Pktstsh::IN_DATA_DONE | vals::Pktstsh::DATA_TOGGLE_ERR | vals::Pktstsh::CHANNEL_HALTED => {
                // These don't have FIFO data to read
            }
            _ => {
                // Unknown status, discard any data
                let words = (len + 3) / 4;
                for _ in 0..words {
                    r.fifo(0).read();
                }
            }
        }
    }

    // Host channel interrupts
    if gintsts.hcint() {
        let haint = r.haint().read().haint();

        for ch in 0..ch_count {
            if haint & (1 << ch) != 0 {
                let hcint = r.hcint(ch).read();

                let result = if hcint.xfrc() {
                    CH_RESULT_COMPLETE
                } else if hcint.stall() {
                    CH_RESULT_STALL
                } else if hcint.bberr() {
                    CH_RESULT_BBERR
                } else if hcint.txerr() {
                    CH_RESULT_TXERR
                } else if hcint.dterr() {
                    CH_RESULT_DTERR
                } else if hcint.frmor() {
                    CH_RESULT_FRMOR
                } else if hcint.nak() {
                    CH_RESULT_NAK
                } else if hcint.chh() {
                    CH_RESULT_HALTED
                } else {
                    CH_RESULT_NONE
                };

                // Clear all channel interrupts
                r.hcint(ch).write_value(hcint);

                if result != CH_RESULT_NONE {
                    state.channels[ch].result.store(result, Ordering::Release);
                    state.channels[ch].waker.wake();
                }
            }
        }
    }
}

/// Clear HPRT W1C interrupt bits without accidentally clearing pena.
fn hprt_clear_interrupts(r: Otg) {
    // Read current value, then write back with W1C status bits set and pena cleared
    // to avoid accidentally disabling the port.
    r.hprt().modify(|w| {
        // Clear the W1C status bits
        let pcdet = w.pcdet();
        let penchng = w.penchng();
        let pocchng = w.pocchng();

        // First, clear all W1C bits to 0 (don't accidentally clear them)
        w.set_pcdet(false);
        w.set_penchng(false);
        w.set_pocchng(false);
        w.set_pena(false); // NEVER write 1 to pena (it would disable the port)

        // Now set only the ones that were actually set (to clear them)
        w.set_pcdet(pcdet);
        w.set_penchng(penchng);
        w.set_pocchng(pocchng);
    });
}

/// Read HPRT for modification, masking W1C bits to prevent accidental clears.
fn hprt_read_safe(r: Otg) -> u32 {
    let val = r.hprt().read().0;
    // Mask out W1C bits: pcdet(1), penchng(3), pena(2), pocchng(5)
    val & !(0x2E)
}

/// USB OTG Host Driver.
pub struct OtgHost<'d, const CH_COUNT: usize> {
    instance: OtgHostInstance<'d, CH_COUNT>,
}

impl<'d, const CH_COUNT: usize> OtgHost<'d, CH_COUNT> {
    /// Create a new OTG host driver.
    pub fn new(instance: OtgHostInstance<'d, CH_COUNT>) -> Self {
        Self { instance }
    }

    fn configure_as_host(&self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;

        r.gusbcfg().modify(|w| {
            // Force host mode
            w.set_fhmod(true);
            w.set_fdmod(false);
            // Enable internal full-speed PHY
            w.set_physel(phy_type.internal() && !phy_type.high_speed());
        });

        // Wait for host mode to take effect (~25ms)
        // We poll cmod bit in gintsts
        let mut timeout = 250_000u32;
        while !r.gintsts().read().cmod() {
            timeout -= 1;
            if timeout == 0 {
                break;
            }
            core::hint::spin_loop();
        }
    }

    fn init_host(&self) {
        let r = self.instance.regs;

        // Configure HCFG: set PHY clock.
        // fslspcs=0: 30/60 MHz (HS PHY); fslspcs=1: 48 MHz (FS PHY).
        r.hcfg().modify(|w| {
            w.set_fslspcs(if self.instance.phy_type.high_speed() { 0 } else { 1 });
        });

        // Configure FIFO sizes for host mode:
        // RX FIFO: half of total
        // Non-periodic TX FIFO: quarter
        // Periodic TX FIFO: quarter
        let total = self.instance.fifo_depth_words;
        let rx_size = total / 2;
        let nptx_size = total / 4;
        let ptx_size = total - rx_size - nptx_size;

        critical_section::with(|_| {
            r.grxfsiz().modify(|w| w.set_rxfd(rx_size));

            // Non-periodic TX FIFO (used for control and bulk OUT)
            r.hnptxfsiz().write(|w| {
                w.set_sa(rx_size);
                w.set_fd(nptx_size);
            });

            // Periodic TX FIFO (used for interrupt and isochronous)
            r.hptxfsiz().write(|w| {
                w.set_sa(rx_size + nptx_size);
                w.set_fd(ptx_size);
            });

            // Flush all FIFOs
            r.grstctl().write(|w| {
                w.set_rxfflsh(true);
                w.set_txfflsh(true);
                w.set_txfnum(0x10); // Flush all TX FIFOs
            });
        });

        // Wait for flush to complete
        while {
            let x = r.grstctl().read();
            x.rxfflsh() || x.txfflsh()
        } {}

        // Power the port
        let safe_val = hprt_read_safe(r);
        r.hprt().write(|w| {
            w.0 = safe_val;
            w.set_ppwr(true);
        });

        // Clear all pending interrupts
        r.gintsts().write_value(crate::otg_v1::regs::Gintsts(0xFFFF_FFFF));

        // Enable host-mode interrupts
        r.gintmsk().write(|w| {
            w.set_prtim(true); // Port interrupt
            w.set_hcim(true); // Host channel interrupt
            w.set_discint(true); // Disconnect interrupt
            w.set_rxflvlm(true); // RX FIFO non-empty
            w.set_sofm(false); // SOF (disabled for now)
        });

        // Enable global interrupt
        r.gahbcfg().write(|w| {
            w.set_gint(true);
        });
    }
}

impl<'d, const CH_COUNT: usize> UsbHostDriver for OtgHost<'d, CH_COUNT> {
    type Channel<T: channel::Type, D: channel::Direction> = Channel<T, D, CH_COUNT>;

    async fn wait_for_device_event(&self) -> DeviceEvent {
        // Lazily initialize the host hardware on first call.
        if !self.instance.state.inited.load(Ordering::Acquire) {
            self.configure_as_host();
            self.init_host();
            self.instance.state.inited.store(true, Ordering::Release);
        }

        loop {
            // Wait for CONNECTED or DISCONNECTED event.
            let event = poll_fn(|cx| {
                let state = self.instance.state;
                state.port_waker.register(cx.waker());

                let ev = state.port_event.swap(0, Ordering::AcqRel);
                match ev {
                    PORT_EVENT_CONNECTED => Poll::Ready(PORT_EVENT_CONNECTED),
                    PORT_EVENT_DISCONNECTED => {
                        // Wake all channels to signal disconnection
                        for ch in &state.channels {
                            if ch.allocated.load(Ordering::Relaxed) {
                                ch.result.store(CH_RESULT_HALTED, Ordering::Release);
                                ch.waker.wake();
                            }
                        }
                        Poll::Ready(PORT_EVENT_DISCONNECTED)
                    }
                    _ => Poll::Pending,
                }
            })
            .await;

            if event == PORT_EVENT_DISCONNECTED {
                return DeviceEvent::Disconnected;
            }

            // Connected: perform bus reset.
            self.bus_reset().await;

            // Now wait for ENABLED or DISCONNECTED.
            let enabled_event = poll_fn(|cx| {
                let state = self.instance.state;
                state.port_waker.register(cx.waker());

                let ev = state.port_event.swap(0, Ordering::AcqRel);
                match ev {
                    PORT_EVENT_ENABLED => Poll::Ready(Some(PORT_EVENT_ENABLED)),
                    PORT_EVENT_DISCONNECTED => {
                        for ch in &state.channels {
                            if ch.allocated.load(Ordering::Relaxed) {
                                ch.result.store(CH_RESULT_HALTED, Ordering::Release);
                                ch.waker.wake();
                            }
                        }
                        Poll::Ready(Some(PORT_EVENT_DISCONNECTED))
                    }
                    _ => Poll::Pending,
                }
            })
            .await;

            match enabled_event {
                Some(PORT_EVENT_ENABLED) => {
                    let speed_code = self.instance.state.port_speed.load(Ordering::Acquire);
                    let speed = match speed_code {
                        0 => Speed::Full,
                        1 => Speed::Low,
                        2 => Speed::High,
                        _ => Speed::Full,
                    };
                    return DeviceEvent::Connected(speed);
                }
                _ => {
                    // Disconnected while waiting for enable; loop and wait again.
                    return DeviceEvent::Disconnected;
                }
            }
        }
    }

    async fn bus_reset(&self) {
        let r = self.instance.regs;

        // Assert reset on the port
        let safe_val = hprt_read_safe(r);
        r.hprt().write(|w| {
            w.0 = safe_val;
            w.set_prst(true);
            w.set_ppwr(true);
        });

        // USB spec requires reset to be held for at least 10ms.
        // We use a busy-wait here since embassy-time may not be available.
        // 50ms to be safe.
        for _ in 0..500_000u32 {
            core::hint::spin_loop();
        }

        // De-assert reset
        let safe_val = hprt_read_safe(r);
        r.hprt().write(|w| {
            w.0 = safe_val;
            w.set_prst(false);
            w.set_ppwr(true);
        });

        // Wait a bit for the device to recover
        for _ in 0..200_000u32 {
            core::hint::spin_loop();
        }
    }

    fn alloc_channel<T: channel::Type, D: channel::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointInfo,
        _pre: bool,
    ) -> Result<Self::Channel<T, D>, HostError> {
        let ep_number = endpoint.addr.index() as u8;
        let max_packet_size = endpoint.max_packet_size;
        let ep_type = endpoint.ep_type;

        // Read device speed from port_speed atomic (stored by ISR)
        let speed_code = self.instance.state.port_speed.load(Ordering::Acquire);
        let is_low_speed = speed_code == 1;

        // Find a free channel using atomic CAS
        for i in 0..self.instance.channel_count.min(CH_COUNT) {
            if self.instance.state.channels[i]
                .allocated
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                self.instance.state.channels[i]
                    .result
                    .store(CH_RESULT_NONE, Ordering::Release);

                return Ok(Channel {
                    regs: self.instance.regs,
                    // SAFETY: state is behind a &'d reference which outlives all channels.
                    // Channel release is atomic via Drop.
                    state: self.instance.state as *const _ as *const HostState<CH_COUNT>,
                    index: i,
                    device_address: addr,
                    ep_number,
                    ep_type,
                    max_packet_size,
                    is_low_speed,
                    data_toggle: false,
                    phantom: PhantomData,
                });
            }
        }

        Err(HostError::OutOfChannels)
    }
}

/// A USB host channel for performing transfers.
///
/// The channel is automatically released when dropped.
pub struct Channel<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> {
    regs: Otg,
    /// Raw pointer to avoid lifetime dependency on OtgHost.
    /// SAFETY: The HostState is always in a static or lives for 'd which outlives all channels.
    state: *const HostState<CH_COUNT>,
    index: usize,
    device_address: u8,
    ep_number: u8,
    ep_type: EndpointType,
    max_packet_size: u16,
    is_low_speed: bool,
    data_toggle: bool,
    phantom: PhantomData<(T, D)>,
}

// SAFETY: Channel access to HostState is through atomics only.
unsafe impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> Send for Channel<T, D, CH_COUNT> {}

impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> Drop for Channel<T, D, CH_COUNT> {
    fn drop(&mut self) {
        // Mark channel as free
        let state = unsafe { &*self.state };
        state.channels[self.index].allocated.store(false, Ordering::Release);
    }
}

impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> Channel<T, D, CH_COUNT> {
    fn state(&self) -> &HostState<CH_COUNT> {
        unsafe { &*self.state }
    }

    fn configure_channel(&self, dir_in: bool, ep_type: EndpointType, pktcnt: u16, xfrsiz: u32, dpid: u8) {
        let r = self.regs;
        let ch = self.index;

        // Configure channel characteristics
        r.hcchar(ch).write(|w| {
            w.set_mpsiz(self.max_packet_size);
            w.set_epnum(self.ep_number);
            w.set_epdir(dir_in);
            w.set_lsdev(self.is_low_speed);
            w.set_eptyp(match ep_type {
                EndpointType::Control => vals::Eptyp::CONTROL,
                EndpointType::Isochronous => vals::Eptyp::ISOCHRONOUS,
                EndpointType::Bulk => vals::Eptyp::BULK,
                EndpointType::Interrupt => vals::Eptyp::INTERRUPT,
            });
            w.set_dad(self.device_address);
        });

        // Configure transfer size
        r.hctsiz(ch).write(|w| {
            w.set_xfrsiz(xfrsiz);
            w.set_pktcnt(pktcnt);
            w.set_dpid(dpid);
        });

        // Enable channel interrupt mask
        r.hcintmsk(ch).write(|w| {
            w.set_xfrcm(true);
            w.set_chhm(true);
            w.set_stallm(true);
            w.set_nakm(true);
            w.set_txerrm(true);
            w.set_bberrm(true);
            w.set_frmorm(true);
            w.set_dterrm(true);
        });

        // Enable this channel in HAINTMSK
        r.haintmsk().modify(|w| {
            w.set_haintm(w.haintm() | (1 << ch));
        });

        // Clear any pending channel interrupts
        r.hcint(ch).write_value(crate::otg_v1::regs::Hcint(0xFFFF_FFFF));

        // Clear result
        self.state().channels[ch]
            .result
            .store(CH_RESULT_NONE, Ordering::Release);
    }

    fn enable_channel(&self) {
        let r = self.regs;
        let ch = self.index;
        r.hcchar(ch).modify(|w| {
            w.set_chena(true);
            w.set_chdis(false);
        });
    }

    #[allow(dead_code)]
    fn halt_channel(&self) {
        let r = self.regs;
        let ch = self.index;
        r.hcchar(ch).modify(|w| {
            w.set_chena(true);
            w.set_chdis(true);
        });
    }

    fn write_fifo(&self, data: &[u8]) {
        let r = self.regs;
        let ch = self.index;

        let mut chunks = data.chunks_exact(4);
        for chunk in &mut chunks {
            let word = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            r.fifo(ch).write_value(crate::otg_v1::regs::Fifo(word));
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            let mut word_bytes = [0u8; 4];
            word_bytes[..rem.len()].copy_from_slice(rem);
            let word = u32::from_ne_bytes(word_bytes);
            r.fifo(ch).write_value(crate::otg_v1::regs::Fifo(word));
        }
    }

    fn setup_rx_buffer(&self, buf: &mut [u8]) {
        let ch_state = &self.state().channels[self.index];
        unsafe {
            *ch_state.rx_buffer.get() = buf.as_mut_ptr();
            *ch_state.rx_count.get() = 0;
            *ch_state.rx_capacity.get() = buf.len();
        }
    }

    fn clear_rx_buffer(&self) {
        let ch_state = &self.state().channels[self.index];
        unsafe {
            *ch_state.rx_buffer.get() = core::ptr::null_mut();
            *ch_state.rx_count.get() = 0;
            *ch_state.rx_capacity.get() = 0;
        }
    }

    fn rx_count(&self) -> usize {
        unsafe { *self.state().channels[self.index].rx_count.get() }
    }

    async fn wait_for_result(&self) -> u8 {
        poll_fn(|cx| {
            let ch_state = &self.state().channels[self.index];
            ch_state.waker.register(cx.waker());

            let result = ch_state.result.load(Ordering::Acquire);
            if result != CH_RESULT_NONE {
                ch_state.result.store(CH_RESULT_NONE, Ordering::Release);
                Poll::Ready(result)
            } else {
                Poll::Pending
            }
        })
        .await
    }

    fn result_to_error(result: u8) -> Result<(), ChannelError> {
        match result {
            CH_RESULT_COMPLETE => Ok(()),
            CH_RESULT_STALL => Err(ChannelError::Stall),
            CH_RESULT_NAK => Ok(()), // NAK is not an error, just retry
            CH_RESULT_TXERR => Err(ChannelError::BadResponse),
            CH_RESULT_BBERR => Err(ChannelError::BadResponse),
            CH_RESULT_FRMOR => Err(ChannelError::BadResponse),
            CH_RESULT_DTERR => Err(ChannelError::BadResponse),
            CH_RESULT_HALTED => Err(ChannelError::Disconnected),
            _ => Err(ChannelError::BadResponse),
        }
    }

    /// Execute an OUT transfer on this channel, retrying on NAK.
    async fn do_out_transfer(&mut self, ep_type: EndpointType, data: &[u8], dpid: u8) -> Result<(), ChannelError> {
        let pktcnt = if data.is_empty() {
            1
        } else {
            ((data.len() as u32 + self.max_packet_size as u32 - 1) / self.max_packet_size as u32) as u16
        };

        loop {
            self.configure_channel(false, ep_type, pktcnt, data.len() as u32, dpid);
            self.enable_channel();
            self.write_fifo(data);

            let result = self.wait_for_result().await;
            if result == CH_RESULT_NAK {
                // Yield and retry
                yield_now().await;
                continue;
            }
            return Self::result_to_error(result);
        }
    }

    /// Execute an IN transfer on this channel, retrying on NAK.
    async fn do_in_transfer(
        &mut self,
        ep_type: EndpointType,
        buf: &mut [u8],
        dpid: u8,
    ) -> Result<usize, ChannelError> {
        let pktcnt = if buf.is_empty() {
            1
        } else {
            ((buf.len() as u32 + self.max_packet_size as u32 - 1) / self.max_packet_size as u32) as u16
        };

        loop {
            self.setup_rx_buffer(buf);
            self.configure_channel(true, ep_type, pktcnt, buf.len() as u32, dpid);
            self.enable_channel();

            let result = self.wait_for_result().await;
            let count = self.rx_count();
            self.clear_rx_buffer();

            if result == CH_RESULT_NAK {
                yield_now().await;
                continue;
            }
            Self::result_to_error(result)?;
            return Ok(count);
        }
    }

    /// Perform a complete control transfer (SETUP -> optional DATA -> STATUS).
    ///
    /// `setup` is the 8-byte SETUP packet bytes.
    /// `dir_in` indicates whether the DATA phase is device-to-host.
    /// `data` is the buffer for the DATA phase.
    async fn do_control_transfer(
        &mut self,
        setup: &[u8],
        dir_in: bool,
        data: &mut [u8],
    ) -> Result<usize, ChannelError> {
        // SETUP phase: always DATA0 (MDATA/SETUP PID)
        self.do_out_transfer(EndpointType::Control, setup, vals::Dpid::SETUP.to_bits())
            .await?;

        // DATA phase
        let mut transferred = 0;
        if !data.is_empty() {
            // Data toggle starts at DATA1
            if dir_in {
                // IN data phase: read from device in MPS-sized chunks
                let mut offset = 0;
                let mut toggle = true; // DATA1
                while offset < data.len() {
                    let dpid = if toggle { vals::Dpid::DATA1 } else { vals::Dpid::DATA0 };
                    let remaining = &mut data[offset..];
                    let chunk_size = remaining.len().min(self.max_packet_size as usize);
                    let n = self
                        .do_in_transfer(EndpointType::Control, &mut remaining[..chunk_size], dpid.to_bits())
                        .await?;
                    offset += n;
                    toggle = !toggle;
                    // Short packet means end of data
                    if n < self.max_packet_size as usize {
                        break;
                    }
                }
                transferred = offset;

                // STATUS phase: OUT ZLP with DATA1
                self.do_out_transfer(EndpointType::Control, &[], vals::Dpid::DATA1.to_bits())
                    .await?;
            } else {
                // OUT data phase: send to device in MPS-sized chunks
                let mut offset = 0;
                let mut toggle = true; // DATA1
                while offset < data.len() {
                    let dpid = if toggle { vals::Dpid::DATA1 } else { vals::Dpid::DATA0 };
                    let remaining = &data[offset..];
                    let chunk_size = remaining.len().min(self.max_packet_size as usize);
                    self.do_out_transfer(EndpointType::Control, &remaining[..chunk_size], dpid.to_bits())
                        .await?;
                    offset += chunk_size;
                    toggle = !toggle;
                }
                transferred = offset;

                // STATUS phase: IN ZLP with DATA1
                let mut status_buf = [0u8; 0];
                self.do_in_transfer(EndpointType::Control, &mut status_buf, vals::Dpid::DATA1.to_bits())
                    .await?;
            }
        } else {
            // No data phase
            // STATUS phase direction is opposite of the request direction
            let req_type = setup[0];
            let is_device_to_host = (req_type & 0x80) != 0;

            if is_device_to_host {
                // Status: OUT ZLP with DATA1
                self.do_out_transfer(EndpointType::Control, &[], vals::Dpid::DATA1.to_bits())
                    .await?;
            } else {
                // Status: IN ZLP with DATA1
                let mut status_buf = [0u8; 0];
                self.do_in_transfer(EndpointType::Control, &mut status_buf, vals::Dpid::DATA1.to_bits())
                    .await?;
            }
        }

        Ok(transferred)
    }
}

impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> UsbChannel<T, D>
    for Channel<T, D, CH_COUNT>
{
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsIn,
    {
        let setup_bytes = setup.as_bytes();
        self.do_control_transfer(setup_bytes, true, buf).await
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<(), ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsOut,
    {
        let setup_bytes = setup.as_bytes();
        // We need a mutable slice for the internal API; OUT data is read-only but we can cast.
        // Create a temporary mutable copy for the interface.
        let mut tmp_buf: [u8; 512] = [0u8; 512];
        let len = buf.len().min(tmp_buf.len());
        tmp_buf[..len].copy_from_slice(&buf[..len]);
        self.do_control_transfer(setup_bytes, false, &mut tmp_buf[..len])
            .await?;
        Ok(())
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsIn,
    {
        let dpid = if self.data_toggle {
            vals::Dpid::DATA1
        } else {
            vals::Dpid::DATA0
        };

        let n = self.do_in_transfer(T::ep_type(), buf, dpid.to_bits()).await?;
        self.data_toggle = !self.data_toggle;
        Ok(n)
    }

    async fn request_out(&mut self, buf: &[u8], _ensure_transaction_end: bool) -> Result<(), ChannelError>
    where
        D: channel::IsOut,
    {
        let dpid = if self.data_toggle {
            vals::Dpid::DATA1
        } else {
            vals::Dpid::DATA0
        };

        self.do_out_transfer(T::ep_type(), buf, dpid.to_bits()).await?;
        self.data_toggle = !self.data_toggle;
        Ok(())
    }

    fn retarget_channel(&mut self, addr: u8, endpoint: &EndpointInfo, _pre: bool) -> Result<(), HostError> {
        self.device_address = addr;
        self.ep_number = endpoint.addr.index() as u8;
        self.max_packet_size = endpoint.max_packet_size;
        self.ep_type = endpoint.ep_type;
        Ok(())
    }

    async fn set_timeout(&mut self, _timeout: embassy_usb_driver::host::TimeoutConfig) {
        // Hardware timeouts; no-op
    }
}

/// Dpid extension for SETUP token.
impl vals::Dpid {
    /// SETUP PID (value 3 = MDATA, used for SETUP token in host mode).
    const SETUP: Self = Self::MDATA;
}

/// Yield to the executor, allowing other tasks to run.
async fn yield_now() {
    let mut yielded = false;
    poll_fn(|cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

