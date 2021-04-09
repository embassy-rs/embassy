#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

pub mod fmt;

#[cfg_attr(feature = "f401", path = "chip/f401.rs")]
#[cfg_attr(feature = "f405", path = "chip/f405.rs")]
#[cfg_attr(feature = "f407", path = "chip/f407.rs")]
#[cfg_attr(feature = "f410", path = "chip/f410.rs")]
#[cfg_attr(feature = "f411", path = "chip/f411.rs")]
#[cfg_attr(feature = "f412", path = "chip/f412.rs")]
#[cfg_attr(feature = "f413", path = "chip/f413.rs")]
#[cfg_attr(feature = "f415", path = "chip/f415.rs")]
#[cfg_attr(feature = "f417", path = "chip/f417.rs")]
#[cfg_attr(feature = "f423", path = "chip/f423.rs")]
#[cfg_attr(feature = "f427", path = "chip/f427.rs")]
#[cfg_attr(feature = "f429", path = "chip/f429.rs")]
#[cfg_attr(feature = "f437", path = "chip/f437.rs")]
#[cfg_attr(feature = "f439", path = "chip/f439.rs")]
#[cfg_attr(feature = "f446", path = "chip/f446.rs")]
#[cfg_attr(feature = "f469", path = "chip/f469.rs")]
#[cfg_attr(feature = "f479", path = "chip/f479.rs")]
mod chip;
pub use chip::{peripherals, Peripherals};

pub mod exti;
pub mod gpio;
//pub mod rtc;
//pub mod interrupt;

pub(crate) use stm32_metapac as pac;
