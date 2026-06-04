#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(async_fn_in_trait)]

//! Embassy HAL for Silicon Labs EFR32 series microcontrollers.

// This must go FIRST so that all the other modules see its macros.
mod fmt;

pub mod eusart;
pub mod gpio;

pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    macro_rules! impl_mode {
        ($name:ident) => {
            impl SealedMode for $name {}
            impl Mode for $name {}
        };
    }

    /// Blocking mode.
    pub struct Blocking;
    /// Async mode.
    pub struct Async;

    impl_mode!(Blocking);
    impl_mode!(Async);
}

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
///     EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
///     EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
/// });
/// ```
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$outer:meta])* $vis:vis struct $name:ident {
        $(
            $(#[doc = $doc:literal])*
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$outer])*
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            $(#[cfg($cond_irq)])?
            $(#[doc = $doc])*
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
