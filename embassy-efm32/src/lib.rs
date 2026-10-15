#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![cfg_attr(
    docsrs,
    doc = "<div style='padding:30px;background:#810;color:#fff;text-align:center;'><p>You might want to <a href='https://docs.embassy.dev/embassy-efm32'>browse the `embassy-efm32` documentation on the Embassy website</a> instead.</p><p>The documentation here on `docs.rs` is built for a single chip only (EFM32GG990 in particular), while on the Embassy website you can pick your exact chip from the top menu. Available peripherals and their APIs change depending on the chip.</p></div>\n\n"
)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod cmu;
pub mod gpio;
pub mod mode;
pub mod time;
pub mod usart;

#[cfg(feature = "_time-driver")]
mod time_driver;

// The time driver's RTC interrupt handler is only defined with `rt`. Host unit tests (`_test`)
// don't need it.
#[cfg(all(feature = "_time-driver", not(feature = "rt"), not(feature = "_test")))]
compile_error!("The `time-driver-rtc` feature requires the `rt` feature.");

// This mod MUST go last, so that it sees all the `impl_*!` macros.
#[cfg_attr(feature = "_efm32gg", path = "chips/efm32gg.rs")]
mod chip;

pub use chip::{Peripherals, interrupt, peripherals};
pub use embassy_hal_internal::{Peri, PeripheralType};

#[cfg(feature = "unstable-pac")]
pub use crate::chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use crate::chip::pac;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [crate::interrupt::typelevel::Binding]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// # Example
///
/// ```rust,ignore
/// use embassy_efm32::{bind_interrupts, peripherals, usart};
///
/// bind_interrupts!(
///     struct Irqs {
///         USART0_RX => usart::RxInterruptHandler<peripherals::USART0>;
///         USART0_TX => usart::TxInterruptHandler<peripherals::USART0>;
///     }
/// );
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

/// HAL configuration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Clock tree (CMU) configuration.
    pub cmu: cmu::Config,
    /// Priority of the GPIO interrupts (`GPIO_EVEN`, `GPIO_ODD`), used to wait for pin edges.
    pub gpio_interrupt_priority: interrupt::Priority,
    /// Priority of the time driver's interrupt (`RTC`).
    #[cfg(feature = "_time-driver")]
    pub time_interrupt_priority: interrupt::Priority,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cmu: Default::default(),
            gpio_interrupt_priority: interrupt::Priority::P0,
            #[cfg(feature = "_time-driver")]
            time_interrupt_priority: interrupt::Priority::P0,
        }
    }
}

/// Initialize the `embassy-efm32` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(config: Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    cmu::init(config.cmu);
    gpio::init(config.gpio_interrupt_priority);

    #[cfg(feature = "_time-driver")]
    time_driver::init(config.time_interrupt_priority);

    peripherals
}
