use heapless::Vec;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::{Dhcpv4Event, Dhcpv4Socket};
use smoltcp::time::Instant;

use super::*;
use crate::device::LinkState;
use crate::Interface;

pub struct DhcpConfigurator {
    handle: Option<SocketHandle>,
}

impl DhcpConfigurator {
    pub fn new() -> Self {
        Self { handle: None }
    }
}

impl Configurator for DhcpConfigurator {
    fn poll(&mut self, iface: &mut Interface, _timestamp: Instant) -> Event {
        if self.handle.is_none() {
            let handle = iface.add_socket(Dhcpv4Socket::new());
            self.handle = Some(handle)
        }

        let link_up = iface.device_mut().device.link_state() == LinkState::Up;

        let socket = iface.get_socket::<Dhcpv4Socket>(self.handle.unwrap());

        if !link_up {
            socket.reset();
            return Event::Deconfigured;
        }

        match socket.poll() {
            None => Event::NoChange,
            Some(Dhcpv4Event::Deconfigured) => Event::Deconfigured,
            Some(Dhcpv4Event::Configured(config)) => {
                let mut dns_servers = Vec::new();
                for s in &config.dns_servers {
                    if let Some(addr) = s {
                        dns_servers.push(addr.clone()).unwrap();
                    }
                }

                Event::Configured(Config {
                    address: config.address,
                    gateway: config.router,
                    dns_servers,
                })
            }
        }
    }
}
