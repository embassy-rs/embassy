#![no_std]
#![doc = include_str!("../README.md")]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
#![warn(missing_docs)]
#![allow(async_fn_in_trait)]

#[cfg(not(feature = "bluetooth"))]
use embassy_futures::select::{Either4 as EitherMany, select4 as select_many};
#[cfg(feature = "bluetooth")]
use embassy_futures::select::{Either5 as EitherMany, select5 as select_many};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

use crate::ioctl::{PendingIoctl, Shared};
use crate::rpc::ioctl_ctx::IoctlMessage;
use crate::rpc::{Backend, HostedEvent, RpcBackend};

pub(crate) mod proto;

// must be first
mod fmt;

#[cfg(feature = "bluetooth")]
pub mod bluetooth;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
enum InterfaceType {
    Invalid,
    Sta,
    Ap,
    Serial,
    Hci,
    Priv,
    Test,
}

const MAX_BUFFER_SIZE: usize = 1600;
// Maximum payload size
const MAX_IOCTL_SIZE: usize = if cfg!(feature = "esp-hosted-fg") {
    // Scan results are unlimited
    MAX_BUFFER_SIZE - 12
} else {
    // Theoretical max overhead is 29 bytes. Biggest message currently is OTA write with 256 bytes.
    256 + 29
};
const HEARTBEAT_MAX_GAP: Duration = Duration::from_secs(20);

/// State for the esp-hosted driver.
pub struct State {
    shared: Shared,
    ioctl_buffer: [u8; MAX_IOCTL_SIZE],
    msg_buffer: IoctlMessage,
    ch: ch::State<MTU, 4, 4>,
    #[cfg(feature = "bluetooth")]
    bt: bluetooth::BtState,
}

impl State {
    /// Create a new state.
    pub fn new() -> Self {
        Self {
            shared: Shared::new(),
            ch: ch::State::new(),
            msg_buffer: IoctlMessage::default(),
            ioctl_buffer: [0u8; MAX_IOCTL_SIZE],
            #[cfg(feature = "bluetooth")]
            bt: bluetooth::BtState::new(),
        }
    }
}

/// Type alias for network driver.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

/// Handles returned by [`new`] for interacting with the esp-hosted driver.
pub struct HostedResources<'a, I, OUT> {
    /// Network device for use with embassy-net.
    pub net_device: NetDriver<'a>,

    /// Bluetooth HCI transport, for use with a `bt-hci` host stack.
    #[cfg(feature = "bluetooth")]
    pub bluetooth: bluetooth::BtDriver<'a>,

    /// Control handle for managing WiFi and driver state.
    pub control: Control<'a>,

    /// Runner driving communication with the coprocessor. Must be spawned.
    pub runner: Runner<'a, I, OUT>,
}

/// Create a new esp-hosted driver.
///
/// Returns a device handle for interfacing with embassy-net, a control handle for
/// interacting with the driver, and a runner for communicating with the WiFi device.
pub async fn new<'a, I, OUT>(state: &'a mut State, iface: I, reset: OUT) -> HostedResources<'a, I, OUT>
where
    I: Interface,
    OUT: OutputPin,
{
    let (ch_runner, device) = ch::new(&mut state.ch, ch::driver::HardwareAddress::Ethernet([0; 6]));
    let state_ch = ch_runner.state_runner();

    #[cfg(feature = "bluetooth")]
    let (bt_runner, bt_driver) = bluetooth::new(&mut state.bt);

    let runner = Runner {
        ch: ch_runner,
        state_ch,
        shared: &state.shared,
        backend: Backend::default(),
        next_seq: 1,
        reset,
        iface,
        heartbeat_deadline: Instant::now() + HEARTBEAT_MAX_GAP,
        #[cfg(feature = "bluetooth")]
        bt: bt_runner,
    };

    HostedResources {
        net_device: device,
        #[cfg(feature = "bluetooth")]
        bluetooth: bt_driver,
        control: Control::new(state_ch, &state.shared, &mut state.ioctl_buffer, &mut state.msg_buffer),
        runner,
    }
}

/// Runner for communicating with the WiFi device.
pub struct Runner<'a, I, OUT> {
    ch: ch::Runner<'a, MTU>,
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,
    backend: Backend,

    next_seq: u16,
    heartbeat_deadline: Instant,

    iface: I,
    reset: OUT,

    #[cfg(feature = "bluetooth")]
    bt: bluetooth::BtRunner<'a>,
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

        self.iface.init(true).await;
        self.shared.interface_ready();

        let mut buffer = [0u8; MAX_BUFFER_SIZE];

        loop {
            if let ioctl::ControlState::Reboot = self.shared.state() {
                self.backend = Backend::default();
                self.iface.init(false).await;
                self.shared.interface_ready();
            }

            self.iface.wait_for_handshake().await;

            let ioctl = self.shared.ioctl_wait_pending();
            let tx = self.ch.tx_buf();
            let ev = self.iface.wait_for_ready();
            let hb = Timer::at(self.heartbeat_deadline);

            let event = select_many(
                ioctl,
                tx,
                ev,
                hb,
                #[cfg(feature = "bluetooth")]
                self.bt.tx_chan.receive(),
            )
            .await;

            let mut tx_len = 0;
            match event {
                EitherMany::First(PendingIoctl { buf, req_len }) => {
                    let if_type_and_num = unwrap!(self.backend.encode_iface_type(InterfaceType::Serial));

                    let payload_len = self
                        .backend
                        .encode_ioctl(&mut buffer[PayloadHeader::SIZE..], &unsafe { &*buf }[..req_len]);

                    let header = PayloadHeader {
                        if_type_and_num,
                        len: payload_len as _,
                        offset: PayloadHeader::SIZE as _,
                        seq_num: self.next_seq,
                        ..Default::default()
                    };
                    self.next_seq = self.next_seq.wrapping_add(1);
                    header.copy(&mut buffer);
                    tx_len = PayloadHeader::SIZE + payload_len;
                }
                EitherMany::Second(packet) => {
                    let packet_len = packet.len();
                    if let Some(if_type_and_num) = self.backend.encode_iface_type(InterfaceType::Sta) {
                        buffer[PayloadHeader::SIZE..][..packet_len].copy_from_slice(&packet);

                        let header = PayloadHeader {
                            if_type_and_num,
                            len: packet_len as _,
                            offset: PayloadHeader::SIZE as _,
                            seq_num: self.next_seq,
                            ..Default::default()
                        };
                        self.next_seq = self.next_seq.wrapping_add(1);
                        header.copy(&mut buffer);
                        tx_len = PayloadHeader::SIZE + packet_len;
                    } else {
                        // Backend doesn't support the requested interface type. Drop the
                        // packet and send nothing this iteration.
                        buffer[..PayloadHeader::SIZE].fill(0);
                    }

                    packet.tx_done();
                }
                EitherMany::Third(()) => {
                    buffer[..PayloadHeader::SIZE].fill(0);
                    tx_len = 0;
                }
                EitherMany::Fourth(()) => {
                    // Extend the deadline if initializing
                    if let ioctl::ControlState::Reboot | ioctl::ControlState::WaitingForInit = self.shared.state() {
                        self.heartbeat_deadline = Instant::now() + HEARTBEAT_MAX_GAP;
                        continue;
                    }
                    panic!("heartbeat from esp32 stopped")
                }

                // Bluetooth HCI packet queued by the host stack.
                #[cfg(feature = "bluetooth")]
                EitherMany::Fifth(slot) => {
                    if let Some(if_type_and_num) = self.backend.encode_iface_type(InterfaceType::Hci) {
                        // `slot.buf[0]` is the H4 packet type indicator; it travels in the
                        // payload header's `hci_priv_packet_type` field, and the remaining
                        // bytes are the HCI packet body.
                        let pkt_type = slot.buf[0];
                        let body = &slot.buf[1..slot.len];
                        buffer[PayloadHeader::SIZE..][..body.len()].copy_from_slice(body);

                        let header = PayloadHeader {
                            if_type_and_num,
                            len: body.len() as _,
                            offset: PayloadHeader::SIZE as _,
                            seq_num: self.next_seq,
                            hci_priv_packet_type: pkt_type,
                            ..Default::default()
                        };
                        self.next_seq = self.next_seq.wrapping_add(1);
                        header.copy(&mut buffer);
                        tx_len = PayloadHeader::SIZE + body.len() as usize;
                    } else {
                        // Backend doesn't support HCI (e.g. not yet initialized). Drop the
                        // packet and send nothing this iteration.
                        buffer[..PayloadHeader::SIZE].fill(0);
                    }
                    slot.receive_done();
                }
            }

            if buffer[0] != 0 {
                #[cfg(feature = "log")]
                trace!("tx: {:02x?}", &buffer[..40]);
                #[cfg(feature = "defmt")]
                trace!("tx: {=[u8]:02x}", &buffer[..40]);
            }

            self.iface.transfer(&mut buffer, tx_len).await;

            self.handle_rx(&mut buffer);
        }
    }

    fn handle_rx(&mut self, buf: &mut [u8]) {
        #[cfg(feature = "log")]
        trace!("rx: {:02x?}", &buf[..40]);
        #[cfg(feature = "defmt")]
        trace!("rx: {=[u8]:02x}", &buf[..40]);

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
                #[cfg(feature = "log")]
                trace!("serial rx: {:02x?}", payload);
                #[cfg(feature = "defmt")]
                trace!("serial rx: {=[u8]:02x}", payload);

                match self.backend.process_serial_data(payload) {
                    Some((true, data)) => self.handle_event(data),
                    Some((false, data)) => self.shared.ioctl_done(data),
                    _ => {}
                }
            }
            #[cfg(feature = "bluetooth")]
            Some(InterfaceType::Hci) => {
                #[cfg(feature = "log")]
                trace!("hci rx: {:02x?}", payload);
                #[cfg(feature = "defmt")]
                trace!("hci rx: {=[u8]:02x}", payload);

                self.bt.rx(payload);
            }
            _ => warn!("unknown iftype {}", if_type_and_num),
        }
    }

    fn handle_event(&mut self, data: &[u8]) {
        match self.backend.normalize_event(data) {
            Some(HostedEvent::Init) => self.shared.init_done(self.backend),
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
