#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[cfg(not(any(
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
)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features: "
);

#[cfg(any(
    all(feature = "stm32f401", feature = "stm32f405"),
    all(feature = "stm32f401", feature = "stm32f407"),
    all(feature = "stm32f401", feature = "stm32f410"),
    all(feature = "stm32f401", feature = "stm32f411"),
    all(feature = "stm32f401", feature = "stm32f412"),
    all(feature = "stm32f401", feature = "stm32f413"),
    all(feature = "stm32f401", feature = "stm32f415"),
    all(feature = "stm32f401", feature = "stm32f417"),
    all(feature = "stm32f401", feature = "stm32f423"),
    all(feature = "stm32f401", feature = "stm32f427"),
    all(feature = "stm32f401", feature = "stm32f429"),
    all(feature = "stm32f401", feature = "stm32f437"),
    all(feature = "stm32f401", feature = "stm32f439"),
    all(feature = "stm32f401", feature = "stm32f446"),
    all(feature = "stm32f401", feature = "stm32f469"),
    all(feature = "stm32f401", feature = "stm32f479"),
    all(feature = "stm32f405", feature = "stm32f401"),
    all(feature = "stm32f405", feature = "stm32f407"),
    all(feature = "stm32f405", feature = "stm32f410"),
    all(feature = "stm32f405", feature = "stm32f411"),
    all(feature = "stm32f405", feature = "stm32f412"),
    all(feature = "stm32f405", feature = "stm32f413"),
    all(feature = "stm32f405", feature = "stm32f415"),
    all(feature = "stm32f405", feature = "stm32f417"),
    all(feature = "stm32f405", feature = "stm32f423"),
    all(feature = "stm32f405", feature = "stm32f427"),
    all(feature = "stm32f405", feature = "stm32f429"),
    all(feature = "stm32f405", feature = "stm32f437"),
    all(feature = "stm32f405", feature = "stm32f439"),
    all(feature = "stm32f405", feature = "stm32f446"),
    all(feature = "stm32f405", feature = "stm32f469"),
    all(feature = "stm32f405", feature = "stm32f479"),
    all(feature = "stm32f407", feature = "stm32f401"),
    all(feature = "stm32f407", feature = "stm32f405"),
    all(feature = "stm32f407", feature = "stm32f410"),
    all(feature = "stm32f407", feature = "stm32f411"),
    all(feature = "stm32f407", feature = "stm32f412"),
    all(feature = "stm32f407", feature = "stm32f413"),
    all(feature = "stm32f407", feature = "stm32f415"),
    all(feature = "stm32f407", feature = "stm32f417"),
    all(feature = "stm32f407", feature = "stm32f423"),
    all(feature = "stm32f407", feature = "stm32f427"),
    all(feature = "stm32f407", feature = "stm32f429"),
    all(feature = "stm32f407", feature = "stm32f437"),
    all(feature = "stm32f407", feature = "stm32f439"),
    all(feature = "stm32f407", feature = "stm32f446"),
    all(feature = "stm32f407", feature = "stm32f469"),
    all(feature = "stm32f407", feature = "stm32f479"),
    all(feature = "stm32f410", feature = "stm32f401"),
    all(feature = "stm32f410", feature = "stm32f405"),
    all(feature = "stm32f410", feature = "stm32f407"),
    all(feature = "stm32f410", feature = "stm32f411"),
    all(feature = "stm32f410", feature = "stm32f412"),
    all(feature = "stm32f410", feature = "stm32f413"),
    all(feature = "stm32f410", feature = "stm32f415"),
    all(feature = "stm32f410", feature = "stm32f417"),
    all(feature = "stm32f410", feature = "stm32f423"),
    all(feature = "stm32f410", feature = "stm32f427"),
    all(feature = "stm32f410", feature = "stm32f429"),
    all(feature = "stm32f410", feature = "stm32f437"),
    all(feature = "stm32f410", feature = "stm32f439"),
    all(feature = "stm32f410", feature = "stm32f446"),
    all(feature = "stm32f410", feature = "stm32f469"),
    all(feature = "stm32f410", feature = "stm32f479"),
    all(feature = "stm32f411", feature = "stm32f401"),
    all(feature = "stm32f411", feature = "stm32f405"),
    all(feature = "stm32f411", feature = "stm32f407"),
    all(feature = "stm32f411", feature = "stm32f410"),
    all(feature = "stm32f411", feature = "stm32f412"),
    all(feature = "stm32f411", feature = "stm32f413"),
    all(feature = "stm32f411", feature = "stm32f415"),
    all(feature = "stm32f411", feature = "stm32f417"),
    all(feature = "stm32f411", feature = "stm32f423"),
    all(feature = "stm32f411", feature = "stm32f427"),
    all(feature = "stm32f411", feature = "stm32f429"),
    all(feature = "stm32f411", feature = "stm32f437"),
    all(feature = "stm32f411", feature = "stm32f439"),
    all(feature = "stm32f411", feature = "stm32f446"),
    all(feature = "stm32f411", feature = "stm32f469"),
    all(feature = "stm32f411", feature = "stm32f479"),
    all(feature = "stm32f412", feature = "stm32f401"),
    all(feature = "stm32f412", feature = "stm32f405"),
    all(feature = "stm32f412", feature = "stm32f407"),
    all(feature = "stm32f412", feature = "stm32f410"),
    all(feature = "stm32f412", feature = "stm32f411"),
    all(feature = "stm32f412", feature = "stm32f413"),
    all(feature = "stm32f412", feature = "stm32f415"),
    all(feature = "stm32f412", feature = "stm32f417"),
    all(feature = "stm32f412", feature = "stm32f423"),
    all(feature = "stm32f412", feature = "stm32f427"),
    all(feature = "stm32f412", feature = "stm32f429"),
    all(feature = "stm32f412", feature = "stm32f437"),
    all(feature = "stm32f412", feature = "stm32f439"),
    all(feature = "stm32f412", feature = "stm32f446"),
    all(feature = "stm32f412", feature = "stm32f469"),
    all(feature = "stm32f412", feature = "stm32f479"),
    all(feature = "stm32f413", feature = "stm32f401"),
    all(feature = "stm32f413", feature = "stm32f405"),
    all(feature = "stm32f413", feature = "stm32f407"),
    all(feature = "stm32f413", feature = "stm32f410"),
    all(feature = "stm32f413", feature = "stm32f411"),
    all(feature = "stm32f413", feature = "stm32f412"),
    all(feature = "stm32f413", feature = "stm32f415"),
    all(feature = "stm32f413", feature = "stm32f417"),
    all(feature = "stm32f413", feature = "stm32f423"),
    all(feature = "stm32f413", feature = "stm32f427"),
    all(feature = "stm32f413", feature = "stm32f429"),
    all(feature = "stm32f413", feature = "stm32f437"),
    all(feature = "stm32f413", feature = "stm32f439"),
    all(feature = "stm32f413", feature = "stm32f446"),
    all(feature = "stm32f413", feature = "stm32f469"),
    all(feature = "stm32f413", feature = "stm32f479"),
    all(feature = "stm32f415", feature = "stm32f401"),
    all(feature = "stm32f415", feature = "stm32f405"),
    all(feature = "stm32f415", feature = "stm32f407"),
    all(feature = "stm32f415", feature = "stm32f410"),
    all(feature = "stm32f415", feature = "stm32f411"),
    all(feature = "stm32f415", feature = "stm32f412"),
    all(feature = "stm32f415", feature = "stm32f413"),
    all(feature = "stm32f415", feature = "stm32f417"),
    all(feature = "stm32f415", feature = "stm32f423"),
    all(feature = "stm32f415", feature = "stm32f427"),
    all(feature = "stm32f415", feature = "stm32f429"),
    all(feature = "stm32f415", feature = "stm32f437"),
    all(feature = "stm32f415", feature = "stm32f439"),
    all(feature = "stm32f415", feature = "stm32f446"),
    all(feature = "stm32f415", feature = "stm32f469"),
    all(feature = "stm32f415", feature = "stm32f479"),
    all(feature = "stm32f417", feature = "stm32f401"),
    all(feature = "stm32f417", feature = "stm32f405"),
    all(feature = "stm32f417", feature = "stm32f407"),
    all(feature = "stm32f417", feature = "stm32f410"),
    all(feature = "stm32f417", feature = "stm32f411"),
    all(feature = "stm32f417", feature = "stm32f412"),
    all(feature = "stm32f417", feature = "stm32f413"),
    all(feature = "stm32f417", feature = "stm32f415"),
    all(feature = "stm32f417", feature = "stm32f423"),
    all(feature = "stm32f417", feature = "stm32f427"),
    all(feature = "stm32f417", feature = "stm32f429"),
    all(feature = "stm32f417", feature = "stm32f437"),
    all(feature = "stm32f417", feature = "stm32f439"),
    all(feature = "stm32f417", feature = "stm32f446"),
    all(feature = "stm32f417", feature = "stm32f469"),
    all(feature = "stm32f417", feature = "stm32f479"),
    all(feature = "stm32f423", feature = "stm32f401"),
    all(feature = "stm32f423", feature = "stm32f405"),
    all(feature = "stm32f423", feature = "stm32f407"),
    all(feature = "stm32f423", feature = "stm32f410"),
    all(feature = "stm32f423", feature = "stm32f411"),
    all(feature = "stm32f423", feature = "stm32f412"),
    all(feature = "stm32f423", feature = "stm32f413"),
    all(feature = "stm32f423", feature = "stm32f415"),
    all(feature = "stm32f423", feature = "stm32f417"),
    all(feature = "stm32f423", feature = "stm32f427"),
    all(feature = "stm32f423", feature = "stm32f429"),
    all(feature = "stm32f423", feature = "stm32f437"),
    all(feature = "stm32f423", feature = "stm32f439"),
    all(feature = "stm32f423", feature = "stm32f446"),
    all(feature = "stm32f423", feature = "stm32f469"),
    all(feature = "stm32f423", feature = "stm32f479"),
    all(feature = "stm32f427", feature = "stm32f401"),
    all(feature = "stm32f427", feature = "stm32f405"),
    all(feature = "stm32f427", feature = "stm32f407"),
    all(feature = "stm32f427", feature = "stm32f410"),
    all(feature = "stm32f427", feature = "stm32f411"),
    all(feature = "stm32f427", feature = "stm32f412"),
    all(feature = "stm32f427", feature = "stm32f413"),
    all(feature = "stm32f427", feature = "stm32f415"),
    all(feature = "stm32f427", feature = "stm32f417"),
    all(feature = "stm32f427", feature = "stm32f423"),
    all(feature = "stm32f427", feature = "stm32f429"),
    all(feature = "stm32f427", feature = "stm32f437"),
    all(feature = "stm32f427", feature = "stm32f439"),
    all(feature = "stm32f427", feature = "stm32f446"),
    all(feature = "stm32f427", feature = "stm32f469"),
    all(feature = "stm32f427", feature = "stm32f479"),
    all(feature = "stm32f429", feature = "stm32f401"),
    all(feature = "stm32f429", feature = "stm32f405"),
    all(feature = "stm32f429", feature = "stm32f407"),
    all(feature = "stm32f429", feature = "stm32f410"),
    all(feature = "stm32f429", feature = "stm32f411"),
    all(feature = "stm32f429", feature = "stm32f412"),
    all(feature = "stm32f429", feature = "stm32f413"),
    all(feature = "stm32f429", feature = "stm32f415"),
    all(feature = "stm32f429", feature = "stm32f417"),
    all(feature = "stm32f429", feature = "stm32f423"),
    all(feature = "stm32f429", feature = "stm32f427"),
    all(feature = "stm32f429", feature = "stm32f437"),
    all(feature = "stm32f429", feature = "stm32f439"),
    all(feature = "stm32f429", feature = "stm32f446"),
    all(feature = "stm32f429", feature = "stm32f469"),
    all(feature = "stm32f429", feature = "stm32f479"),
    all(feature = "stm32f437", feature = "stm32f401"),
    all(feature = "stm32f437", feature = "stm32f405"),
    all(feature = "stm32f437", feature = "stm32f407"),
    all(feature = "stm32f437", feature = "stm32f410"),
    all(feature = "stm32f437", feature = "stm32f411"),
    all(feature = "stm32f437", feature = "stm32f412"),
    all(feature = "stm32f437", feature = "stm32f413"),
    all(feature = "stm32f437", feature = "stm32f415"),
    all(feature = "stm32f437", feature = "stm32f417"),
    all(feature = "stm32f437", feature = "stm32f423"),
    all(feature = "stm32f437", feature = "stm32f427"),
    all(feature = "stm32f437", feature = "stm32f429"),
    all(feature = "stm32f437", feature = "stm32f439"),
    all(feature = "stm32f437", feature = "stm32f446"),
    all(feature = "stm32f437", feature = "stm32f469"),
    all(feature = "stm32f437", feature = "stm32f479"),
    all(feature = "stm32f439", feature = "stm32f401"),
    all(feature = "stm32f439", feature = "stm32f405"),
    all(feature = "stm32f439", feature = "stm32f407"),
    all(feature = "stm32f439", feature = "stm32f410"),
    all(feature = "stm32f439", feature = "stm32f411"),
    all(feature = "stm32f439", feature = "stm32f412"),
    all(feature = "stm32f439", feature = "stm32f413"),
    all(feature = "stm32f439", feature = "stm32f415"),
    all(feature = "stm32f439", feature = "stm32f417"),
    all(feature = "stm32f439", feature = "stm32f423"),
    all(feature = "stm32f439", feature = "stm32f427"),
    all(feature = "stm32f439", feature = "stm32f429"),
    all(feature = "stm32f439", feature = "stm32f437"),
    all(feature = "stm32f439", feature = "stm32f446"),
    all(feature = "stm32f439", feature = "stm32f469"),
    all(feature = "stm32f439", feature = "stm32f479"),
    all(feature = "stm32f446", feature = "stm32f401"),
    all(feature = "stm32f446", feature = "stm32f405"),
    all(feature = "stm32f446", feature = "stm32f407"),
    all(feature = "stm32f446", feature = "stm32f410"),
    all(feature = "stm32f446", feature = "stm32f411"),
    all(feature = "stm32f446", feature = "stm32f412"),
    all(feature = "stm32f446", feature = "stm32f413"),
    all(feature = "stm32f446", feature = "stm32f415"),
    all(feature = "stm32f446", feature = "stm32f417"),
    all(feature = "stm32f446", feature = "stm32f423"),
    all(feature = "stm32f446", feature = "stm32f427"),
    all(feature = "stm32f446", feature = "stm32f429"),
    all(feature = "stm32f446", feature = "stm32f437"),
    all(feature = "stm32f446", feature = "stm32f439"),
    all(feature = "stm32f446", feature = "stm32f469"),
    all(feature = "stm32f446", feature = "stm32f479"),
    all(feature = "stm32f469", feature = "stm32f401"),
    all(feature = "stm32f469", feature = "stm32f405"),
    all(feature = "stm32f469", feature = "stm32f407"),
    all(feature = "stm32f469", feature = "stm32f410"),
    all(feature = "stm32f469", feature = "stm32f411"),
    all(feature = "stm32f469", feature = "stm32f412"),
    all(feature = "stm32f469", feature = "stm32f413"),
    all(feature = "stm32f469", feature = "stm32f415"),
    all(feature = "stm32f469", feature = "stm32f417"),
    all(feature = "stm32f469", feature = "stm32f423"),
    all(feature = "stm32f469", feature = "stm32f427"),
    all(feature = "stm32f469", feature = "stm32f429"),
    all(feature = "stm32f469", feature = "stm32f437"),
    all(feature = "stm32f469", feature = "stm32f439"),
    all(feature = "stm32f469", feature = "stm32f446"),
    all(feature = "stm32f469", feature = "stm32f479"),
    all(feature = "stm32f479", feature = "stm32f401"),
    all(feature = "stm32f479", feature = "stm32f405"),
    all(feature = "stm32f479", feature = "stm32f407"),
    all(feature = "stm32f479", feature = "stm32f410"),
    all(feature = "stm32f479", feature = "stm32f411"),
    all(feature = "stm32f479", feature = "stm32f412"),
    all(feature = "stm32f479", feature = "stm32f413"),
    all(feature = "stm32f479", feature = "stm32f415"),
    all(feature = "stm32f479", feature = "stm32f417"),
    all(feature = "stm32f479", feature = "stm32f423"),
    all(feature = "stm32f479", feature = "stm32f427"),
    all(feature = "stm32f479", feature = "stm32f429"),
    all(feature = "stm32f479", feature = "stm32f437"),
    all(feature = "stm32f479", feature = "stm32f439"),
    all(feature = "stm32f479", feature = "stm32f446"),
    all(feature = "stm32f479", feature = "stm32f469"),
))]
compile_error!(
    "Multile chip features activated. You must activate exactly one of the following features: "
);

pub use stm32f4xx_hal as hal;
pub use stm32f4xx_hal::stm32 as pac;

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(not(any(feature = "stm32f401", feature = "stm32f410", feature = "stm32f411",)))]
pub mod can;
pub mod exti;
pub mod i2c;
pub mod interrupt;
#[cfg(not(feature = "stm32f410"))]
pub mod qei;
pub mod rtc;
pub mod serial;
