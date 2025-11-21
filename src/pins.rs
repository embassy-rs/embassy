//! Pin configuration helpers (separate from peripheral drivers).
use crate::pac;

pub unsafe fn configure_uart2_pins_port2() {
    // P2_2 = LPUART2_TX ALT3, P2_3 = LPUART2_RX ALT3 with pull-up, input enable, high drive, slow slew.
    let port2 = &*pac::Port2::ptr();
    port2.pcr2().write(|w| {
        w.ps()
            .ps1()
            .pe()
            .pe1()
            .sre()
            .sre1()
            .dse()
            .dse1()
            .mux()
            .mux3()
            .ibe()
            .ibe1()
    });
    port2.pcr3().write(|w| {
        w.ps()
            .ps1()
            .pe()
            .pe1()
            .sre()
            .sre1()
            .dse()
            .dse1()
            .mux()
            .mux3()
            .ibe()
            .ibe1()
    });
    core::arch::asm!("dsb sy; isb sy");
}

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
