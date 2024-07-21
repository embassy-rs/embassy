//! Blocking mutex.
//!
//! This module provides a blocking mutex that can be used to synchronize data.
pub mod raw;

use core::ops::Deref;

use scoped_mutex::BlockingMutex;
// Semver re-exports
pub use scoped_mutex::BlockingMutex as Mutex;

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
    /// Creates a new mutex in an unlocked state ready for use.
    #[inline]
    pub const fn const_new(data: T) -> Self {
        Self {
            inner: BlockingMutex::const_new(raw::CriticalSectionRawMutex::new(), data),
        }
    }

    /// Creates a new mutex based on a pre-existing raw mutex.
    ///
    /// This allows creating a mutex in a constant context on stable Rust.
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            inner: BlockingMutex::new(data),
        }
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
    /// Creates a new mutex in an unlocked state ready for use.
    #[inline]
    pub const fn const_new(data: T) -> Self {
        Self {
            inner: BlockingMutex::const_new(raw::NoopRawMutex::new(), data),
        }
    }

    /// Creates a new mutex based on a pre-existing raw mutex.
    ///
    /// This allows creating a mutex in a constant context on stable Rust.
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            inner: BlockingMutex::new(data),
        }
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
        inner: BlockingMutex<raw::ThreadModeRawMutex, T>,
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
                inner: BlockingMutex::const_new(raw::ThreadModeRawMutex::new(), value),
            }
        }
    }

    impl<T> Deref for ThreadModeMutex<T> {
        type Target = BlockingMutex<raw::ThreadModeRawMutex, T>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<T: ?Sized> Drop for ThreadModeMutex<T> {
        fn drop(&mut self) {
            // Only allow dropping from thread mode. Dropping calls drop on the inner `T`, so
            // `drop` needs the same guarantees as `lock`. `ThreadModeMutex<T>` is Send even if
            // T isn't, so without this check a user could create a ThreadModeMutex in thread mode,
            // send it to interrupt context and drop it there, which would "send" a T even if T is not Send.
            assert!(
                scoped_mutex::impls::thread_mode::in_thread_mode(),
                "ThreadModeMutex can only be dropped from thread mode."
            );

            // Drop of the inner `T` happens after this.
        }
    }
}
