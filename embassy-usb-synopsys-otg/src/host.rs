//! USB Host mode driver for Synopsys DWC2 OTG controllers.

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::host::{ChannelError, DeviceEvent, HostError, SetupPacket, UsbChannel, UsbHostDriver, channel};
use embassy_usb_driver::{EndpointInfo, EndpointType, Speed};
use portable_atomic::{AtomicBool, AtomicU8, Ordering};

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

/// Maximum number of NAK retries before returning a timeout error.
const NAK_RETRY_LIMIT: u32 = 5000;

// Port event bitflags (OR'd together, not mutually exclusive).
const PORT_EVENT_CONNECTED: u8 = 1 << 0;
const PORT_EVENT_DISCONNECTED: u8 = 1 << 1;
const PORT_EVENT_ENABLED: u8 = 1 << 2;
const PORT_EVENT_OVERCURRENT: u8 = 1 << 3;

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

    // Clear SOF interrupt immediately to avoid flooding.
    if gintsts.sof() {
        r.gintsts().write(|w| w.set_sof(true));
    }

    // Host port interrupt
    if gintsts.hprtint() {
        let hprt = r.hprt().read();

        if hprt.pcdet() {
            // Port connect detected
            state.port_event.fetch_or(PORT_EVENT_CONNECTED, Ordering::Release);
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
                state.port_event.fetch_or(PORT_EVENT_ENABLED, Ordering::Release);
            } else {
                // Port disabled
                state.port_event.fetch_or(PORT_EVENT_DISCONNECTED, Ordering::Release);
            }
            state.port_waker.wake();
        }

        if hprt.pocchng() {
            if hprt.poca() {
                state.port_event.fetch_or(PORT_EVENT_OVERCURRENT, Ordering::Release);
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
        state.port_event.fetch_or(PORT_EVENT_DISCONNECTED, Ordering::Release);
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

                    // Discard any remaining bytes we couldn't store (buffer was smaller than packet)
                    if to_copy < len {
                        // We already read ceil(to_copy/4) words; drain the rest.
                        let words_read = (to_copy + 3) / 4;
                        let total_words = (len + 3) / 4;
                        for _ in words_read..total_words {
                            r.fifo(0).read();
                        }
                    }
                } else {
                    // Discard all data if buffer full or null
                    let words = (len + 3) / 4;
                    for _ in 0..words {
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

    async fn configure_as_host(&self) {
        let r = self.instance.regs;
        let phy_type = self.instance.phy_type;

        // Wait for AHB bus to be idle before configuring registers.
        while !r.grstctl().read().ahbidl() {}

        // Configure GCCFG for host mode based on core version.
        // The register layout varies across DWC2 revisions; use CID to select.
        let core_id = r.cid().read().0;
        match core_id {
            0x0000_1000 | 0x0000_1100 | 0x0000_1200 => {
                // v1 cores (STM32F2/F4): power up internal transceiver, disable VBUS sensing.
                r.gccfg_v1().modify(|w| {
                    w.set_pwrdwn(phy_type.internal());
                    w.set_novbussens(true);
                    w.set_vbusasen(false);
                    w.set_vbusbsen(false);
                });
            }
            0x0000_2000 | 0x0000_2100 | 0x0000_2300 | 0x0000_3000 | 0x0000_3100 => {
                // v2/v3 cores (STM32F446/H7): power up PHY, disable VBUS detection.
                r.gccfg_v2().modify(|w| {
                    w.set_pwrdwn(phy_type.internal() && !phy_type.high_speed());
                    w.set_phyhsen(phy_type.internal() && phy_type.high_speed());
                    w.set_vbden(false);
                });
            }
            0x0000_5000 | 0x0000_6100 => {
                // v5 cores (STM32U5/WBA): enable host pull-downs, clear VBUS override.
                r.gccfg_v3().modify(|w| {
                    w.set_forcehostpd(true);
                    w.set_vbvaloval(false);
                    w.set_vbvaloven(false);
                });
            }
            _ => {} // Unknown core; rely on HAL-layer configuration.
        }

        r.gusbcfg().modify(|w| {
            // Force host mode
            w.set_fhmod(true);
            w.set_fdmod(false);
            // Enable internal full-speed PHY
            w.set_physel(phy_type.internal() && !phy_type.high_speed());
        });

        // Wait for host mode to take effect (~25ms).
        // Poll cmod bit with async yield between attempts.
        for _ in 0..50u32 {
            if r.gintsts().read().cmod() {
                break;
            }
            embassy_time::Timer::after_millis(1).await;
        }
    }

    fn init_host(&self) {
        let r = self.instance.regs;

        // Ensure PHY clock is running (clear any power/clock gating).
        r.pcgcctl().write_value(Default::default());

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
            w.set_sofm(true); // SOF required for periodic (interrupt/isochronous) transfers
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
            self.configure_as_host().await;
            self.init_host();
            self.instance.state.inited.store(true, Ordering::Release);
        }

        loop {
            // Wait for CONNECTED or DISCONNECTED event.
            let event = poll_fn(|cx| {
                let state = self.instance.state;
                state.port_waker.register(cx.waker());

                let ev = state.port_event.load(Ordering::Acquire);
                if ev & (PORT_EVENT_DISCONNECTED | PORT_EVENT_OVERCURRENT) != 0 {
                    state
                        .port_event
                        .fetch_and(!(PORT_EVENT_DISCONNECTED | PORT_EVENT_OVERCURRENT), Ordering::AcqRel);
                    // Wake all channels to signal disconnection
                    for ch in &state.channels {
                        if ch.allocated.load(Ordering::Relaxed) {
                            ch.result.store(CH_RESULT_HALTED, Ordering::Release);
                            ch.waker.wake();
                        }
                    }
                    return Poll::Ready(PORT_EVENT_DISCONNECTED);
                }
                if ev & PORT_EVENT_CONNECTED != 0 {
                    state.port_event.fetch_and(!PORT_EVENT_CONNECTED, Ordering::AcqRel);
                    return Poll::Ready(PORT_EVENT_CONNECTED);
                }
                Poll::Pending
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

                let ev = state.port_event.load(Ordering::Acquire);
                if ev & (PORT_EVENT_DISCONNECTED | PORT_EVENT_OVERCURRENT) != 0 {
                    state
                        .port_event
                        .fetch_and(!(PORT_EVENT_DISCONNECTED | PORT_EVENT_OVERCURRENT), Ordering::AcqRel);
                    for ch in &state.channels {
                        if ch.allocated.load(Ordering::Relaxed) {
                            ch.result.store(CH_RESULT_HALTED, Ordering::Release);
                            ch.waker.wake();
                        }
                    }
                    return Poll::Ready(PORT_EVENT_DISCONNECTED);
                }
                if ev & PORT_EVENT_ENABLED != 0 {
                    state.port_event.fetch_and(!PORT_EVENT_ENABLED, Ordering::AcqRel);
                    return Poll::Ready(PORT_EVENT_ENABLED);
                }
                Poll::Pending
            })
            .await;

            match enabled_event {
                PORT_EVENT_ENABLED => {
                    let speed_code = self.instance.state.port_speed.load(Ordering::Acquire);
                    let speed = match speed_code {
                        0 => Speed::Full,
                        1 => Speed::Low,
                        2 => Speed::High,
                        _ => Speed::Full,
                    };

                    // Program the frame interval for the detected device speed.
                    // The PHY clock rate determines the HFIR value:
                    //   Internal HS PHY (UTMI): 60 MHz → HFIR = 60000 for FS
                    //   Internal FS PHY:        48 MHz → HFIR = 48000 for FS
                    let r = self.instance.regs;
                    let phy_type = self.instance.phy_type;
                    match speed {
                        Speed::Full | Speed::Low => {
                            // Both FS and LS use 1 ms frame intervals.
                            // PHY clock: HS PHY = 60 MHz, FS PHY = 48 MHz.
                            let frivl = if phy_type.high_speed() { 60_000 } else { 48_000 };
                            r.hfir().write(|w| w.set_frivl(frivl));
                        }
                        Speed::High => {} // Keep HS defaults
                    }

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
        let ch_count = self.instance.channel_count.min(CH_COUNT);

        // Halt any still-active hardware channels left over from a
        // previous session (e.g. interrupt endpoints that were polling
        // when the device disconnected).
        for ch in 0..ch_count {
            let hcchar = r.hcchar(ch).read();
            if hcchar.chena() {
                r.hcchar(ch).modify(|w| {
                    w.set_chena(true);
                    w.set_chdis(true);
                });
            }
        }

        // Brief wait for channel halts to take effect.
        embassy_time::Timer::after_millis(2).await;

        // Clear all channel interrupts and mask them.
        for ch in 0..ch_count {
            r.hcint(ch).write_value(crate::otg_v1::regs::Hcint(0xFFFF_FFFF));
        }
        r.haintmsk().write(|w| w.set_haintm(0));

        // Flush RX and TX FIFOs to discard any stale data from the
        // previous device session.
        r.grstctl().write(|w| {
            w.set_rxfflsh(true);
            w.set_txfflsh(true);
            w.set_txfnum(0x10); // all TX FIFOs
        });
        while {
            let x = r.grstctl().read();
            x.rxfflsh() || x.txfflsh()
        } {}

        // Assert reset on the port.
        let safe_val = hprt_read_safe(r);
        r.hprt().write(|w| {
            w.0 = safe_val;
            w.set_prst(true);
            w.set_ppwr(true);
        });

        // USB spec requires reset to be held for at least 10ms; use 50ms to be safe.
        embassy_time::Timer::after_millis(50).await;

        // De-assert reset.
        let safe_val = hprt_read_safe(r);
        r.hprt().write(|w| {
            w.0 = safe_val;
            w.set_prst(false);
            w.set_ppwr(true);
        });

        // Wait for the device to recover after reset de-assertion.
        embassy_time::Timer::after_millis(20).await;
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
        let r = self.regs;
        let ch = self.index;

        // Request the hardware to halt this channel. We can't wait for
        // completion (Drop is synchronous), but the halt ensures the
        // controller stops issuing tokens for this endpoint. Any
        // resulting CHH interrupt is harmless — we mask it below.
        let hcchar = r.hcchar(ch).read();
        if hcchar.chena() {
            r.hcchar(ch).modify(|w| {
                w.set_chena(true);
                w.set_chdis(true);
            });
        }

        // Mask this channel's interrupt so the ISR won't deliver stale
        // results to whatever gets allocated at this index next.
        critical_section::with(|_| {
            r.haintmsk().modify(|w| {
                w.set_haintm(w.haintm() & !(1 << ch));
            });
        });

        // Clear any pending channel interrupts.
        r.hcint(ch).write_value(crate::otg_v1::regs::Hcint(0xFFFF_FFFF));

        let state = unsafe { &*self.state };
        state.channels[ch].result.store(CH_RESULT_NONE, Ordering::Release);
        state.channels[ch].allocated.store(false, Ordering::Release);
    }
}

impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> Channel<T, D, CH_COUNT> {
    fn state(&self) -> &HostState<CH_COUNT> {
        unsafe { &*self.state }
    }

    fn configure_channel(&self, dir_in: bool, ep_type: EndpointType, pktcnt: u16, xfrsiz: u32, dpid: u8) {
        let r = self.regs;
        let ch = self.index;

        // The DWC2 does not auto-clear CHENA after transfer completion.
        // If the channel is still active from a previous transfer, we
        // must explicitly halt it and wait for the hardware to confirm
        // (CHENA cleared) before reconfiguring. Without this, writing
        // new HCCHAR/HCTSIZ while the channel is active causes undefined
        // behavior — typically a permanent hang or stale interrupts.
        if r.hcchar(ch).read().chena() {
            r.hcchar(ch).modify(|w| {
                w.set_chena(true);
                w.set_chdis(true);
            });
            while r.hcchar(ch).read().chena() {
                core::hint::spin_loop();
            }
            r.hcint(ch).write_value(crate::otg_v1::regs::Hcint(0xFFFF_FFFF));
        }

        // Configure channel characteristics
        let is_periodic = matches!(ep_type, EndpointType::Interrupt | EndpointType::Isochronous);
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
            if is_periodic {
                // Multi Count must be at least 1 for periodic endpoints (0 is reserved).
                w.set_mcnt(1);
                // Schedule on the opposite frame parity so the transfer starts on the next SOF.
                let current_frame = r.hfnum().read().frnum();
                w.set_oddfrm(current_frame & 1 == 0);
            }
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

        // Enable this channel in HAINTMSK (critical section guards the RMW against concurrent alloc_channel)
        critical_section::with(|_| {
            r.haintmsk().modify(|w| {
                w.set_haintm(w.haintm() | (1 << ch));
            });
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

    /// Execute an OUT transfer on this channel, retrying on NAK up to [`NAK_RETRY_LIMIT`] times.
    async fn do_out_transfer(&mut self, ep_type: EndpointType, data: &[u8], dpid: u8) -> Result<(), ChannelError> {
        let pktcnt = if data.is_empty() {
            1
        } else {
            ((data.len() as u32 + self.max_packet_size as u32 - 1) / self.max_packet_size as u32) as u16
        };

        let mut nak_retries = 0u32;
        loop {
            self.configure_channel(false, ep_type, pktcnt, data.len() as u32, dpid);
            self.enable_channel();
            self.write_fifo(data);

            let result = self.wait_for_result().await;
            if result == CH_RESULT_NAK {
                nak_retries += 1;
                if nak_retries >= NAK_RETRY_LIMIT {
                    return Err(ChannelError::Timeout);
                }
                yield_now().await;
                continue;
            }
            return Self::result_to_error(result);
        }
    }

    /// Execute an IN transfer on this channel, retrying on NAK up to [`NAK_RETRY_LIMIT`] times.
    async fn do_in_transfer(&mut self, ep_type: EndpointType, buf: &mut [u8], dpid: u8) -> Result<usize, ChannelError> {
        // For interrupt/isochronous endpoints, only request one packet per transfer.
        // The device sends at most one packet per (micro)frame.
        let is_periodic = matches!(ep_type, EndpointType::Interrupt | EndpointType::Isochronous);
        let xfer_size = if is_periodic {
            (buf.len() as u32).min(self.max_packet_size as u32)
        } else {
            buf.len() as u32
        };
        let pktcnt: u16 = if is_periodic {
            1
        } else if buf.is_empty() {
            1
        } else {
            ((buf.len() as u32 + self.max_packet_size as u32 - 1) / self.max_packet_size as u32) as u16
        };

        let mut nak_retries = 0u32;
        loop {
            self.setup_rx_buffer(&mut buf[..xfer_size as usize]);
            self.configure_channel(true, ep_type, pktcnt, xfer_size, dpid);
            self.enable_channel();

            let result = self.wait_for_result().await;
            let count = self.rx_count();
            self.clear_rx_buffer();

            if result == CH_RESULT_COMPLETE {
                return Ok(count);
            }

            if result == CH_RESULT_NAK {
                if is_periodic {
                    // For periodic endpoints, the hardware may start halting the
                    // channel after NAK. Explicitly halt and wait for completion
                    // (CHH) before reconfiguring, otherwise the retry races with
                    // the in-progress halt and the new transfer never starts.
                    self.halt_channel();
                    let _halt = self.wait_for_result().await; // expect CHH
                } else {
                    // NAK retry limit only applies to non-periodic (control/bulk)
                    // transfers where NAK means "busy, try later". For periodic
                    // (interrupt/iso) endpoints, NAK is the normal idle response
                    // and polling should continue indefinitely.
                    nak_retries += 1;
                    if nak_retries >= NAK_RETRY_LIMIT {
                        return Err(ChannelError::Timeout);
                    }
                }
                yield_now().await;
                continue;
            }

            Self::result_to_error(result)?;
            return Ok(count);
        }
    }

    /// Perform a control IN transfer (SETUP -> DATA IN -> STATUS OUT).
    async fn do_control_in(&mut self, setup: &[u8], buf: &mut [u8]) -> Result<usize, ChannelError> {
        // SETUP phase
        self.do_out_transfer(EndpointType::Control, setup, vals::Dpid::SETUP.to_bits())
            .await?;

        // DATA IN phase
        let mut transferred = 0;
        if !buf.is_empty() {
            let mut offset = 0;
            let mut toggle = true; // DATA1
            while offset < buf.len() {
                let dpid = if toggle { vals::Dpid::DATA1 } else { vals::Dpid::DATA0 };
                let remaining = &mut buf[offset..];
                let chunk_size = remaining.len().min(self.max_packet_size as usize);
                let n = self
                    .do_in_transfer(EndpointType::Control, &mut remaining[..chunk_size], dpid.to_bits())
                    .await?;
                offset += n;
                toggle = !toggle;
                if n < self.max_packet_size as usize {
                    break;
                }
            }
            transferred = offset;
        }

        // STATUS phase: OUT ZLP with DATA1
        self.do_out_transfer(EndpointType::Control, &[], vals::Dpid::DATA1.to_bits())
            .await?;

        Ok(transferred)
    }

    /// Perform a control OUT transfer (SETUP -> DATA OUT -> STATUS IN).
    async fn do_control_out(&mut self, setup: &[u8], data: &[u8]) -> Result<(), ChannelError> {
        // SETUP phase
        self.do_out_transfer(EndpointType::Control, setup, vals::Dpid::SETUP.to_bits())
            .await?;

        if !data.is_empty() {
            // DATA OUT phase
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

            // STATUS phase: IN ZLP with DATA1
            let mut status_buf = [0u8; 0];
            self.do_in_transfer(EndpointType::Control, &mut status_buf, vals::Dpid::DATA1.to_bits())
                .await?;
        } else {
            // No data phase — STATUS direction is opposite of request direction
            let req_type = setup[0];
            let is_device_to_host = (req_type & 0x80) != 0;

            if is_device_to_host {
                self.do_out_transfer(EndpointType::Control, &[], vals::Dpid::DATA1.to_bits())
                    .await?;
            } else {
                let mut status_buf = [0u8; 0];
                self.do_in_transfer(EndpointType::Control, &mut status_buf, vals::Dpid::DATA1.to_bits())
                    .await?;
            }
        }

        Ok(())
    }
}

impl<T: channel::Type, D: channel::Direction, const CH_COUNT: usize> UsbChannel<T, D> for Channel<T, D, CH_COUNT> {
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsIn,
    {
        self.do_control_in(setup.as_bytes(), buf).await
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<(), ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsOut,
    {
        self.do_control_out(setup.as_bytes(), buf).await
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

    fn set_timeout(&mut self, _timeout: embassy_usb_driver::host::TimeoutConfig) {
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
