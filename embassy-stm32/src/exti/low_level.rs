//! Low-level EXTI (External Interrupt) functionality
//!
//! This module provides low-level functions for configuring and controlling
//! the External Interrupt (EXTI) lines on STM32 microcontrollers.

use super::{cpu_regs, exticr_regs};
use crate::gpio::{Input, Pin as GpioPin, PinNumber};
use crate::pac::EXTI;
use crate::pac::exti::regs::Lines;

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
pub(super) fn configure_exti_pin(pin: PinNumber, port: PinNumber, trigger_edge: TriggerEdge) {
    critical_section::with(|_| {
        let pin_num = pin as usize;
        #[cfg(exti_n6)]
        // Ports N and above (starting index 13) use value 8 and higher, as Ports I-M are not present
        let port = {
            const STM32_PORTI: PinNumber = 0x8;
            const STM32_PORTN: PinNumber = 0xD;
            if port >= STM32_PORTN {
                port - (STM32_PORTN - STM32_PORTI) // N-Q = 8-12
            } else {
                port // A-H = 0-7
            }
        };
        // Cast needed: on N6, PinNumber is u16 (for total pin counting), but port is always 0-15.
        exticr_regs()
            .exticr(pin_num / 4)
            .modify(|w| w.set_exti(pin_num % 4, port as _));

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

/// EXTI interrupt state
pub enum InterruptState {
    /// Interrupt enabled
    Enabled,
    /// Interrupt disabled
    Disabled,
}

impl From<InterruptState> for bool {
    fn from(state: InterruptState) -> bool {
        matches!(state, InterruptState::Enabled)
    }
}

/// Enables the EXTI interrupt for a specific pin
pub(super) fn set_exti_interrupt_enabled(pin: PinNumber, state: InterruptState) {
    critical_section::with(|_| {
        let pin = pin as usize;
        cpu_regs().imr(0).modify(|w| w.set_line(pin, state.into()));
    });
}

/// Configures and enables an EXTI line from a GPIO Input in a single critical section
pub(super) fn configure_and_enable_exti(pin: &Input, trigger_edge: TriggerEdge) {
    let pin_num = pin.pin.pin.pin();
    let port_num = pin.pin.pin.port();

    critical_section::with(|_| {
        configure_exti_pin(pin_num, port_num, trigger_edge);
        set_exti_interrupt_enabled(pin_num, InterruptState::Enabled);
    });
}

/// Conditional compilation for STM32 variants with separate RPR/FPR registers.
/// These variants use separate Rising/Falling Pending Registers instead of the unified PR register.
macro_rules! cfg_has_rpr_fpr {
    ($($tokens:tt)*) => {
        #[cfg(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6))]
        $($tokens)*
    };
}

/// Conditional compilation for STM32 variants with unified PR register.
/// Represents variants that use a single Pending Register for both rising and falling edges.
macro_rules! cfg_no_rpr_fpr {
    ($($tokens:tt)*) => {
        #[cfg(not(any(exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6)))]
        $($tokens)*
    };
}

/// Clears any pending EXTI interrupt flag for a specific bit mask
pub(super) fn clear_exti_pending_mask(mask: u32) {
    cfg_no_rpr_fpr! {
        EXTI.pr(0).write_value(Lines(mask));
    }

    cfg_has_rpr_fpr!({
        EXTI.rpr(0).write_value(Lines(mask));
        EXTI.fpr(0).write_value(Lines(mask));
    })
}

/// Clears the pending EXTI interrupt flag for a specific pin
pub(super) fn clear_exti_pending(pin: PinNumber) {
    let mask = 1u32 << pin;

    critical_section::with(|_| {
        clear_exti_pending_mask(mask);
    });
}

/// Checks if an EXTI interrupt is pending for a specific pin
///
/// # Returns
/// `true` if an interrupt is pending, `false` otherwise
pub(super) fn is_exti_pending(pin: PinNumber) -> bool {
    let pin = pin as usize;

    cfg_no_rpr_fpr! {
        return EXTI.pr(0).read().line(pin);
    }

    cfg_has_rpr_fpr! {
        return EXTI.rpr(0).read().line(pin) || EXTI.fpr(0).read().line(pin);
    }
}
