#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod config;
mod device;
mod packet_pool;
mod stack;

#[cfg(feature = "dhcpv4")]
pub use config::DhcpConfigurator;
pub use config::{Config, Configurator, Event as ConfigEvent, StaticConfigurator};

pub use device::{Device, LinkState};
pub use packet_pool::{Packet, PacketBox, PacketBoxExt, PacketBuf, MTU};
pub use stack::{
    config, ethernet_address, init, is_config_up, is_init, is_link_up, run, StackResources,
};

#[cfg(feature = "tcp")]
mod tcp_socket;
#[cfg(feature = "tcp")]
pub use tcp_socket::TcpSocket;

// smoltcp reexports
pub use smoltcp::phy::{DeviceCapabilities, Medium};
pub use smoltcp::time::Duration as SmolDuration;
pub use smoltcp::time::Instant as SmolInstant;
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::{EthernetAddress, HardwareAddress};
pub use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
pub type Interface = smoltcp::iface::Interface<'static, device::DeviceAdapter>;
pub use smoltcp::{Error, Result};
