#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(async_fn_in_trait)]

//! Embassy HAL for Silicon Labs EFR32 series microcontrollers.

// This must go FIRST so that all the other modules see its macros.
mod fmt;

pub mod gpio;
pub mod rcc;
pub mod time;
#[cfg(feature = "_time-driver")]
mod time_driver;

pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use silabs_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use silabs_metapac as pac;

// Peripheral and interrupt singletons are emitted at build time from
// `silabs_metapac::metadata::METADATA`. See `build.rs`.
include!(concat!(env!("OUT_DIR"), "/_generated.rs"));

/// Macro to bind interrupts to handlers.
///
/// ```rust,ignore
/// use embassy_silabs::{bind_interrupts, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     EUSART0_RX => eusart::InterruptHandler<peripherals::EUSART0>;
/// });
/// ```
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$outer:meta])* $vis:vis struct $name:ident {
        $(
            $(#[$irq_meta:meta])*
            $irq:ident => $($handler:ty),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$outer])*
        $vis struct $name;

        $(
            $(#[$irq_meta])*
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::typelevel::Handler<
                        $crate::interrupt::typelevel::$irq,
                    >>::on_interrupt();
                )*
            }

            $(
                $(#[$irq_meta])*
                unsafe impl $crate::interrupt::typelevel::Binding<
                    $crate::interrupt::typelevel::$irq,
                    $handler,
                > for $name {}
            )*
        )*
    };
}

/// `embassy-silabs` global configuration.
pub use rcc::Config;

/// Initialize embassy.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panics.
pub fn init(config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let peripherals = Peripherals::take_with_cs(cs);

        rcc::init_clocks(cs, config);

        #[cfg(feature = "_time-driver")]
        time_driver::init(cs);

        peripherals
    })
}
