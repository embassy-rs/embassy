#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

pub(crate) use stm32_metapac as pac;

#[macro_use]
pub mod exti;
#[macro_use]
pub mod gpio;
//pub mod rtc;
//pub mod interrupt;
#[macro_use]
pub mod usart;

// This must go LAST so that it sees the `impl_foo!` macros
mod chip;
pub use chip::{peripherals, Peripherals};
