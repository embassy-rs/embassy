//! Module for MCXA2xx family

pub use inner_periph::{Peripherals, peripherals};

use crate::interrupt::InterruptExt;

// NOTE: macro generates missing safety docs and unsafe calls in unsafe blocks,
// allow for now, and put in a module so we can apply the rule to that scope.
#[allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
mod inner_periph {
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
}

// NOTE: Macro has missing safety docs and makes unsafe calls in unsafe fns
pub use inner_interrupt::*;
#[allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
mod inner_interrupt {
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
}

// Use cortex-m-rt's #[interrupt] attribute directly; PAC does not re-export it.

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

// Chip specific GPIO impls
mod gpio_impls {
    use crate::gpio::{AnyPin, GpioPin, Pull, SealedPin};
    use crate::impl_pin;
    use crate::pac::common::{RW, Reg};
    use crate::pac::gpio::vals::{Pdd, Pid};
    use crate::pac::port::regs::Pcr;
    use crate::pac::port::vals::{Dse, Ibe, Mux, Sre};

    #[cfg(feature = "swd-as-gpio")]
    impl_pin!(P0_0, 0, 0, GPIO0);
    #[cfg(feature = "swd-as-gpio")]
    impl_pin!(P0_1, 0, 1, GPIO0);
    #[cfg(feature = "swd-swo-as-gpio")]
    impl_pin!(P0_2, 0, 2, GPIO0);
    #[cfg(feature = "jtag-extras-as-gpio")]
    impl_pin!(P0_3, 0, 3, GPIO0);
    impl_pin!(P0_4, 0, 4, GPIO0);
    impl_pin!(P0_5, 0, 5, GPIO0);
    #[cfg(feature = "jtag-extras-as-gpio")]
    impl_pin!(P0_6, 0, 6, GPIO0);
    impl_pin!(P0_7, 0, 7, GPIO0);
    impl_pin!(P0_8, 0, 8, GPIO0);
    impl_pin!(P0_9, 0, 9, GPIO0);
    impl_pin!(P0_10, 0, 10, GPIO0);
    impl_pin!(P0_11, 0, 11, GPIO0);
    impl_pin!(P0_12, 0, 12, GPIO0);
    impl_pin!(P0_13, 0, 13, GPIO0);
    impl_pin!(P0_14, 0, 14, GPIO0);
    impl_pin!(P0_15, 0, 15, GPIO0);
    impl_pin!(P0_16, 0, 16, GPIO0);
    impl_pin!(P0_17, 0, 17, GPIO0);
    impl_pin!(P0_18, 0, 18, GPIO0);
    impl_pin!(P0_19, 0, 19, GPIO0);
    impl_pin!(P0_20, 0, 20, GPIO0);
    impl_pin!(P0_21, 0, 21, GPIO0);
    impl_pin!(P0_22, 0, 22, GPIO0);
    impl_pin!(P0_23, 0, 23, GPIO0);
    impl_pin!(P0_24, 0, 24, GPIO0);
    impl_pin!(P0_25, 0, 25, GPIO0);
    impl_pin!(P0_26, 0, 26, GPIO0);
    impl_pin!(P0_27, 0, 27, GPIO0);
    impl_pin!(P0_28, 0, 28, GPIO0);
    impl_pin!(P0_29, 0, 29, GPIO0);
    impl_pin!(P0_30, 0, 30, GPIO0);
    impl_pin!(P0_31, 0, 31, GPIO0);

    impl_pin!(P1_0, 1, 0, GPIO1);
    impl_pin!(P1_1, 1, 1, GPIO1);
    impl_pin!(P1_2, 1, 2, GPIO1);
    impl_pin!(P1_3, 1, 3, GPIO1);
    impl_pin!(P1_4, 1, 4, GPIO1);
    impl_pin!(P1_5, 1, 5, GPIO1);
    impl_pin!(P1_6, 1, 6, GPIO1);
    impl_pin!(P1_7, 1, 7, GPIO1);
    impl_pin!(P1_8, 1, 8, GPIO1);
    impl_pin!(P1_9, 1, 9, GPIO1);
    impl_pin!(P1_10, 1, 10, GPIO1);
    impl_pin!(P1_11, 1, 11, GPIO1);
    impl_pin!(P1_12, 1, 12, GPIO1);
    impl_pin!(P1_13, 1, 13, GPIO1);
    impl_pin!(P1_14, 1, 14, GPIO1);
    impl_pin!(P1_15, 1, 15, GPIO1);
    impl_pin!(P1_16, 1, 16, GPIO1);
    impl_pin!(P1_17, 1, 17, GPIO1);
    impl_pin!(P1_18, 1, 18, GPIO1);
    impl_pin!(P1_19, 1, 19, GPIO1);
    impl_pin!(P1_20, 1, 20, GPIO1);
    impl_pin!(P1_21, 1, 21, GPIO1);
    impl_pin!(P1_22, 1, 22, GPIO1);
    impl_pin!(P1_23, 1, 23, GPIO1);
    impl_pin!(P1_24, 1, 24, GPIO1);
    impl_pin!(P1_25, 1, 25, GPIO1);
    impl_pin!(P1_26, 1, 26, GPIO1);
    impl_pin!(P1_27, 1, 27, GPIO1);
    impl_pin!(P1_28, 1, 28, GPIO1);
    #[cfg(feature = "dangerous-reset-as-gpio")]
    impl_pin!(P1_29, 1, 29, GPIO1);
    #[cfg(feature = "sosc-as-gpio")]
    impl_pin!(P1_30, 1, 30, GPIO1);
    #[cfg(feature = "sosc-as-gpio")]
    impl_pin!(P1_31, 1, 31, GPIO1);

    impl_pin!(P2_0, 2, 0, GPIO2);
    impl_pin!(P2_1, 2, 1, GPIO2);
    impl_pin!(P2_2, 2, 2, GPIO2);
    impl_pin!(P2_3, 2, 3, GPIO2);
    impl_pin!(P2_4, 2, 4, GPIO2);
    impl_pin!(P2_5, 2, 5, GPIO2);
    impl_pin!(P2_6, 2, 6, GPIO2);
    impl_pin!(P2_7, 2, 7, GPIO2);
    impl_pin!(P2_8, 2, 8, GPIO2);
    impl_pin!(P2_9, 2, 9, GPIO2);
    impl_pin!(P2_10, 2, 10, GPIO2);
    impl_pin!(P2_11, 2, 11, GPIO2);
    impl_pin!(P2_12, 2, 12, GPIO2);
    impl_pin!(P2_13, 2, 13, GPIO2);
    impl_pin!(P2_14, 2, 14, GPIO2);
    impl_pin!(P2_15, 2, 15, GPIO2);
    impl_pin!(P2_16, 2, 16, GPIO2);
    impl_pin!(P2_17, 2, 17, GPIO2);
    impl_pin!(P2_18, 2, 18, GPIO2);
    impl_pin!(P2_19, 2, 19, GPIO2);
    impl_pin!(P2_20, 2, 20, GPIO2);
    impl_pin!(P2_21, 2, 21, GPIO2);
    impl_pin!(P2_22, 2, 22, GPIO2);
    impl_pin!(P2_23, 2, 23, GPIO2);
    impl_pin!(P2_24, 2, 24, GPIO2);
    impl_pin!(P2_25, 2, 25, GPIO2);
    impl_pin!(P2_26, 2, 26, GPIO2);
    // impl_pin!(P2_27, 2, 27, GPIO2);
    // impl_pin!(P2_28, 2, 28, GPIO2);
    // impl_pin!(P2_29, 2, 29, GPIO2);
    // impl_pin!(P2_30, 2, 30, GPIO2);
    // impl_pin!(P2_31, 2, 31, GPIO2);

    impl_pin!(P3_0, 3, 0, GPIO3);
    impl_pin!(P3_1, 3, 1, GPIO3);
    impl_pin!(P3_2, 3, 2, GPIO3);
    impl_pin!(P3_3, 3, 3, GPIO3);
    impl_pin!(P3_4, 3, 4, GPIO3);
    impl_pin!(P3_5, 3, 5, GPIO3);
    impl_pin!(P3_6, 3, 6, GPIO3);
    impl_pin!(P3_7, 3, 7, GPIO3);
    impl_pin!(P3_8, 3, 8, GPIO3);
    impl_pin!(P3_9, 3, 9, GPIO3);
    impl_pin!(P3_10, 3, 10, GPIO3);
    impl_pin!(P3_11, 3, 11, GPIO3);
    impl_pin!(P3_12, 3, 12, GPIO3);
    impl_pin!(P3_13, 3, 13, GPIO3);
    impl_pin!(P3_14, 3, 14, GPIO3);
    impl_pin!(P3_15, 3, 15, GPIO3);
    impl_pin!(P3_16, 3, 16, GPIO3);
    impl_pin!(P3_17, 3, 17, GPIO3);
    impl_pin!(P3_18, 3, 18, GPIO3);
    impl_pin!(P3_19, 3, 19, GPIO3);
    impl_pin!(P3_20, 3, 20, GPIO3);
    impl_pin!(P3_21, 3, 21, GPIO3);
    impl_pin!(P3_22, 3, 22, GPIO3);
    impl_pin!(P3_23, 3, 23, GPIO3);
    impl_pin!(P3_24, 3, 24, GPIO3);
    impl_pin!(P3_25, 3, 25, GPIO3);
    impl_pin!(P3_26, 3, 26, GPIO3);
    impl_pin!(P3_27, 3, 27, GPIO3);
    impl_pin!(P3_28, 3, 28, GPIO3);
    impl_pin!(P3_29, 3, 29, GPIO3);
    impl_pin!(P3_30, 3, 30, GPIO3);
    impl_pin!(P3_31, 3, 31, GPIO3);

    impl_pin!(P4_0, 4, 0, GPIO4);
    impl_pin!(P4_1, 4, 1, GPIO4);
    impl_pin!(P4_2, 4, 2, GPIO4);
    impl_pin!(P4_3, 4, 3, GPIO4);
    impl_pin!(P4_4, 4, 4, GPIO4);
    impl_pin!(P4_5, 4, 5, GPIO4);
    impl_pin!(P4_6, 4, 6, GPIO4);
    impl_pin!(P4_7, 4, 7, GPIO4);
    // impl_pin!(P4_8, 4, 8, GPIO4);
    // impl_pin!(P4_9, 4, 9, GPIO4);
    // impl_pin!(P4_10, 4, 10, GPIO4);
    // impl_pin!(P4_11, 4, 11, GPIO4);
    // impl_pin!(P4_12, 4, 12, GPIO4);
    // impl_pin!(P4_13, 4, 13, GPIO4);
    // impl_pin!(P4_14, 4, 14, GPIO4);
    // impl_pin!(P4_15, 4, 15, GPIO4);
    // impl_pin!(P4_16, 4, 16, GPIO4);
    // impl_pin!(P4_17, 4, 17, GPIO4);
    // impl_pin!(P4_18, 4, 18, GPIO4);
    // impl_pin!(P4_19, 4, 19, GPIO4);
    // impl_pin!(P4_20, 4, 20, GPIO4);
    // impl_pin!(P4_21, 4, 21, GPIO4);
    // impl_pin!(P4_22, 4, 22, GPIO4);
    // impl_pin!(P4_23, 4, 23, GPIO4);
    // impl_pin!(P4_24, 4, 24, GPIO4);
    // impl_pin!(P4_25, 4, 25, GPIO4);
    // impl_pin!(P4_26, 4, 26, GPIO4);
    // impl_pin!(P4_27, 4, 27, GPIO4);
    // impl_pin!(P4_28, 4, 28, GPIO4);
    // impl_pin!(P4_29, 4, 29, GPIO4);
    // impl_pin!(P4_30, 4, 30, GPIO4);
    // impl_pin!(P4_31, 4, 31, GPIO4);
}
