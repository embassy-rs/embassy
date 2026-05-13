#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(async_fn_in_trait)]

//! Embassy HAL for Silicon Labs EFR32 series microcontrollers.

// This must go FIRST so that all the other modules see its macros.
mod fmt;

pub mod gpio;
pub mod rcc;
#[cfg(feature = "_time-driver")]
mod time_driver;

pub use embassy_hal_internal::{Peri, PeripheralType};
#[cfg(feature = "unstable-pac")]
pub use silabs_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use silabs_metapac as pac;

embassy_hal_internal::peripherals! {
    TIMER0, TIMER1, TIMER2, TIMER3, TIMER4,
    TIMER5, TIMER6, TIMER7, TIMER8, TIMER9,
    GPIO,

    // Port A
    PA00, PA01, PA02, PA03, PA04, PA05, PA06, PA07,
    PA08, PA09, PA10, PA11, PA12, PA13, PA14, PA15,
    // Port B
    PB00, PB01, PB02, PB03, PB04, PB05, PB06, PB07,
    PB08, PB09, PB10, PB11, PB12, PB13, PB14, PB15,
    // Port C
    PC00, PC01, PC02, PC03, PC04, PC05, PC06, PC07,
    PC08, PC09, PC10, PC11, PC12, PC13, PC14, PC15,
    // Port D
    PD00, PD01, PD02, PD03, PD04, PD05, PD06, PD07,
    PD08, PD09, PD10, PD11, PD12, PD13, PD14, PD15,
}

embassy_hal_internal::interrupt_mod!(
    SMU_SECURE,
    SMU_S_PRIVILEGED,
    EMU,
    TIMER0,
    TIMER1,
    TIMER2,
    TIMER3,
    TIMER4,
    TIMER5,
    TIMER6,
    TIMER7,
    TIMER8,
    TIMER9,
    USART0_RX,
    USART0_TX,
    USART1_RX,
    USART1_TX,
    USART2_RX,
    USART2_TX,
    EUSART0_RX,
    EUSART0_TX,
    EUSART1_RX,
    EUSART1_TX,
    EUSART2_RX,
    EUSART2_TX,
    EUSART3_RX,
    EUSART3_TX,
    ICACHE0,
    BURTC,
    LETIMER0,
    SYSCFG,
    LDMA,
    LFXO,
    LFRCO,
    ULFRCO,
    GPIO_ODD,
    GPIO_EVEN,
    I2C0,
    I2C1,
    I2C2,
    I2C3,
    EMUDG,
    HOSTMAILBOX,
    ACMP0,
    ACMP1,
    WDOG0,
    WDOG1,
    HFXO0,
    HFRCO0,
    HFRCOEM23,
    CMU,
    AES,
    IADC,
    MSC,
    DPLL0,
    DCDC,
    PCNT0,
    SW0,
    SW1,
    SW2,
    SW3,
    SEMBRX,
    SEMBTX,
    SYSRTC_APP,
    SYSRTC_SEQ,
    KEYSCAN,
    VDAC0,
    VDAC1,
    LCD,
);

/// Macro to bind interrupts to handlers.
///
/// ```rust,ignore
/// use embassy_silabs::{bind_interrupts, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     EUSART0_RX => eusart::InterruptHandler<peripherals::EUSART0>;
/// });
/// ```
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$outer:meta])* $vis:vis struct $name:ident {
        $(
            $(#[$irq_meta:meta])*
            $irq:ident => $($handler:ty),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$outer])*
        $vis struct $name;

        $(
            $(#[$irq_meta])*
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $irq() {
                $(
                    <$handler as $crate::interrupt::typelevel::Handler<
                        $crate::interrupt::typelevel::$irq,
                    >>::on_interrupt();
                )*
            }

            $(
                $(#[$irq_meta])*
                unsafe impl $crate::interrupt::typelevel::Binding<
                    $crate::interrupt::typelevel::$irq,
                    $handler,
                > for $name {}
            )*
        )*
    };
}

/// `embassy-silabs` global configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Config {}

/// Initialize embassy.
///
/// This returns the peripheral singletons that can be used for creating drivers.
///
/// This should only be called once at startup, otherwise it panics.
pub fn init(_config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let peripherals = Peripherals::take_with_cs(cs);

        rcc::init_clocks(cs);

        #[cfg(feature = "_time-driver")]
        time_driver::init(cs);

        peripherals
    })
}
