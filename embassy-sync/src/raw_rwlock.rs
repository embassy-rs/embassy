use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::Waker;
use core::cell::UnsafeCell;

pub trait RawRwLock {
    fn lock_read(&self);
    fn try_lock_read(&self) -> bool;
    fn unlock_read(&self);
    fn lock_write(&self);
    fn try_lock_write(&self) -> bool;
    fn unlock_write(&self);
}

pub struct RawRwLockImpl {
    state: AtomicUsize,
    waker: UnsafeCell<Option<Waker>>,
}

impl RawRwLockImpl {
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
            waker: UnsafeCell::new(None),
        }
    }
}

unsafe impl Send for RawRwLockImpl {}
unsafe impl Sync for RawRwLockImpl {}

impl RawRwLock for RawRwLockImpl {
    fn lock_read(&self) {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state & 1 == 0 {
                if self.state.compare_and_swap(state, state + 2, Ordering::AcqRel) == state {
                    break;
                }
            }
        }
    }

    fn try_lock_read(&self) -> bool {
        let state = self.state.load(Ordering::Acquire);
        if state & 1 == 0 {
            if self.state.compare_and_swap(state, state + 2, Ordering::AcqRel) == state {
                return true;
            }
        }
        false
    }

    fn unlock_read(&self) {
        self.state.fetch_sub(2, Ordering::Release);
        if self.state.load(Ordering::Acquire) == 0 {
            if let Some(waker) = unsafe { &*self.waker.get() } {
                waker.wake_by_ref();
            }
        }
    }

    fn lock_write(&self) {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state == 0 {
                if self.state.compare_and_swap(0, 1, Ordering::AcqRel) == 0 {
                    break;
                }
            }
        }
    }

    fn try_lock_write(&self) -> bool {
        if self.state.compare_and_swap(0, 1, Ordering::AcqRel) == 0 {
            return true;
        }
        false
    }

    fn unlock_write(&self) {
        self.state.store(0, Ordering::Release);
        if let Some(waker) = unsafe { &*self.waker.get() } {
            waker.wake_by_ref();
        }
    }
}
