#![no_std]

#[cfg(not(any(feature = "lpc55", feature = "mimxrt633s", feature = "mimxrt685s",)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features:
	lpc55,
	mimxrt633s,
	mimxrt685s
"
);

#[cfg(feature = "_gpio")]
pub mod gpio;

#[cfg(feature = "_pint")]
pub mod pint;

#[cfg(feature = "lpc55")]
mod pac_utils;

// This mod MUST go last, so that it sees all the `impl_foo!` macros.
#[cfg_attr(feature = "lpc55", path = "chips/lpc55.rs")]
#[cfg_attr(feature = "mimxrt633s", path = "chips/mimxrt633s.rs")]
#[cfg_attr(feature = "mimxrt685s", path = "chips/mimxrt685s.rs")]
mod chip;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right \[`Binding`\]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_imxrt::{bind_interrupts, flexspi, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     FLEXSPI_IRQ => flexspi::InterruptHandler<peripherals::FLEXSPI>;
/// });
/// ```
///
// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident { $($irq:ident => $($handler:ty),*;)* }) => {
            #[derive(Copy, Clone)]
            $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();
                )*
            }

            $(
                unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
            )*
        )*
    };
}

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub use crate::chip::interrupt;
#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

/// Initialize the `embassy-nxp` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(_config: config::Config) -> Peripherals {
    #[cfg(feature = "_gpio")]
    gpio::init();

    #[cfg(feature = "_pint")]
    pint::init();

    crate::Peripherals::take()
}

/// HAL configuration for the NXP board.
pub mod config {
    #[derive(Default)]
    pub struct Config {}
}
