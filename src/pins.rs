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
            .mux11()
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
            .mux11()
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
            .mux00()
            .ibe()
            .ibe0()
            .inv()
            .inv0()
            .lk()
            .lk0()
    });
    core::arch::asm!("dsb sy; isb sy");
}

/// Configure a pin for a specific mux alternative.
///
/// # Arguments
/// * `port` - Port number (0-4)
/// * `pin` - Pin number (varies by port: PORT0=0-7, PORT1=0-19, PORT2=0-26, PORT3=0-31, PORT4=0-7)
/// * `mux` - Mux alternative (0-15, where 0 = GPIO, 1-15 = other functions)
pub unsafe fn set_pin_mux(port: u8, pin: u8, mux: u8) {
    // Validate mux value (0-15)
    if mux > 15 {
        panic!("Invalid mux value: {}, must be 0-15", mux);
    }

    // Validate pin number based on port
    let max_pin = match port {
        0 => 7,  // PORT0: pins 0-7
        1 => 19, // PORT1: pins 0-19
        2 => 26, // PORT2: pins 0-26
        3 => 31, // PORT3: pins 0-31
        4 => 7,  // PORT4: pins 0-7
        _ => panic!("Unsupported GPIO port: {}", port),
    };

    if pin > max_pin {
        panic!(
            "Invalid pin {} for PORT{}, max pin is {}",
            pin, port, max_pin
        );
    }

    // Get the base address for the port
    let port_base: *mut u32 = match port {
        0 => pac::Port0::ptr() as *mut u32,
        1 => pac::Port1::ptr() as *mut u32,
        2 => pac::Port2::ptr() as *mut u32,
        3 => pac::Port3::ptr() as *mut u32,
        4 => pac::Port4::ptr() as *mut u32,
        _ => panic!("Unsupported GPIO port: {}", port),
    };

    // PCR registers are 4 bytes apart, starting at offset 0 for PCR0
    let pcr_addr = port_base.add(pin as usize);

    // Read current PCR value
    let current_val = pcr_addr.read_volatile();

    // Clear mux bits (bits 8-11) and set new mux value
    let new_val = (current_val & !(0xF << 8)) | ((mux as u32) << 8);

    // Write back the new value
    pcr_addr.write_volatile(new_val);

    core::arch::asm!("dsb sy; isb sy");
}

/// Configure a pin for GPIO mode (ALT0).
/// This is a convenience function that calls set_pin_mux with mux=0.
///
/// # Arguments
/// * `port` - Port number (0-4)
/// * `pin` - Pin number (varies by port: PORT0=0-7, PORT1=0-19, PORT2=0-26, PORT3=0-31, PORT4=0-7)
pub unsafe fn set_pin_mux_gpio(port: u8, pin: u8) {
    set_pin_mux(port, pin, 0);
}
