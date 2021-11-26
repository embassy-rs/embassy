use heapless::Vec;
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

use crate::Interface;

mod statik;
pub use statik::StaticConfigurator;

#[cfg(feature = "dhcpv4")]
mod dhcp;
#[cfg(feature = "dhcpv4")]
pub use dhcp::DhcpConfigurator;

/// Return value for the `Configurator::poll` function
#[derive(Debug, Clone)]
pub enum Event {
    /// No change has occured to the configuration.
    NoChange,
    /// Configuration has been lost (for example, DHCP lease has expired)
    Deconfigured,
    /// Configuration has been newly acquired, or modified.
    Configured(Config),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub address: Ipv4Cidr,
    pub gateway: Option<Ipv4Address>,
    pub dns_servers: Vec<Ipv4Address, 3>,
}

pub trait Configurator {
    fn poll(&mut self, iface: &mut Interface, timestamp: Instant) -> Event;
}
