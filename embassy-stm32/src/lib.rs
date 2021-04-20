#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

pub mod fmt;

mod chip;
pub use chip::{peripherals, Peripherals};

pub mod exti;
pub mod gpio;
//pub mod rtc;
//pub mod interrupt;
//pub mod usart;

pub(crate) use stm32_metapac as pac;
