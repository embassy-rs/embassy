#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

use embassy::interrupt::{Interrupt, InterruptExt};
//pub(crate) use stm32_metapac as pac;

pub mod pac {
    pub use stm32_metapac::*;

    #[cfg(any(feature = "_syscfg_f4"))]
    pub use stm32_metapac::syscfg_f4 as syscfg;

    #[cfg(any(feature = "_syscfg_l4"))]
    pub use stm32_metapac::syscfg_l4 as syscfg;
}


#[macro_use]
pub mod exti;
#[macro_use]
pub mod gpio;
//pub mod rtc;
#[macro_use]
pub mod usart;

#[macro_use]
pub mod rng;

// This must go LAST so that it sees the `impl_foo!` macros
mod chip;
pub use chip::{interrupt, peripherals, Peripherals};
pub use embassy_macros::interrupt;

#[non_exhaustive]
pub struct Config {
    _private: (),
}

impl Default for Config {
    fn default() -> Self {
        Self { _private: () }
    }
}

/// Initialize embassy.
pub fn init(_config: Config) -> Peripherals {
    let p = Peripherals::take();

    unsafe {
        interrupt::EXTI0::steal().enable();
        interrupt::EXTI1::steal().enable();
        interrupt::EXTI2::steal().enable();
        interrupt::EXTI3::steal().enable();
        interrupt::EXTI4::steal().enable();
        interrupt::EXTI9_5::steal().enable();
        interrupt::EXTI15_10::steal().enable();
    }

    p
}
