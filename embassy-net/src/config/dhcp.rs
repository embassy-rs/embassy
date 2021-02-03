use embassy::util::Forever;
use heapless::consts::*;
use heapless::Vec;
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::socket::{RawPacketMetadata, RawSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

use super::*;
use crate::{device::LinkState, fmt::*};
use crate::{Interface, SocketSet};

pub struct DhcpResources {
    rx_buffer: [u8; 900],
    tx_buffer: [u8; 600],
    rx_meta: [RawPacketMetadata; 1],
    tx_meta: [RawPacketMetadata; 1],
}

pub struct DhcpConfigurator {
    client: Option<Dhcpv4Client>,
}

impl DhcpConfigurator {
    pub fn new() -> Self {
        Self { client: None }
    }
}

static DHCP_RESOURCES: Forever<DhcpResources> = Forever::new();

impl Configurator for DhcpConfigurator {
    fn poll(
        &mut self,
        iface: &mut Interface,
        sockets: &mut SocketSet,
        timestamp: Instant,
    ) -> Option<Config> {
        if self.client.is_none() {
            let res = DHCP_RESOURCES.put(DhcpResources {
                rx_buffer: [0; 900],
                tx_buffer: [0; 600],
                rx_meta: [RawPacketMetadata::EMPTY; 1],
                tx_meta: [RawPacketMetadata::EMPTY; 1],
            });
            let rx_buffer = RawSocketBuffer::new(&mut res.rx_meta[..], &mut res.rx_buffer[..]);
            let tx_buffer = RawSocketBuffer::new(&mut res.tx_meta[..], &mut res.tx_buffer[..]);
            let dhcp = Dhcpv4Client::new(sockets, rx_buffer, tx_buffer, timestamp);
            info!("created dhcp");
            self.client = Some(dhcp)
        }

        let client = self.client.as_mut().unwrap();

        let link_up = iface.device_mut().device.link_state() == LinkState::Up;
        if !link_up {
            client.reset(timestamp);
            return Some(Config::Down);
        }

        let config = client.poll(iface, sockets, timestamp).unwrap_or(None)?;

        if config.address.is_none() {
            return Some(Config::Down);
        }

        let mut dns_servers = Vec::new();
        for s in &config.dns_servers {
            if let Some(addr) = s {
                dns_servers.push(addr.clone()).unwrap();
            }
        }

        return Some(Config::Up(UpConfig {
            address: config.address.unwrap(),
            gateway: config.router.unwrap_or(Ipv4Address::UNSPECIFIED),
            dns_servers,
        }));
    }
}
