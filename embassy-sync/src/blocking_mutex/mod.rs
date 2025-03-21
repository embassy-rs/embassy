//! Blocking mutex.
//!
//! This module provides a blocking mutex that can be used to synchronize data.
pub mod raw;

use core::cell::UnsafeCell;

use self::raw::RawMutex;

/// Blocking mutex (not async)
///
/// Provides a blocking mutual exclusion primitive backed by an implementation of [`raw::RawMutex`].
///
/// Which implementation you select depends on the context in which you're using the mutex, and you can choose which kind
/// of interior mutability fits your use case.
///
/// Use [`CriticalSectionMutex`] when data can be shared between threads and interrupts.
///
/// Use [`NoopMutex`] when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeMutex`] when data is shared between tasks running on the same executor but you want a global singleton.
///
/// In all cases, the blocking mutex is intended to be short lived and not held across await points.
/// Use the async [`Mutex`](crate::mutex::Mutex) if you need a lock that is held across await points.
pub struct Mutex<R, T: ?Sized> {
    // NOTE: `raw` must be FIRST, so when using ThreadModeMutex the "can't drop in non-thread-mode" gets
    // to run BEFORE dropping `data`.
    raw: R,
    data: UnsafeCell<T>,
}

unsafe impl<R: RawMutex + Send, T: ?Sized + Send> Send for Mutex<R, T> {}
unsafe impl<R: RawMutex + Sync, T: ?Sized + Send> Sync for Mutex<R, T> {}

impl<R: RawMutex, T> Mutex<R, T> {
    /// Creates a new mutex in an unlocked state ready for use.
    #[inline]
    pub const fn new(val: T) -> Mutex<R, T> {
        Mutex {
            raw: R::INIT,
            data: UnsafeCell::new(val),
        }
    }

    /// Creates a critical section and grants temporary access to the protected data.
    pub fn lock<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.raw.lock(|| {
            let ptr = self.data.get() as *const T;
            let inner = unsafe { &*ptr };
            f(inner)
        })
    }

    /// Creates a critical section and grants temporary mutable access to the protected data.
    ///
    /// # Safety
    ///
    /// This method is marked unsafe because calling this method re-entrantly, i.e. within
    /// another `lock_mut` or `lock` closure, violates Rust's aliasing rules. Calling this
    /// method at the same time from different tasks is safe. For a safe alternative with
    /// mutable access that never causes UB, use a `RefCell` in a `Mutex`.
    pub unsafe fn lock_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        self.raw.lock(|| {
            let ptr = self.data.get() as *mut T;
            // Safety: we have exclusive access to the data, as long as this mutex is not locked re-entrantly
            let inner = unsafe { &mut *ptr };
            f(inner)
        })
    }
}

impl<R, T> Mutex<R, T> {
    /// Creates a new mutex based on a pre-existing raw mutex.
    ///
    /// This allows creating a mutex in a constant context on stable Rust.
    #[inline]
    pub const fn const_new(raw_mutex: R, val: T) -> Mutex<R, T> {
        Mutex {
            raw: raw_mutex,
            data: UnsafeCell::new(val),
        }
    }

    /// Consumes this mutex, returning the underlying data.
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place---the mutable borrow statically guarantees no locks exist.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

/// A mutex that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub type CriticalSectionMutex<T> = Mutex<raw::CriticalSectionRawMutex, T>;

/// A mutex that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Mutex is only safe within a single executor.**
pub type NoopMutex<T> = Mutex<raw::NoopRawMutex, T>;

impl<T> Mutex<raw::CriticalSectionRawMutex, T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&'cs self, _cs: critical_section::CriticalSection<'cs>) -> &'cs T {
        let ptr = self.data.get() as *const T;
        unsafe { &*ptr }
    }
}

impl<T> Mutex<raw::NoopRawMutex, T> {
    /// Borrows the data
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &T {
        let ptr = self.data.get() as *const T;
        unsafe { &*ptr }
    }
}

// ThreadModeMutex does NOT use the generic mutex from above because it's special:
// it's Send+Sync even if T: !Send. There's no way to do that without specialization (I think?).
//
// There's still a ThreadModeRawMutex for use with the generic Mutex (handy with Channel, for example),
// but that will require T: Send even though it shouldn't be needed.

#[cfg(any(cortex_m, feature = "std"))]
pub use thread_mode_mutex::*;
#[cfg(any(cortex_m, feature = "std"))]
mod thread_mode_mutex {
    use super::*;

    /// A "mutex" that only allows borrowing from thread mode.
    ///
    /// # Safety
    ///
    /// **This Mutex is only safe on single-core systems.**
    ///
    /// On multi-core systems, a `ThreadModeMutex` **is not sufficient** to ensure exclusive access.
    pub struct ThreadModeMutex<T: ?Sized> {
        inner: UnsafeCell<T>,
    }

    // NOTE: ThreadModeMutex only allows borrowing from one execution context ever: thread mode.
    // Therefore it cannot be used to send non-sendable stuff between execution contexts, so it can
    // be Send+Sync even if T is not Send (unlike CriticalSectionMutex)
    unsafe impl<T: ?Sized> Sync for ThreadModeMutex<T> {}
    unsafe impl<T: ?Sized> Send for ThreadModeMutex<T> {}

    impl<T> ThreadModeMutex<T> {
        /// Creates a new mutex
        pub const fn new(value: T) -> Self {
            ThreadModeMutex {
                inner: UnsafeCell::new(value),
            }
        }
    }

    impl<T: ?Sized> ThreadModeMutex<T> {
        /// Lock the `ThreadModeMutex`, granting access to the data.
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn lock<R>(&self, f: impl FnOnce(&T) -> R) -> R {
            f(self.borrow())
        }

        /// Borrows the data
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn borrow(&self) -> &T {
            assert!(
                raw::in_thread_mode(),
                "ThreadModeMutex can only be borrowed from thread mode."
            );
            unsafe { &*self.inner.get() }
        }
    }

    impl<T: ?Sized> Drop for ThreadModeMutex<T> {
        fn drop(&mut self) {
            // Only allow dropping from thread mode. Dropping calls drop on the inner `T`, so
            // `drop` needs the same guarantees as `lock`. `ThreadModeMutex<T>` is Send even if
            // T isn't, so without this check a user could create a ThreadModeMutex in thread mode,
            // send it to interrupt context and drop it there, which would "send" a T even if T is not Send.
            assert!(
                raw::in_thread_mode(),
                "ThreadModeMutex can only be dropped from thread mode."
            );

            // Drop of the inner `T` happens after this.
        }
    }
}
