use heapless::consts::*;
use heapless::Vec;
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

use crate::fmt::*;
use crate::{Interface, SocketSet};

mod dhcp;
mod statik;
pub use dhcp::DhcpConfigurator;
pub use statik::StaticConfigurator;

#[derive(Debug, Clone)]
pub enum Config {
    Down,
    Up(UpConfig),
}

#[derive(Debug, Clone)]
pub struct UpConfig {
    pub address: Ipv4Cidr,
    pub gateway: Ipv4Address,
    pub dns_servers: Vec<Ipv4Address, U3>,
}

pub trait Configurator {
    fn poll(
        &mut self,
        iface: &mut Interface,
        sockets: &mut SocketSet,
        timestamp: Instant,
    ) -> Option<Config>;
}
