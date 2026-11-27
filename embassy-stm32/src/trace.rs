//! Interrupt tracing support for the STM32 time driver and `bind_interrupts!`.
//!
//! The IRQ entry and exit callbacks follow the Rust ABI convention used by `embassy-mcxa`,
//! with symbols specific to the STM32 HAL.

#[cfg(feature = "trace")]
unsafe extern "Rust" {
    /// An interrupt has started executing
    safe fn _embassy_stm32_trace_irq_start(irq: u16);
    /// An interrupt is done executing
    safe fn _embassy_stm32_trace_irq_end(irq: u16);
}

#[doc(hidden)]
#[inline(always)]
pub fn irq_start(_irq: crate::interrupt::Interrupt) {
    #[cfg(feature = "trace")]
    {
        use cortex_m::interrupt::InterruptNumber;
        _embassy_stm32_trace_irq_start(_irq.number());
    }
}

#[doc(hidden)]
#[inline(always)]
pub fn irq_end(_irq: crate::interrupt::Interrupt) {
    #[cfg(feature = "trace")]
    {
        use cortex_m::interrupt::InterruptNumber;
        _embassy_stm32_trace_irq_end(_irq.number());
    }
}
