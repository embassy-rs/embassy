//! Mutex primitives.
//!
//! This module provides a trait for mutexes that can be used in different contexts.
//!
//! The traits come from the [`mutex-traits`](https://docs.rs/mutex-traits) crate. Implement
//! [`ScopedRawMutex`] and [`ConstInit`] for a type to use it with embassy-sync.
use core::cell::Cell;
#[cfg(not(feature = "portable-atomic"))]
use core::sync::atomic::{AtomicBool, Ordering};

pub use mutex_traits::{ConstInit, ScopedRawMutex};
#[cfg(feature = "portable-atomic")]
use portable_atomic::{AtomicBool, Ordering};

/// Raw mutex trait.
///
/// This mutex is "raw", which means it does not actually contain the protected data, it
/// just implements the mutex mechanism. For most uses you should use [`super::Mutex`] instead,
/// which is generic over a RawMutex and contains the protected data.
///
/// This trait is implemented for every type that implements [`ScopedRawMutex`] and [`ConstInit`].
/// Implement those traits, not this one.
pub trait RawMutex: ScopedRawMutex + ConstInit {}

impl<M: ScopedRawMutex + ConstInit> RawMutex for M {}

/// Clears a lock flag when dropped, also when the closure that holds the lock panics.
struct Unlock<'a>(&'a AtomicBool);

impl Drop for Unlock<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// A mutex that allows borrowing data across executors and interrupts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub struct CriticalSectionRawMutex {
    // Only accessed inside a critical section, which gives exclusive access.
    locked: Cell<bool>,
}
unsafe impl Send for CriticalSectionRawMutex {}
unsafe impl Sync for CriticalSectionRawMutex {}

impl core::fmt::Debug for CriticalSectionRawMutex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Doesn't print the lock flag, which can only be read inside a critical section.
        f.debug_struct("CriticalSectionRawMutex").finish_non_exhaustive()
    }
}

impl CriticalSectionRawMutex {
    /// Create a new `CriticalSectionRawMutex`.
    pub const fn new() -> Self {
        Self {
            locked: Cell::new(false),
        }
    }
}

impl ConstInit for CriticalSectionRawMutex {
    const INIT: Self = Self::new();
}

impl CriticalSectionRawMutex {
    /// Enters a critical section and sets the lock flag.
    ///
    /// Returns `None`, outside the critical section, if the flag is already set. The critical
    /// section keeps other contexts out, so the flag only sees locking from inside the lock.
    fn acquire(&self) -> Option<CsLock<'_>> {
        // Safety: the restore state is released in `CsLock::drop`, or right here.
        let restore_state = unsafe { critical_section::acquire() };
        if self.locked.get() {
            // Safety: `restore_state` comes from the `acquire` above.
            unsafe { critical_section::release(restore_state) };
            return None;
        }
        self.locked.set(true);
        Some(CsLock {
            mutex: self,
            restore_state,
        })
    }

    /// Like `acquire`, but panics if the lock flag is already set.
    fn lock(&self) -> CsLock<'_> {
        match self.acquire() {
            Some(lock) => lock,
            None => panic!("CriticalSectionRawMutex locked reentrantly"),
        }
    }
}

/// A held `CriticalSectionRawMutex`. Clears the lock flag and leaves the critical section when
/// dropped, also when the closure that holds the lock panics.
struct CsLock<'a> {
    mutex: &'a CriticalSectionRawMutex,
    restore_state: critical_section::RestoreState,
}

impl Drop for CsLock<'_> {
    fn drop(&mut self) {
        self.mutex.locked.set(false);
        // Safety: `restore_state` comes from `CriticalSectionRawMutex::acquire`, and is released once.
        unsafe { critical_section::release(self.restore_state) };
    }
}

unsafe impl ScopedRawMutex for CriticalSectionRawMutex {
    fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        let _lock = self.acquire()?;
        Some(f())
    }

    fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let _lock = self.lock();
        f()
    }

    fn is_locked(&self) -> bool {
        // Another core can write the flag, so it can only be read inside a critical section.
        critical_section::with(|_| self.locked.get())
    }
}

/// A mutex that panics if locked from multiple contexts.
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
#[derive(Debug)]
pub struct PanicRawMutex {
    locked: AtomicBool,
}
unsafe impl Send for PanicRawMutex {}
unsafe impl Sync for PanicRawMutex {}

impl PanicRawMutex {
    /// Create a new `PanicRawMutex`.
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    fn try_acquire(&self) -> bool {
        #[cfg(target_has_atomic = "8")]
        return !self.locked.swap(true, Ordering::Acquire);

        #[cfg(not(target_has_atomic = "8"))]
        critical_section::with(|_| {
            if self.locked.load(Ordering::Acquire) {
                return false;
            }
            self.locked.store(true, Ordering::Relaxed);
            true
        })
    }
}

impl ConstInit for PanicRawMutex {
    const INIT: Self = Self::new();
}

unsafe impl ScopedRawMutex for PanicRawMutex {
    fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        if !self.try_acquire() {
            return None;
        }
        let _unlock = Unlock(&self.locked);
        Some(f())
    }

    fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.try_with_lock(f)
            .expect("PanicRawMutex locked from multiple contexts")
    }

    fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}

// ================

/// A mutex that allows borrowing data in the context of a single executor.
///
/// # Safety
///
/// **This Mutex is only safe within a single executor.**
#[derive(Debug)]
pub struct NoopRawMutex {
    // `Cell` makes the type `!Sync`.
    locked: Cell<bool>,
}

unsafe impl Send for NoopRawMutex {}

impl NoopRawMutex {
    /// Create a new `NoopRawMutex`.
    pub const fn new() -> Self {
        Self {
            locked: Cell::new(false),
        }
    }
}

impl ConstInit for NoopRawMutex {
    const INIT: Self = Self::new();
}

unsafe impl ScopedRawMutex for NoopRawMutex {
    fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        struct Unlock<'a>(&'a Cell<bool>);

        impl Drop for Unlock<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }

        if self.locked.replace(true) {
            return None;
        }
        let _unlock = Unlock(&self.locked);
        Some(f())
    }

    fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        self.try_with_lock(f).expect("NoopRawMutex locked reentrantly")
    }

    fn is_locked(&self) -> bool {
        self.locked.get()
    }
}

// ================

#[cfg(any(cortex_m, doc, feature = "std"))]
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
        locked: AtomicBool,
    }

    unsafe impl Send for ThreadModeRawMutex {}
    unsafe impl Sync for ThreadModeRawMutex {}

    impl ThreadModeRawMutex {
        /// Create a new `ThreadModeRawMutex`.
        pub const fn new() -> Self {
            Self {
                locked: AtomicBool::new(false),
            }
        }
    }

    impl ConstInit for ThreadModeRawMutex {
        const INIT: Self = Self::new();
    }

    unsafe impl ScopedRawMutex for ThreadModeRawMutex {
        fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
            assert!(in_thread_mode(), "ThreadModeMutex can only be locked from thread mode.");

            // Only thread mode locks the mutex, so the flag only sees locking from inside `f`.
            if self.locked.load(Ordering::Relaxed) {
                return None;
            }
            self.locked.store(true, Ordering::Relaxed);
            let _unlock = Unlock(&self.locked);
            Some(f())
        }

        fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
            assert!(in_thread_mode(), "ThreadModeMutex can only be locked from thread mode.");

            if self.locked.load(Ordering::Relaxed) {
                panic!("ThreadModeRawMutex locked reentrantly");
            }
            self.locked.store(true, Ordering::Relaxed);
            let _unlock = Unlock(&self.locked);
            f()
        }

        fn is_locked(&self) -> bool {
            self.locked.load(Ordering::Relaxed)
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
#[cfg(any(cortex_m, doc, feature = "std"))]
pub use thread_mode::*;
