#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(async_fn_in_trait)]

use embassy_futures::select::{Either4, select4};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

use crate::ioctl::{PendingIoctl, Shared};
use crate::rpc::{FgBackend, HostedEvent, RpcBackend};

mod proto;

// must be first
mod fmt;

mod control;
mod iface;
mod ioctl;
mod rpc;

pub use control::*;
pub use iface::*;

const MTU: usize = 1514;

macro_rules! impl_bytes {
    ($t:ident) => {
        impl $t {
            pub const SIZE: usize = core::mem::size_of::<Self>();

            #[allow(unused)]
            pub fn to_bytes(&self) -> [u8; Self::SIZE] {
                unsafe { core::mem::transmute(*self) }
            }

            #[allow(unused)]
            pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> &Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );
                unsafe { core::mem::transmute(bytes) }
            }

            #[allow(unused)]
            pub fn from_bytes_mut(bytes: &mut [u8; Self::SIZE]) -> &mut Self {
                let alignment = core::mem::align_of::<Self>();
                assert_eq!(
                    bytes.as_ptr().align_offset(alignment),
                    0,
                    "{} is not aligned",
                    core::any::type_name::<Self>()
                );

                unsafe { core::mem::transmute(bytes) }
            }
        }
    };
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
struct PayloadHeader {
    /// InterfaceType on lower 4 bits, number on higher 4 bits.
    if_type_and_num: u8,

    /// Flags.
    ///
    /// bit 0: more fragments.
    flags: u8,

    len: u16,
    offset: u16,
    checksum: u16,
    seq_num: u16,
    reserved2: u8,

    /// Packet type for HCI or PRIV interface, reserved otherwise
    hci_priv_packet_type: u8,
}
impl_bytes!(PayloadHeader);

impl PayloadHeader {
    #[inline]
    fn copy(mut self, buffer: &mut [u8; MAX_BUFFER_SIZE]) {
        buffer[0..PayloadHeader::SIZE].copy_from_slice(&self.to_bytes());
        self.checksum = checksum(&buffer[..PayloadHeader::SIZE + self.len as usize]);
        buffer[0..PayloadHeader::SIZE].copy_from_slice(&self.to_bytes());
    }
}

#[allow(unused)]
#[repr(u8)]
enum InterfaceType {
    Sta,
    Ap,
    Serial,
    Hci,
    Priv,
    Test,
}

const MAX_BUFFER_SIZE: usize = 1600;
const HEARTBEAT_MAX_GAP: Duration = Duration::from_secs(20);

/// State for the esp-hosted driver.
pub struct State {
    shared: Shared,
    ch: ch::State<MTU, 4, 4>,
}

impl State {
    /// Create a new state.
    pub fn new() -> Self {
        Self {
            shared: Shared::new(),
            ch: ch::State::new(),
        }
    }
}

/// Type alias for network driver.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

/// Create a new esp-hosted driver using the provided state, interface, and reset pin.
///
/// Returns a device handle for interfacing with embassy-net, a control handle for
/// interacting with the driver, and a runner for communicating with the WiFi device.
pub async fn new<'a, I, OUT>(
    state: &'a mut State,
    iface: I,
    reset: OUT,
) -> (NetDriver<'a>, Control<'a>, Runner<'a, I, OUT>)
where
    I: Interface,
    OUT: OutputPin,
{
    let (ch_runner, device) = ch::new(&mut state.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    let runner = Runner {
        ch: ch_runner,
        state_ch,
        shared: &state.shared,
        backend: FgBackend,
        next_seq: 1,
        reset,
        iface,
        heartbeat_deadline: Instant::now() + HEARTBEAT_MAX_GAP,
    };

    (device, Control::new(state_ch, &state.shared), runner)
}

/// Runner for communicating with the WiFi device.
pub struct Runner<'a, I, OUT> {
    ch: ch::Runner<'a, MTU>,
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
    backend: FgBackend,

    next_seq: u16,
    heartbeat_deadline: Instant,

    iface: I,
    reset: OUT,
}

impl<'a, I, OUT> Runner<'a, I, OUT>
where
    I: Interface,
    OUT: OutputPin,
{
    /// Run the packet processing.
    pub async fn run(mut self) -> ! {
        debug!("resetting...");
        self.reset.set_low().unwrap();
        Timer::after_millis(100).await;
        self.reset.set_high().unwrap();
        Timer::after_millis(1000).await;

        let mut buffer = [0u8; MAX_BUFFER_SIZE];

        loop {
            self.iface.wait_for_handshake().await;

            let ioctl = self.shared.ioctl_wait_pending();
            let tx = self.ch.tx_buf();
            let ev = self.iface.wait_for_ready();
            let hb = Timer::at(self.heartbeat_deadline);

            match select4(ioctl, tx, ev, hb).await {
                Either4::First(PendingIoctl { buf, req_len }) => {
                    let payload_len = self
                        .backend
                        .encode_ioctl(&mut buffer[PayloadHeader::SIZE..], &unsafe { &*buf }[..req_len]);

                    let header = PayloadHeader {
                        if_type_and_num: self.backend.encode_iface_type(InterfaceType::Serial),
                        len: payload_len as _,
                        offset: PayloadHeader::SIZE as _,
                        seq_num: self.next_seq,
                        ..Default::default()
                    };
                    self.next_seq = self.next_seq.wrapping_add(1);
                    header.copy(&mut buffer);
                }
                Either4::Second(packet) => {
                    buffer[PayloadHeader::SIZE..][..packet.len()].copy_from_slice(&packet);

                    let header = PayloadHeader {
                        if_type_and_num: self.backend.encode_iface_type(InterfaceType::Sta),
                        len: packet.len() as _,
                        offset: PayloadHeader::SIZE as _,
                        seq_num: self.next_seq,
                        ..Default::default()
                    };
                    self.next_seq = self.next_seq.wrapping_add(1);
                    header.copy(&mut buffer);

                    packet.tx_done();
                }
                Either4::Third(()) => {
                    buffer[..PayloadHeader::SIZE].fill(0);
                }
                Either4::Fourth(()) => {
                    // Extend the deadline if initializing
                    if let ioctl::ControlState::Reboot = self.shared.state() {
                        self.heartbeat_deadline = Instant::now() + HEARTBEAT_MAX_GAP;
                        continue;
                    }
                    panic!("heartbeat from esp32 stopped")
                }
            }

            if buffer[0] != 0 {
                trace!("tx: {:02x}", &buffer[..40]);
            }

            self.iface.transfer(&mut buffer).await;

            self.handle_rx(&mut buffer);
        }
    }

    fn handle_rx(&mut self, buf: &mut [u8]) {
        trace!("rx: {:02x}", &buf[..40]);

        let buf_len = buf.len();
        let h = PayloadHeader::from_bytes_mut((&mut buf[..PayloadHeader::SIZE]).try_into().unwrap());

        if h.len == 0 || h.offset as usize != PayloadHeader::SIZE {
            return;
        }

        let payload_len = h.len as usize;
        if buf_len < PayloadHeader::SIZE + payload_len {
            warn!("rx: len too big");
            return;
        }

        let if_type_and_num = h.if_type_and_num;
        let want_checksum = h.checksum;
        h.checksum = 0;
        let got_checksum = checksum(&buf[..PayloadHeader::SIZE + payload_len]);
        if want_checksum != got_checksum {
            warn!("rx: bad checksum. Got {:04x}, want {:04x}", got_checksum, want_checksum);
            return;
        }

        let payload = &buf[PayloadHeader::SIZE..][..payload_len];
        let if_type = self.backend.decode_iface_type(if_type_and_num & 0x0f);

        match if_type {
            Some(InterfaceType::Sta) => match self.ch.try_rx_buf() {
                Some(mut buf) => {
                    buf[..payload.len()].copy_from_slice(payload);
                    buf.rx_done(payload.len())
                }
                None => warn!("failed to push rxd packet to the channel."),
            },
            Some(InterfaceType::Serial) => {
                trace!("serial rx: {:02x}", payload);

                match self.backend.process_serial_data(payload) {
                    Some((true, data)) => self.handle_event(data),
                    Some((false, data)) => self.shared.ioctl_done(data),
                    _ => {}
                }
            }
            _ => warn!("unknown iftype {}", if_type_and_num),
        }
    }

    fn handle_event(&mut self, data: &[u8]) {
        match self.backend.normalize_event(data) {
            Some(HostedEvent::Init) => self.shared.init_done(),
            Some(HostedEvent::Heartbeat) => self.heartbeat_deadline = Instant::now() + HEARTBEAT_MAX_GAP,
            Some(HostedEvent::StaConnected { resp }) => {
                info!("connected, code {}", resp);
                if self.shared.connect_is_pending() {
                    self.shared.connect_done();
                }
                self.state_ch.set_link_state(LinkState::Up);
            }
            Some(HostedEvent::StaDisconnected { reason }) => {
                info!("disconnected, reason {}", reason);
                if self.shared.connect_is_pending() {
                    self.shared.connect_failed(reason);
                }
                self.state_ch.set_link_state(LinkState::Down);
            }
            None => {}
        }
    }
}

fn checksum(buf: &[u8]) -> u16 {
    let mut res = 0u16;
    for &b in buf {
        res = res.wrapping_add(b as _);
    }
    res
}
