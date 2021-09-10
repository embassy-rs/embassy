//! Blocking mutex (not async)

use core::cell::UnsafeCell;
use critical_section::CriticalSection;

/// Any object implementing this trait guarantees exclusive access to the data contained
/// within the mutex for the duration of the lock.
/// Adapted from https://github.com/rust-embedded/mutex-trait.
pub trait Mutex {
    /// Data protected by the mutex.
    type Data;

    fn new(data: Self::Data) -> Self;

    /// Creates a critical section and grants temporary access to the protected data.
    fn lock<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R;
}

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
    pub fn borrow<'cs>(&'cs self, _cs: CriticalSection<'cs>) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

impl<T> Mutex for CriticalSectionMutex<T> {
    type Data = T;

    fn new(data: T) -> Self {
        Self::new(data)
    }

    fn lock<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R {
        critical_section::with(|cs| f(self.borrow(cs)))
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

    /// Borrows the data
    pub fn borrow(&self) -> &T {
        assert!(
            in_thread_mode(),
            "ThreadModeMutex can only be borrowed from thread mode."
        );
        unsafe { &*self.inner.get() }
    }
}

impl<T> Mutex for ThreadModeMutex<T> {
    type Data = T;

    fn new(data: T) -> Self {
        Self::new(data)
    }

    fn lock<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R {
        f(self.borrow())
    }
}

impl<T> Drop for ThreadModeMutex<T> {
    fn drop(&mut self) {
        // Only allow dropping from thread mode. Dropping calls drop on the inner `T`, so
        // `drop` needs the same guarantees as `lock`. `ThreadModeMutex<T>` is Send even if
        // T isn't, so without this check a user could create a ThreadModeMutex in thread mode,
        // send it to interrupt context and drop it there, which would "send" a T even if T is not Send.
        assert!(
            in_thread_mode(),
            "ThreadModeMutex can only be dropped from thread mode."
        );

        // Drop of the inner `T` happens after this.
    }
}

pub fn in_thread_mode() -> bool {
    #[cfg(feature = "std")]
    return Some("main") == std::thread::current().name();

    #[cfg(not(feature = "std"))]
    return cortex_m::peripheral::SCB::vect_active()
        == cortex_m::peripheral::scb::VectActive::ThreadMode;
}

/// A "mutex" that does nothing and cannot be shared between threads.
pub struct NoopMutex<T> {
    inner: T,
}

impl<T> NoopMutex<T> {
    pub const fn new(value: T) -> Self {
        NoopMutex { inner: value }
    }
}

impl<T> NoopMutex<T> {
    pub fn borrow(&self) -> &T {
        &self.inner
    }
}

impl<T> Mutex for NoopMutex<T> {
    type Data = T;

    fn new(data: T) -> Self {
        Self::new(data)
    }

    fn lock<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R {
        f(self.borrow())
    }
}
