use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex};

/// A handle to the lock that guards the driver's hardware access.
///
/// The driver passes the lock to several parts, so it takes a handle that it can duplicate. Every
/// handle that [`share`](Self::share) returns locks the same lock.
///
/// Handles are passed to other contexts only as their `Send` and `Sync` implementations allow.
///
/// # Safety
///
/// - [`share`](Self::share) returns a handle that locks the same lock as `self`.
/// - While `lock` runs `f`, no other context can run `lock` on a handle to the same lock.
pub unsafe trait SharedRawMutex {
    /// Returns another handle to the same lock.
    fn share(&self) -> Self;

    /// Locks the lock while `f` runs.
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R;
}

unsafe impl SharedRawMutex for CriticalSectionRawMutex {
    fn share(&self) -> Self {
        // All critical-section mutexes lock the same global critical section.
        Self::new()
    }

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        // Take the critical section directly. The handles don't share the lock flag of
        // `CriticalSectionRawMutex`, so they must not use it.
        critical_section::with(|_| f())
    }
}

unsafe impl<M: RawMutex> SharedRawMutex for &M {
    fn share(&self) -> Self {
        *self
    }

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        (*self).with_lock(f)
    }
}
