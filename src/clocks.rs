//! Clock control helpers (no magic numbers, PAC field access only).
//! Provides reusable gate abstractions for peripherals used by the examples.
use crate::pac;

/// Trait describing an AHB clock gate that can be toggled through MRCC.
pub trait Gate {
    /// Enable the clock gate.
    unsafe fn enable(mrcc: &pac::mrcc0::RegisterBlock);

    /// Return whether the clock gate is currently enabled.
    fn is_enabled(mrcc: &pac::mrcc0::RegisterBlock) -> bool;
}

/// Enable a clock gate for the given peripheral set.
#[inline]
pub unsafe fn enable<G: Gate>(peripherals: &pac::Peripherals) {
    let mrcc = &peripherals.mrcc0;
    G::enable(mrcc);
    while !G::is_enabled(mrcc) {}
    core::arch::asm!("dsb sy; isb sy", options(nomem, nostack, preserves_flags));
}

/// Check whether a gate is currently enabled.
#[inline]
pub fn is_enabled<G: Gate>(peripherals: &pac::Peripherals) -> bool {
    G::is_enabled(&peripherals.mrcc0)
}

macro_rules! impl_cc_gate {
    ($name:ident, $reg:ident, $field:ident) => {
        pub struct $name;

        impl Gate for $name {
            #[inline]
            unsafe fn enable(mrcc: &pac::mrcc0::RegisterBlock) {
                mrcc.$reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            fn is_enabled(mrcc: &pac::mrcc0::RegisterBlock) -> bool {
                mrcc.$reg().read().$field().is_enabled()
            }
        }
    };
}

pub mod gate {
    use super::*;

    impl_cc_gate!(Port2, mrcc_glb_cc1, port2);
    impl_cc_gate!(Port3, mrcc_glb_cc1, port3);
    impl_cc_gate!(Ostimer0, mrcc_glb_cc1, ostimer0);
    impl_cc_gate!(Lpuart2, mrcc_glb_cc0, lpuart2);
    impl_cc_gate!(Gpio3, mrcc_glb_cc2, gpio3);
    impl_cc_gate!(Port1, mrcc_glb_cc1, port1);
    impl_cc_gate!(Adc1, mrcc_glb_cc1, adc1);
}

/// Convenience helper enabling the PORT2 and LPUART2 gates required for the debug UART.
pub unsafe fn enable_uart2_port2(peripherals: &pac::Peripherals) {
    enable::<gate::Port2>(peripherals);
    enable::<gate::Lpuart2>(peripherals);
}

/// Convenience helper enabling the PORT3 and GPIO3 gates used by the LED in the examples.
pub unsafe fn enable_led_port(peripherals: &pac::Peripherals) {
    enable::<gate::Port3>(peripherals);
    enable::<gate::Gpio3>(peripherals);
}

/// Convenience helper enabling the OSTIMER0 clock gate.
pub unsafe fn enable_ostimer0(peripherals: &pac::Peripherals) {
    enable::<gate::Ostimer0>(peripherals);
}

pub unsafe fn select_uart2_clock(peripherals: &pac::Peripherals) {
    // Use FRO_LF_DIV (already running) MUX=0 DIV=0
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_lpuart2_clksel()
        .write(|w| w.mux().clkroot_func_0());
    mrcc.mrcc_lpuart2_clkdiv().write(|w| unsafe { w.bits(0) });
}

pub unsafe fn ensure_frolf_running(peripherals: &pac::Peripherals) {
    // Ensure FRO_LF divider clock is running (reset default HALT=1 stops it)
    let sys = &peripherals.syscon;
    sys.frolfdiv().modify(|_, w| {
        // DIV defaults to 0; keep it explicit and clear HALT
        unsafe { w.div().bits(0) }.halt().run()
    });
}

/// Compute the FRO_LF_DIV output frequency currently selected for LPUART2.
/// Assumes select_uart2_clock() has chosen MUX=0 (FRO_LF_DIV) and DIV is set in SYSCON.FRO_LF_DIV.
pub unsafe fn uart2_src_hz(peripherals: &pac::Peripherals) -> u32 {
    // SYSCON.FRO_LF_DIV: DIV field is simple divider: freq_out = 12_000_000 / (DIV+1) for many NXP parts.
    // On MCXA276 FRO_LF base is 12 MHz; our init keeps DIV=0, so result=12_000_000.
    // Read it anyway for future generality.
    let div = peripherals.syscon.frolfdiv().read().div().bits() as u32;
    let base = 12_000_000u32;
    base / (div + 1)
}

/// Enable clock gate and release reset for OSTIMER0.
/// Select OSTIMER0 clock source = 1 MHz root (working bring-up configuration).
pub unsafe fn select_ostimer0_clock_1m(peripherals: &pac::Peripherals) {
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_ostimer0_clksel().write(|w| w.mux().clkroot_1m());
}

pub unsafe fn init_fro16k(peripherals: &pac::Peripherals) {
    let vbat = &peripherals.vbat0;
    // Enable FRO16K oscillator
    vbat.froctla().modify(|_, w| w.fro_en().set_bit());

    // Lock the control register
    vbat.frolcka().modify(|_, w| w.lock().set_bit());

    // Enable clock outputs to both VSYS and VDD_CORE domains
    // Bit 0: clk_16k0 to VSYS domain
    // Bit 1: clk_16k1 to VDD_CORE domain
    vbat.froclke().modify(|_, w| unsafe { w.clke().bits(0x3) });
}

pub unsafe fn enable_adc(peripherals: &pac::Peripherals) {
    enable::<gate::Port1>(peripherals);
    enable::<gate::Adc1>(peripherals);
}

pub unsafe fn select_adc_clock(peripherals: &pac::Peripherals) {
    // Use FRO_LF_DIV (already running) MUX=0 DIV=0
    let mrcc = &peripherals.mrcc0;
    mrcc.mrcc_adc_clksel().write(|w| w.mux().clkroot_func_0());
    mrcc.mrcc_adc_clkdiv().write(|w| unsafe { w.bits(0) });
}
