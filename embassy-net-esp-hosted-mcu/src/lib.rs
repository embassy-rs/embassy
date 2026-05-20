#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(async_fn_in_trait)]

use embassy_futures::select::{Either5, select5};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

use crate::ioctl::{PendingIoctl, Shared};
use crate::proto::{Rpc, Rpc_};

#[allow(unused)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(missing_docs)]
#[allow(clippy::all)]
mod proto;

// must be first
mod fmt;

mod config;
mod control;
mod iface;
mod ioctl;

pub use config::*;
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

#[allow(unused)]
#[repr(u8)]
enum InterfaceType {
    Invalid = 0,
    Sta = 1,
    Ap = 2,
    Serial = 3,
    Hci = 4,
    Priv = 5,
    Test = 6,
}
const ESP_STA_IF: u8 = 1;
const ESP_SERIAL_IF: u8 = 3;

const SERIAL_MSG_HEADER_LEN: usize = 12;

/// Maximum size of the SPI buffers for the esp-hosted driver.
pub const MAX_SPI_BUFFER_SIZE: usize = 1600;
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

/// Events emitted by the esp-hosted driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EspHostedEvents {
    /// The heartbeat deadline has been reached.
    Deadline,
    /// The wifi link has been lost.
    Disconnected,
}

/// Type alias for network driver.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

/// Create a new esp-hosted driver using the provided state, interface, and reset pin.
///
/// Returns a device handle for interfacing with embassy-net, a control handle for
/// interacting with the driver, and a runner for communicating with the WiFi device.
/// The optional events_notifier is used to signal the main loop when the heartbeat deadline is reached or when the wifi link is lost.
/// If not provided, the main loop will panic if the heartbeat deadline is reached.
pub async fn new<'a, I, OUT>(
    state: &'a mut State,
    iface: I,
    reset: OUT,
    events_notifier: Option<&'static Signal<NoopRawMutex, EspHostedEvents>>,
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
        next_seq: 1,
        heartbeat_deadline: Instant::now() + HEARTBEAT_MAX_GAP,
        iface,
        reset,
        events_notifier,
    };

    (device, Control::new(state_ch, &state.shared), runner)
}

/// Runner for communicating with the WiFi device.
pub struct Runner<'a, I, OUT> {
    ch: ch::Runner<'a, MTU>,
    state_ch: ch::StateRunner<'a>,
    shared: &'a Shared,

    next_seq: u16,
    heartbeat_deadline: Instant,

    iface: I,
    reset: OUT,

    events_notifier: Option<&'static Signal<NoopRawMutex, EspHostedEvents>>,
}

#[cfg_attr(all(not(feature = "log"), not(feature = "defmt")), allow(unused))]
struct HexSlice<'a>(&'a [u8]);

#[cfg(feature = "log")]
impl<'a> core::fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HexSlice<'_> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "[");
        for byte in self.0 {
            defmt::write!(f, "{:02x}", byte);
        }
        defmt::write!(f, "]");
    }
}

#[allow(unused)]
fn print_eth_frame(frame: &[u8]) {
    let dest_mac_address = &frame[0..6];
    let source_mac_address = &frame[6..12];
    let ethertype = u16::from_be_bytes(frame[12..14].try_into().unwrap());

    debug!("dest_mac_address: {}", HexSlice(dest_mac_address));
    debug!("source_mac_address: {}", HexSlice(source_mac_address));
    debug!("ethertype: {:04x}", ethertype);
}

impl<'a, I, OUT> Runner<'a, I, OUT>
where
    I: Interface,
    OUT: OutputPin,
{
    /// Run the packet processing.
    pub async fn run(mut self, tx_buf: &mut [u8], rx_buf: &mut [u8]) -> ! {
        debug!("resetting...");
        self.reset_device().await;

        loop {
            self.iface.wait_for_handshake().await;

            let ioctl = self.shared.ioctl_wait_pending();
            let tx = self.ch.tx_buf();
            let ev = async { self.iface.wait_for_ready().await };
            let hb = Timer::at(self.heartbeat_deadline);
            let reset_request = self.shared.wait_for_reset_request();

            match select5(ioctl, tx, ev, hb, reset_request).await {
                Either5::First(PendingIoctl { buf, req_len }) => {
                    tx_buf[12..22].copy_from_slice(b"\x01\x06\x00RPCRsp\x02");
                    tx_buf[22..24].copy_from_slice(&(req_len as u16).to_le_bytes());
                    tx_buf[24..][..req_len].copy_from_slice(&unsafe { &*buf }[..req_len]);

                    let mut header = PayloadHeader {
                        if_type_and_num: InterfaceType::Serial as _,
                        len: (req_len + 12) as _,
                        offset: PayloadHeader::SIZE as _,
                        seq_num: self.next_seq,
                        ..Default::default()
                    };
                    self.next_seq = self.next_seq.wrapping_add(1);

                    // Calculate checksum
                    tx_buf[0..12].copy_from_slice(&header.to_bytes());
                    header.checksum = checksum(&tx_buf[..24 + req_len]);
                    tx_buf[0..12].copy_from_slice(&header.to_bytes());
                }
                Either5::Second(packet) => {
                    tx_buf[12..][..packet.len()].copy_from_slice(packet);

                    let mut header = PayloadHeader {
                        if_type_and_num: InterfaceType::Sta as _,
                        len: packet.len() as _,
                        offset: PayloadHeader::SIZE as _,
                        seq_num: self.next_seq,
                        ..Default::default()
                    };
                    self.next_seq = self.next_seq.wrapping_add(1);

                    // Calculate checksum
                    tx_buf[0..12].copy_from_slice(&header.to_bytes());
                    header.checksum = checksum(&tx_buf[..12 + packet.len()]);
                    tx_buf[0..12].copy_from_slice(&header.to_bytes());

                    self.ch.tx_done();
                }
                Either5::Third(()) => {
                    tx_buf[..PayloadHeader::SIZE].fill(0);
                }
                Either5::Fourth(()) => {
                    // Extend the deadline if initializing
                    if let ioctl::ControlState::Reboot = self.shared.state() {
                        self.reset_heartbeat_deadline();
                        continue;
                    }
                    match self.events_notifier {
                        Some(notifier) => {
                            notifier.signal(EspHostedEvents::Deadline);
                            self.reset_heartbeat_deadline();
                        }
                        None => {
                            panic!("heartbeat from esp32 stopped");
                        }
                    }
                }
                Either5::Fifth(()) => {
                    self.reset_device().await;
                    self.state_ch.set_link_state(LinkState::Down);
                    self.shared.reset_request_done();
                }
            }

            if tx_buf[0] != 0 {
                trace!("tx: {}", HexSlice(&tx_buf[..40]));
            }

            self.iface.transfer(rx_buf, tx_buf).await;

            self.handle_rx(rx_buf);
        }
    }

    fn handle_rx(&mut self, buf: &mut [u8]) {
        trace!("rx: {}", HexSlice(&buf[..40]));

        let buf_len = buf.len();
        let h =
            PayloadHeader::from_bytes_mut((&mut buf[..PayloadHeader::SIZE]).try_into().unwrap());

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
            warn!(
                "rx: bad checksum. Got {:04x}, want {:04x}",
                got_checksum, want_checksum
            );
            return;
        }

        let payload = &mut buf[PayloadHeader::SIZE..][..payload_len];

        match if_type_and_num & 0x0f {
            // STA
            ESP_STA_IF => match self.ch.try_rx_buf() {
                Some(buf) => {
                    buf[..payload.len()].copy_from_slice(payload);
                    self.ch.rx_done(payload.len())
                }
                None => warn!("failed to push rxd packet to the channel."),
            },
            // serial
            ESP_SERIAL_IF => {
                trace!("serial rx: {}", HexSlice(payload));
                if payload.len() < SERIAL_MSG_HEADER_LEN {
                    warn!("serial rx: too short");
                    return;
                }

                let is_event = match &payload[..(SERIAL_MSG_HEADER_LEN - 2)] {
                    b"\x01\x06\x00RPCRsp\x02" => false,
                    b"\x01\x06\x00RPCEvt\x02" => true,
                    _ => {
                        warn!("serial rx: bad tlv");
                        return;
                    }
                };

                let len = u16::from_le_bytes(
                    payload[(SERIAL_MSG_HEADER_LEN - 2)..SERIAL_MSG_HEADER_LEN]
                        .try_into()
                        .unwrap(),
                ) as usize;
                if payload.len() < SERIAL_MSG_HEADER_LEN + len {
                    warn!("serial rx: too short 2");
                    return;
                }
                let data = &payload[SERIAL_MSG_HEADER_LEN..][..len];

                if is_event {
                    self.handle_event(data);
                } else {
                    self.shared.ioctl_done(data);
                }
            }
            _ => warn!("unknown iftype {}", if_type_and_num),
        }
    }

    fn handle_event(&mut self, data: &[u8]) {
        use micropb::MessageDecode;
        let mut event = Rpc::default();
        if event.decode_from_bytes(data).is_err() {
            warn!("failed to parse event");
            return;
        }

        debug!("event: {:?}", &event);

        let Some(payload) = &event.payload else {
            warn!("event without payload?");
            return;
        };

        match payload {
            Rpc_::Payload::EventEspInit(_) => {
                self.shared.init_done();
                self.reset_heartbeat_deadline();
            }
            Rpc_::Payload::EventHeartbeat(_) => {
                self.reset_heartbeat_deadline();
            }
            Rpc_::Payload::EventStaConnected(e) => {
                info!("connected, code {}", e.resp);
                self.state_ch.set_link_state(LinkState::Up);
                self.shared.link_up_done();

                // ESP-Hosted firmware sends disconnect event even for failed connection attempts.
                // We reset the signal to avoid spurious disconnection events.
                if let Some(notifier) = self.events_notifier {
                    notifier.reset();
                }
            }
            Rpc_::Payload::EventStaDisconnected(e) => {
                info!("disconnected, code {}", e.resp);
                self.state_ch.set_link_state(LinkState::Down);
                self.shared.link_down_done();

                if let Some(notifier) = self.events_notifier {
                    notifier.signal(EspHostedEvents::Disconnected);
                }
            }
            _ => {}
        }
    }

    fn reset_heartbeat_deadline(&mut self) {
        self.heartbeat_deadline = Instant::now() + HEARTBEAT_MAX_GAP;
    }

    async fn reset_device(&mut self) {
        self.reset.set_low().unwrap();
        Timer::after_millis(100).await;
        self.reset.set_high().unwrap();
        Timer::after_millis(1000).await;
        self.next_seq = 1;
    }
}

fn checksum(buf: &[u8]) -> u16 {
    let mut res = 0u16;
    for &b in buf {
        res = res.wrapping_add(b as _);
    }
    res
}
