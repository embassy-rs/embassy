use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use cortex_m::peripheral::NVIC;

pub use embassy_macros::interrupt_declare as declare;
pub use embassy_macros::interrupt_take as take;

struct NrWrap(u8);
unsafe impl cortex_m::interrupt::Nr for NrWrap {
    fn nr(&self) -> u8 {
        self.0
    }
}

pub unsafe trait OwnedInterrupt {
    type Priority: From<u8> + Into<u8> + Copy;
    fn number(&self) -> u8;
    #[doc(hidden)]
    unsafe fn __handler(&self) -> &'static AtomicPtr<u32>;

    fn set_handler(&self, handler: unsafe fn()) {
        unsafe { self.__handler() }.store(handler as *mut u32, Ordering::Release);
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
