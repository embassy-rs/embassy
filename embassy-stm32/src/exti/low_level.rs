//! Low-level EXTI (External Interrupt) functionality
//!
//! This module provides low-level functions for configuring and controlling
//! the External Interrupt (EXTI) lines on STM32 microcontrollers. These functions
//! are primarily intended for internal use by higher-level EXTI drivers.
//!
//! These functions provide access to core EXTI functionality without the async
//! overhead of the main embassy EXTI implementation, making them suitable for use
//! with frameworks like RTIC or custom interrupt handlers.
use super::{cpu_regs, exticr_regs};
use crate::pac::EXTI;

/// Defines the edge triggering mode for EXTI interrupts
///
/// This enum specifies which signal edges should trigger an EXTI interrupt:
/// - `Falling`: Only falling edges (high to low transitions) trigger interrupts
/// - `Rising`: Only rising edges (low to high transitions) trigger interrupts
/// - `Any`: Both rising and falling edges trigger interrupts
pub enum TriggerEdge {
    /// Detect only falling edges (high to low transitions)
    Falling,
    /// Detect only rising edges (low to high transitions)
    Rising,
    /// Detect both rising and falling edges
    Any,
}

/// Configures an EXTI line for a specific GPIO pin
///
/// This function configures the EXTI line corresponding to the provided pin number
/// for the specified port, and sets up edge detection based on the provided parameters.
///
/// # Arguments
/// * `pin` - The pin number
/// * `port` - The GPIO port number
/// * `trigger_edge` - The edge triggering mode (falling, rising, or both)
pub(super) fn configure_exti_pin(pin: u8, port: u8, trigger_edge: TriggerEdge) {
    critical_section::with(|_| {
        let pin_num = pin as usize;
        exticr_regs()
            .exticr(pin_num / 4)
            .modify(|w| w.set_exti(pin_num % 4, port));

        let (rising, falling) = match trigger_edge {
            TriggerEdge::Falling => (false, true),
            TriggerEdge::Rising => (true, false),
            TriggerEdge::Any => (true, true),
        };

        EXTI.rtsr(0).modify(|w| w.set_line(pin_num, rising));
        EXTI.ftsr(0).modify(|w| w.set_line(pin_num, falling));

        clear_exti_pending(pin);
    });
}

/// Enables the EXTI interrupt for a specific pin
///
/// This function enables interrupt generation for the EXTI line
/// corresponding to the provided pin number.
///
/// # Arguments
/// * `pin` - The pin number
pub(super) fn enable_exti_interrupt(pin: u8) {
    critical_section::with(|_| {
        let pin = pin as usize;
        cpu_regs().imr(0).modify(|w| w.set_line(pin, true));
    });
}

/// Configures and enables an EXTI line for a specific GPIO pin in a single critical section
///
/// This function ensures that both configuration and enabling happen atomically
/// within a single critical section.
///
/// # Arguments
/// * `pin` - The pin number
/// * `port` - The GPIO port number
/// * `trigger_edge` - The edge triggering mode (falling, rising, or both)
pub(super) fn configure_and_enable_exti(pin: u8, port: u8, trigger_edge: TriggerEdge) {
    critical_section::with(|_| {
        configure_exti_pin(pin, port, trigger_edge);
        enable_exti_interrupt(pin);
    });
}

/// Disables the EXTI interrupt for a specific pin
///
/// This function disables interrupt generation for the EXTI line
/// corresponding to the provided pin number.
///
/// # Arguments
/// * `pin` - The pin number
pub(super) fn disable_exti_interrupt(pin: u8) {
    critical_section::with(|_| {
        let pin = pin as usize;
        cpu_regs().imr(0).modify(|w| w.set_line(pin, false));
    });
}

/// Clears the pending EXTI interrupt flag for a specific pin
///
/// This function clears any pending interrupt flag for the EXTI line
/// corresponding to the provided pin number.
///
/// # Arguments
/// * `pin` - The pin number
pub(super) fn clear_exti_pending(pin: u8) {
    critical_section::with(|_| {
        let pin = pin as usize;

        #[cfg(not(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50)))]
        EXTI.pr(0).write(|w| w.set_line(pin, true));
        #[cfg(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50))]
        {
            EXTI.rpr(0).write(|w| w.set_line(pin, true));
            EXTI.fpr(0).write(|w| w.set_line(pin, true));
        }
    });
}

/// Checks if an EXTI interrupt is pending for a specific pin
///
/// This function checks if there is a pending interrupt on the EXTI line
/// corresponding to the provided pin number.
///
/// # Arguments
/// * `pin` - The pin number
///
/// # Returns
/// `true` if an interrupt is pending, `false` otherwise
#[cfg(feature = "exti-with-custom-handlers")]
pub(super) fn is_exti_pending(pin: u8) -> bool {
    let pin = pin as usize;

    #[cfg(not(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50)))]
    return EXTI.pr(0).read().line(pin);
    #[cfg(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_h5, exti_h50))]
    return EXTI.rpr(0).read().line(pin) || EXTI.fpr(0).read().line(pin);
}
