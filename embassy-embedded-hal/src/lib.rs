#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

pub mod adapter;
pub mod shared_bus;

pub trait SetConfig<C> {
    fn set_config(&mut self, config: &C);
}
