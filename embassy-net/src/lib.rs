#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![feature(generic_associated_types, type_alias_impl_trait)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod device;
mod packet_pool;
mod stack;

pub use device::{Device, LinkState};
pub use packet_pool::{Packet, PacketBox, PacketBoxExt, PacketBuf, MTU};
pub use stack::{Config, ConfigStrategy, Stack, StackResources};

#[cfg(feature = "tcp")]
pub mod tcp;

// smoltcp reexports
pub use smoltcp::phy::{DeviceCapabilities, Medium};
pub use smoltcp::time::Duration as SmolDuration;
pub use smoltcp::time::Instant as SmolInstant;
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::{EthernetAddress, HardwareAddress};
pub use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
