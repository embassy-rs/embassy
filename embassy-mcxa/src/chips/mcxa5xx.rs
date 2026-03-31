//! Module for MCXA5xx family

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

        // AOI0,
        // AOI1,

        // CAN0,
        // CAN1,

        CDOG0,
        CDOG1,

        // CLKOUT is not specifically a peripheral (it's part of SYSCON),
        // but we still want it to be a singleton.
        CLKOUT,

        // CMC,
        // CMP0,
        // CMP1,
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

        // DBGMAILBOX,

        DMA0,
        DMA0_CH0,
        DMA0_CH1,
        DMA0_CH2,
        DMA0_CH3,
        DMA0_CH4,
        DMA0_CH5,
        DMA0_CH6,
        DMA0_CH7,
        // Need more work on the DMA driver before we can enable these
        // DMA_CH8,
        // DMA_CH9,
        // DMA_CH10,
        // DMA_CH11,
        EDMA0_TCD0,
        // Need more work on the DMA driver before we can enable this
        // EDMA0_TCD1,

        // EIM0,
        // EQDC0,
        // EQDC1,
        // ERM0,
        // FLEXIO0,
        // FLEXPWM0,
        // FLEXPWM1,
        // FMC0,
        // FMU0,
        // FREQME0,
        // GLIKEY0,

        GPIO0,
        GPIO1,
        GPIO2,
        GPIO3,
        GPIO4,
        GPIO5,

        I3C0,
        I3C1,
        I3C2,
        I3C3,
        INPUTMUX0,

        LPI2C0,
        LPI2C1,
        LPI2C2,
        LPI2C3,

        LPSPI0,
        LPSPI1,
        LPSPI2,
        LPSPI3,
        LPSPI4,
        LPSPI5,

        // LPTMR0,

        LPUART0,
        LPUART1,
        LPUART2,
        LPUART3,
        LPUART4,
        LPUART5,

        // MAU0,
        // MBC0,
        // MRCC0,
        // OPAMP0,
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
        // Normally RESET_B!
        #[cfg(feature = "dangerous-reset-as-gpio")]
        P1_29,
        // Normally XTAL48M!
        #[cfg(feature = "sosc-as-gpio")]
        P1_30,
        // Normally EXTAL48M!
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

        // Normally EXTAL32K!
        #[cfg(feature = "rosc-32k-as-gpio")]
        P5_0,
        // Normally XTAL32K!
        #[cfg(feature = "rosc-32k-as-gpio")]
        P5_1,
        P5_2,
        P5_3,
        P5_4,
        P5_5,
        P5_6,
        P5_7,
        P5_8,
        P5_9,

        // PKC0,

        PORT0,
        PORT1,
        PORT2,
        PORT3,
        PORT4,
        PORT5,

        RTC0,
        // SAU,
        // SCG0,
        // SCN_SCB,
        // SGI0,
        // SMARTDMA0,
        // SPC0,
        // SYSCON,
        // TDET0,
        TRNG0,
        // UDF0,
        // USB0,
        // UTICK0,
        // VBAT0,
        // WAKETIMER0,
        // WUU0,
        WWDT0,
        WWDT1,
    );
}

// NOTE: Macro has missing safety docs and makes unsafe calls in unsafe fns
pub use inner_interrupt::*;
#[allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
mod inner_interrupt {
    #[rustfmt::skip]
    embassy_hal_internal::interrupt_mod!(
        ADC0,
        ADC1,

        // CAN0,
        // CAN1,

        CDOG0,
        CDOG1,

        // CMC,

        // CMP0,
        // CMP1,
        // CMP2,

        CTIMER0,
        CTIMER1,
        CTIMER2,
        CTIMER3,
        CTIMER4,

        // DAC0,

        DMA0_CH0,
        DMA0_CH1,
        DMA0_CH2,
        DMA0_CH3,
        DMA0_CH4,
        DMA0_CH5,
        DMA0_CH6,
        DMA0_CH7,
        DMA0_CH8,
        DMA0_CH9,
        DMA0_CH10,
        DMA0_CH11,
        DMA1_CH0,
        DMA1_CH1,
        DMA1_CH2,
        DMA1_CH3,

        // EQDC0_COMPARE,
        // EQDC0_HOME,
        // EQDC0_INDEX,
        // EQDC0_WATCHDOG,
        // EQDC1_COMPARE,
        // EQDC1_HOME,
        // EQDC1_INDEX,
        // EQDC1_WATCHDOG,
        // ERM0_MULTI_BIT,
        // ERM0_SINGLE_BIT,
        // FLEXIO,
        // FLEXPWM0_FAULT,
        // FLEXPWM0_RELOAD_ERROR,
        // FLEXPWM0_SUBMODULE0,
        // FLEXPWM0_SUBMODULE1,
        // FLEXPWM0_SUBMODULE2,
        // FLEXPWM0_SUBMODULE3,
        // FLEXPWM1_FAULT,
        // FLEXPWM1_RELOAD_ERROR,
        // FLEXPWM1_SUBMODULE0,
        // FLEXPWM1_SUBMODULE1,
        // FLEXPWM1_SUBMODULE2,
        // FLEXPWM1_SUBMODULE3,
        // FMU0,
        // FREQME0,
        // GLIKEY0,

        GPIO0,
        GPIO1,
        GPIO2,
        GPIO3,
        GPIO4,
        GPIO5,

        I3C0,
        I3C1,
        I3C2,
        I3C3,

        LPI2C0,
        LPI2C1,
        LPI2C2,
        LPI2C3,

        LPSPI0,
        LPSPI1,
        LPSPI2,
        LPSPI3,
        LPSPI4,
        LPSPI5,

        // LPTMR0,
        LPUART0,
        LPUART1,
        LPUART2,
        LPUART3,
        LPUART4,
        LPUART5,
        // MAU,
        // MBC0,
        OS_EVENT,
        // PKC,
        RTC0,
        // SCG0,
        // SGI,
        // SLCD,
        // SMARTDMA,
        // SPC0,
        // TDET,
        TRNG0,
        // USB0,
        // UTICK0,
        // WAKETIMER0,
        // WUU0,
        WWDT0,
        WWDT1,
    );
}

/// Initialize HAL with configuration (mirrors embassy-imxrt style). Minimal: just take peripherals.
/// Also applies configurable NVIC priority for the OSTIMER OS_EVENT interrupt (no enabling).
pub fn init(cfg: crate::config::Config) -> Peripherals {
    // Might not need to be mutable if none of the `...-as-gpio` features are active.
    #[allow(unused_mut)]
    let mut peripherals = Peripherals::take();

    // crate::interrupt::RTC.set_priority(cfg.rtc_interrupt_priority);
    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO1.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO2.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO4.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO5.set_priority(cfg.gpio_interrupt_priority);

    // Configure clocks
    crate::clocks::init(cfg.clock_cfg).unwrap();

    // Initialize embassy-time global driver backed by OSTIMER0
    // NOTE: As early as possible, but MUST be AFTER clocks!
    crate::ostimer::init(cfg.time_interrupt_priority);

    // Initialize the INPUTMUX peripheral
    crate::inputmux::init();

    // Enable interrupts
    unsafe {
        cortex_m::interrupt::enable();
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

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT5>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO5>(&crate::clocks::periph_helpers::NoConfig);
    }

    // import may be unused if none of the following features are set
    #[allow(unused_imports)]
    use crate::gpio::SealedPin;

    // If we are not using pins for specialized purposes, set them as disabled
    #[cfg(feature = "rosc-32k-as-gpio")]
    {
        peripherals.P5_0.set_as_disabled();
        peripherals.P5_1.set_as_disabled();
    }
    #[cfg(feature = "sosc-as-gpio")]
    {
        peripherals.P1_30.set_as_disabled();
        peripherals.P1_31.set_as_disabled();
    }
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
        peripherals.P1_29.set_as_disabled();
    }

    peripherals
}

// Chip specific GPIO impls
mod gpio_impls {
    use crate::gpio::{AnyPin, GpioPin, Pull, SealedPin};
    use crate::impl_pin;
    use crate::pac::common::{RW, Reg};
    use crate::pac::gpio::{Pdd, Pid};
    use crate::pac::port::{Dse, Ibe, Mux, Pcr, Sre};

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
    impl_pin!(P2_28, 2, 28, GPIO2);
    impl_pin!(P2_29, 2, 29, GPIO2);
    impl_pin!(P2_30, 2, 30, GPIO2);
    impl_pin!(P2_31, 2, 31, GPIO2);

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
    impl_pin!(P4_8, 4, 8, GPIO4);
    impl_pin!(P4_9, 4, 9, GPIO4);
    impl_pin!(P4_10, 4, 10, GPIO4);
    impl_pin!(P4_11, 4, 11, GPIO4);
    impl_pin!(P4_12, 4, 12, GPIO4);
    impl_pin!(P4_13, 4, 13, GPIO4);

    #[cfg(feature = "rosc-32k-as-gpio")]
    impl_pin!(P5_0, 5, 0, GPIO5);
    #[cfg(feature = "rosc-32k-as-gpio")]
    impl_pin!(P5_1, 5, 1, GPIO5);
    impl_pin!(P5_2, 5, 2, GPIO5);
    impl_pin!(P5_3, 5, 3, GPIO5);
    impl_pin!(P5_4, 5, 4, GPIO5);
    impl_pin!(P5_5, 5, 5, GPIO5);
    impl_pin!(P5_6, 5, 6, GPIO5);
    impl_pin!(P5_7, 5, 7, GPIO5);
    impl_pin!(P5_8, 5, 8, GPIO5);
    impl_pin!(P5_9, 5, 9, GPIO5);
}

/// This module contains implementations of MRCC APIs, specifically of the [`Gate`] trait,
/// for various low level peripherals.
pub(crate) mod peripheral_gating {
    use paste::paste;

    use crate::clocks::Gate;
    use crate::clocks::periph_helpers::{
        AdcConfig, CTimerConfig, Clk1MConfig, I3cConfig, Lpi2cConfig, LpspiConfig, LpuartConfig, NoConfig,
        OsTimerConfig,
    };
    use crate::{impl_cc_gate, pac};

    // These peripherals have no additional upstream clocks or configuration required
    // other than enabling through the MRCC gate. Currently, these peripherals will
    // ALWAYS return `Ok(0)` when calling [`enable_and_reset()`] and/or
    // [`SPConfHelper::post_enable_config()`].
    impl_cc_gate!(PORT0, mrcc_glb_cc1, mrcc_glb_rst1, port0, NoConfig);
    impl_cc_gate!(PORT1, mrcc_glb_cc1, mrcc_glb_rst1, port1, NoConfig);
    impl_cc_gate!(PORT2, mrcc_glb_cc1, mrcc_glb_rst1, port2, NoConfig);
    impl_cc_gate!(PORT3, mrcc_glb_cc1, mrcc_glb_rst1, port3, NoConfig);
    impl_cc_gate!(PORT4, mrcc_glb_cc1, mrcc_glb_rst1, port4, NoConfig);
    impl_cc_gate!(PORT5, mrcc_glb_cc1, port5, NoConfig);

    impl_cc_gate!(CRC0, mrcc_glb_cc0, mrcc_glb_rst0, crc0, NoConfig);
    impl_cc_gate!(INPUTMUX0, mrcc_glb_cc0, mrcc_glb_rst0, inputmux0, NoConfig);

    // These peripherals DO have meaningful configuration, and could fail if the system
    // clocks do not match their needs.
    impl_cc_gate!(ADC0, mrcc_glb_cc1, mrcc_glb_rst1, adc0, AdcConfig);
    impl_cc_gate!(ADC1, mrcc_glb_cc1, mrcc_glb_rst1, adc1, AdcConfig);

    impl_cc_gate!(I3C0, mrcc_glb_acc2, mrcc_glb_rst2, i3c0, I3cConfig);
    impl_cc_gate!(I3C1, mrcc_glb_acc2, mrcc_glb_rst2, i3c1, I3cConfig);
    impl_cc_gate!(I3C2, mrcc_glb_acc2, mrcc_glb_rst2, i3c2, I3cConfig);
    impl_cc_gate!(I3C3, mrcc_glb_acc2, mrcc_glb_rst2, i3c3, I3cConfig);

    impl_cc_gate!(CTIMER0, mrcc_glb_acc0, mrcc_glb_rst0, ctimer0, CTimerConfig);
    impl_cc_gate!(CTIMER1, mrcc_glb_acc0, mrcc_glb_rst0, ctimer1, CTimerConfig);
    impl_cc_gate!(CTIMER2, mrcc_glb_acc0, mrcc_glb_rst0, ctimer2, CTimerConfig);
    impl_cc_gate!(CTIMER3, mrcc_glb_acc0, mrcc_glb_rst0, ctimer3, CTimerConfig);
    impl_cc_gate!(CTIMER4, mrcc_glb_acc0, mrcc_glb_rst0, ctimer4, CTimerConfig);
    impl_cc_gate!(OSTIMER0, mrcc_glb_cc0, mrcc_glb_rst0, ostimer0, OsTimerConfig);

    // TRNG peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(TRNG0, mrcc_glb_acc4, mrcc_glb_rst4, trng0, NoConfig);

    // Peripherals that use ACC instead of CC!
    impl_cc_gate!(LPUART0, mrcc_glb_acc0, mrcc_glb_rst0, lpuart0, LpuartConfig);
    impl_cc_gate!(LPUART1, mrcc_glb_acc0, mrcc_glb_rst0, lpuart1, LpuartConfig);
    impl_cc_gate!(LPUART2, mrcc_glb_acc0, mrcc_glb_rst0, lpuart2, LpuartConfig);
    impl_cc_gate!(LPUART3, mrcc_glb_acc0, mrcc_glb_rst0, lpuart3, LpuartConfig);
    impl_cc_gate!(LPUART4, mrcc_glb_acc0, mrcc_glb_rst0, lpuart4, LpuartConfig);
    impl_cc_gate!(LPUART5, mrcc_glb_acc0, mrcc_glb_rst0, lpuart5, LpuartConfig);

    // DMA0 peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(DMA0, mrcc_glb_acc0, mrcc_glb_rst0, dma0, NoConfig);

    impl_cc_gate!(GPIO0, mrcc_glb_acc3, mrcc_glb_rst3, gpio0, NoConfig);
    impl_cc_gate!(GPIO1, mrcc_glb_acc3, mrcc_glb_rst3, gpio1, NoConfig);
    impl_cc_gate!(GPIO2, mrcc_glb_acc3, mrcc_glb_rst3, gpio2, NoConfig);
    impl_cc_gate!(GPIO3, mrcc_glb_acc3, mrcc_glb_rst3, gpio3, NoConfig);
    impl_cc_gate!(GPIO4, mrcc_glb_acc3, mrcc_glb_rst3, gpio4, NoConfig);
    impl_cc_gate!(GPIO5, mrcc_glb_cc3, gpio5, NoConfig);

    impl_cc_gate!(LPI2C0, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c0, Lpi2cConfig);
    impl_cc_gate!(LPI2C1, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c1, Lpi2cConfig);
    impl_cc_gate!(LPI2C2, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c2, Lpi2cConfig);
    impl_cc_gate!(LPI2C3, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c3, Lpi2cConfig);

    impl_cc_gate!(LPSPI0, mrcc_glb_acc1, mrcc_glb_rst1, lpspi0, LpspiConfig);
    impl_cc_gate!(LPSPI1, mrcc_glb_acc1, mrcc_glb_rst1, lpspi1, LpspiConfig);
    impl_cc_gate!(LPSPI2, mrcc_glb_acc1, mrcc_glb_rst1, lpspi2, LpspiConfig);
    impl_cc_gate!(LPSPI3, mrcc_glb_acc1, mrcc_glb_rst1, lpspi3, LpspiConfig);
    impl_cc_gate!(LPSPI4, mrcc_glb_acc1, mrcc_glb_rst1, lpspi4, LpspiConfig);
    impl_cc_gate!(LPSPI5, mrcc_glb_acc1, mrcc_glb_rst1, lpspi5, LpspiConfig);

    impl_cc_gate!(WWDT0, mrcc_glb_acc0, wwdt0, Clk1MConfig);
    impl_cc_gate!(WWDT1, mrcc_glb_acc0, wwdt1, Clk1MConfig);
}

pub(crate) mod clock_limits {
    #![allow(dead_code)]

    use crate::chips::ClockLimits;

    pub const VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[(24_000_000, 0b0000)];
    // <= 48MHz
    pub const VDD_CORE_MID_DRIVE_MAX_WAIT_STATES: u8 = 0b0001;

    pub const VDD_CORE_NORMAL_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] =
        &[(30_000_000, 0b0000), (60_000_000, 0b0001), (90_000_000, 0b0010)];
    // <= 120MHz
    pub const VDD_CORE_NORMAL_DRIVE_MAX_WAIT_STATES: u8 = 0b0011;

    pub const VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[
        (40_000_000, 0b0000),
        (80_000_000, 0b0001),
        (120_000_000, 0b0010),
        (160_000_000, 0b0011),
        (200_000_000, 0b0100),
    ];
    // <= 250MHz
    pub const VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES: u8 = 0b0101;

    impl ClockLimits {
        pub const MID_DRIVE: Self = Self {
            fro_hf: 96_000_000,
            fro_hf_div: 48_000_000,
            pll1_clk: 100_000_000,
            pll1_clk_div: 100_000_000,
            main_clk: 96_000_000,
            cpu_clk: 48_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };

        pub const NORMAL_DRIVE: Self = Self {
            fro_hf: 192_000_000,
            fro_hf_div: 192_000_000,
            pll1_clk: 300_000_000,
            pll1_clk_div: 150_000_000,
            main_clk: 120_000_000,
            cpu_clk: 120_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };

        pub const OVER_DRIVE: Self = Self {
            fro_hf: 192_000_000,
            fro_hf_div: 192_000_000,
            pll1_clk: 400_000_000,
            pll1_clk_div: 200_000_000,
            main_clk: 240_000_000,
            cpu_clk: 240_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };
    }
}
