#![no_std]

//! Minimal Embassy support for ASR microcontrollers.
//!
//! This crate currently provides only initialization and an `embassy-time`
//! driver. Peripheral HAL drivers are intentionally out of scope.

#[cfg(feature = "asr6601")]
pub mod pac {
    pub use asr6601_pac::*;
    pub use cortex_m_rt::interrupt;

    pub mod interrupt {
        pub use asr6601_pac::Interrupt;
        pub use asr6601_pac::Interrupt::*;
    }
}

#[cfg(feature = "time-driver-rtc")]
mod time_driver;

// There are no HAL peripheral singletons yet. Keeping the standard Embassy
// `Peripherals` return value makes `init` forward-compatible with adding them.
embassy_hal_internal::peripherals! {}

/// Initialize minimal ASR chip support.
///
/// With `time-driver-rtc` enabled, this resets and uses RTC exclusively for
/// Embassy time. Applications must not access RTC through the raw PAC.
pub fn init() -> Peripherals {
    let peripherals = Peripherals::take();

    #[cfg(feature = "time-driver-rtc")]
    crate::time_driver::init();

    unsafe {
        // The thread-mode Cortex-M executor uses WFE and must not inherit deep
        // sleep state from a bootloader.
        cortex_m::peripheral::Peripherals::steal()
            .SCB
            .clear_sleepdeep();
        cortex_m::interrupt::enable();
    }

    peripherals
}
