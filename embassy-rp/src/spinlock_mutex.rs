//! Mutex implementation utilizing an hardware spinlock

use core::marker::PhantomData;
use core::sync::atomic::Ordering;

use embassy_sync::blocking_mutex::raw::RawMutex;

use crate::spinlock::{Spinlock, SpinlockValid};

/// A mutex that allows borrowing data across executors and interrupts by utilizing an hardware spinlock
///
/// # Safety
///
/// This mutex is safe to share between different executors and interrupts.
pub struct SpinlockRawMutex<const N: usize> {
    _phantom: PhantomData<()>,
}
unsafe impl<const N: usize> Send for SpinlockRawMutex<N> {}
unsafe impl<const N: usize> Sync for SpinlockRawMutex<N> {}

impl<const N: usize> SpinlockRawMutex<N> {
    /// Create a new `SpinlockRawMutex`.
    pub const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

unsafe impl<const N: usize> RawMutex for SpinlockRawMutex<N>
where
    Spinlock<N>: SpinlockValid,
{
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        // Store the initial interrupt state in stack variable
        let interrupts_active = cortex_m::register::primask::read().is_active();

        // Spin until we get the lock
        loop {
            // Need to disable interrupts to ensure that we will not deadlock
            // if an interrupt or higher prio locks the spinlock after we acquire the lock
            cortex_m::interrupt::disable();
            // Ensure the compiler doesn't re-order accesses and violate safety here
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
            if let Some(lock) = Spinlock::<N>::try_claim() {
                // We just acquired the lock.
                // 1. Forget it, so we don't immediately unlock
                core::mem::forget(lock);
                break;
            }
            // We didn't get the lock, enable interrupts if they were enabled before we started
            if interrupts_active {
                // safety: interrupts are only enabled, if they had been enabled before
                unsafe {
                    cortex_m::interrupt::enable();
                }
            }
        }

        let retval = f();

        // Ensure the compiler doesn't re-order accesses and violate safety here
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        // Release the spinlock to allow others to lock mutex again
        // safety: this point is only reached a spinlock was acquired before
        unsafe {
            Spinlock::<N>::release();
        }

        // Re-enable interrupts if they were enabled before the mutex was locked
        if interrupts_active {
            // safety: interrupts are only enabled, if they had been enabled before
            unsafe {
                cortex_m::interrupt::enable();
            }
        }

        retval
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
