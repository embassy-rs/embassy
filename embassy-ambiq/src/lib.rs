#![no_std]

#[cfg(feature = "apollo3")]
pub use apollo3_pac as pac;

pub mod gpio;
#[cfg(any(feature = "_stimer", feature = "time-driver-ctimer0"))]
pub mod time_driver;

embassy_hal_internal::peripherals! {
    P0, P1, P2, P3, P4, P5, P6, P7, P8, P9,
    P10, P11, P12, P13, P14, P15, P16, P17, P18, P19,
    P20, P21, P22, P23, P24, P25, P26, P27, P28, P29,
    P30, P31, P32, P33, P34, P35, P36, P37, P38, P39,
    P40, P41, P42, P43, P44, P45, P46, P47, P48, P49,
}

pub fn init() -> Peripherals {
    let p = Peripherals::take();
    #[cfg(any(feature = "_stimer", feature = "time-driver-ctimer0"))]
    crate::time_driver::Apollo3TimeDriver::init();

    unsafe {
        cortex_m::interrupt::enable();

        let mut scb = cortex_m::peripheral::Peripherals::steal().SCB;
        scb.clear_sleepdeep();

        #[cfg(feature = "svl-vtor")]
        {
            // The SparkFun SVL bootloader leaves VTOR at 0x0 pointing at its own vector table.
            // Repoint to our app vectors at 0x10000 (matches memory.x FLASH ORIGIN).
            // We do this before setting up any interrupts.
            scb.vtor.write(0x0001_0000);
        }

        // Enable the global GPIO interrupt. Individual pins configure their own routing.
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::GPIO);
    }

    p
}
