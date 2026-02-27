#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

#[cfg(not(any(
    feature = "mec1721n_b0_lj",
    feature = "mec1721n_b0_sz",
    feature = "mec1723n_b0_lj",
    feature = "mec1723n_b0_sz",
    feature = "mec1723n_f0_sz",
    feature = "mec1723n_p0_9y",
    feature = "mec1724n_b0_lj",
    feature = "mec1724n_b0_sz",
    feature = "mec1725n_b0_lj",
    feature = "mec1727n_b0_sz",
)))]
compile_error!(
    "No chip feature activated. You must activate exactcly one of the following features:
    mec1721n_b0_lj,
    mec1721n_b0_sz,
    mec1723n_b0_lj,
    mec1723n_b0_sz,
    mec1723n_f0_sz,
    mec1723n_p0_9y,
    mec1724n_b0_lj,
    mec1724n_b0_sz,
    mec1725n_b0_lj,
    mec1727n_b0_sz,
    "
);

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod gpio;
pub mod i2c;
pub mod pwm;
pub mod tach;
pub mod time_driver;
pub mod uart;

// Reexports
pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use mec17xx_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use mec17xx_pac as pac;

#[cfg(feature = "rt")]
pub use crate::pac::NVIC_PRIO_BITS;

embassy_hal_internal::interrupt_mod!(
    ADC_RPT,
    ADC_SNGL,
    AEC0_IBF,
    AEC0_OBE,
    AEC1_IBF,
    AEC1_OBE,
    AEC2_IBF,
    AEC2_OBE,
    AEC3_IBF,
    AEC3_OBE,
    AEC4_IBF,
    AEC4_OBE,
    APM1_CTL,
    APM1_EN,
    APM1_STS,
    ASIF,
    BCM_BUSY_CLR_0,
    BCM_ERR_0,
    CCT,
    CCT_CAP0,
    CCT_CAP1,
    CCT_CAP2,
    CCT_CAP3,
    CCT_CAP4,
    CCT_CAP5,
    CCT_CMP0,
    CCT_CMP1,
    CNTR_TMR0,
    CNTR_TMR1,
    CNTR_TMR2,
    CNTR_TMR3,
    DMA_CH00,
    DMA_CH01,
    DMA_CH02,
    DMA_CH03,
    DMA_CH04,
    DMA_CH05,
    DMA_CH06,
    DMA_CH07,
    DMA_CH08,
    DMA_CH09,
    DMA_CH10,
    DMA_CH11,
    DMA_CH12,
    DMA_CH13,
    DMA_CH14,
    DMA_CH15,
    EMI0,
    EMI1,
    EMI2,
    ESPI_RESET,
    ESPI_VWIRE,
    GIRQ08,
    GIRQ09,
    GIRQ10,
    GIRQ11,
    GIRQ12,
    GIRQ13,
    GIRQ14,
    GIRQ15,
    GIRQ17,
    GIRQ18,
    GIRQ19,
    GIRQ20,
    GIRQ21,
    GIRQ23,
    GIRQ24,
    GIRQ25,
    GIRQ26,
    HTMR0,
    HTMR1,
    I2CSMB0,
    I2CSMB1,
    I2CSMB2,
    I2CSMB3,
    I2CSMB4,
    INTR_BM1,
    INTR_BM2,
    INTR_FLASH,
    INTR_LTR,
    INTR_OOB_DOWN,
    INTR_OOB_UP,
    INTR_PC,
    KBC_IBF,
    KBC_OBE,
    KEYSCAN,
    LED0,
    LED1,
    LED2,
    LED3,
    MBOX,
    P80CAP0,
    PECI,
    PHOT,
    POWERGUARD_0,
    POWERGUARD_1,
    PS2_0A_WAKE,
    PS2_0B_WAKE,
    PS2_0_ACT,
    QMSPI,
    RC_ID0,
    RC_ID1,
    RC_ID2,
    RPM2PWM_0_SPIN,
    RPM2PWM_0_STALL,
    RPM2PWM_1_SPIN,
    RPM2PWM_1_STALL,
    RTC,
    RTC_ALARM,
    RTMR,
    RX0,
    RX1,
    SAF_DONE,
    SAF_ERR,
    SPISLV,
    SYSPWR,
    TACH0,
    TACH1,
    TACH2,
    TACH3,
    TIMER16_0,
    TIMER16_1,
    TIMER16_2,
    TIMER16_3,
    TIMER32_0,
    TIMER32_1,
    TX0,
    TX1,
    UART0,
    UART1,
    VCI_IN0,
    VCI_IN1,
    VCI_IN2,
    VCI_IN3,
    VCI_OVRD_IN,
    WDT,
    WK,
    WKSEC,
    WKSUB,
    WKSUBSEC,
);

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_microchip::{bind_interrupts, usb, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     USBCTRL_IRQ => usb::InterruptHandler<peripherals::USB>;
/// });
/// ```
///
// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                $(
                    $(#[cfg($cond_handler)])?
                    <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();

                )*
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

embassy_hal_internal::peripherals! {
    // Port 0
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    GPIO5,
    GPIO6,
    GPIO7,

    // Port 1
    GPIO10,
    GPIO11,
    GPIO12,
    GPIO13,
    GPIO14,
    GPIO15,
    GPIO16,
    GPIO17,

    // Port 2
    GPIO20,
    GPIO21,
    GPIO22,
    GPIO23,
    GPIO24,
    GPIO25,
    GPIO26,
    GPIO27,

    // Port 3
    GPIO30,
    GPIO31,
    GPIO32,
    GPIO33,
    GPIO34,
    GPIO35,
    GPIO36,

    // Port 4
    GPIO40,
    GPIO41,
    GPIO42,
    GPIO43,
    GPIO44,
    GPIO45,
    GPIO46,
    GPIO47,

    // Port 5
    GPIO50,
    GPIO51,
    GPIO52,
    GPIO53,
    GPIO54,
    GPIO55,
    GPIO56,
    GPIO57,

    // Port 6
    GPIO60,
    GPIO61,
    GPIO62,
    GPIO63,
    GPIO64,
    GPIO65,
    GPIO66,
    GPIO67,

    // Port 7
    GPIO70,
    GPIO71,
    GPIO72,
    GPIO73,
    GPIO74,
    GPIO75,
    GPIO76,

    // Ports 8 and 9 don't exist

    // Port 10
    GPIO100,
    GPIO101,
    GPIO102,
    GPIO103,
    GPIO104,
    GPIO105,
    GPIO106,
    GPIO107,

    // Port 11
    GPIO112,
    GPIO113,
    GPIO114,
    GPIO115,
    GPIO116,
    GPIO117,

    // Port 12
    GPIO120,
    GPIO121,
    GPIO122,
    GPIO123,
    GPIO124,
    GPIO125,
    GPIO126,
    GPIO127,

    // Port 13
    GPIO130,
    GPIO131,
    GPIO132,
    GPIO133,
    GPIO134,
    GPIO135,

    // Port 14
    GPIO140,
    GPIO141,
    GPIO142,
    GPIO143,
    GPIO144,
    GPIO145,
    GPIO146,
    GPIO147,

    // Port 15
    GPIO150,
    GPIO151,
    GPIO152,
    GPIO153,
    GPIO154,
    GPIO155,
    GPIO156,
    GPIO157,

    // Port 16
    GPIO160,
    GPIO161,
    GPIO162,
    GPIO165,
    GPIO166,
    GPIO167,

    // Port 17
    GPIO170,
    GPIO171,
    GPIO172,
    GPIO173,
    GPIO174,
    GPIO175,

    // Ports 18 and 19 don't exist

    // Port 20
    GPIO200,
    GPIO201,
    GPIO202,
    GPIO203,
    GPIO204,
    GPIO205,
    GPIO206,
    GPIO207,

    // Port 21
    GPIO210,
    GPIO211,
    GPIO212,
    GPIO213,
    GPIO214,
    GPIO215,
    GPIO216,
    GPIO217,

    // Port 22
    GPIO221,
    GPIO222,
    GPIO223,
    GPIO224,
    GPIO225,
    GPIO226,
    GPIO227,

    // Port 23
    GPIO230,
    GPIO231,

    // Port 24
    GPIO240,
    GPIO241,
    GPIO242,
    GPIO243,
    GPIO244,
    GPIO245,
    GPIO246,
    GPIO247,

    // Port 25
    // GPIO253, no interrupt
    GPIO254,
    GPIO255,
    // GPIO256, no interrupt
    // GPIO257, no interrupt

    // Port 26
    // GPIO260, no interrupt

    ESPI,

    PWM0,
    PWM1,
    PWM2,
    PWM3,
    PWM4,
    PWM5,
    PWM6,
    PWM7,
    PWM8,
    PWM9,
    PWM10,
    PWM11,

    SMB0,
    SMB1,
    SMB2,
    SMB3,
    SMB4,
    TACH0,
    TACH1,
    TACH2,
    TACH3,
    UART0,
    UART1,
}

/// HAL configuration for Microchip.
pub mod config {
    /// HAL configuration passed when initializing.
    #[non_exhaustive]
    pub struct Config {}

    impl Default for Config {
        fn default() -> Self {
            Self {}
        }
    }

    impl Config {
        /// Create a new configuration
        pub fn new() -> Self {
            Self {}
        }
    }
}

/// Initialize the `embassy-microchip` HAL with the provided configuration.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panis.
pub fn init(_config: config::Config) -> Peripherals {
    let peripherals = Peripherals::take();

    unsafe {
        // ROM code leaves interrupts globally disabled, d'oh.
        cortex_m::interrupt::enable();
        gpio::init();
    }

    time_driver::init();

    peripherals
}
