#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]

//! Embassy support for ASR microcontrollers.
//!
//! This crate provides the shared ownership, interrupt, and initialization
//! infrastructure used by ASR peripheral drivers.

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

#[cfg(feature = "asr6601")]
pub mod adc;
#[cfg(feature = "asr6601")]
pub mod afec;
#[cfg(feature = "asr6601")]
pub mod bstimer;
#[cfg(feature = "asr6601")]
pub mod crc;
#[cfg(feature = "asr6601")]
pub mod dac;
#[cfg(feature = "asr6601")]
pub mod dma;
#[cfg(feature = "asr6601")]
pub mod flash;
#[cfg(feature = "asr6601")]
pub mod gpio;
#[cfg(feature = "asr6601")]
pub mod i2c;
#[cfg(feature = "asr6601")]
pub mod iwdg;
#[cfg(feature = "asr6601")]
pub mod lptimer;
#[cfg(feature = "asr6601")]
pub mod lpuart;
#[cfg(feature = "asr6601")]
pub mod pwr;
#[cfg(feature = "asr6601")]
pub mod rcc;
#[cfg(feature = "asr6601")]
pub mod rng;
#[cfg(feature = "asr6601")]
pub mod sec;
#[cfg(feature = "asr6601")]
pub mod spi;
#[cfg(feature = "asr6601")]
pub mod syscfg;
#[cfg(feature = "asr6601")]
pub mod timer;
#[cfg(feature = "asr6601")]
pub mod uart;
#[cfg(feature = "asr6601")]
pub mod wdg;

pub use embassy_hal_internal::{Peri, PeripheralType};

/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    /// Blocking mode.
    #[derive(Clone, Copy, Debug)]
    pub struct Blocking;

    /// Asynchronous mode.
    #[derive(Clone, Copy, Debug)]
    pub struct Async;

    impl SealedMode for Blocking {}
    impl Mode for Blocking {}
    impl SealedMode for Async {}
    impl Mode for Async {}
}

#[cfg(feature = "asr6601")]
embassy_hal_internal::interrupt_mod!(
    SEC, RTC, WDG, EFC, UART3, I2C2, UART0, UART1, UART2, LPUART, SSP0, SSP1, QSPI, I2C0, I2C1, SCC, ADC, AFEC, SSP2,
    DMA1, DAC, LORA, GPIO, TIMER0, TIMER1, TIMER2, TIMER3, BSTIMER0, BSTIMER1, LPTIMER0, SAC, DMA0, I2S, LCD, PWR,
    LPTIMER1, IWDG,
);

/// Bind interrupt vectors to HAL interrupt handlers.
///
/// The generated unit struct proves at compile time that each listed
/// interrupt is bound to the requested handler or handlers.
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
                        <$handler as $crate::interrupt::typelevel::Handler<
                            $crate::interrupt::typelevel::$irq
                        >>::on_interrupt();
                    )*
                }
            }

            $(#[cfg($cond_irq)])?
            $crate::bind_interrupts!(@inner
                $(
                    $(#[cfg($cond_handler)])?
                    unsafe impl $crate::interrupt::typelevel::Binding<
                        $crate::interrupt::typelevel::$irq,
                        $handler
                    > for $name {}
                )*
            );
        )*
    };
    (@inner $($t:tt)*) => {
        $($t)*
    }
}

embassy_hal_internal::peripherals! {
    // GPIO port A
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7,
    PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15,

    // GPIO port B
    PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7,
    PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15,

    // GPIO port C
    PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7,
    PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15,

    // GPIO port D
    PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7,
    PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15,

    // GPIO hardware blocks
    GPIOA, GPIOB, GPIOC, GPIOD,

    // DMA hardware blocks and channels
    DMA0, DMA1,
    DMA0_CH0, DMA0_CH1, DMA0_CH2, DMA0_CH3,
    DMA1_CH0, DMA1_CH1, DMA1_CH2, DMA1_CH3,

    // Clock, system, and power control
    RCC,
    SYSCFG,
    AFEC,
    PWR,

    // Communications
    UART0,
    UART1,
    UART2,
    UART3,
    LPUART,
    SSP0,
    SSP1,
    SSP2,
    QSPI,
    I2C0,
    I2C1,
    I2C2,
    I2S,

    // Timers
    TIMER0,
    TIMER1,
    TIMER2,
    TIMER3,
    BSTIMER0,
    BSTIMER1,
    LPTIMER0,
    LPTIMER1,
    #[cfg(not(feature = "time-driver-rtc"))]
    RTC,

    // Analog and display
    ADC,
    DAC,
    LCD,

    // Storage and data processing
    EFC,
    CRC,
    RNG,
    SAE,
    SEC,

    // Radio and supervision
    LORAC,
    IWDG,
    WDG,
}

/// Global HAL configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Oscillator and core/bus clock configuration.
    pub rcc: rcc::Config,
}

impl Config {
    /// Create the default HAL configuration.
    pub const fn new() -> Self {
        Self {
            rcc: rcc::Config::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the ASR HAL.
///
/// With `time-driver-rtc` enabled, this resets and uses RTC exclusively for
/// Embassy time. Applications must not access RTC through the raw PAC.
pub fn init(config: Config) -> Peripherals {
    let peripherals = Peripherals::take();

    #[cfg(feature = "asr6601")]
    {
        // Consume RCC into the shared clock tree. Peripheral drivers continue
        // to use crate-private RCC helpers for clock/reset gates.
        rcc::init(unsafe { peripherals.RCC.clone_unchecked() }, config.rcc)
            .expect("embassy-asr: RCC initialization failed");
        unsafe {
            dma::init();
        }
    }

    #[cfg(feature = "time-driver-rtc")]
    crate::time_driver::init();

    unsafe {
        // The thread-mode Cortex-M executor uses WFE and must not inherit deep
        // sleep state from a bootloader.
        cortex_m::peripheral::Peripherals::steal().SCB.clear_sleepdeep();
        cortex_m::interrupt::enable();
    }

    peripherals
}
