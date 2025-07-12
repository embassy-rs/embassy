#![no_std]

pub mod gpio;
#[cfg(feature = "lpc55")]
pub mod pint;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(feature = "lpc55", path = "chips/lpc55.rs")]
mod chip;

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals};
pub use embassy_hal_internal::{Peri, PeripheralType};

/// Initialize the `embassy-nxp` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(_config: config::Config) -> Peripherals {
    #[cfg(feature = "lpc55")]
    {
        gpio::init();
        pint::init();
    }

    crate::Peripherals::take()
}

/// HAL configuration for the NXP board.
pub mod config {
    #[derive(Default)]
    pub struct Config {}
}
