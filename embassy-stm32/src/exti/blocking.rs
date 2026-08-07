//! Blocking-mode EXTI utilities
//!
//! This module provides types and functions for manual EXTI interrupt handling.
//! Use these when you need direct control over EXTI interrupts outside of
//! Embassy's async executor.
//!
//! The main blocking EXTI driver is [`ExtiInput<Blocking>`](super::ExtiInput).

use super::{ExtiInput, low_level};
use crate::mode::Blocking;

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
    /// * `inputs` - Slice of references to [`ExtiInput<Blocking>`](super::ExtiInput) instances that
    ///   belong to the same EXTI IRQ group.
    ///
    /// # Safety Notes
    /// * All inputs **must** map to the same EXTI IRQ group (for instance all pins
    ///   in the range 10–15 for `EXTI15_10` handler).
    /// * Mixing pins from different IRQ groups will cause `clear()` to
    ///   incorrectly clear pending flags belonging to other interrupt vectors,
    ///   potentially causing missed interrupts.
    /// * Consult your STM32 reference manual for the EXTI line to IRQ
    ///   vector mapping for your specific chip.
    pub fn new(inputs: &[&ExtiInput<Blocking>]) -> Self {
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
