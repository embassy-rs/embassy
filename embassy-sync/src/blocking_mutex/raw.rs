//! Mutex primitives.
//!
//! This module provides a trait for mutexes that can be used in different contexts.
use core::marker::PhantomData;

pub use raw_mutex_traits::RawMutex;

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
