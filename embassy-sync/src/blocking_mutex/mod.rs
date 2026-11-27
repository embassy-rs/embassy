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
/// Which implementation you select depends on the context in which you're using the mutex.
///
/// Use [`CriticalSectionMutex`] when data can be shared between threads and interrupts.
///
/// Use [`NoopMutex`] when data is only shared between tasks running on the same executor.
///
/// Use [`ThreadModeMutex`] when data is shared between tasks running on the same executor but you want a global singleton.
///
/// The mutex is not reentrant: locking it again while it is locked in the same context panics or
/// deadlocks, depending on the raw mutex. The raw mutexes of this crate panic.
///
/// In all cases, the blocking mutex is intended to be short lived and not held across await points.
/// Use the async [`Mutex`](crate::mutex::Mutex) if you need a lock that is held across await points.
#[derive(Debug)]
pub struct Mutex<R, T: ?Sized> {
    // NOTE: `raw` must be FIRST, so when using ThreadModeMutex the "can't drop in non-thread-mode" gets
    // to run BEFORE dropping `data`.
    raw: R,
    data: UnsafeCell<T>,
}

unsafe impl<R: Send, T: ?Sized + Send> Send for Mutex<R, T> {}
unsafe impl<R: Sync, T: ?Sized + Send> Sync for Mutex<R, T> {}

impl<R: RawMutex, T> Mutex<R, T> {
    /// Creates a new mutex in an unlocked state ready for use.
    #[inline]
    pub const fn new(val: T) -> Mutex<R, T> {
        Mutex {
            raw: R::INIT,
            data: UnsafeCell::new(val),
        }
    }
}

impl<R: RawMutex, T: ?Sized> Mutex<R, T> {
    /// Locks the mutex and grants temporary mutable access to the protected data.
    ///
    /// # Panics
    ///
    /// Locking the mutex again from `f` panics or deadlocks, depending on the raw mutex. When another
    /// context holds the lock, some raw mutexes panic and others block until it is released.
    pub fn lock<U>(&self, f: impl FnOnce(&mut T) -> U) -> U {
        self.raw.with_lock(|| {
            // Safety: the raw mutex is held, so no other reference to the data exists.
            f(unsafe { &mut *self.data.get() })
        })
    }

    /// Locks the mutex if it is not locked, and grants temporary mutable access to the protected data.
    ///
    /// Returns `None` if the mutex is already locked.
    pub fn try_lock<U>(&self, f: impl FnOnce(&mut T) -> U) -> Option<U> {
        self.raw.try_with_lock(|| {
            // Safety: the raw mutex is held, so no other reference to the data exists.
            f(unsafe { &mut *self.data.get() })
        })
    }

    /// Returns `true` if the mutex is locked.
    pub fn is_locked(&self) -> bool {
        self.raw.is_locked()
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
}

impl<R, T: ?Sized> Mutex<R, T> {
    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place---the mutable borrow statically guarantees no locks exist.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

/// A mutex that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub type CriticalSectionMutex<T> = Mutex<raw::CriticalSectionRawMutex, T>;

/// A mutex that panics if locked from multiple contexts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub type PanicMutex<T> = Mutex<raw::PanicRawMutex, T>;

/// A mutex that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Mutex is only safe within a single executor.**
pub type NoopMutex<T> = Mutex<raw::NoopRawMutex, T>;

// ThreadModeMutex does NOT use the generic mutex from above because it's special:
// it's Send+Sync even if T: !Send. There's no way to do that without specialization (I think?).
//
// There's still a ThreadModeRawMutex for use with the generic Mutex (handy with Channel, for example),
// but that will require T: Send even though it shouldn't be needed.

#[cfg(any(cortex_m, doc, feature = "std"))]
pub use thread_mode_mutex::*;
#[cfg(any(cortex_m, doc, feature = "std"))]
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
