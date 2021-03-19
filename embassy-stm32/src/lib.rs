#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479",
))]
pub use {stm32f4xx_hal as hal, stm32f4xx_hal::stm32 as pac};

#[cfg(any(feature = "stm32l0x1", feature = "stm32l0x2", feature = "stm32l0x3",))]
pub use {stm32l0xx_hal as hal, stm32l0xx_hal::pac};

pub mod fmt;

pub mod interrupt;
