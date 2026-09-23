//! DHCPv4 client.
//!
//! The client is part of an interface. Turn it on with [`Iface::set_dhcpv4`] and it
//! runs from the [`Runner`](crate::Runner): it finds a server, gets a lease, and
//! renews it. The leased address and default route are installed on the interface by
//! the stack itself. Read the lease with [`Iface::dhcpv4_lease`], and watch
//! [`Iface::config_generation`] to notice changes.
//!
//! Only Ethernet interfaces are supported.
//!
//! [`Iface::set_dhcpv4`]: super::Iface::set_dhcpv4
//! [`Iface::dhcpv4_lease`]: super::Iface::dhcpv4_lease
//! [`Iface::config_generation`]: super::Iface::config_generation

use embassy_time::Duration;
pub use xarxa::iface::dhcpv4::*;

use crate::time::duration_to_xarxa;
use crate::wire::DhcpOption;

/// Configuration of the DHCP client, passed to [`Iface::set_dhcpv4`].
///
/// Start from [`DhcpConfig::default`] and change the fields you need.
///
/// [`Iface::set_dhcpv4`]: super::Iface::set_dhcpv4
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DhcpConfig {
    /// Extra options added to every outgoing packet.
    pub outgoing_options: &'static [DhcpOption<'static>],
    /// The parameter request list option sent to the server.
    ///
    /// `None` asks for the subnet mask, router and DNS servers. Changing this does
    /// not change which options the client itself reads from the lease.
    pub parameter_request_list: Option<&'static [u8]>,
    /// A cap on the lease duration the server gives.
    ///
    /// Useful to react faster to IP configuration changes, and to test renewals.
    pub max_lease_duration: Option<Duration>,
    /// Ignore NAKs from the server.
    ///
    /// This is not RFC compliant. It is a workaround for servers that send spurious
    /// NAKs, for example when several servers share a network.
    pub ignore_naks: bool,
}

impl Default for DhcpConfig {
    fn default() -> Self {
        Self {
            outgoing_options: &[],
            parameter_request_list: None,
            max_lease_duration: None,
            ignore_naks: false,
        }
    }
}

impl DhcpConfig {
    pub(crate) fn to_xarxa(&self) -> xarxa::iface::dhcpv4::DhcpConfig {
        let mut config = xarxa::iface::dhcpv4::DhcpConfig::default();
        config.outgoing_options = self.outgoing_options;
        config.parameter_request_list = self.parameter_request_list;
        config.max_lease_duration = self.max_lease_duration.map(duration_to_xarxa);
        config.ignore_naks = self.ignore_naks;
        config
    }
}
