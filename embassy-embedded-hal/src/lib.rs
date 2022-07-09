#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "nightly")]
pub mod adapter;

pub mod shared_bus;

pub trait SetConfig {
    type Config;
    fn set_config(&mut self, config: &Self::Config);
}
