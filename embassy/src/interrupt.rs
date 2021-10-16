use atomic_polyfill::{compiler_fence, AtomicPtr, Ordering};
use core::mem;
use core::ptr;
use cortex_m::peripheral::NVIC;

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

pub unsafe trait Interrupt: crate::util::Unborrow<Target = Self> {
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
        compiler_fence(Ordering::SeqCst);
        let handler = unsafe { self.__handler() };
        handler.func.store(func as *mut (), Ordering::Relaxed);
        compiler_fence(Ordering::SeqCst);
    }

    fn remove_handler(&self) {
        compiler_fence(Ordering::SeqCst);
        let handler = unsafe { self.__handler() };
        handler.func.store(ptr::null_mut(), Ordering::Relaxed);
        compiler_fence(Ordering::SeqCst);
    }

    fn set_handler_context(&self, ctx: *mut ()) {
        let handler = unsafe { self.__handler() };
        handler.ctx.store(ctx, Ordering::Relaxed);
    }

    #[inline]
    fn enable(&self) {
        compiler_fence(Ordering::SeqCst);
        unsafe {
            NVIC::unmask(NrWrap(self.number()));
        }
    }

    #[inline]
    fn disable(&self) {
        NVIC::mask(NrWrap(self.number()));
        compiler_fence(Ordering::SeqCst);
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
            let mut nvic: cortex_m::peripheral::NVIC = mem::transmute(());
            nvic.set_priority(NrWrap(self.number()), prio.into())
        }
    }
}
