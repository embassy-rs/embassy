#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::task::Context;

/// Representation of an hardware address, such as an Ethernet address or an IEEE802.15.4 address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HardwareAddress {
    /// A six-octet Ethernet address
    Ethernet([u8; 6]),
    /// An eight-octet IEEE802.15.4 address
    Ieee802154([u8; 8]),
    /// Indicates that a Driver is IP-native, and has no hardware address
    Ip,
}

/// Main `embassy-net` driver API.
///
/// This is essentially an interface for sending and receiving raw network frames.
///
/// The interface is based on _tokens_, which are types that allow to receive/transmit a
/// single packet. The `receive` and `transmit` functions only construct such tokens, the
/// real sending/receiving operation are performed when the tokens are consumed.
pub trait Driver {
    /// A token to receive a single network packet.
    type RxToken<'a>: RxToken
    where
        Self: 'a;

    /// A token to transmit a single network packet.
    type TxToken<'a>: TxToken
    where
        Self: 'a;

    /// Construct a token pair consisting of one receive token and one transmit token.
    ///
    /// If there is a packet ready to be received, this function must return `Some`.
    /// If there isn't, it must return `None`, and wake `cx.waker()` when a packet is ready.
    ///
    /// The additional transmit token makes it possible to generate a reply packet based
    /// on the contents of the received packet. For example, this makes it possible to
    /// handle arbitrarily large ICMP echo ("ping") requests, where the all received bytes
    /// need to be sent back, without heap allocation.
    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)>;

    /// Construct a transmit token.
    ///
    /// If there is free space in the transmit buffer to transmit a packet, this function must return `Some`.
    /// If there isn't, it must return `None`, and wake `cx.waker()` when space becomes available.
    ///
    /// Note that [`TxToken::consume`] is infallible, so it is not allowed to return a token
    /// if there is no free space and fail later.
    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>>;

    /// Get the link state.
    ///
    /// This function must return the current link state of the device, and wake `cx.waker()` when
    /// the link state changes.
    fn link_state(&mut self, cx: &mut Context) -> LinkState;

    /// Get a description of device capabilities.
    fn capabilities(&self) -> Capabilities;

    /// Get the device's hardware address.
    fn hardware_address(&self) -> HardwareAddress;
}

impl<T: ?Sized + Driver> Driver for &mut T {
    type RxToken<'a> = T::RxToken<'a>
    where
        Self: 'a;
    type TxToken<'a> = T::TxToken<'a>
    where
        Self: 'a;

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        T::transmit(self, cx)
    }
    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        T::receive(self, cx)
    }
    fn capabilities(&self) -> Capabilities {
        T::capabilities(self)
    }
    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        T::link_state(self, cx)
    }
    fn hardware_address(&self) -> HardwareAddress {
        T::hardware_address(self)
    }
}

/// A token to receive a single network packet.
pub trait RxToken {
    /// Consumes the token to receive a single network packet.
    ///
    /// This method receives a packet and then calls the given closure `f` with the raw
    /// packet bytes as argument.
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

/// A token to transmit a single network packet.
pub trait TxToken {
    /// Consumes the token to send a single network packet.
    ///
    /// This method constructs a transmit buffer of size `len` and calls the passed
    /// closure `f` with a mutable reference to that buffer. The closure should construct
    /// a valid network packet (e.g. an ethernet packet) in the buffer. When the closure
    /// returns, the transmit buffer is sent out.
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

/// A description of device capabilities.
///
/// Higher-level protocols may achieve higher throughput or lower latency if they consider
/// the bandwidth or packet size limitations.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Capabilities {
    /// Medium of the device.
    ///
    /// This indicates what kind of packet the sent/received bytes are, and determines
    /// some behaviors of Interface. For example, ARP/NDISC address resolution is only done
    /// for Ethernet mediums.
    pub medium: Medium,

    /// Maximum transmission unit.
    ///
    /// The network device is unable to send or receive frames larger than the value returned
    /// by this function.
    ///
    /// For Ethernet devices, this is the maximum Ethernet frame size, including the Ethernet header (14 octets), but
    /// *not* including the Ethernet FCS (4 octets). Therefore, Ethernet MTU = IP MTU + 14.
    ///
    /// Note that in Linux and other OSes, "MTU" is the IP MTU, not the Ethernet MTU, even for Ethernet
    /// devices. This is a common source of confusion.
    ///
    /// Most common IP MTU is 1500. Minimum is 576 (for IPv4) or 1280 (for IPv6). Maximum is 9216 octets.
    pub max_transmission_unit: usize,

    /// Maximum burst size, in terms of MTU.
    ///
    /// The network device is unable to send or receive bursts large than the value returned
    /// by this function.
    ///
    /// If `None`, there is no fixed limit on burst size, e.g. if network buffers are
    /// dynamically allocated.
    pub max_burst_size: Option<usize>,

    /// Checksum behavior.
    ///
    /// If the network device is capable of verifying or computing checksums for some protocols,
    /// it can request that the stack not do so in software to improve performance.
    pub checksum: ChecksumCapabilities,
}

/// Type of medium of a device.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Medium {
    /// Ethernet medium. Devices of this type send and receive Ethernet frames,
    /// and interfaces using it must do neighbor discovery via ARP or NDISC.
    ///
    /// Examples of devices of this type are Ethernet, WiFi (802.11), Linux `tap`, and VPNs in tap (layer 2) mode.
    Ethernet,

    /// IP medium. Devices of this type send and receive IP frames, without an
    /// Ethernet header. MAC addresses are not used, and no neighbor discovery (ARP, NDISC) is done.
    ///
    /// Examples of devices of this type are the Linux `tun`, PPP interfaces, VPNs in tun (layer 3) mode.
    Ip,

    /// IEEE 802_15_4 medium
    Ieee802154,
}

impl Default for Medium {
    fn default() -> Medium {
        Medium::Ethernet
    }
}

/// A description of checksum behavior for every supported protocol.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ChecksumCapabilities {
    /// Checksum behavior for IPv4.
    pub ipv4: Checksum,
    /// Checksum behavior for UDP.
    pub udp: Checksum,
    /// Checksum behavior for TCP.
    pub tcp: Checksum,
    /// Checksum behavior for ICMPv4.
    pub icmpv4: Checksum,
    /// Checksum behavior for ICMPv6.
    pub icmpv6: Checksum,
}

/// A description of checksum behavior for a particular protocol.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Checksum {
    /// Verify checksum when receiving and compute checksum when sending.
    Both,
    /// Verify checksum when receiving.
    Rx,
    /// Compute checksum before sending.
    Tx,
    /// Ignore checksum completely.
    None,
}

impl Default for Checksum {
    fn default() -> Checksum {
        Checksum::Both
    }
}

/// The link state of a network device.
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkState {
    /// The link is down.
    Down,
    /// The link is up.
    Up,
}
