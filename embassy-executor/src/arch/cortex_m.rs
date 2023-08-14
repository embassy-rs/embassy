#[cfg(feature = "executor-thread")]
pub use thread::*;

#[cfg(feature = "executor-thread")]
mod thread {

    #[cfg(feature = "nightly")]
    pub use embassy_macros::main_cortex_m as main;

    use crate::raw::OpaqueThreadContext;
    use crate::thread::ThreadContext;

    #[export_name = "__thread_mode_pender"]
    fn __thread_mode_pender(_context: OpaqueThreadContext) {
        unsafe { core::arch::asm!("sev") }
    }

    /// TODO
    // Name pending
    #[derive(Default)] // Default enables Executor::new
    pub struct Context;

    impl ThreadContext for Context {
        fn context(&self) -> OpaqueThreadContext {
            OpaqueThreadContext(0)
        }

        fn wait(&mut self) {
            unsafe { core::arch::asm!("wfe") }
        }
    }

    /// TODO
    // Type alias for backwards compatibility
    pub type Executor = crate::thread::ThreadModeExecutor<Context>;
}

// None of this has to be public, I guess?
#[cfg(feature = "executor-interrupt")]
pub use interrupt::*;
#[cfg(feature = "executor-interrupt")]
mod interrupt {
    use cortex_m::interrupt::InterruptNumber;
    use cortex_m::peripheral::NVIC;

    use crate::interrupt::InterruptContext;
    use crate::raw::OpaqueInterruptContext;

    #[derive(Clone, Copy)]
    struct CortexMInterruptContext(u16);

    unsafe impl cortex_m::interrupt::InterruptNumber for CortexMInterruptContext {
        fn number(self) -> u16 {
            self.0
        }
    }

    impl<T> InterruptContext for T
    where
        T: InterruptNumber,
    {
        fn context(&self) -> OpaqueInterruptContext {
            OpaqueInterruptContext(self.number() as usize)
        }

        fn enable(&self) {
            unsafe { NVIC::unmask(*self) }
        }
    }

    #[export_name = "__interrupt_mode_pender"]
    fn __interrupt_mode_pender(interrupt: OpaqueInterruptContext) {
        let interrupt = CortexMInterruptContext(unsafe { core::mem::transmute::<_, usize>(interrupt) as u16 });

        // STIR is faster, but is only available in v7 and higher.
        #[cfg(not(armv6m))]
        {
            let mut nvic: NVIC = unsafe { core::mem::transmute(()) };
            nvic.request(interrupt);
        }

        #[cfg(armv6m)]
        NVIC::pend(interrupt);
    }

    /// TODO
    // Type alias for backwards compatibility
    pub type InterruptExecutor = crate::interrupt::InterruptModeExecutor;
}
