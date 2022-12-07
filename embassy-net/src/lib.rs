#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "nightly",
    feature(type_alias_impl_trait, async_fn_in_trait, impl_trait_projections)
)]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod device;
mod stack;

pub use stack::{Config, ConfigStrategy, Stack, StackResources};

#[cfg(feature = "tcp")]
pub mod tcp;

#[cfg(feature = "udp")]
pub mod udp;

// smoltcp reexports
pub use smoltcp::time::{Duration as SmolDuration, Instant as SmolInstant};
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::{EthernetAddress, HardwareAddress};
pub use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
#[cfg(feature = "proto-ipv6")]
pub use smoltcp::wire::{Ipv6Address, Ipv6Cidr};
#[cfg(feature = "udp")]
pub use smoltcp::{socket::udp::PacketMetadata, wire::IpListenEndpoint};
