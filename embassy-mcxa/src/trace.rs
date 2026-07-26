#[cfg(feature = "trace")]
unsafe extern "Rust" {
    /// An interrupt has started executing
    safe fn _embassy_mcxa_trace_irq_start(irq: u16);
    /// An interrupt is done executing
    safe fn _embassy_mcxa_trace_irq_end(irq: u16);
}

#[doc(hidden)]
pub fn irq_start(_irq: crate::interrupt::Interrupt) {
    #[cfg(feature = "trace")]
    {
        use cortex_m::interrupt::InterruptNumber;
        _embassy_mcxa_trace_irq_start(_irq.number());
    }
}

#[doc(hidden)]
pub fn irq_end(_irq: crate::interrupt::Interrupt) {
    #[cfg(feature = "trace")]
    {
        use cortex_m::interrupt::InterruptNumber;
        _embassy_mcxa_trace_irq_end(_irq.number());
    }
}
