//! Blocking EXTI (External Interrupt) functionality
//!
//! This module provides a non-async version of the EXTI input driver designed
//! for use with custom interrupt handlers or frameworks such as RTIC.

use super::low_level::{self, InterruptState, TriggerEdge};
use crate::Peri;
use crate::gpio::{ExtiPin, Input, Level, Pin as GpioPin, Pull};

/// Blocking EXTI input driver for custom interrupt handlers or frameworks such as RTIC.
///
/// This driver augments a GPIO `Input` with EXTI functionality but is designed
/// for use with interrupt-driven frameworks rather than async/await patterns.
pub struct ExtiInput<'d> {
    pin: Input<'d>,
}

impl<'d> Unpin for ExtiInput<'d> {}

impl<'d> ExtiInput<'d> {
    /// Creates a new EXTI input for interrupt-driven usage
    ///
    /// This function creates a new EXTI input that can be used with custom interrupt handlers or
    /// interrupt-driven frameworks like RTIC. It initializes the GPIO pin with the specified pull
    /// configuration, sets up the EXTI channel for the pin, and enables the interrupt.
    ///
    /// # Arguments
    /// * `pin` - The GPIO pin to use
    /// * `ch` - The EXTI channel corresponding to the pin (consumed for ownership tracking)
    /// * `pull` - The pull configuration for the pin
    /// * `trigger_edge` - The edge triggering mode (falling, rising, or any)
    ///
    /// # Returns
    /// A new `ExtiInput` instance with interrupts enabled
    pub fn new<T: GpioPin + ExtiPin>(
        pin: Peri<'d, T>,
        _ch: Peri<'d, T::ExtiChannel>, // Consumed for ownership tracking
        pull: Pull,
        trigger_edge: TriggerEdge,
    ) -> Self {
        let pin = Input::new(pin, pull);

        low_level::configure_and_enable_exti(&pin, trigger_edge);

        Self { pin }
    }

    /// Reconfigures the edge detection mode for this pin's EXTI line
    ///
    /// This method updates which edges (rising, falling, or any) will trigger
    /// interrupts for this pin.
    /// Note that reconfiguring the edge detection will clear any pending
    /// interrupt flag for this pin.
    pub fn set_edge_detection(&mut self, trigger_edge: TriggerEdge) {
        let pin_num = self.pin.pin.pin.pin();
        let port_num = self.pin.pin.pin.port();
        low_level::configure_exti_pin(pin_num, port_num, trigger_edge);
    }

    /// Enables the EXTI interrupt for this pin
    pub fn enable_interrupt(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::set_exti_interrupt_enabled(pin_num, InterruptState::Enabled);
    }

    /// Disables the EXTI interrupt for this pin
    pub fn disable_interrupt(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::set_exti_interrupt_enabled(pin_num, InterruptState::Disabled);
    }

    /// Clears any pending interrupt for this pin
    ///
    /// This method clears the pending interrupt flag for the EXTI line
    /// associated with this pin. This should typically be called from
    /// the interrupt handler after processing an interrupt.
    pub fn clear_pending(&mut self) {
        let pin_num = self.pin.pin.pin.pin();
        low_level::clear_exti_pending(pin_num);
    }

    /// Checks if an interrupt is pending for the current pin
    ///
    /// This method checks if there is a pending interrupt on the EXTI line
    /// associated with this pin.
    ///
    /// # Returns
    /// `true` if an interrupt is pending, `false` otherwise
    pub fn is_pending(&self) -> bool {
        let pin_num = self.pin.pin.pin.pin();
        low_level::is_exti_pending(pin_num)
    }

    /// Read the current pin level and checks if it is high
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Read the current pin level and checks if it is low
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Read the pin level
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    fn pin_mask(&self) -> u32 {
        1 << self.pin.pin.pin.pin()
    }
}

/// Pre-computed bitmask for clearing multiple EXTI interrupt lines at once.
///
/// This type is intended for use with shared EXTI IRQ handlers
/// (for example `EXTI15_10` or `EXTI9_5`), where multiple pins share
/// a single interrupt vector.
///
/// Instead of checking and clearing each pin individually in your interrupt
/// handler, you can pre-compute a mask during initialization and use it to
/// clear all pending flags with a single register write.
pub struct ExtiGroupMask(u32);

impl ExtiGroupMask {
    /// Creates a new EXTI group mask from a list of inputs.
    ///
    /// The mask is computed once and can be reused inside an interrupt
    /// handler for fast clearing of pending flags.
    ///
    /// # Benefits
    /// * **Reduced verbosity**: Clear all grouped pins with a single call
    ///   instead of checking and clearing each pin individually.
    /// * **Performance**: Clears all pending flags in one register write
    ///   rather than multiple critical sections.
    ///
    /// # Parameters
    /// * `inputs` - Slice of references to [`ExtiInput`] instances that
    ///   belong to the same EXTI IRQ group.
    ///
    /// # Safety Notes
    /// * All inputs **must** map to the same EXTI IRQ group (e.g. all pins
    ///   in the range 10–15 for `EXTI15_10` handler).
    /// * Mixing pins from different IRQ groups is not validated and will
    ///   result in the mask only clearing flags for pins handled by the
    ///   current interrupt vector.
    /// * Consult your STM32 reference manual for the EXTI line to IRQ
    ///   vector mapping for your specific chip.
    pub fn new(inputs: &[&ExtiInput]) -> Self {
        let mut mask = 0;
        for input in inputs {
            mask |= input.pin_mask();
        }
        Self(mask)
    }

    /// Clears all pending EXTI interrupt flags contained in this mask.
    ///
    /// This performs a single write to the EXTI pending register,
    /// making it suitable for use inside interrupt handlers.
    pub fn clear(&self) {
        critical_section::with(|_| low_level::clear_exti_pending_mask(self.0))
    }
}
