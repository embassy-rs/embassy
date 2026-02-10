#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]

// //! ## Feature flags
// #![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

pub mod adc;
pub mod cdog;
pub mod clkout;
pub mod clocks; // still provide clock helpers
pub mod config;
pub mod crc;
pub mod ctimer;
pub mod dma;
#[cfg(feature = "custom-executor")]
pub mod executor;
pub mod gpio;
pub mod i2c;
pub mod i3c;
pub mod inputmux;
pub mod lpuart;
pub mod ostimer;
pub mod perf_counters;
pub mod reset_reason;
pub mod rtc;
pub mod spi;
pub mod trng;
pub mod wwdt;

use crate::interrupt::InterruptExt;
pub use crate::pac::NVIC_PRIO_BITS;

#[rustfmt::skip]
embassy_hal_internal::peripherals!(
    ADC0,
    ADC1,
    ADC2,
    ADC3,

    AOI0,
    AOI1,

    CAN0,
    CAN1,

    CDOG0,
    CDOG1,

    // CLKOUT is not specifically a peripheral (it's part of SYSCON),
    // but we still want it to be a singleton.
    CLKOUT,

    CMC,
    CMP0,
    CMP1,
    CRC0,

    CTIMER0,

    CTIMER0_CH0,
    CTIMER0_CH1,
    CTIMER0_CH2,
    CTIMER0_CH3,

    CTIMER1,

    CTIMER1_CH0,
    CTIMER1_CH1,
    CTIMER1_CH2,
    CTIMER1_CH3,

    CTIMER2,

    CTIMER2_CH0,
    CTIMER2_CH1,
    CTIMER2_CH2,
    CTIMER2_CH3,

    CTIMER3,

    CTIMER3_CH0,
    CTIMER3_CH1,
    CTIMER3_CH2,
    CTIMER3_CH3,

    CTIMER4,

    CTIMER4_CH0,
    CTIMER4_CH1,
    CTIMER4_CH2,
    CTIMER4_CH3,

    DBGMAILBOX,
    DMA0,
    DMA_CH0,
    DMA_CH1,
    DMA_CH2,
    DMA_CH3,
    DMA_CH4,
    DMA_CH5,
    DMA_CH6,
    DMA_CH7,
    EDMA0_TCD0,
    EIM0,
    EQDC0,
    EQDC1,
    ERM0,
    FLEXIO0,
    FLEXPWM0,
    FLEXPWM1,
    FMC0,
    FMU0,
    FREQME0,
    GLIKEY0,

    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,

    I3C0,
    INPUTMUX0,

    LPI2C0,
    LPI2C1,
    LPI2C2,
    LPI2C3,

    LPSPI0,
    LPSPI1,

    LPTMR0,

    LPUART0,
    LPUART1,
    LPUART2,
    LPUART3,
    LPUART4,
    LPUART5,

    MAU0,
    MBC0,
    MRCC0,
    OPAMP0,
    OSTIMER0,

    // Normally SWDIO!
    #[cfg(feature = "swd-as-gpio")]
    P0_0,
    // Normally SWCLK!
    #[cfg(feature = "swd-as-gpio")]
    P0_1,
    // Normally SWO!
    #[cfg(feature = "swd-swo-as-gpio")]
    P0_2,
    // Normally JTAG TDI!
    #[cfg(feature = "jtag-extras-as-gpio")]
    P0_3,
    P0_4,
    P0_5,
    // Normally JTAG ISPMODE_N!
    #[cfg(feature = "jtag-extras-as-gpio")]
    P0_6,
    P0_7,
    P0_8,
    P0_9,
    P0_10,
    P0_11,
    P0_12,
    P0_13,
    P0_14,
    P0_15,
    P0_16,
    P0_17,
    P0_18,
    P0_19,
    P0_20,
    P0_21,
    P0_22,
    P0_23,
    P0_24,
    P0_25,
    P0_26,
    P0_27,
    P0_28,
    P0_29,
    P0_30,
    P0_31,

    P1_0,
    P1_1,
    P1_2,
    P1_3,
    P1_4,
    P1_5,
    P1_6,
    P1_7,
    P1_8,
    P1_9,
    P1_10,
    P1_11,
    P1_12,
    P1_13,
    P1_14,
    P1_15,
    P1_16,
    P1_17,
    P1_18,
    P1_19,
    P1_20,
    P1_21,
    P1_22,
    P1_23,
    P1_24,
    P1_25,
    P1_26,
    P1_27,
    P1_28,
    #[cfg(feature = "dangerous-reset-as-gpio")]
    P1_29,
    #[cfg(feature = "sosc-as-gpio")]
    P1_30,
    #[cfg(feature = "sosc-as-gpio")]
    P1_31,

    P2_0,
    P2_1,
    P2_2,
    P2_3,
    P2_4,
    P2_5,
    P2_6,
    P2_7,
    P2_8,
    P2_9,
    P2_10,
    P2_11,
    P2_12,
    P2_13,
    P2_14,
    P2_15,
    P2_16,
    P2_17,
    P2_18,
    P2_19,
    P2_20,
    P2_21,
    P2_22,
    P2_23,
    P2_24,
    P2_25,
    P2_26,
    P2_27,
    P2_28,
    P2_29,
    P2_30,
    P2_31,

    P3_0,
    P3_1,
    P3_2,
    P3_3,
    P3_4,
    P3_5,
    P3_6,
    P3_7,
    P3_8,
    P3_9,
    P3_10,
    P3_11,
    P3_12,
    P3_13,
    P3_14,
    P3_15,
    P3_16,
    P3_17,
    P3_18,
    P3_19,
    P3_20,
    P3_21,
    P3_22,
    P3_23,
    P3_24,
    P3_25,
    P3_26,
    P3_27,
    P3_28,
    P3_29,
    P3_30,
    P3_31,

    P4_0,
    P4_1,
    P4_2,
    P4_3,
    P4_4,
    P4_5,
    P4_6,
    P4_7,
    P4_8,
    P4_9,
    P4_10,
    P4_11,
    P4_12,
    P4_13,
    P4_14,
    P4_15,
    P4_16,
    P4_17,
    P4_18,
    P4_19,
    P4_20,
    P4_21,
    P4_22,
    P4_23,
    P4_24,
    P4_25,
    P4_26,
    P4_27,
    P4_28,
    P4_29,
    P4_30,
    P4_31,

    P5_0,
    P5_1,
    P5_2,
    P5_3,
    P5_4,
    P5_5,
    P5_6,
    P5_7,
    P5_8,
    P5_9,
    P5_10,
    P5_11,
    P5_12,
    P5_13,
    P5_14,
    P5_15,
    P5_16,
    P5_17,
    P5_18,
    P5_19,
    P5_20,
    P5_21,
    P5_22,
    P5_23,
    P5_24,
    P5_25,
    P5_26,
    P5_27,
    P5_28,
    P5_29,
    P5_30,
    P5_31,

    PKC0,

    PORT0,
    PORT1,
    PORT2,
    PORT3,
    PORT4,

    RTC0,
    SAU,
    SCG0,
    SCN_SCB,
    SGI0,
    SMARTDMA0,
    SPC0,
    SYSCON,
    TDET0,
    TRNG0,
    UDF0,
    USB0,
    UTICK0,
    VBAT0,
    WAKETIMER0,
    WUU0,
    WWDT0,
);

embassy_hal_internal::interrupt_mod!(
    ADC0,
    ADC1,
    ADC2,
    ADC3,
    CAN0,
    CAN1,
    CDOG0,
    CDOG1,
    CMC,
    CMP0,
    CMP1,
    CMP2,
    CTIMER0,
    CTIMER1,
    CTIMER2,
    CTIMER3,
    CTIMER4,
    DAC0,
    DMA_CH0,
    DMA_CH1,
    DMA_CH2,
    DMA_CH3,
    DMA_CH4,
    DMA_CH5,
    DMA_CH6,
    DMA_CH7,
    EQDC0_COMPARE,
    EQDC0_HOME,
    EQDC0_INDEX,
    EQDC0_WATCHDOG,
    EQDC1_COMPARE,
    EQDC1_HOME,
    EQDC1_INDEX,
    EQDC1_WATCHDOG,
    ERM0_MULTI_BIT,
    ERM0_SINGLE_BIT,
    FLEXIO,
    FLEXPWM0_FAULT,
    FLEXPWM0_RELOAD_ERROR,
    FLEXPWM0_SUBMODULE0,
    FLEXPWM0_SUBMODULE1,
    FLEXPWM0_SUBMODULE2,
    FLEXPWM0_SUBMODULE3,
    FLEXPWM1_FAULT,
    FLEXPWM1_RELOAD_ERROR,
    FLEXPWM1_SUBMODULE0,
    FLEXPWM1_SUBMODULE1,
    FLEXPWM1_SUBMODULE2,
    FLEXPWM1_SUBMODULE3,
    FMU0,
    FREQME0,
    GLIKEY0,
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    I3C0,
    LPI2C0,
    LPI2C1,
    LPI2C2,
    LPI2C3,
    LPSPI0,
    LPSPI1,
    LPTMR0,
    LPUART0,
    LPUART1,
    LPUART2,
    LPUART3,
    LPUART4,
    LPUART5,
    MAU,
    MBC0,
    OS_EVENT,
    PKC,
    RTC,
    RTC_1HZ,
    SCG0,
    SGI,
    SLCD,
    SMARTDMA,
    SPC0,
    TDET,
    TRNG0,
    USB0,
    UTICK0,
    WAKETIMER0,
    WUU0,
    WWDT0,
);

// Use cortex-m-rt's #[interrupt] attribute directly; PAC does not re-export it.

// Re-export interrupt traits and types
#[cfg(feature = "unstable-pac")]
pub use nxp_pac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use nxp_pac as pac;

/// Initialize HAL with configuration (mirrors embassy-imxrt style). Minimal: just take peripherals.
/// Also applies configurable NVIC priority for the OSTIMER OS_EVENT interrupt (no enabling).
pub fn init(cfg: crate::config::Config) -> Peripherals {
    // Might not need to be mutable if none of the `...-as-gpio` features are active.
    #[allow(unused_mut)]
    let mut peripherals = Peripherals::take();

    crate::interrupt::RTC.set_priority(cfg.rtc_interrupt_priority);
    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO1.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO2.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO4.set_priority(cfg.gpio_interrupt_priority);

    // Configure clocks
    crate::clocks::init(cfg.clock_cfg).unwrap();

    // Initialize embassy-time global driver backed by OSTIMER0
    // NOTE: As early as possible, but MUST be AFTER clocks!
    crate::ostimer::init(cfg.time_interrupt_priority);

    // Initialize the INPUTMUX peripheral
    crate::inputmux::init();

    unsafe {
        crate::gpio::interrupt_init();
    }

    // Initialize DMA controller (clock, reset, configuration)
    crate::dma::init();

    // Enable GPIO clocks
    unsafe {
        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT0>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO0>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT1>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO1>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT2>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO2>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT3>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO3>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT4>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO4>(&crate::clocks::periph_helpers::NoConfig);
    }

    // import may be unused if none of the following features are set
    #[allow(unused_imports)]
    use crate::gpio::SealedPin;

    // If we are not using SWD pins for SWD reasons, make them floating inputs
    #[cfg(feature = "swd-as-gpio")]
    {
        peripherals.P0_0.set_as_disabled();
        peripherals.P0_1.set_as_disabled();
    }
    #[cfg(feature = "swd-swo-as-gpio")]
    {
        peripherals.P0_2.set_as_disabled();
    }
    #[cfg(feature = "jtag-extras-as-gpio")]
    {
        peripherals.P0_3.set_as_disabled();
        peripherals.P0_6.set_as_disabled();
    }
    #[cfg(feature = "dangerous-reset-as-gpio")]
    {
        // DANGER DANGER DANGER
        peripherals.P0_29.set_as_disabled();
    }

    peripherals
}

/// Macro to bind interrupts to handlers, similar to embassy-imxrt.
///
/// Example:
/// - Bind OS_EVENT to the OSTIMER time-driver handler
///   bind_interrupts!(struct Irqs { OS_EVENT => crate::ostimer::time_driver::OsEventHandler; });
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
