//! Read-Write Lock primitives.
//!
//! This module provides a trait for read-write locks that can be used in different contexts.
use core::{cell::RefCell, marker::PhantomData};

/// Raw read-write lock trait.
///
/// This read-write lock is "raw", which means it does not actually contain the protected data, it
/// just implements the read-write lock mechanism. For most uses you should use [`super::RwLock`] instead,
/// which is generic over a RawRwLock and contains the protected data.
///
/// Note that, unlike other read-write locks, implementations only guarantee no
/// concurrent access from other threads: concurrent access from the current
/// thread is allowed. For example, it's possible to lock the same read-write lock multiple times reentrantly.
///
/// Therefore, locking a `RawRwLock` is only enough to guarantee safe shared (`&`) access
/// to the data, it is not enough to guarantee exclusive (`&mut`) access.
///
/// # Safety
///
/// RawRwLock implementations must ensure that, while locked, no other thread can lock
/// the RawRwLock concurrently.
///
/// Unsafe code is allowed to rely on this fact, so incorrect implementations will cause undefined behavior.
pub unsafe trait RawRwLock {
    /// Create a new `RawRwLock` instance.
    ///
    /// This is a const instead of a method to allow creating instances in const context.
    const INIT: Self;

    /// Lock this `RawRwLock` for reading.
    fn read_lock<R>(&self, f: impl FnOnce() -> R) -> R;

    /// Lock this `RawRwLock` for writing.
    fn write_lock<R>(&self, f: impl FnOnce() -> R) -> R;
}

/// A read-write lock that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This read-write lock is safe to share between different executors and interrupts.
pub struct CriticalSectionRawRwLock {
    state: RefCell<isize>,
}

unsafe impl Send for CriticalSectionRawRwLock {}
unsafe impl Sync for CriticalSectionRawRwLock {}

impl CriticalSectionRawRwLock {
    /// Creates a new [`CriticalSectionRawRwLock`].
    pub const fn new() -> Self {
        Self { state: RefCell::new(0) }
    }

    fn lock_read(&self) {
        critical_section::with(|_| {
            let mut state = self.state.borrow_mut();

            while *state & WRITER != 0 {
                // Spin until the writer releases the lock
            }
            *state += 1;
        });
    }

    fn unlock_read(&self) {
        critical_section::with(|_| {
            *self.state.borrow_mut() -= 1;
        });
    }

    fn lock_write(&self) {
        critical_section::with(|_| {
            let mut state = self.state.borrow_mut();

            while *state != 0 {
                // Spin until all readers and writers release the lock
            }
            *state = WRITER;
        });
    }

    fn unlock_write(&self) {
        critical_section::with(|_| {
            *self.state.borrow_mut() = 0;
        });
    }
}

unsafe impl RawRwLock for CriticalSectionRawRwLock {
    const INIT: Self = Self::new();

    fn read_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.lock_read();
        let result = f();
        self.unlock_read();
        result
    }

    fn write_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.lock_write();
        let result = f();
        self.unlock_write();
        result
    }
}

const WRITER: isize = -1;

// The rest of the file remains unchanged

// ================

/// A read-write lock that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Read-Write Lock is only safe within a single executor.**
pub struct NoopRawRwLock {
    _phantom: PhantomData<*mut ()>,
}

unsafe impl Send for NoopRawRwLock {}

impl NoopRawRwLock {
    /// Create a new `NoopRawRwLock`.
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

unsafe impl RawRwLock for NoopRawRwLock {
    const INIT: Self = Self::new();
    fn read_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }

    fn write_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}

// ================

#[cfg(any(cortex_m, feature = "std"))]
mod thread_mode {
    use super::*;

    /// A "read-write lock" that only allows borrowing from thread mode.
    ///
    /// # Safety
    ///
    /// **This Read-Write Lock is only safe on single-core systems.**
    ///
    /// On multi-core systems, a `ThreadModeRawRwLock` **is not sufficient** to ensure exclusive access.
    pub struct ThreadModeRawRwLock {
        _phantom: PhantomData<()>,
    }

    unsafe impl Send for ThreadModeRawRwLock {}
    unsafe impl Sync for ThreadModeRawRwLock {}

    impl ThreadModeRawRwLock {
        /// Create a new `ThreadModeRawRwLock`.
        pub const fn new() -> Self {
            Self { _phantom: PhantomData }
        }
    }

    unsafe impl RawRwLock for ThreadModeRawRwLock {
        const INIT: Self = Self::new();
        fn read_lock<R>(&self, f: impl FnOnce() -> R) -> R {
            assert!(
                in_thread_mode(),
                "ThreadModeRwLock can only be locked from thread mode."
            );

            f()
        }

        fn write_lock<R>(&self, f: impl FnOnce() -> R) -> R {
            assert!(
                in_thread_mode(),
                "ThreadModeRwLock can only be locked from thread mode."
            );

            f()
        }
    }

    impl Drop for ThreadModeRawRwLock {
        fn drop(&mut self) {
            assert!(
                in_thread_mode(),
                "ThreadModeRwLock can only be dropped from thread mode."
            );
        }
    }

    pub(crate) fn in_thread_mode() -> bool {
        #[cfg(feature = "std")]
        return Some("main") == std::thread::current().name();

        #[cfg(not(feature = "std"))]
        return unsafe { (0xE000ED04 as *const u32).read_volatile() } & 0x1FF == 0;
    }
}
#[cfg(any(cortex_m, feature = "std"))]
pub use thread_mode::*;
