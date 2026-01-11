#![doc = include_str!("../README.md")]
#![no_std]
pub mod dma;
pub mod gpio;
pub mod interrupts;
pub mod sysinfo;
#[cfg(feature = "time-driver")]
mod time_driver;
pub mod uart;

// Peripherals and interrupts supported by the NEORV32 chip
mod chip {
    #[rustfmt::skip]
    embassy_hal_internal::peripherals!(
        UART0, UART1,
        GPIO,
        PORT0, PORT1, PORT2, PORT3, PORT4, PORT5, PORT6, PORT7,
        PORT8, PORT9, PORT10, PORT11, PORT12, PORT13, PORT14, PORT15,
        PORT16, PORT17, PORT18, PORT19, PORT20, PORT21, PORT22, PORT23,
        PORT24, PORT25, PORT26, PORT27, PORT28, PORT29, PORT30, PORT31,
        DMA,
    );
    pub mod interrupts {
        crate::interrupt_mod!(UART0, UART1, GPIO, DMA);
    }
}

pub use chip::interrupts::*;
pub use chip::{Peripherals, peripherals};
pub use neorv32_pac as pac;

/// Initialize the HAL. This must only be called from hart 0.
///
/// # Panics
///
/// Panics if this has already been called once before or not called from hart 0.
///
/// Panics if `time-driver` feature is enabled but `CLINT` is not supported.
pub fn init() -> Peripherals {
    // Attempt to take first so we panic before doing anything else
    let p = Peripherals::take();

    // SAFETY: We're not worried about breaking any critical sections here
    unsafe { riscv::interrupt::enable() }

    #[cfg(feature = "time-driver")]
    time_driver::init();

    p
}
