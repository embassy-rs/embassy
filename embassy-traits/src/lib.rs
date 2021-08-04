#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait)]

pub mod delay;
pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod rng;
pub mod spi;
pub mod uart;
