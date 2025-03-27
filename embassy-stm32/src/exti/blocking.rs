//! Blocking EXTI (External Interrupt) functionality
//!
//! This module provides a non-async version of the EXTI input driver designed
//! for use with custom interrupt handlers or frameworks like RTIC.
use super::low_level::{self, TriggerEdge};
use super::Channel;
use crate::gpio::{Input, Level, Pin as GpioPin, Pull};
use crate::Peri;

/// EXTI input driver for custom interrupt handlers or frameworks like RTIC.
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
    /// * `ch` - The EXTI channel corresponding to the pin
    /// * `pull` - The pull configuration for the pin
    /// * `trigger_edge` - The edge triggering mode (falling, rising, or both)
    ///
    /// # Returns
    /// A new `ExtiInput` instance with interrupts enabled
    pub fn new<T: GpioPin>(
        pin: Peri<'d, T>,
        ch: Peri<'d, T::ExtiChannel>,
        pull: Pull,
        trigger_edge: TriggerEdge,
    ) -> Self {
        // Needed if using AnyPin+AnyChannel.
        assert_eq!(pin.pin(), ch.number());

        let pin_num = pin.pin();
        let port = pin.port();

        low_level::configure_and_enable_exti(pin_num, port, trigger_edge);

        Self {
            pin: Input::new(pin, pull),
        }
    }

    /// Configures the EXTI line to detect rising edges
    ///
    /// After calling this method, the EXTI line will generate interrupts
    /// on rising edges of the input signal.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn rising_edge(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        let port_num = self.pin.pin.pin.port();
        low_level::configure_exti_pin(pin_num, port_num, TriggerEdge::Rising);
        self
    }

    /// Configures the EXTI line to detect falling edges
    ///
    /// After calling this method, the EXTI line will generate interrupts
    /// on falling edges of the input signal.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn falling_edge(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        let port_num = self.pin.pin.pin.port();
        low_level::configure_exti_pin(pin_num, port_num, TriggerEdge::Falling);
        self
    }

    /// Configures the EXTI line to detect both rising and falling edges
    ///
    /// After calling this method, the EXTI line will generate interrupts
    /// on both rising and falling edges of the input signal.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn any_edge(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        let port_num = self.pin.pin.pin.port();
        low_level::configure_exti_pin(pin_num, port_num, TriggerEdge::Any);
        self
    }

    /// Enables the EXTI interrupt for this pin
    ///
    /// After calling this method, interrupts will be generated based on
    /// the configured edge detection settings.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn enable(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        low_level::enable_exti_interrupt(pin_num);
        self
    }

    /// Disables the EXTI interrupt for this pin
    ///
    /// After calling this method, no interrupts will be generated.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn disable(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        low_level::disable_exti_interrupt(pin_num);
        self
    }

    /// Clears any pending interrupt for this pin
    ///
    /// This method clears the pending interrupt flag for the EXTI line
    /// associated with this pin. This should typically be called from
    /// the interrupt handler after processing an interrupt.
    ///
    /// # Returns
    /// A mutable reference to self for method chaining
    pub fn clear_pending(&mut self) -> &mut Self {
        let pin_num = self.pin.pin.pin.pin();
        low_level::clear_exti_pending(pin_num);
        self
    }

    /// Checks if an interrupt is pending for this pin
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

    /// Get the current pin level
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get the current pin level
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }
}
