use core::ptr::{self, NonNull};
use core::task::Waker;

use atomic_polyfill::{AtomicPtr, Ordering};

use crate::executor::raw::{task_from_waker, wake_task, Task};

/// Utility struct to register and wake a waker.
#[derive(Debug)]
pub struct WakerRegistration {
    waker: Option<NonNull<Task>>,
}

impl WakerRegistration {
    pub const fn new() -> Self {
        Self { waker: None }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&mut self, w: &Waker) {
        let w = unsafe { task_from_waker(w) };
        match self.waker {
            // Optimization: If both the old and new Wakers wake the same task, do nothing.
            Some(w2) if w == w2 => {}
            Some(w2) => {
                // We had a waker registered for another task. Wake it, so the other task can
                // reregister itself if it's still interested.
                //
                // If two tasks are waiting on the same thing concurrently, this will cause them
                // to wake each other in a loop fighting over this WakerRegistration. This wastes
                // CPU but things will still work.
                //
                // If the user wants to have two tasks waiting on the same thing they should use
                // a more appropriate primitive that can store multiple wakers.

                unsafe { wake_task(w2) }
                self.waker = Some(w);
            }
            None => self.waker = Some(w),
        }
    }

    /// Wake the registered waker, if any.
    pub fn wake(&mut self) {
        if let Some(w) = self.waker.take() {
            unsafe { wake_task(w) }
        }
    }
}

pub struct AtomicWakerRegistration {
    waker: AtomicPtr<Task>,
}

impl AtomicWakerRegistration {
    pub const fn new() -> Self {
        Self {
            waker: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        let w = unsafe { task_from_waker(w) };
        let w2 = self.waker.swap(w.as_ptr(), Ordering::Relaxed);
        if !w2.is_null() && w2 != w.as_ptr() {
            unsafe { wake_task(NonNull::new_unchecked(w2)) };
        }
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        let w2 = self.waker.swap(ptr::null_mut(), Ordering::Relaxed);
        if !w2.is_null() {
            unsafe { wake_task(NonNull::new_unchecked(w2)) };
        }
    }
}
