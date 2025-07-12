//! Mutex primitives.
//!
//! This module provides a trait for mutexes that can be used in different contexts.
use core::marker::PhantomData;

/// Raw mutex trait.
///
/// This mutex is "raw", which means it does not actually contain the protected data, it
/// just implements the mutex mechanism. For most uses you should use [`super::Mutex`] instead,
/// which is generic over a RawMutex and contains the protected data.
///
/// Note that, unlike other mutexes, implementations only guarantee no
/// concurrent access from other threads: concurrent access from the current
/// thread is allowed. For example, it's possible to lock the same mutex multiple times reentrantly.
///
/// Therefore, locking a `RawMutex` is only enough to guarantee safe shared (`&`) access
/// to the data, it is not enough to guarantee exclusive (`&mut`) access.
///
/// # Safety
///
/// RawMutex implementations must ensure that, while locked, no other thread can lock
/// the RawMutex concurrently.
///
/// Unsafe code is allowed to rely on this fact, so incorrect implementations will cause undefined behavior.
pub unsafe trait RawMutex {
    /// Create a new `RawMutex` instance.
    ///
    /// This is a const instead of a method to allow creating instances in const context.
    const INIT: Self;

    /// Lock this `RawMutex`.
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R;
}

/// A mutex that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub struct CriticalSectionRawMutex {
    _phantom: PhantomData<()>,
}
unsafe impl Send for CriticalSectionRawMutex {}
unsafe impl Sync for CriticalSectionRawMutex {}

impl CriticalSectionRawMutex {
    /// Create a new `CriticalSectionRawMutex`.
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

unsafe impl RawMutex for CriticalSectionRawMutex {
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        critical_section::with(|_| f())
    }
}

// ================

/// A mutex that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Mutex is only safe within a single executor.**
pub struct NoopRawMutex {
    _phantom: PhantomData<*mut ()>,
}

unsafe impl Send for NoopRawMutex {}

impl NoopRawMutex {
    /// Create a new `NoopRawMutex`.
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

unsafe impl RawMutex for NoopRawMutex {
    const INIT: Self = Self::new();
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}

// ================

#[cfg(any(cortex_m, feature = "std"))]
mod thread_mode {
    use super::*;

    /// A "mutex" that only allows borrowing from thread mode.
    ///
    /// # Safety
    ///
    /// **This Mutex is only safe on single-core systems.**
    ///
    /// On multi-core systems, a `ThreadModeRawMutex` **is not sufficient** to ensure exclusive access.
    pub struct ThreadModeRawMutex {
        _phantom: PhantomData<()>,
    }

    unsafe impl Send for ThreadModeRawMutex {}
    unsafe impl Sync for ThreadModeRawMutex {}

    impl ThreadModeRawMutex {
        /// Create a new `ThreadModeRawMutex`.
        pub const fn new() -> Self {
            Self { _phantom: PhantomData }
        }
    }

    unsafe impl RawMutex for ThreadModeRawMutex {
        const INIT: Self = Self::new();
        fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
            assert!(in_thread_mode(), "ThreadModeMutex can only be locked from thread mode.");

            f()
        }
    }

    impl Drop for ThreadModeRawMutex {
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

    pub(crate) fn in_thread_mode() -> bool {
        #[cfg(feature = "std")]
        return Some("main") == std::thread::current().name();

        #[cfg(not(feature = "std"))]
        // ICSR.VECTACTIVE == 0
        return unsafe { (0xE000ED04 as *const u32).read_volatile() } & 0x1FF == 0;
    }
}
#[cfg(any(cortex_m, feature = "std"))]
pub use thread_mode::*;
