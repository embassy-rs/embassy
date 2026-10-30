//! Shared helpers for the ASR6601 hardware-verification binaries.

#![no_std]

use embassy_asr::pac;

/// Keep SWD alive when the core enters sleep/stop/standby.
pub fn enable_debug_in_low_power() {
    let rcc = unsafe { pac::Rcc::steal() };
    rcc.cgr0().modify(|_, w| w.syscfg_clk_en().set_bit());

    let syscfg = unsafe { pac::Syscfg::steal() };
    syscfg.cr2().modify(|_, w| w.syscfg_dbg_sleep().set_bit());
    syscfg
        .cr3()
        .modify(|_, w| w.syscfg_dbg_stop().set_bit().syscfg_dbg_standby().set_bit());
}

/// Short settle delay for GPIO level tests.
#[inline]
pub fn settle_gpio() {
    for _ in 0..10_000 {
        cortex_m::asm::nop();
    }
}

/// Settle delay for DAC/ADC measurements.
#[inline]
pub fn settle_analog() {
    cortex_m::asm::delay(240_000);
}
