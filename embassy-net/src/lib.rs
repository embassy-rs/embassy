#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![allow(incomplete_features)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod config;
mod device;
mod packet_pool;
mod stack;

pub use config::{Config, Configurator, DhcpConfigurator, StaticConfigurator};
pub use device::{Device, LinkState};
pub use packet_pool::{Packet, PacketBox, PacketBoxExt, PacketBuf};
pub use stack::{init, is_init, run};

#[cfg(feature = "tcp")]
mod tcp_socket;
#[cfg(feature = "tcp")]
pub use tcp_socket::TcpSocket;

// smoltcp reexports
pub use smoltcp::phy::{DeviceCapabilities, Medium};
pub use smoltcp::time::Duration as SmolDuration;
pub use smoltcp::time::Instant as SmolInstant;
pub use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
pub type Interface = smoltcp::iface::Interface<'static, device::DeviceAdapter>;
pub type SocketSet = smoltcp::socket::SocketSet<'static>;
pub use smoltcp::{Error, Result};
