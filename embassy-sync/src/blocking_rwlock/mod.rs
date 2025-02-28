//! Blocking read-write lock.
//!
//! This module provides a blocking read-write lock that can be used to synchronize data.
pub mod raw_rwlock;

use core::cell::UnsafeCell;

use self::raw_rwlock::RawRwLock;

/// Blocking read-write lock (not async)
///
/// Provides a blocking read-write lock primitive backed by an implementation of [`raw_rwlock::RawRwLock`].
///
/// Which implementation you select depends on the context in which you're using the read-write lock, and you can choose which kind
/// of interior mutability fits your use case.
///
/// Use [`CriticalSectionRwLock`] when data can be shared between threads and interrupts.
///
/// Use [`NoopRwLock`] when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeRwLock`] when data is shared between tasks running on the same executor but you want a global singleton.
///
/// In all cases, the blocking read-write lock is intended to be short lived and not held across await points.
/// Use the async [`RwLock`](crate::rwlock::RwLock) if you need a lock that is held across await points.
pub struct RwLock<R, T: ?Sized> {
    // NOTE: `raw` must be FIRST, so when using ThreadModeRwLock the "can't drop in non-thread-mode" gets
    // to run BEFORE dropping `data`.
    raw: R,
    data: UnsafeCell<T>,
}

unsafe impl<R: RawRwLock + Send, T: ?Sized + Send> Send for RwLock<R, T> {}
unsafe impl<R: RawRwLock + Sync, T: ?Sized + Send> Sync for RwLock<R, T> {}

impl<R: RawRwLock, T> RwLock<R, T> {
    /// Creates a new read-write lock in an unlocked state ready for use.
    #[inline]
    pub const fn new(val: T) -> RwLock<R, T> {
        RwLock {
            raw: R::INIT,
            data: UnsafeCell::new(val),
        }
    }

    /// Creates a critical section and grants temporary read access to the protected data.
    pub fn read_lock<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.raw.read_lock(|| {
            let ptr = self.data.get() as *const T;
            let inner = unsafe { &*ptr };
            f(inner)
        })
    }

    /// Creates a critical section and grants temporary write access to the protected data.
    pub fn write_lock<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        self.raw.write_lock(|| {
            let ptr = self.data.get() as *mut T;
            let inner = unsafe { &mut *ptr };
            f(inner)
        })
    }
}

impl<R, T> RwLock<R, T> {
    /// Creates a new read-write lock based on a pre-existing raw read-write lock.
    ///
    /// This allows creating a read-write lock in a constant context on stable Rust.
    #[inline]
    pub const fn const_new(raw_rwlock: R, val: T) -> RwLock<R, T> {
        RwLock {
            raw: raw_rwlock,
            data: UnsafeCell::new(val),
        }
    }

    /// Consumes this read-write lock, returning the underlying data.
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place---the mutable borrow statically guarantees no locks exist.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

/// A read-write lock that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This read-write lock is safe to share between different executors and interrupts.
pub type CriticalSectionRwLock<T> = RwLock<raw_rwlock::CriticalSectionRawRwLock, T>;

/// A read-write lock that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Read-Write Lock is only safe within a single executor.**
pub type NoopRwLock<T> = RwLock<raw_rwlock::NoopRawRwLock, T>;

impl<T> RwLock<raw_rwlock::CriticalSectionRawRwLock, T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&'cs self, _cs: critical_section::CriticalSection<'cs>) -> &'cs T {
        let ptr = self.data.get() as *const T;
        unsafe { &*ptr }
    }
}

impl<T> RwLock<raw_rwlock::NoopRawRwLock, T> {
    /// Borrows the data
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &T {
        let ptr = self.data.get() as *const T;
        unsafe { &*ptr }
    }
}

// ThreadModeRwLock does NOT use the generic read-write lock from above because it's special:
// it's Send+Sync even if T: !Send. There's no way to do that without specialization (I think?).
//
// There's still a ThreadModeRawRwLock for use with the generic RwLock (handy with Channel, for example),
// but that will require T: Send even though it shouldn't be needed.

#[cfg(any(cortex_m, feature = "std"))]
pub use thread_mode_rwlock::*;
#[cfg(any(cortex_m, feature = "std"))]
mod thread_mode_rwlock {
    use super::*;

    /// A "read-write lock" that only allows borrowing from thread mode.
    ///
    /// # Safety
    ///
    /// **This Read-Write Lock is only safe on single-core systems.**
    ///
    /// On multi-core systems, a `ThreadModeRwLock` **is not sufficient** to ensure exclusive access.
    pub struct ThreadModeRwLock<T: ?Sized> {
        inner: UnsafeCell<T>,
    }

    // NOTE: ThreadModeRwLock only allows borrowing from one execution context ever: thread mode.
    // Therefore it cannot be used to send non-sendable stuff between execution contexts, so it can
    // be Send+Sync even if T is not Send (unlike CriticalSectionRwLock)
    unsafe impl<T: ?Sized> Sync for ThreadModeRwLock<T> {}
    unsafe impl<T: ?Sized> Send for ThreadModeRwLock<T> {}

    impl<T> ThreadModeRwLock<T> {
        /// Creates a new read-write lock
        pub const fn new(value: T) -> Self {
            ThreadModeRwLock {
                inner: UnsafeCell::new(value),
            }
        }
    }

    impl<T: ?Sized> ThreadModeRwLock<T> {
        /// Lock the `ThreadModeRwLock` for reading, granting access to the data.
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn read_lock<R>(&self, f: impl FnOnce(&T) -> R) -> R {
            f(self.borrow())
        }

        /// Lock the `ThreadModeRwLock` for writing, granting access to the data.
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn write_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            f(self.borrow_mut())
        }

        /// Borrows the data
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn borrow(&self) -> &T {
            assert!(
                raw_rwlock::in_thread_mode(),
                "ThreadModeRwLock can only be borrowed from thread mode."
            );
            unsafe { &*self.inner.get() }
        }

        /// Mutably borrows the data
        ///
        /// # Panics
        ///
        /// This will panic if not currently running in thread mode.
        pub fn borrow_mut(&self) -> &mut T {
            assert!(
                raw_rwlock::in_thread_mode(),
                "ThreadModeRwLock can only be borrowed from thread mode."
            );
            unsafe { &mut *self.inner.get() }
        }
    }

    impl<T: ?Sized> Drop for ThreadModeRwLock<T> {
        fn drop(&mut self) {
            // Only allow dropping from thread mode. Dropping calls drop on the inner `T`, so
            // `drop` needs the same guarantees as `lock`. `ThreadModeRwLock<T>` is Send even if
            // T isn't, so without this check a user could create a ThreadModeRwLock in thread mode,
            // send it to interrupt context and drop it there, which would "send" a T even if T is not Send.
            assert!(
                raw_rwlock::in_thread_mode(),
                "ThreadModeRwLock can only be dropped from thread mode."
            );

            // Drop of the inner `T` happens after this.
        }
    }
}