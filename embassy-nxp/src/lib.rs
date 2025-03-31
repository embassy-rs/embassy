#![no_std]

pub mod gpio;
mod pac_utils;
pub mod pint;

pub use embassy_hal_internal::Peri;
pub use lpc55_pac as pac;

/// Initialize the `embassy-nxp` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once and at startup, otherwise it panics.
pub fn init(_config: config::Config) -> Peripherals {
    gpio::init();
    pint::init();

    crate::Peripherals::take()
}

embassy_hal_internal::peripherals! {
    // External pins. These are not only GPIOs, they are multi-purpose pins and can be used by other
    // peripheral types (e.g. I2C).
    PIO0_0,
    PIO0_1,
    PIO0_2,
    PIO0_3,
    PIO0_4,
    PIO0_5,
    PIO0_6,
    PIO0_7,
    PIO0_8,
    PIO0_9,
    PIO0_10,
    PIO0_11,
    PIO0_12,
    PIO0_13,
    PIO0_14,
    PIO0_15,
    PIO0_16,
    PIO0_17,
    PIO0_18,
    PIO0_19,
    PIO0_20,
    PIO0_21,
    PIO0_22,
    PIO0_23,
    PIO0_24,
    PIO0_25,
    PIO0_26,
    PIO0_27,
    PIO0_28,
    PIO0_29,
    PIO0_30,
    PIO0_31,
    PIO1_0,
    PIO1_1,
    PIO1_2,
    PIO1_3,
    PIO1_4,
    PIO1_5,
    PIO1_6,
    PIO1_7,
    PIO1_8,
    PIO1_9,
    PIO1_10,
    PIO1_11,
    PIO1_12,
    PIO1_13,
    PIO1_14,
    PIO1_15,
    PIO1_16,
    PIO1_17,
    PIO1_18,
    PIO1_19,
    PIO1_20,
    PIO1_21,
    PIO1_22,
    PIO1_23,
    PIO1_24,
    PIO1_25,
    PIO1_26,
    PIO1_27,
    PIO1_28,
    PIO1_29,
    PIO1_30,
    PIO1_31,
}

/// HAL configuration for the NXP board.
pub mod config {
    #[derive(Default)]
    pub struct Config {}
}
