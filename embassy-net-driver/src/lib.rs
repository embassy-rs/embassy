#![no_std]

use core::task::Context;

pub trait Driver {
    type RxToken<'a>: RxToken
    where
        Self: 'a;
    type TxToken<'a>: TxToken
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)>;
    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>>;
    fn link_state(&mut self, cx: &mut Context) -> LinkState;

    fn capabilities(&self) -> Capabilities;
    fn ethernet_address(&self) -> [u8; 6];
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
    fn ethernet_address(&self) -> [u8; 6] {
        T::ethernet_address(self)
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
    pub ipv4: Checksum,
    pub udp: Checksum,
    pub tcp: Checksum,
    pub icmpv4: Checksum,
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

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkState {
    Down,
    Up,
}
