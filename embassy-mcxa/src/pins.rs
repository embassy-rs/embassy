//! Pin configuration helpers (separate from peripheral drivers).
use crate::pac;

pub unsafe fn configure_adc_pins() {
    // P1_10 = ADC1_A8
    let port1 = &*pac::Port1::ptr();
    port1.pcr10().write(|w| {
        w.ps()
            .ps0()
            .pe()
            .pe0()
            .sre()
            .sre0()
            .ode()
            .ode0()
            .dse()
            .dse0()
            .mux()
            .mux0()
            .ibe()
            .ibe0()
            .inv()
            .inv0()
            .lk()
            .lk0()
    });
    core::arch::asm!("dsb sy; isb sy");
}
