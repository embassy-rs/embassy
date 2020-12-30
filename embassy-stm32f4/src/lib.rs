#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]
#![feature(let_chains)]

#[cfg(not(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479",
)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features: "
);

#[cfg(any(
    all(feature = "stm32f401", feature = "stm32f405"),
    all(feature = "stm32f401", feature = "stm32f407"),
    all(feature = "stm32f401", feature = "stm32f410"),
    all(feature = "stm32f401", feature = "stm32f411"),
    all(feature = "stm32f401", feature = "stm32f412"),
    all(feature = "stm32f401", feature = "stm32f413"),
    all(feature = "stm32f401", feature = "stm32f415"),
    all(feature = "stm32f401", feature = "stm32f417"),
    all(feature = "stm32f401", feature = "stm32f423"),
    all(feature = "stm32f401", feature = "stm32f427"),
    all(feature = "stm32f401", feature = "stm32f429"),
    all(feature = "stm32f401", feature = "stm32f437"),
    all(feature = "stm32f401", feature = "stm32f439"),
    all(feature = "stm32f401", feature = "stm32f446"),
    all(feature = "stm32f401", feature = "stm32f469"),
    all(feature = "stm32f401", feature = "stm32f479"),
    all(feature = "stm32f405", feature = "stm32f401"),
    all(feature = "stm32f405", feature = "stm32f407"),
    all(feature = "stm32f405", feature = "stm32f410"),
    all(feature = "stm32f405", feature = "stm32f411"),
    all(feature = "stm32f405", feature = "stm32f412"),
    all(feature = "stm32f405", feature = "stm32f413"),
    all(feature = "stm32f405", feature = "stm32f415"),
    all(feature = "stm32f405", feature = "stm32f417"),
    all(feature = "stm32f405", feature = "stm32f423"),
    all(feature = "stm32f405", feature = "stm32f427"),
    all(feature = "stm32f405", feature = "stm32f429"),
    all(feature = "stm32f405", feature = "stm32f437"),
    all(feature = "stm32f405", feature = "stm32f439"),
    all(feature = "stm32f405", feature = "stm32f446"),
    all(feature = "stm32f405", feature = "stm32f469"),
    all(feature = "stm32f405", feature = "stm32f479"),
    all(feature = "stm32f407", feature = "stm32f401"),
    all(feature = "stm32f407", feature = "stm32f405"),
    all(feature = "stm32f407", feature = "stm32f410"),
    all(feature = "stm32f407", feature = "stm32f411"),
    all(feature = "stm32f407", feature = "stm32f412"),
    all(feature = "stm32f407", feature = "stm32f413"),
    all(feature = "stm32f407", feature = "stm32f415"),
    all(feature = "stm32f407", feature = "stm32f417"),
    all(feature = "stm32f407", feature = "stm32f423"),
    all(feature = "stm32f407", feature = "stm32f427"),
    all(feature = "stm32f407", feature = "stm32f429"),
    all(feature = "stm32f407", feature = "stm32f437"),
    all(feature = "stm32f407", feature = "stm32f439"),
    all(feature = "stm32f407", feature = "stm32f446"),
    all(feature = "stm32f407", feature = "stm32f469"),
    all(feature = "stm32f407", feature = "stm32f479"),
    all(feature = "stm32f410", feature = "stm32f401"),
    all(feature = "stm32f410", feature = "stm32f405"),
    all(feature = "stm32f410", feature = "stm32f407"),
    all(feature = "stm32f410", feature = "stm32f411"),
    all(feature = "stm32f410", feature = "stm32f412"),
    all(feature = "stm32f410", feature = "stm32f413"),
    all(feature = "stm32f410", feature = "stm32f415"),
    all(feature = "stm32f410", feature = "stm32f417"),
    all(feature = "stm32f410", feature = "stm32f423"),
    all(feature = "stm32f410", feature = "stm32f427"),
    all(feature = "stm32f410", feature = "stm32f429"),
    all(feature = "stm32f410", feature = "stm32f437"),
    all(feature = "stm32f410", feature = "stm32f439"),
    all(feature = "stm32f410", feature = "stm32f446"),
    all(feature = "stm32f410", feature = "stm32f469"),
    all(feature = "stm32f410", feature = "stm32f479"),
    all(feature = "stm32f411", feature = "stm32f401"),
    all(feature = "stm32f411", feature = "stm32f405"),
    all(feature = "stm32f411", feature = "stm32f407"),
    all(feature = "stm32f411", feature = "stm32f410"),
    all(feature = "stm32f411", feature = "stm32f412"),
    all(feature = "stm32f411", feature = "stm32f413"),
    all(feature = "stm32f411", feature = "stm32f415"),
    all(feature = "stm32f411", feature = "stm32f417"),
    all(feature = "stm32f411", feature = "stm32f423"),
    all(feature = "stm32f411", feature = "stm32f427"),
    all(feature = "stm32f411", feature = "stm32f429"),
    all(feature = "stm32f411", feature = "stm32f437"),
    all(feature = "stm32f411", feature = "stm32f439"),
    all(feature = "stm32f411", feature = "stm32f446"),
    all(feature = "stm32f411", feature = "stm32f469"),
    all(feature = "stm32f411", feature = "stm32f479"),
    all(feature = "stm32f412", feature = "stm32f401"),
    all(feature = "stm32f412", feature = "stm32f405"),
    all(feature = "stm32f412", feature = "stm32f407"),
    all(feature = "stm32f412", feature = "stm32f410"),
    all(feature = "stm32f412", feature = "stm32f411"),
    all(feature = "stm32f412", feature = "stm32f413"),
    all(feature = "stm32f412", feature = "stm32f415"),
    all(feature = "stm32f412", feature = "stm32f417"),
    all(feature = "stm32f412", feature = "stm32f423"),
    all(feature = "stm32f412", feature = "stm32f427"),
    all(feature = "stm32f412", feature = "stm32f429"),
    all(feature = "stm32f412", feature = "stm32f437"),
    all(feature = "stm32f412", feature = "stm32f439"),
    all(feature = "stm32f412", feature = "stm32f446"),
    all(feature = "stm32f412", feature = "stm32f469"),
    all(feature = "stm32f412", feature = "stm32f479"),
    all(feature = "stm32f413", feature = "stm32f401"),
    all(feature = "stm32f413", feature = "stm32f405"),
    all(feature = "stm32f413", feature = "stm32f407"),
    all(feature = "stm32f413", feature = "stm32f410"),
    all(feature = "stm32f413", feature = "stm32f411"),
    all(feature = "stm32f413", feature = "stm32f412"),
    all(feature = "stm32f413", feature = "stm32f415"),
    all(feature = "stm32f413", feature = "stm32f417"),
    all(feature = "stm32f413", feature = "stm32f423"),
    all(feature = "stm32f413", feature = "stm32f427"),
    all(feature = "stm32f413", feature = "stm32f429"),
    all(feature = "stm32f413", feature = "stm32f437"),
    all(feature = "stm32f413", feature = "stm32f439"),
    all(feature = "stm32f413", feature = "stm32f446"),
    all(feature = "stm32f413", feature = "stm32f469"),
    all(feature = "stm32f413", feature = "stm32f479"),
    all(feature = "stm32f415", feature = "stm32f401"),
    all(feature = "stm32f415", feature = "stm32f405"),
    all(feature = "stm32f415", feature = "stm32f407"),
    all(feature = "stm32f415", feature = "stm32f410"),
    all(feature = "stm32f415", feature = "stm32f411"),
    all(feature = "stm32f415", feature = "stm32f412"),
    all(feature = "stm32f415", feature = "stm32f413"),
    all(feature = "stm32f415", feature = "stm32f417"),
    all(feature = "stm32f415", feature = "stm32f423"),
    all(feature = "stm32f415", feature = "stm32f427"),
    all(feature = "stm32f415", feature = "stm32f429"),
    all(feature = "stm32f415", feature = "stm32f437"),
    all(feature = "stm32f415", feature = "stm32f439"),
    all(feature = "stm32f415", feature = "stm32f446"),
    all(feature = "stm32f415", feature = "stm32f469"),
    all(feature = "stm32f415", feature = "stm32f479"),
    all(feature = "stm32f417", feature = "stm32f401"),
    all(feature = "stm32f417", feature = "stm32f405"),
    all(feature = "stm32f417", feature = "stm32f407"),
    all(feature = "stm32f417", feature = "stm32f410"),
    all(feature = "stm32f417", feature = "stm32f411"),
    all(feature = "stm32f417", feature = "stm32f412"),
    all(feature = "stm32f417", feature = "stm32f413"),
    all(feature = "stm32f417", feature = "stm32f415"),
    all(feature = "stm32f417", feature = "stm32f423"),
    all(feature = "stm32f417", feature = "stm32f427"),
    all(feature = "stm32f417", feature = "stm32f429"),
    all(feature = "stm32f417", feature = "stm32f437"),
    all(feature = "stm32f417", feature = "stm32f439"),
    all(feature = "stm32f417", feature = "stm32f446"),
    all(feature = "stm32f417", feature = "stm32f469"),
    all(feature = "stm32f417", feature = "stm32f479"),
    all(feature = "stm32f423", feature = "stm32f401"),
    all(feature = "stm32f423", feature = "stm32f405"),
    all(feature = "stm32f423", feature = "stm32f407"),
    all(feature = "stm32f423", feature = "stm32f410"),
    all(feature = "stm32f423", feature = "stm32f411"),
    all(feature = "stm32f423", feature = "stm32f412"),
    all(feature = "stm32f423", feature = "stm32f413"),
    all(feature = "stm32f423", feature = "stm32f415"),
    all(feature = "stm32f423", feature = "stm32f417"),
    all(feature = "stm32f423", feature = "stm32f427"),
    all(feature = "stm32f423", feature = "stm32f429"),
    all(feature = "stm32f423", feature = "stm32f437"),
    all(feature = "stm32f423", feature = "stm32f439"),
    all(feature = "stm32f423", feature = "stm32f446"),
    all(feature = "stm32f423", feature = "stm32f469"),
    all(feature = "stm32f423", feature = "stm32f479"),
    all(feature = "stm32f427", feature = "stm32f401"),
    all(feature = "stm32f427", feature = "stm32f405"),
    all(feature = "stm32f427", feature = "stm32f407"),
    all(feature = "stm32f427", feature = "stm32f410"),
    all(feature = "stm32f427", feature = "stm32f411"),
    all(feature = "stm32f427", feature = "stm32f412"),
    all(feature = "stm32f427", feature = "stm32f413"),
    all(feature = "stm32f427", feature = "stm32f415"),
    all(feature = "stm32f427", feature = "stm32f417"),
    all(feature = "stm32f427", feature = "stm32f423"),
    all(feature = "stm32f427", feature = "stm32f429"),
    all(feature = "stm32f427", feature = "stm32f437"),
    all(feature = "stm32f427", feature = "stm32f439"),
    all(feature = "stm32f427", feature = "stm32f446"),
    all(feature = "stm32f427", feature = "stm32f469"),
    all(feature = "stm32f427", feature = "stm32f479"),
    all(feature = "stm32f429", feature = "stm32f401"),
    all(feature = "stm32f429", feature = "stm32f405"),
    all(feature = "stm32f429", feature = "stm32f407"),
    all(feature = "stm32f429", feature = "stm32f410"),
    all(feature = "stm32f429", feature = "stm32f411"),
    all(feature = "stm32f429", feature = "stm32f412"),
    all(feature = "stm32f429", feature = "stm32f413"),
    all(feature = "stm32f429", feature = "stm32f415"),
    all(feature = "stm32f429", feature = "stm32f417"),
    all(feature = "stm32f429", feature = "stm32f423"),
    all(feature = "stm32f429", feature = "stm32f427"),
    all(feature = "stm32f429", feature = "stm32f437"),
    all(feature = "stm32f429", feature = "stm32f439"),
    all(feature = "stm32f429", feature = "stm32f446"),
    all(feature = "stm32f429", feature = "stm32f469"),
    all(feature = "stm32f429", feature = "stm32f479"),
    all(feature = "stm32f437", feature = "stm32f401"),
    all(feature = "stm32f437", feature = "stm32f405"),
    all(feature = "stm32f437", feature = "stm32f407"),
    all(feature = "stm32f437", feature = "stm32f410"),
    all(feature = "stm32f437", feature = "stm32f411"),
    all(feature = "stm32f437", feature = "stm32f412"),
    all(feature = "stm32f437", feature = "stm32f413"),
    all(feature = "stm32f437", feature = "stm32f415"),
    all(feature = "stm32f437", feature = "stm32f417"),
    all(feature = "stm32f437", feature = "stm32f423"),
    all(feature = "stm32f437", feature = "stm32f427"),
    all(feature = "stm32f437", feature = "stm32f429"),
    all(feature = "stm32f437", feature = "stm32f439"),
    all(feature = "stm32f437", feature = "stm32f446"),
    all(feature = "stm32f437", feature = "stm32f469"),
    all(feature = "stm32f437", feature = "stm32f479"),
    all(feature = "stm32f439", feature = "stm32f401"),
    all(feature = "stm32f439", feature = "stm32f405"),
    all(feature = "stm32f439", feature = "stm32f407"),
    all(feature = "stm32f439", feature = "stm32f410"),
    all(feature = "stm32f439", feature = "stm32f411"),
    all(feature = "stm32f439", feature = "stm32f412"),
    all(feature = "stm32f439", feature = "stm32f413"),
    all(feature = "stm32f439", feature = "stm32f415"),
    all(feature = "stm32f439", feature = "stm32f417"),
    all(feature = "stm32f439", feature = "stm32f423"),
    all(feature = "stm32f439", feature = "stm32f427"),
    all(feature = "stm32f439", feature = "stm32f429"),
    all(feature = "stm32f439", feature = "stm32f437"),
    all(feature = "stm32f439", feature = "stm32f446"),
    all(feature = "stm32f439", feature = "stm32f469"),
    all(feature = "stm32f439", feature = "stm32f479"),
    all(feature = "stm32f446", feature = "stm32f401"),
    all(feature = "stm32f446", feature = "stm32f405"),
    all(feature = "stm32f446", feature = "stm32f407"),
    all(feature = "stm32f446", feature = "stm32f410"),
    all(feature = "stm32f446", feature = "stm32f411"),
    all(feature = "stm32f446", feature = "stm32f412"),
    all(feature = "stm32f446", feature = "stm32f413"),
    all(feature = "stm32f446", feature = "stm32f415"),
    all(feature = "stm32f446", feature = "stm32f417"),
    all(feature = "stm32f446", feature = "stm32f423"),
    all(feature = "stm32f446", feature = "stm32f427"),
    all(feature = "stm32f446", feature = "stm32f429"),
    all(feature = "stm32f446", feature = "stm32f437"),
    all(feature = "stm32f446", feature = "stm32f439"),
    all(feature = "stm32f446", feature = "stm32f469"),
    all(feature = "stm32f446", feature = "stm32f479"),
    all(feature = "stm32f469", feature = "stm32f401"),
    all(feature = "stm32f469", feature = "stm32f405"),
    all(feature = "stm32f469", feature = "stm32f407"),
    all(feature = "stm32f469", feature = "stm32f410"),
    all(feature = "stm32f469", feature = "stm32f411"),
    all(feature = "stm32f469", feature = "stm32f412"),
    all(feature = "stm32f469", feature = "stm32f413"),
    all(feature = "stm32f469", feature = "stm32f415"),
    all(feature = "stm32f469", feature = "stm32f417"),
    all(feature = "stm32f469", feature = "stm32f423"),
    all(feature = "stm32f469", feature = "stm32f427"),
    all(feature = "stm32f469", feature = "stm32f429"),
    all(feature = "stm32f469", feature = "stm32f437"),
    all(feature = "stm32f469", feature = "stm32f439"),
    all(feature = "stm32f469", feature = "stm32f446"),
    all(feature = "stm32f469", feature = "stm32f479"),
    all(feature = "stm32f479", feature = "stm32f401"),
    all(feature = "stm32f479", feature = "stm32f405"),
    all(feature = "stm32f479", feature = "stm32f407"),
    all(feature = "stm32f479", feature = "stm32f410"),
    all(feature = "stm32f479", feature = "stm32f411"),
    all(feature = "stm32f479", feature = "stm32f412"),
    all(feature = "stm32f479", feature = "stm32f413"),
    all(feature = "stm32f479", feature = "stm32f415"),
    all(feature = "stm32f479", feature = "stm32f417"),
    all(feature = "stm32f479", feature = "stm32f423"),
    all(feature = "stm32f479", feature = "stm32f427"),
    all(feature = "stm32f479", feature = "stm32f429"),
    all(feature = "stm32f479", feature = "stm32f437"),
    all(feature = "stm32f479", feature = "stm32f439"),
    all(feature = "stm32f479", feature = "stm32f446"),
    all(feature = "stm32f479", feature = "stm32f469"),
))]
compile_error!(
    "Multile chip features activated. You must activate exactly one of the following features: "
);

pub use stm32f4xx_hal as hal;
pub use stm32f4xx_hal::stm32 as pac;

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
        use core::task::Waker;
        use stm32f4xx_hal::pac::{interrupt, Interrupt, NVIC};

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

pub mod serial;

pub use cortex_m_rt::interrupt;
