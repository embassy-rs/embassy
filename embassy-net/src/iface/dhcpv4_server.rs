//! DHCPv4 server.
//!
//! The server is part of an interface. Turn it on with
//! [`Iface::set_dhcpv4_server`] and it answers DHCP requests arriving on that
//! interface, handing out addresses from a configured pool. Inspect the leases
//! with [`Iface::dhcpv4_server_leases`] and remove one with
//! [`Iface::remove_dhcpv4_server_lease`].
//!
//! The interface must have an IPv4 address: it is the server's own address and
//! its subnet provides the subnet mask sent to clients. The pool must be inside
//! that subnet.
//!
//! Only Ethernet interfaces are supported.
//!
//! [`Iface::set_dhcpv4_server`]: super::Iface::set_dhcpv4_server
//! [`Iface::dhcpv4_server_leases`]: super::Iface::dhcpv4_server_leases
//! [`Iface::remove_dhcpv4_server_lease`]: super::Iface::remove_dhcpv4_server_lease

use embassy_time::{Duration, Instant};
use heapless::Vec;
pub use xarxa::iface::dhcpv4_server::*;

use crate::config::DHCP_MAX_DNS_SERVER_COUNT;
use crate::time::{duration_to_xarxa, instant_from_xarxa};
use crate::wire::{DhcpOption, EthernetAddress, Ipv4Address};

/// Configuration of the DHCP server, passed to [`Iface::set_dhcpv4_server`].
///
/// Start from [`DhcpServerConfig::new`] and change the fields you need.
///
/// [`Iface::set_dhcpv4_server`]: super::Iface::set_dhcpv4_server
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DhcpServerConfig {
    /// First address of the pool leases are taken from.
    pub pool_start: Ipv4Address,
    /// Last address of the pool, inclusive.
    pub pool_end: Ipv4Address,
    /// How long a lease lasts. Clients asking for a shorter lease get it.
    pub lease_duration: Duration,
    /// The default gateway sent to clients, if any.
    pub gateway: Option<Ipv4Address>,
    /// The DNS servers sent to clients. Empty sends none.
    pub dns_servers: Vec<Ipv4Address, DHCP_MAX_DNS_SERVER_COUNT>,
    /// Extra options added to every OFFER and ACK.
    pub outgoing_options: &'static [DhcpOption<'static>],
}

impl DhcpServerConfig {
    /// A configuration leasing addresses from `pool_start` to `pool_end`
    /// (inclusive) for one hour, with no gateway and no DNS servers.
    pub fn new(pool_start: Ipv4Address, pool_end: Ipv4Address) -> Self {
        Self {
            pool_start,
            pool_end,
            lease_duration: Duration::from_secs(3600),
            gateway: None,
            dns_servers: Vec::new(),
            outgoing_options: &[],
        }
    }

    pub(crate) fn to_xarxa(&self) -> xarxa::iface::dhcpv4_server::DhcpServerConfig {
        let mut config = xarxa::iface::dhcpv4_server::DhcpServerConfig::new(self.pool_start, self.pool_end);
        config.lease_duration = duration_to_xarxa(self.lease_duration);
        config.gateway = self.gateway;
        config.dns_servers = self.dns_servers.clone();
        config.outgoing_options = self.outgoing_options;
        config
    }
}

/// One entry of the DHCP server's lease table.
///
/// Read them with [`Iface::dhcpv4_server_leases`].
///
/// [`Iface::dhcpv4_server_leases`]: super::Iface::dhcpv4_server_leases
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DhcpServerLease(xarxa::iface::dhcpv4_server::DhcpServerLease);

impl DhcpServerLease {
    pub(crate) fn from_xarxa(lease: xarxa::iface::dhcpv4_server::DhcpServerLease) -> Self {
        Self(lease)
    }

    /// The leased address.
    pub fn address(&self) -> Ipv4Address {
        self.0.address()
    }

    /// The client's hardware address, from its latest message.
    pub fn hardware_addr(&self) -> EthernetAddress {
        self.0.hardware_addr()
    }

    /// The client identifier the client sent, or `None` if it sent none and is
    /// identified by its hardware address.
    pub fn client_id(&self) -> Option<&[u8]> {
        self.0.client_id()
    }

    /// The state of the lease.
    pub fn state(&self) -> DhcpServerLeaseState {
        self.0.state()
    }

    /// When the lease stops holding its address.
    ///
    /// For an offered lease this is when the unanswered offer lapses, for a
    /// bound one the end of the lease, and for a declined one the end of the
    /// hold that keeps the address out of the pool. A released lease is already
    /// past it. Past this time the entry is only a record: the address is free,
    /// and the entry makes a returning client get it again.
    pub fn expires_at(&self) -> Instant {
        instant_from_xarxa(self.0.expires_at())
    }
}
