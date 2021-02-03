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

// NOTE: A `CriticalSectionMutex` can be used as a channel so the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. access tokens) across different
// execution contexts (e.g. interrupts)
unsafe impl<T> Sync for CriticalSectionMutex<T> where T: Send {}

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

// NOTE: ThreadModeMutex only allows borrowing from one execution context ever: thread mode.
// Therefore it cannot be used to send non-sendable stuff between execution contexts, so it can
// be Send+Sync even if T is not Send (unlike CriticalSectionMutex)
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
