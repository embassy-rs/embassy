//! Mutex implementation utilizing an hardware spinlock

use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

use embassy_sync::blocking_mutex::raw::{ConstInit, ScopedRawMutex};

use crate::pac;
use crate::spinlock::{Spinlock, SpinlockValid};

/// A mutex that allows borrowing data across executors and interrupts by utilizing an hardware spinlock
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
///
/// Locking the mutex again while it is locked on the same core spins forever.
pub struct SpinlockRawMutex<const N: usize> {
    _phantom: PhantomData<()>,
}
unsafe impl<const N: usize> Send for SpinlockRawMutex<N> {}
unsafe impl<const N: usize> Sync for SpinlockRawMutex<N> {}

impl<const N: usize> Default for SpinlockRawMutex<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SpinlockRawMutex<N> {
    /// Create a new `SpinlockRawMutex`.
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<const N: usize> SpinlockRawMutex<N>
where
    Spinlock<N>: SpinlockValid,
{
    /// Disables interrupts and tries to claim the spinlock once.
    ///
    /// Returns `true` with interrupts disabled if the spinlock was claimed. Otherwise it
    /// re-enables interrupts if they were enabled before, and returns `false`.
    fn try_acquire(interrupts_active: bool) -> bool {
        // Need to disable interrupts to ensure that we will not deadlock
        // if an interrupt or higher prio locks the spinlock after we acquire the lock
        cortex_m::interrupt::disable();
        // Ensure the compiler doesn't re-order accesses and violate safety here
        compiler_fence(Ordering::SeqCst);
        if let Some(lock) = Spinlock::<N>::try_claim() {
            // Forget the lock, so we don't immediately unlock
            core::mem::forget(lock);
            return true;
        }
        if interrupts_active {
            // safety: interrupts are only enabled, if they had been enabled before
            unsafe { cortex_m::interrupt::enable() };
        }
        false
    }

    /// Runs `f`, then releases the spinlock and restores the interrupt state, also if `f` panics.
    fn run_locked<R>(interrupts_active: bool, f: impl FnOnce() -> R) -> R {
        struct Release<const N: usize>(bool)
        where
            Spinlock<N>: SpinlockValid;

        impl<const N: usize> Drop for Release<N>
        where
            Spinlock<N>: SpinlockValid,
        {
            fn drop(&mut self) {
                // Ensure the compiler doesn't re-order accesses and violate safety here
                compiler_fence(Ordering::SeqCst);
                // safety: `Release` is only created after the spinlock was acquired
                unsafe { Spinlock::<N>::release() };
                if self.0 {
                    // safety: interrupts are only enabled, if they had been enabled before
                    unsafe { cortex_m::interrupt::enable() };
                }
            }
        }

        let _release = Release::<N>(interrupts_active);
        f()
    }
}

impl<const N: usize> ConstInit for SpinlockRawMutex<N> {
    const INIT: Self = Self::new();
}

unsafe impl<const N: usize> ScopedRawMutex for SpinlockRawMutex<N>
where
    Spinlock<N>: SpinlockValid,
{
    fn try_with_lock<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        let interrupts_active = cortex_m::register::primask::read().is_active();
        if !Self::try_acquire(interrupts_active) {
            return None;
        }
        Some(Self::run_locked(interrupts_active, f))
    }

    fn with_lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let interrupts_active = cortex_m::register::primask::read().is_active();
        while !Self::try_acquire(interrupts_active) {}
        Self::run_locked(interrupts_active, f)
    }

    fn is_locked(&self) -> bool {
        pac::SIO.spinlock_st().read() & (1 << N) != 0
    }
}

pub mod blocking_mutex {
    //! Mutex implementation utilizing an hardware spinlock
    use embassy_sync::blocking_mutex::Mutex;

    use crate::spinlock_mutex::SpinlockRawMutex;
    /// A mutex that allows borrowing data across executors and interrupts by utilizing an hardware spinlock.
    ///
    /// # Safety
    ///
    /// This mutex is safe to share between different executors and interrupts.
    pub type SpinlockMutex<const N: usize, T> = Mutex<SpinlockRawMutex<N>, T>;
}
