use core::ptr;
use cortex_m::peripheral::NVIC;

use atomic_polyfill::{AtomicPtr, Ordering};

pub use embassy_macros::interrupt_declare as declare;
pub use embassy_macros::interrupt_take as take;

/// Implementation detail, do not use outside embassy crates.
#[doc(hidden)]
pub struct Handler {
    pub func: AtomicPtr<()>,
    pub ctx: AtomicPtr<()>,
}

impl Handler {
    pub const fn new() -> Self {
        Self {
            func: AtomicPtr::new(ptr::null_mut()),
            ctx: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NrWrap(pub(crate) u16);
unsafe impl cortex_m::interrupt::InterruptNumber for NrWrap {
    fn number(self) -> u16 {
        self.0
    }
}

pub unsafe trait Interrupt {
    type Priority: From<u8> + Into<u8> + Copy;
    fn number(&self) -> u16;
    unsafe fn steal() -> Self;

    /// Implementation detail, do not use outside embassy crates.
    #[doc(hidden)]
    unsafe fn __handler(&self) -> &'static Handler;
}

pub trait InterruptExt: Interrupt {
    fn set_handler(&self, func: unsafe fn(*mut ()));
    fn remove_handler(&self);
    fn set_handler_context(&self, ctx: *mut ());
    fn enable(&self);
    fn disable(&self);
    #[cfg(not(armv6m))]
    fn is_active(&self) -> bool;
    fn is_enabled(&self) -> bool;
    fn is_pending(&self) -> bool;
    fn pend(&self);
    fn unpend(&self);
    fn get_priority(&self) -> Self::Priority;
    fn set_priority(&self, prio: Self::Priority);
}

impl<T: Interrupt + ?Sized> InterruptExt for T {
    fn set_handler(&self, func: unsafe fn(*mut ())) {
        let handler = unsafe { self.__handler() };
        handler.func.store(func as *mut (), Ordering::Release);
    }

    fn remove_handler(&self) {
        let handler = unsafe { self.__handler() };
        handler.func.store(ptr::null_mut(), Ordering::Release);
    }

    fn set_handler_context(&self, ctx: *mut ()) {
        let handler = unsafe { self.__handler() };
        handler.ctx.store(ctx, Ordering::Release);
    }

    #[inline]
    fn enable(&self) {
        unsafe {
            NVIC::unmask(NrWrap(self.number()));
        }
    }

    #[inline]
    fn disable(&self) {
        NVIC::mask(NrWrap(self.number()));
    }

    #[inline]
    #[cfg(not(armv6m))]
    fn is_active(&self) -> bool {
        NVIC::is_active(NrWrap(self.number()))
    }

    #[inline]
    fn is_enabled(&self) -> bool {
        NVIC::is_enabled(NrWrap(self.number()))
    }

    #[inline]
    fn is_pending(&self) -> bool {
        NVIC::is_pending(NrWrap(self.number()))
    }

    #[inline]
    fn pend(&self) {
        NVIC::pend(NrWrap(self.number()))
    }

    #[inline]
    fn unpend(&self) {
        NVIC::unpend(NrWrap(self.number()))
    }

    #[inline]
    fn get_priority(&self) -> Self::Priority {
        Self::Priority::from(NVIC::get_priority(NrWrap(self.number())))
    }

    #[inline]
    fn set_priority(&self, prio: Self::Priority) {
        unsafe {
            cortex_m::peripheral::Peripherals::steal()
                .NVIC
                .set_priority(NrWrap(self.number()), prio.into())
        }
    }
}

pub struct InterruptTable {}

impl InterruptTable {
    /*
        Register the interrupt table with the NVIC
    */
    pub fn register_table() {
        // unsafe { p.core.SCB.vtor.write(0x4000) }
    }

    /*
        JIT a trampoline function, given a fn and ctx
    */
    pub fn generate_fn(func: unsafe fn(*mut ()), ctx: *mut ()) -> [u8] {}

    /*
        Set the handler in the interrupt table
    */
    pub fn set_handler(irq: u8, func: unsafe fn(*mut ())) {}

    /*
        Remove the handler in the interrupt table
    */
    pub fn remove_handler(irq: u8) {}
}
