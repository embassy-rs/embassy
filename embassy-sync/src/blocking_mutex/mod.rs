//! Blocking mutex.
//!
//! This module provides a blocking mutex that can be used to synchronize data.
pub mod raw;

use core::ops::Deref;

use raw_mutex_traits::BlockingMutex;
// Semver re-export
pub use raw_mutex_traits::BlockingMutex as Mutex;

/// A mutex that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub struct CriticalSectionMutex<T> {
    pub(crate) inner: BlockingMutex<raw::CriticalSectionRawMutex, T>,
}

/// A mutex that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Mutex is only safe within a single executor.**
pub struct NoopMutex<T> {
    pub(crate) inner: BlockingMutex<raw::NoopRawMutex, T>,
}

impl<T> Deref for CriticalSectionMutex<T> {
    type Target = BlockingMutex<raw::CriticalSectionRawMutex, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> CriticalSectionMutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&'cs self, _cs: critical_section::CriticalSection<'cs>) -> &'cs T {
        let ptr = unsafe { self.inner.get_unchecked() } as *const T;
        unsafe { &*ptr }
    }
}

impl<T> Deref for NoopMutex<T> {
    type Target = BlockingMutex<raw::NoopRawMutex, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> NoopMutex<T> {
    /// Borrows the data
    pub fn borrow(&self) -> &T {
        let ptr = unsafe { self.inner.get_unchecked() } as *const T;
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
    use core::cell::UnsafeCell;

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
