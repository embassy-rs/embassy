use core::cell::UnsafeCell;
use cortex_m::interrupt::CriticalSection;

use crate::fmt::{assert, panic, *};

/// A "mutex" based on critical sections
///
/// # Safety
///
/// **This Mutex is only safe on single-core systems.**
///
/// On multi-core systems, a `CriticalSection` **is not sufficient** to ensure exclusive access.
pub struct CriticalSectionMutex<T> {
    inner: UnsafeCell<T>,
}
unsafe impl<T> Sync for CriticalSectionMutex<T> {}
unsafe impl<T> Send for CriticalSectionMutex<T> {}

impl<T> CriticalSectionMutex<T> {
    /// Creates a new mutex
    pub const fn new(value: T) -> Self {
        CriticalSectionMutex {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> CriticalSectionMutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&'cs self, _cs: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

/// A "mutex" that only allows borrowing from thread mode.
///
/// # Safety
///
/// **This Mutex is only safe on single-core systems.**
///
/// On multi-core systems, a `ThreadModeMutex` **is not sufficient** to ensure exclusive access.
pub struct ThreadModeMutex<T> {
    inner: UnsafeCell<T>,
}
unsafe impl<T> Sync for ThreadModeMutex<T> {}
unsafe impl<T> Send for ThreadModeMutex<T> {}

impl<T> ThreadModeMutex<T> {
    /// Creates a new mutex
    pub const fn new(value: T) -> Self {
        ThreadModeMutex {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> ThreadModeMutex<T> {
    /// Borrows the data
    pub fn borrow(&self) -> &T {
        assert!(
            in_thread_mode(),
            "ThreadModeMutex can only be borrowed from thread mode."
        );
        unsafe { &*self.inner.get() }
    }
}

pub fn in_thread_mode() -> bool {
    #[cfg(feature = "std")]
    return Some("main") == std::thread::current().name();

    #[cfg(not(feature = "std"))]
    return cortex_m::peripheral::SCB::vect_active()
        == cortex_m::peripheral::scb::VectActive::ThreadMode;
}
