use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::task::Waker;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one without waking it.
/// The intended use case is to wake tasks from interrupts. Therefore, it is generally not expected,
/// that multiple tasks register try to register a waker simultaneously.
pub struct AtomicWaker {
    waker: AtomicPtr<()>,
}

impl AtomicWaker {
    /// Create a new `AtomicWaker`.
    pub const fn new() -> Self {
        Self {
            waker: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        self.waker.store(w.as_turbo_ptr().as_ptr() as _, Ordering::Release);
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        if let Some(ptr) = NonNull::new(self.waker.load(Ordering::Acquire)) {
            unsafe { Waker::from_turbo_ptr(ptr) }.wake();
        }
    }
}
