#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
// Clippy Exceptions
//
// Allow functions with too many args - we have a lot of HAL constructors like this for now
#![allow(clippy::too_many_arguments)]

/// Module for MCXA2xx-specific HAL drivers
///
/// NOTE: *for now*, some items are here because we haven't validated them on the MCXA5xx yet.
/// This note will be removed when the two reach parity.
#[path = "."]
mod mcxa2xx_exlusive {
    pub mod adc;
    pub mod cdog;
    pub mod clkout;
    pub mod clocks; // still provide clock helpers
    pub mod config;
    pub mod crc;
    pub mod ctimer;
    pub mod dma;
    #[cfg(feature = "custom-executor")]
    pub mod executor;
    pub mod flash;
    pub mod gpio;
    pub mod i2c;
    pub mod i3c;
    pub mod inputmux;
    pub mod lpuart;
    pub mod ostimer;
    pub mod perf_counters;
    pub mod reset_reason;
    pub mod rtc;
    pub mod spi;
    pub mod trng;
    pub mod wwdt;

    pub use crate::chips::mcxa2xx::{Peripherals, init, interrupt, peripherals};
}

/// Module for MCXA5xx-specific HAL drivers
#[path = "."]
mod mcxa5xx_exclusive {}

/// Module for HAL drivers supported by all chips
#[path = "."]
mod all_chips {}

#[allow(unused_imports)]
pub use all_chips::*;
#[cfg(feature = "mcxa2xx")]
pub use mcxa2xx_exlusive::*;
#[cfg(feature = "mcxa5xx")]
pub use mcxa5xx_exlusive::*;

pub(crate) mod chips;

// Re-export interrupt traits and types
#[cfg(feature = "unstable-pac")]
pub use nxp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use nxp_pac as pac;

const HALS_SELECTED: usize = const { cfg!(feature = "mcxa2xx") as usize + cfg!(feature = "mcxa5xx") as usize };

/// Ensure exactly one chip feature is set.
#[doc(hidden)]
pub const _SINGLE_HAL_CHECK: bool = const {
    assert!(HALS_SELECTED == 1, "Select exactly one chip feature!");
    HALS_SELECTED == 1
};

/// Macro to bind interrupts to handlers, similar to embassy-imxrt.
///
/// Example:
/// - Bind OS_EVENT to the OSTIMER time-driver handler
///   bind_interrupts!(struct Irqs { OS_EVENT => crate::ostimer::time_driver::OsEventHandler; });
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$attr:meta])* $vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$attr])*
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                unsafe {
                    $(
                        $(#[cfg($cond_handler)])?
                        <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();
                    )*
                }
            }

            $(#[cfg($cond_irq)])?
            $crate::bind_interrupts!(@inner
                $(
                    $(#[cfg($cond_handler)])?
                    unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
                )*
            );
        )*
    };
    (@inner $($t:tt)*) => {
        $($t)*
    }
}
