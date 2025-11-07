//! Reset control helpers built on PAC field writers.
use crate::pac;

/// Trait describing a reset line that can be asserted/deasserted.
pub trait ResetLine {
    /// Drive the peripheral out of reset.
    unsafe fn release(mrcc: &pac::mrcc0::RegisterBlock);

    /// Drive the peripheral into reset.
    unsafe fn assert(mrcc: &pac::mrcc0::RegisterBlock);

    /// Check whether the peripheral is currently released.
    fn is_released(mrcc: &pac::mrcc0::RegisterBlock) -> bool;
}

/// Release a reset line for the given peripheral set.
#[inline]
pub unsafe fn release<R: ResetLine>(peripherals: &pac::Peripherals) {
    R::release(&peripherals.mrcc0);
}

/// Assert a reset line for the given peripheral set.
#[inline]
pub unsafe fn assert<R: ResetLine>(peripherals: &pac::Peripherals) {
    R::assert(&peripherals.mrcc0);
}

/// Pulse a reset line (assert then release) with a short delay.
#[inline]
pub unsafe fn pulse<R: ResetLine>(peripherals: &pac::Peripherals) {
    let mrcc = &peripherals.mrcc0;
    R::assert(mrcc);
    cortex_m::asm::nop();
    cortex_m::asm::nop();
    R::release(mrcc);
}

macro_rules! impl_reset_line {
    ($name:ident, $reg:ident, $field:ident) => {
        pub struct $name;

        impl ResetLine for $name {
            #[inline]
            unsafe fn release(mrcc: &pac::mrcc0::RegisterBlock) {
                mrcc.$reg().modify(|_, w| w.$field().enabled());
            }

            #[inline]
            unsafe fn assert(mrcc: &pac::mrcc0::RegisterBlock) {
                mrcc.$reg().modify(|_, w| w.$field().disabled());
            }

            #[inline]
            fn is_released(mrcc: &pac::mrcc0::RegisterBlock) -> bool {
                mrcc.$reg().read().$field().is_enabled()
            }
        }
    };
}

pub mod line {
    use super::*;

    impl_reset_line!(Port2, mrcc_glb_rst1, port2);
    impl_reset_line!(Port3, mrcc_glb_rst1, port3);
    impl_reset_line!(Gpio3, mrcc_glb_rst2, gpio3);
    impl_reset_line!(Lpuart2, mrcc_glb_rst0, lpuart2);
    impl_reset_line!(Ostimer0, mrcc_glb_rst1, ostimer0);
    impl_reset_line!(Port1, mrcc_glb_rst1, port1);
    impl_reset_line!(Adc1, mrcc_glb_rst1, adc1);
}

#[inline]
pub unsafe fn release_reset_port2(peripherals: &pac::Peripherals) {
    release::<line::Port2>(peripherals);
}

#[inline]
pub unsafe fn release_reset_port3(peripherals: &pac::Peripherals) {
    release::<line::Port3>(peripherals);
}

#[inline]
pub unsafe fn release_reset_gpio3(peripherals: &pac::Peripherals) {
    release::<line::Gpio3>(peripherals);
}

#[inline]
pub unsafe fn release_reset_lpuart2(peripherals: &pac::Peripherals) {
    release::<line::Lpuart2>(peripherals);
}

#[inline]
pub unsafe fn release_reset_ostimer0(peripherals: &pac::Peripherals) {
    release::<line::Ostimer0>(peripherals);
}

/// Convenience shim retained for existing call sites.
#[inline]
pub unsafe fn reset_ostimer0(peripherals: &pac::Peripherals) {
    pulse::<line::Ostimer0>(peripherals);
}

#[inline]
pub unsafe fn release_reset_port1(peripherals: &pac::Peripherals) {
    release::<line::Port1>(peripherals);
}

#[inline]
pub unsafe fn release_reset_adc1(peripherals: &pac::Peripherals) {
    release::<line::Adc1>(peripherals);
}
