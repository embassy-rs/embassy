//! Firmware side of the host-driven USB conformance test.
//!
//! One generic implementation, instantiated once per controller by
//! `src/bin/usb_hs_conformance.rs` and `src/bin/usb_fs_conformance.rs`. The
//! device exposes a single vendor interface with two alternate settings and a
//! vendor control protocol the host script (`host/conformance.py`) uses to put
//! the device into a mode, trigger single transfers and read back counters:
//!
//! - alt 0: bulk OUT, bulk IN, interrupt IN
//! - alt 1: isochronous OUT, isochronous IN
//!
//! Nothing here decides pass or fail on its own except the invariants only the
//! device can observe: those are asserted once the host sends `FINISH`. Every
//! host-observable expectation lives in the host script.

use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

use defmt::info;
use embassy_futures::join::join5;
use embassy_futures::select::{Either, select};
use embassy_nxp::usb::{Driver, Instance};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::watch::{Receiver, Watch};
use embassy_time::Timer;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::descriptor::{SynchronizationType, UsageType};
use embassy_usb::driver::{EndpointError, EndpointIn, EndpointOut};
use embassy_usb::{Handler, UsbDeviceSpeed};

/// Per-controller parameters. Everything else about the device is identical
/// across instantiations, which is what makes the two runs comparable.
pub struct Params {
    pub pid: u16,
    pub product: &'static str,
    pub serial: &'static str,
    pub max_speed: UsbDeviceSpeed,
    pub bulk_mps: u16,
    /// 16 on both controllers.
    pub int_mps: u16,
    pub iso_out_mps: u16,
    pub iso_in_mps: u16,
}

/// Largest bulk IN slot on either controller. Also the bulk task's buffer size,
/// so a single `write` can hand hardware a whole multi-packet slot.
const BULK_BUF: usize = 3584;
/// Largest isochronous packet on either controller.
const ISO_BUF: usize = 1024;
/// Largest control-OUT payload `ECHO_WRITE` accepts.
const ECHO_MAX: usize = 200;
/// Size of the `GET_REPORT` payload. Layout, little-endian:
///
/// ```text
///  0..4   u32 bulk_out_bytes     12..16  u32 disabled_errors
///  4..8   u32 bulk_in_bytes      16..20  u32 overflow_errors
///  8..12  u32 iso_out_bytes      20..22  u16 iso_out_packets
/// 22..24  u16 iso_in_packets     24..26  u16 int_in_packets
/// 26..28  u16 ramp_mismatches    28..30  u16 zlp_out_count
/// ```
const REPORT_LEN: usize = 30;

/// Byte `k` of every test payload. Matches `ramp` in
/// `examples/lpc55s69/scripts/usb_throughput.py` and `host/conformance.py`.
fn ramp_byte(k: u32) -> u8 {
    (k % 512) as u8
}

/// What the data tasks should be doing. Broadcast over [`MODE`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Idle = 0,
    /// Bulk: echo every OUT packet back on IN.
    Echo = 1,
    /// Bulk: drain OUT, checking the payload against the ramp.
    Sink = 2,
    /// Bulk: send `mode_param` ramp bytes on IN, then return to [`Mode::Idle`].
    Source = 3,
    /// Isochronous: drain OUT, checking every packet against the ramp.
    IsoSink = 4,
    /// Isochronous: send ramp packets on IN forever.
    IsoSource = 5,
}

impl Mode {
    fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Mode::Idle,
            1 => Mode::Echo,
            2 => Mode::Sink,
            3 => Mode::Source,
            4 => Mode::IsoSink,
            5 => Mode::IsoSource,
            _ => return None,
        })
    }
}

/// Counters the host reads back with `GET_REPORT`.
struct ConformanceState {
    bulk_out_bytes: AtomicU32,
    bulk_in_bytes: AtomicU32,
    iso_out_bytes: AtomicU32,
    iso_out_packets: AtomicU16,
    iso_in_packets: AtomicU16,
    int_in_packets: AtomicU16,
    ramp_mismatches: AtomicU16,
    zlp_out_count: AtomicU16,
    /// A halted endpoint can report `Disabled` at roughly 1 kHz. Saturation
    /// prevents a long-running firmware session from wrapping this counter.
    disabled_errors: AtomicU32,
    overflow_errors: AtomicU32,
    /// `wValue`-independent parameter of the last `SET_MODE`.
    mode_param: AtomicU16,
}

impl ConformanceState {
    /// Zeroes every counter so a host run's absolute expectations hold even
    /// when the script is re-run against a firmware that is already up.
    fn reset(&self) {
        self.bulk_out_bytes.store(0, Ordering::Relaxed);
        self.bulk_in_bytes.store(0, Ordering::Relaxed);
        self.iso_out_bytes.store(0, Ordering::Relaxed);
        self.iso_out_packets.store(0, Ordering::Relaxed);
        self.iso_in_packets.store(0, Ordering::Relaxed);
        self.int_in_packets.store(0, Ordering::Relaxed);
        self.ramp_mismatches.store(0, Ordering::Relaxed);
        self.zlp_out_count.store(0, Ordering::Relaxed);
        self.disabled_errors.store(0, Ordering::Relaxed);
        self.overflow_errors.store(0, Ordering::Relaxed);
    }
}

static STATE: ConformanceState = ConformanceState {
    bulk_out_bytes: AtomicU32::new(0),
    bulk_in_bytes: AtomicU32::new(0),
    iso_out_bytes: AtomicU32::new(0),
    iso_out_packets: AtomicU16::new(0),
    iso_in_packets: AtomicU16::new(0),
    int_in_packets: AtomicU16::new(0),
    ramp_mismatches: AtomicU16::new(0),
    zlp_out_count: AtomicU16::new(0),
    disabled_errors: AtomicU32::new(0),
    overflow_errors: AtomicU32::new(0),
    mode_param: AtomicU16::new(0),
};

fn saturating_add_u16(counter: &AtomicU16, value: u16) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}

fn saturating_add_u32(counter: &AtomicU32, value: u32) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}

/// A [`Watch`] and not a [`Signal`]: `Signal::wait` consumes the value, so with
/// two consumers only one of them would ever see a given mode change. `N = 2`
/// is exactly the number of receivers taken below.
static MODE: Watch<CriticalSectionRawMutex, Mode, 2> = Watch::new();
/// Requested interrupt-IN payload length. Single consumer.
static INT_TRIGGER: Signal<CriticalSectionRawMutex, u16> = Signal::new();
/// The host is done; check the device-side invariants. Single consumer.
static FINISH: Signal<CriticalSectionRawMutex, ()> = Signal::new();

type ModeRx = Receiver<'static, CriticalSectionRawMutex, Mode, 2>;

fn mode_of(rx: &mut ModeRx) -> Mode {
    rx.try_get().unwrap_or(Mode::Idle)
}

// Vendor request numbers. Recipient is always `Interface`, index always 0.
const SET_MODE: u8 = 0x01;
const GET_REPORT: u8 = 0x02;
const ECHO_WRITE: u8 = 0x03;
const ECHO_READ: u8 = 0x04;
const TRIGGER_INT: u8 = 0x05;
const FINISH_REQ: u8 = 0x06;
const UNSUPPORTED: u8 = 0x07;
const RESET: u8 = 0x08;

/// Serves the vendor control protocol.
///
/// The echo payload lives here rather than in a static: both callbacks take
/// `&mut self`, so no locking is needed.
struct ControlHandler {
    int_mps: u16,
    echo: [u8; ECHO_MAX],
    echo_len: usize,
}

impl ControlHandler {
    fn new(int_mps: u16) -> Self {
        Self {
            int_mps,
            echo: [0; ECHO_MAX],
            echo_len: 0,
        }
    }

    /// True for the vendor requests this handler owns; everything else must
    /// fall through to the rest of the stack.
    fn ours(req: &Request) -> bool {
        req.request_type == RequestType::Vendor && req.recipient == Recipient::Interface && req.index == 0
    }
}

impl Handler for ControlHandler {
    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if !Self::ours(&req) {
            return None;
        }
        Some(match req.request {
            SET_MODE => {
                if data.len() != 4 {
                    OutResponse::Rejected
                } else if let Some(mode) = Mode::from_u8(data[0]) {
                    STATE
                        .mode_param
                        .store(u16::from_le_bytes([data[2], data[3]]), Ordering::Relaxed);
                    MODE.sender().send(mode);
                    OutResponse::Accepted
                } else {
                    OutResponse::Rejected
                }
            }
            ECHO_WRITE => {
                if data.len() > ECHO_MAX {
                    OutResponse::Rejected
                } else {
                    self.echo[..data.len()].copy_from_slice(data);
                    self.echo_len = data.len();
                    OutResponse::Accepted
                }
            }
            TRIGGER_INT => {
                if req.value == 0 || req.value > self.int_mps {
                    OutResponse::Rejected
                } else {
                    INT_TRIGGER.signal(req.value);
                    OutResponse::Accepted
                }
            }
            RESET => {
                STATE.reset();
                STATE.mode_param.store(0, Ordering::Relaxed);
                MODE.sender().send(Mode::Idle);
                OutResponse::Accepted
            }
            FINISH_REQ => {
                FINISH.signal(());
                OutResponse::Accepted
            }
            UNSUPPORTED => OutResponse::Rejected,
            _ => OutResponse::Rejected,
        })
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if !Self::ours(&req) {
            return None;
        }
        Some(match req.request {
            GET_REPORT => {
                if buf.len() < REPORT_LEN {
                    return Some(InResponse::Rejected);
                }
                buf[0..4].copy_from_slice(&STATE.bulk_out_bytes.load(Ordering::Relaxed).to_le_bytes());
                buf[4..8].copy_from_slice(&STATE.bulk_in_bytes.load(Ordering::Relaxed).to_le_bytes());
                buf[8..12].copy_from_slice(&STATE.iso_out_bytes.load(Ordering::Relaxed).to_le_bytes());
                buf[12..16].copy_from_slice(&STATE.disabled_errors.load(Ordering::Relaxed).to_le_bytes());
                buf[16..20].copy_from_slice(&STATE.overflow_errors.load(Ordering::Relaxed).to_le_bytes());
                buf[20..22].copy_from_slice(&STATE.iso_out_packets.load(Ordering::Relaxed).to_le_bytes());
                buf[22..24].copy_from_slice(&STATE.iso_in_packets.load(Ordering::Relaxed).to_le_bytes());
                buf[24..26].copy_from_slice(&STATE.int_in_packets.load(Ordering::Relaxed).to_le_bytes());
                buf[26..28].copy_from_slice(&STATE.ramp_mismatches.load(Ordering::Relaxed).to_le_bytes());
                buf[28..30].copy_from_slice(&STATE.zlp_out_count.load(Ordering::Relaxed).to_le_bytes());
                InResponse::Accepted(&buf[..REPORT_LEN])
            }
            ECHO_READ => {
                let n = self.echo_len;
                buf[..n].copy_from_slice(&self.echo[..n]);
                InResponse::Accepted(&buf[..n])
            }
            UNSUPPORTED => InResponse::Rejected,
            _ => InResponse::Rejected,
        })
    }
}

/// Runs the conformance device. Never returns: [`finish_task`] halts the core at
/// a breakpoint once the host has finished and the device invariants hold.
pub async fn run<'d, T: Instance>(driver: Driver<'d, T>, params: Params) -> ! {
    let mut config = embassy_usb::Config::new(0xc0de, params.pid);
    config.manufacturer = Some("Embassy");
    config.product = Some(params.product);
    config.serial_number = Some(params.serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = params.max_speed;

    let mut config_descriptor = [0u8; 256];
    let mut bos_descriptor = [0u8; 256];
    // Must exceed the 200-byte `ECHO_WRITE` payload.
    let mut control_buf = [0u8; 256];
    let mut handler = ControlHandler::new(params.int_mps);

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // One vendor-specific function, one interface, two alternate settings. The
    // vendor class keeps the kernel from binding a driver, so the host script
    // owns the interface outright.
    let (bulk_out, bulk_in, int_in, iso_out, iso_in) = {
        let mut function = builder.function(0xff, 0x00, 0x00);
        let mut iface = function.interface();
        let (bulk_out, bulk_in, int_in) = {
            let mut alt = iface.alt_setting(0xff, 0x00, 0x00, None);
            let bulk_out = alt.endpoint_bulk_out(None, params.bulk_mps);
            let bulk_in = alt.endpoint_bulk_in(None, params.bulk_mps);
            let int_in = alt.endpoint_interrupt_in(None, params.int_mps, 1);
            (bulk_out, bulk_in, int_in)
        };
        let (iso_out, iso_in) = {
            let mut alt = iface.alt_setting(0xff, 0x00, 0x00, None);
            let iso_out = alt.endpoint_isochronous_out(
                None,
                params.iso_out_mps,
                1,
                SynchronizationType::Asynchronous,
                UsageType::DataEndpoint,
                &[],
            );
            let iso_in = alt.endpoint_isochronous_in(
                None,
                params.iso_in_mps,
                1,
                SynchronizationType::Asynchronous,
                UsageType::DataEndpoint,
                &[],
            );
            (iso_out, iso_in)
        };
        (bulk_out, bulk_in, int_in, iso_out, iso_in)
    };

    builder.handler(&mut handler);
    let mut usb = builder.build();

    // The orchestrator's ready banner: printed before anything blocks.
    info!("conformance ready");

    let joined = join5(
        usb.run(),
        bulk_task(bulk_out, bulk_in),
        iso_task(iso_out, iso_in, params.iso_in_mps),
        int_task(int_in),
        finish_task(),
    )
    .await;
    let _ = joined;
    // `UsbDevice::run` never returns and `finish_task` halts the core.
    defmt::unreachable!()
}

/// Bulk OUT/IN worker.
///
/// Every await that can block on the host is wrapped in a `select` against the
/// mode watch, so a `SET_MODE` always takes effect even when the current mode's
/// transfer never completes.
async fn bulk_task<O: EndpointOut, I: EndpointIn>(mut ep_out: O, mut ep_in: I) -> ! {
    let mut rx = MODE.receiver().unwrap();
    let mut buf = [0u8; BULK_BUF];
    // Ramp position for `Sink` and `Source`, reset whenever either is entered.
    let mut offset: u32 = 0;
    let mut remaining: u32 = 0;
    let mut current = Mode::Idle;

    ep_out.wait_enabled().await;

    loop {
        // Detect the transition here rather than in the `Idle` arm: a mode
        // change can also arrive while a transfer is pending, in which case the
        // `select` below just restarts the loop.
        let mode = mode_of(&mut rx);
        if mode != current {
            current = mode;
            if mode == Mode::Sink || mode == Mode::Source {
                offset = 0;
                remaining = STATE.mode_param.load(Ordering::Relaxed) as u32;
            }
        }
        match mode {
            Mode::Idle | Mode::IsoSink | Mode::IsoSource => {
                rx.changed().await;
            }
            Mode::Echo => {
                let n = match select(ep_out.read(&mut buf), rx.changed()).await {
                    Either::First(r) => match bulk_err(r, &mut ep_out).await {
                        Some(n) => n,
                        None => continue,
                    },
                    Either::Second(_) => continue,
                };
                saturating_add_u32(&STATE.bulk_out_bytes, n as u32);
                if n == 0 {
                    saturating_add_u16(&STATE.zlp_out_count, 1);
                }
                // `needs_zlp` is what makes an exact-multiple-of-mps echo
                // terminate; for `n == 0` it is what emits the echoed ZLP.
                match select(ep_in.write_transfer(&buf[..n], true), rx.changed()).await {
                    Either::First(Err(e)) => {
                        count_err(e);
                        continue;
                    }
                    Either::First(Ok(())) => {
                        saturating_add_u32(&STATE.bulk_in_bytes, n as u32);
                    }
                    Either::Second(_) => continue,
                }
            }
            Mode::Sink => {
                let n = match select(ep_out.read(&mut buf), rx.changed()).await {
                    Either::First(r) => match bulk_err(r, &mut ep_out).await {
                        Some(n) => n,
                        None => continue,
                    },
                    Either::Second(_) => continue,
                };
                saturating_add_u32(&STATE.bulk_out_bytes, n as u32);
                check_ramp(&buf[..n], offset);
                offset += n as u32;
            }
            Mode::Source => {
                if remaining == 0 {
                    // Done: hand the tasks back to `Idle` and let the host
                    // observe the transition through the counters.
                    MODE.sender().send(Mode::Idle);
                    continue;
                }
                let chunk = remaining.min(BULK_BUF as u32) as usize;
                fill_ramp(&mut buf[..chunk], offset);
                match select(ep_in.write_transfer(&buf[..chunk], false), rx.changed()).await {
                    Either::First(Err(e)) => {
                        count_err(e);
                        continue;
                    }
                    Either::First(Ok(())) => {
                        saturating_add_u32(&STATE.bulk_in_bytes, chunk as u32);
                        offset += chunk as u32;
                        remaining -= chunk as u32;
                    }
                    Either::Second(_) => continue,
                }
            }
        }
    }
}

/// Isochronous OUT/IN worker. Iso payloads carry no stream offset: every packet
/// is `ramp_byte(0..len)`, because a dropped packet would otherwise desync every
/// later content check and isochronous has no retry.
async fn iso_task<O: EndpointOut, I: EndpointIn>(mut ep_out: O, mut ep_in: I, iso_in_mps: u16) -> ! {
    let mut rx = MODE.receiver().unwrap();
    let mut buf = [0u8; ISO_BUF];
    let mut tx = [0u8; ISO_BUF];
    fill_ramp(&mut tx, 0);
    let in_len = iso_in_mps as usize;

    loop {
        match mode_of(&mut rx) {
            Mode::IsoSink => {
                if let Either::Second(_) = select(ep_out.wait_enabled(), rx.changed()).await {
                    continue;
                }
                loop {
                    match select(ep_out.read(&mut buf), rx.changed()).await {
                        Either::First(Ok(n)) => {
                            saturating_add_u16(&STATE.iso_out_packets, 1);
                            saturating_add_u32(&STATE.iso_out_bytes, n as u32);
                            // `Ok(0)` is how the driver reports a dropped or
                            // errored isochronous packet: a lost frame, not a
                            // content error.
                            if n != 0 {
                                check_ramp(&buf[..n], 0);
                            }
                        }
                        Either::First(Err(e)) => {
                            count_err(e);
                            Timer::after_millis(1).await;
                            break;
                        }
                        Either::Second(_) => break,
                    }
                }
            }
            Mode::IsoSource => {
                if let Either::Second(_) = select(ep_in.wait_enabled(), rx.changed()).await {
                    continue;
                }
                loop {
                    match select(ep_in.write(&tx[..in_len]), rx.changed()).await {
                        Either::First(Ok(())) => {
                            saturating_add_u16(&STATE.iso_in_packets, 1);
                        }
                        Either::First(Err(e)) => {
                            count_err(e);
                            Timer::after_millis(1).await;
                            break;
                        }
                        Either::Second(_) => break,
                    }
                }
            }
            _ => {
                rx.changed().await;
            }
        }
    }
}

/// Sends one interrupt-IN packet per `TRIGGER_INT`.
async fn int_task<I: EndpointIn>(mut ep_in: I) -> ! {
    let mut tx = [0u8; 64];
    fill_ramp(&mut tx, 0);
    loop {
        let len = INT_TRIGGER.wait().await as usize;
        ep_in.wait_enabled().await;
        match ep_in.write(&tx[..len]).await {
            Ok(()) => {
                saturating_add_u16(&STATE.int_in_packets, 1);
            }
            Err(e) => count_err(e),
        }
    }
}

/// Checks the invariants only the device can see, then halts.
async fn finish_task() -> ! {
    FINISH.wait().await;
    // Let the control status stage finish before the core stops.
    Timer::after_millis(100).await;

    let ramp_mismatches = STATE.ramp_mismatches.load(Ordering::Relaxed);
    let overflow_errors = STATE.overflow_errors.load(Ordering::Relaxed);
    let disabled_errors = STATE.disabled_errors.load(Ordering::Relaxed);
    let bulk_out_bytes = STATE.bulk_out_bytes.load(Ordering::Relaxed);
    let bulk_in_bytes = STATE.bulk_in_bytes.load(Ordering::Relaxed);
    let int_in_packets = STATE.int_in_packets.load(Ordering::Relaxed);

    info!(
        "report: bulk_out={} bulk_in={} iso_out_bytes={} iso_out_pkts={} iso_in_pkts={} int_in_pkts={} ramp_mismatches={} zlp_out={} disabled_errors={} overflow_errors={}",
        bulk_out_bytes,
        bulk_in_bytes,
        STATE.iso_out_bytes.load(Ordering::Relaxed),
        STATE.iso_out_packets.load(Ordering::Relaxed),
        STATE.iso_in_packets.load(Ordering::Relaxed),
        int_in_packets,
        ramp_mismatches,
        STATE.zlp_out_count.load(Ordering::Relaxed),
        disabled_errors,
        overflow_errors,
    );

    defmt::assert_eq!(ramp_mismatches, 0, "payload ramp mismatches");
    defmt::assert_eq!(overflow_errors, 0, "endpoint buffer overflows");
    // At least one per halted direction. The host asserts the halt semantics
    // itself; this only cross-checks that the driver really reported the halt
    // to the endpoint tasks.
    defmt::assert!(
        disabled_errors >= 2,
        "expected >= 2 Disabled reports from the halt and alt-setting phases, got {}",
        disabled_errors
    );
    defmt::assert!(bulk_out_bytes > 0, "no bulk OUT traffic");
    defmt::assert!(bulk_in_bytes > 0, "no bulk IN traffic");
    defmt::assert_eq!(int_in_packets, 4, "expected exactly 4 interrupt IN packets");

    info!("Test OK");
    cortex_m::asm::bkpt();
    loop {
        Timer::after_millis(1000).await;
    }
}

/// Maps a bulk read result onto a length, recovering from a disabled or halted
/// endpoint.
///
/// The 1 ms backoff is load-bearing: a *stalled* endpoint is still enabled, so
/// `wait_enabled` returns immediately and this would otherwise spin hot for as
/// long as the host leaves the halt feature set.
async fn bulk_err<O: EndpointOut>(r: Result<usize, EndpointError>, ep_out: &mut O) -> Option<usize> {
    match r {
        Ok(n) => Some(n),
        Err(e) => {
            count_err(e);
            Timer::after_millis(1).await;
            ep_out.wait_enabled().await;
            None
        }
    }
}

fn count_err(e: EndpointError) {
    match e {
        EndpointError::Disabled => saturating_add_u32(&STATE.disabled_errors, 1),
        EndpointError::BufferOverflow => saturating_add_u32(&STATE.overflow_errors, 1),
    };
}

fn fill_ramp(buf: &mut [u8], offset: u32) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = ramp_byte(offset + i as u32);
    }
}

fn check_ramp(buf: &[u8], offset: u32) {
    let mut bad = 0u16;
    for (i, b) in buf.iter().enumerate() {
        if *b != ramp_byte(offset + i as u32) {
            bad += 1;
        }
    }
    if bad != 0 {
        saturating_add_u16(&STATE.ramp_mismatches, bad);
    }
}
