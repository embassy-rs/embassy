#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
// Clippy Exceptions
//
// Allow functions with too many args - we have a lot of HAL constructors like this for now
#![allow(clippy::too_many_arguments)]

#[cfg(feature = "mcxn947")]
#[path = "."]
mod mcxn947_exclusive {
    pub use crate::chips::mcxn947::init;
}

pub mod clocks;
pub mod config;
pub mod gpio;

#[cfg(feature = "mcxn947")]
pub use mcxn947_exclusive::*;

pub(crate) mod chips;

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]
    #![allow(clippy::missing_safety_doc)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Re-export Peri and PeripheralType to allow applications to express Peri types and requirements.
pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use nxp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use nxp_pac as pac;

pub use crate::_generated::{Peripherals, interrupt, peripherals};

const HALS_SELECTED: usize = const { cfg!(feature = "mcxn947") as usize };

/// Ensure exactly one chip feature is set.
#[doc(hidden)]
pub const _SINGLE_HAL_CHECK: bool = const {
    assert!(HALS_SELECTED == 1, "Select exactly one chip feature!");
    HALS_SELECTED == 1
};

/// Bind interrupts to their handlers.
///
/// Defines a struct `$name` that implements
/// [`interrupt::typelevel::Binding`] for each `IRQ => Handler` pair, and the
/// interrupt vector that calls each handler. Async driver constructors take
/// that struct as proof the interrupt is wired up.
///
/// ```ignore
/// bind_interrupts!(struct Irqs {
///     SOME_IRQ => some_driver::InterruptHandler<SOME_PERIPHERAL>;
/// });
/// ```
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
                use $crate::interrupt::typelevel::Interrupt;

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
