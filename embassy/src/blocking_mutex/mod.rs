//! Blocking mutex (not async)

pub mod raw;

use self::raw::RawMutex;
use core::cell::UnsafeCell;

/// Any object implementing this trait guarantees exclusive access to the data contained
/// within the mutex for the duration of the lock.
/// Adapted from <https://github.com/rust-embedded/mutex-trait>.
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
    #[cfg(feature = "nightly")]
    #[inline]
    pub const fn new(val: T) -> Mutex<R, T> {
        Mutex {
            raw: R::INIT,
            data: UnsafeCell::new(val),
        }
    }

    /// Creates a new mutex in an unlocked state ready for use.
    #[cfg(not(feature = "nightly"))]
    #[inline]
    pub fn new(val: T) -> Mutex<R, T> {
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

pub type CriticalSectionMutex<T> = Mutex<raw::CriticalSectionRawMutex, T>;
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
        pub fn lock<R>(&self, f: impl FnOnce(&T) -> R) -> R {
            f(self.borrow())
        }

        /// Borrows the data
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
