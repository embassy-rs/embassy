#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]

#[cfg(not(any(
feature = "55",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: 55");

#[cfg(feature = "55")]
pub use stm32wb_hal as hal;

#[cfg(feature = "55")]
pub use stm32wb_hal::pac as pac;

/// Creates a new interrupt waking a [`Waker`].
///
/// As this interrupt will be declared in this macro, it can't be used for anything else.
///
/// # Examples
///
/// This macro is useful for implementing [`Future::poll`]:
///
/// ```
/// fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
///     if self.is_ready() {
///         Poll::Ready(())
///     } else {
///         waker_interrupt!(TIM2, cx.waker().clone());
///         Poll::Pending
///     }
/// }
/// ```
///
/// [`Waker`]: core::task::Waker
/// [`Future::poll`]: core::future::Future::poll
macro_rules! waker_interrupt {
    ($INT:ident, $waker:expr) => {{
        use core::sync::atomic::{self, Ordering};
        use stm32wb_hal::pac::{interrupt, Interrupt, NVIC};

        static mut WAKER: Option<Waker> = None;

        #[interrupt]
        fn $INT() {
            // Safety: This context is disabled while the lower priority context accesses WAKER
            if let Some(waker) = unsafe { WAKER.as_ref() } {
                waker.wake_by_ref();

                NVIC::mask(Interrupt::$INT);
            }
        }

        NVIC::mask(Interrupt::$INT);
        atomic::compiler_fence(Ordering::Acquire);
        // Safety: The other relevant context, the interrupt, is disabled
        unsafe { WAKER = Some($waker) }
        NVIC::unpend(Interrupt::$INT);
        atomic::compiler_fence(Ordering::Release);
        // Safety: This is the end of a mask-based critical section
        unsafe { NVIC::unmask(Interrupt::$INT) }
    }};
}

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod i2c;

pub use cortex_m_rt::interrupt;
