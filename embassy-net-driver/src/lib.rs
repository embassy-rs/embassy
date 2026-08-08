#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::task::Context;

/// Metadata associated to a packet.
///
/// The packet metadata is a set of attributes associated to network packets
/// as they travel up or down the stack. The metadata is get/set by the
/// [`Driver`] implementations or by the user when sending/receiving packets from a
/// socket.
///
/// Metadata fields are enabled via Cargo features. If no field is enabled, this
/// struct becomes zero-sized, which allows the compiler to optimize it out as if
/// the packet metadata mechanism didn't exist at all.
///
/// This struct is marked as `#[non_exhaustive]`. This means it is not possible to
/// create it directly by specifying all fields. You have to instead create it with
/// default values and then set the fields you want. This makes adding metadata
/// fields a non-breaking change.
///
/// ```rust
/// let mut meta = xarxa_driver::PacketMeta::default();
/// #[cfg(feature = "packetmeta-id")]
/// {
///     meta.id = 15;
/// }
/// ```
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[non_exhaustive]
pub struct PacketMeta {
    #[cfg(feature = "packetmeta-id")]
    /// An opaque identifier for this packet.
    ///
    /// On received packets, this is set by the [`Device`]. On packets to transmit,
    /// this is set by the user and passed down to the [`Device`]; it is also what
    /// correlates a transmit timestamp back to the packet that produced it, see
    /// [`Device::poll_tx_timestamp`].
    pub id: u32,

    #[cfg(feature = "packetmeta-timestamp")]
    /// The time at which this packet was received, as measured by the device.
    ///
    /// `None` if the device did not timestamp this packet. Devices commonly only
    /// timestamp a subset of received packets, e.g. only PTP event messages.
    ///
    /// This field is only meaningful on received packets. It is ignored on packets
    /// to transmit: at the time a packet is handed to the device, it has not been
    /// transmitted yet, so its transmit timestamp does not exist yet. Use
    /// [`Self::request_timestamp`] and [`Device::poll_tx_timestamp`] instead.
    pub timestamp: Option<Timestamp>,

    #[cfg(feature = "packetmeta-timestamp")]
    /// Request that the device timestamp this packet as it is transmitted.
    ///
    /// The resulting timestamp is reported back later, out of band, via
    /// [`Device::poll_tx_timestamp`], tagged with this packet's [`Self::id`].
    ///
    /// This field is only meaningful on packets to transmit. It is ignored on
    /// received packets.
    ///
    /// Timestamping is opt-in per packet because hardware typically has only a
    /// handful of transmit timestamp slots. Requesting a timestamp for every packet
    /// will cause most of them to be dropped.
    pub request_timestamp: bool,
}

/// The timestamp of a transmitted packet, reported by [`Device::poll_tx_timestamp`].
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TxTimestamp {
    /// The [`PacketMeta::id`] of the packet this timestamp belongs to.
    pub id: u32,

    /// The time at which the packet was transmitted, as measured by the device.
    pub timestamp: Timestamp,
}

/// Representation of an hardware address, such as an Ethernet address or an IEEE802.15.4 address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum HardwareAddress {
    /// Ethernet medium, with a A six-octet Ethernet address.
    ///
    /// Devices of this type send and receive Ethernet frames,
    /// and interfaces using it must do neighbor discovery via ARP or NDISC.
    ///
    /// Examples of devices of this type are Ethernet, WiFi (802.11), Linux `tap`, and VPNs in tap (layer 2) mode.
    Ethernet([u8; 6]),
    /// 6LoWPAN over IEEE802.15.4, with an eight-octet address.
    Ieee802154([u8; 8]),
    /// Indicates that a Driver is IP-native, and has no hardware address.
    ///
    /// Devices of this type send and receive IP frames, without an
    /// Ethernet header. MAC addresses are not used, and no neighbor discovery (ARP, NDISC) is done.
    ///
    /// Examples of devices of this type are the Linux `tun`, PPP interfaces, VPNs in tun (layer 3) mode.
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

    /// Poll the driver for timestamps and return a pair of (id, `Timestamp`)
    ///
    /// Ids may be reused, and therefore this method should be called before calling
    /// `receive` or `transmit` to avoid deducing an incorrect association.
    #[allow(unused_variables)]
    fn poll_timestamp(&mut self, cx: &mut Context) -> Option<TxTimestamp> {
        None
    }

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
    ///
    /// The returned hardware address also determines the "medium" of this driver. This indicates
    /// what kind of packet the sent/received bytes are, and determines some behaviors of
    /// the interface. For example, ARP/NDISC address resolution is only done for Ethernet mediums.
    fn hardware_address(&self) -> HardwareAddress;
}

impl<T: ?Sized + Driver> Driver for &mut T {
    type RxToken<'a>
        = T::RxToken<'a>
    where
        Self: 'a;
    type TxToken<'a>
        = T::TxToken<'a>
    where
        Self: 'a;

    fn poll_timestamp(&mut self, cx: &mut Context) -> Option<TxTimestamp> {
        T::poll_timestamp(self, cx)
    }

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

/// A representation of a hardware packet timestamp.
///
/// This is a reading of the *device's own clock*, not of the `Instant` the stack is polled
/// with. Such a clock is usually called a "PTP hardware clock" or PHC. It has an arbitrary
/// epoch (often, but not necessarily, the time since the device was reset) and it drifts
/// with respect to any other clock in the system unless something is actively disciplining
/// it. Do not mix `Timestamp` and `Instant` values.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Timestamp {
    /// Whole seconds.
    pub seconds: u32,
    /// Fraction of a second, in units of 0.25 nanoseconds.
    ///
    /// Always less than `4_000_000_000`, i.e. less than one whole second.
    pub quarter_nanos: u32,
}

impl Timestamp {
    /// Construct a timestamp from seconds and nanoseconds
    #[inline]
    pub const fn from_seconds_and_nanos(seconds: u32, nanos: u32) -> Self {
        Self {
            seconds,
            quarter_nanos: nanos << 2,
        }
    }

    /// Get the nanoseconds for this timestamp
    #[inline]
    pub const fn nanos(&self) -> u32 {
        self.quarter_nanos >> 2
    }
}

/// A token to receive a single network packet.
pub trait RxToken {
    /// Get the buffer for this packet.
    fn buf(&mut self) -> &mut [u8] {
        &mut []
    }

    /// Consumes the token to receive a single network packet.
    ///
    /// This method receives a packet and then calls the given closure `f` with the raw
    /// packet bytes as argument.
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;

    /// The Packet metadata associated with the frame received by this [`RxToken`]
    fn meta(&self) -> PacketMeta {
        PacketMeta::default()
    }
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

    /// The Packet metadata to be associated with the frame to be transmitted by
    /// this [`TxToken`].
    #[allow(unused_variables)]
    fn set_meta(&mut self, meta: PacketMeta) {}
}

/// A description of device capabilities.
///
/// Higher-level protocols may achieve higher throughput or lower latency if they consider
/// the bandwidth or packet size limitations.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Capabilities {
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

    /// If set to true, hardware timestamps are supported.
    pub timestamp: bool,
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
