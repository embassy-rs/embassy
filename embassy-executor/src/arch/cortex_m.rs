#[cfg(feature = "executor-thread")]
pub use thread::*;

use crate::raw::PenderContext;

#[cfg(feature = "executor-interrupt")]

/// # Safety
///
/// `irq` must be a valid interrupt request number
unsafe fn nvic_pend(irq: u16) {
    use cortex_m::interrupt::InterruptNumber;

    #[derive(Clone, Copy)]
    struct Irq(u16);
    unsafe impl InterruptNumber for Irq {
        fn number(self) -> u16 {
            self.0
        }
    }

    let irq = Irq(irq);

    // STIR is faster, but is only available in v7 and higher.
    #[cfg(not(armv6m))]
    {
        let mut nvic: cortex_m::peripheral::NVIC = unsafe { core::mem::transmute(()) };
        nvic.request(irq);
    }

    #[cfg(armv6m)]
    cortex_m::peripheral::NVIC::pend(irq);
}

#[cfg(all(feature = "executor-thread", feature = "executor-interrupt"))]
#[export_name = "__pender"]
fn __pender(context: PenderContext) {
    unsafe {
        let context: usize = core::mem::transmute(context);
        // Safety: `context` is either `usize::MAX` created by `Executor::run`, or a valid interrupt
        // request number given to `InterruptExecutor::start`.
        if context as usize == usize::MAX {
            core::arch::asm!("sev")
        } else {
            nvic_pend(context as u16)
        }
    }
}

#[cfg(all(feature = "executor-thread", not(feature = "executor-interrupt")))]
#[export_name = "__pender"]
fn __pender(_context: PenderContext) {
    unsafe { core::arch::asm!("sev") }
}

#[cfg(all(not(feature = "executor-thread"), feature = "executor-interrupt"))]
#[export_name = "__pender"]
fn __pender(context: PenderContext) {
    unsafe {
        let context: usize = core::mem::transmute(context);
        // Safety: `context` is the same value we passed to `InterruptExecutor::start`, which must
        // be a valid interrupt request number.
        nvic_pend(context as u16)
    }
}

#[cfg(feature = "executor-thread")]
mod thread {

    #[cfg(feature = "nightly")]
    pub use embassy_macros::main_cortex_m as main;

    use crate::raw::PenderContext;
    use crate::thread::ThreadContext;

    /// TODO
    // Name pending
    #[derive(Default)] // Default enables Executor::new
    pub struct Context;

    impl ThreadContext for Context {
        fn context(&self) -> PenderContext {
            unsafe { core::mem::transmute(usize::MAX) }
        }

        fn wait(&mut self) {
            unsafe { core::arch::asm!("wfe") }
        }
    }

    /// TODO
    // Type alias for backwards compatibility
    pub type Executor = crate::thread::ThreadModeExecutor<Context>;
}

#[cfg(feature = "executor-interrupt")]
pub use interrupt::*;
#[cfg(feature = "executor-interrupt")]
mod interrupt {
    use cortex_m::interrupt::InterruptNumber;
    use cortex_m::peripheral::NVIC;

    use crate::interrupt::InterruptContext;
    use crate::raw::PenderContext;

    impl<T> InterruptContext for T
    where
        T: InterruptNumber,
    {
        fn context(&self) -> PenderContext {
            unsafe { core::mem::transmute(self.number() as usize) }
        }

        fn enable(&self) {
            unsafe { NVIC::unmask(*self) }
        }
    }

    /// TODO
    // Type alias for backwards compatibility
    pub type InterruptExecutor = crate::interrupt::InterruptModeExecutor;
}
