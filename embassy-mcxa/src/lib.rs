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
#[cfg(feature = "mcxa2xx")]
#[path = "."]
mod mcxa2xx_exclusive {
    pub mod flash; // TODO: Add dummy driver to metadata

    pub use crate::chips::mcxa2xx::init;
}

/// Module for MCXA5xx-specific HAL drivers
#[cfg(feature = "mcxa5xx")]
#[path = "."]
mod mcxa5xx_exclusive {
    pub use crate::chips::mcxa5xx::init;
}

#[cfg(mcxa_adc)]
pub mod adc;
#[cfg(mcxa_cdog)]
pub mod cdog;
#[cfg(any(mcxa_mrcc5xx, mcxa_mrcc2xx))]
pub mod clkout; // TODO: Add dummy driver to metadata
#[cfg(any(mcxa_mrcc5xx, mcxa_mrcc2xx))]
pub mod clocks;
pub mod config;
#[cfg(mcxa_crc)]
pub mod crc;
#[cfg(mcxa_ctimer)]
pub mod ctimer;
#[cfg(mcxa_dma)]
pub mod dma;
#[cfg(feature = "executor-platform")]
pub mod executor;
#[cfg(mcxa_flexspi)]
pub mod flexspi;
#[cfg(mcxa_gpio)]
pub mod gpio;
#[cfg(mcxa_lpi2c)]
pub mod i2c;
#[cfg(mcxa_i3c)]
pub mod i3c;
#[cfg(mcxa_inputmux)]
pub mod inputmux;
#[cfg(mcxa_lpuart)]
pub mod lpuart;
#[cfg(mcxa_ostimer)]
pub mod ostimer;
pub mod perf_counters;
#[cfg(mcxa_cmc)]
pub mod reset_reason;
#[cfg(mcxa_rtc5xx)]
#[path = "rtc/mcxa5xx.rs"]
pub mod rtc;
#[cfg(mcxa_rtc2xx)]
#[path = "rtc/mcxa2xx.rs"]
pub mod rtc;
#[cfg(mcxa_lpspi)]
pub mod spi;
#[cfg(mcxa_trng)]
pub mod trng;
#[cfg(mcxa_wwdt)]
pub mod wwdt;

#[cfg(feature = "mcxa2xx")]
pub use mcxa2xx_exclusive::*;
#[cfg(feature = "mcxa5xx")]
pub use mcxa5xx_exclusive::*;

pub(crate) mod chips;

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Re-export interrupt traits and types
// Re-export Peri and PeripheralType to allow applications to express Peri types and requirements.
pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use nxp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use nxp_pac as pac;

pub use crate::_generated::{Peripherals, interrupt, peripherals};

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
