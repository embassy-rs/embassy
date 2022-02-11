use core::cell::Cell;
use core::mem;
use core::task::Waker;

use crate::blocking_mutex::raw::CriticalSectionRawMutex;
use crate::blocking_mutex::Mutex;

/// Utility struct to register and wake a waker.
#[derive(Debug)]
pub struct WakerRegistration {
    waker: Option<Waker>,
}

impl WakerRegistration {
    pub const fn new() -> Self {
        Self { waker: None }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&mut self, w: &Waker) {
        match self.waker {
            // Optimization: If both the old and new Wakers wake the same task, we can simply
            // keep the old waker, skipping the clone. (In most executor implementations,
            // cloning a waker is somewhat expensive, comparable to cloning an Arc).
            Some(ref w2) if (w2.will_wake(w)) => {}
            _ => {
                // clone the new waker and store it
                if let Some(old_waker) = mem::replace(&mut self.waker, Some(w.clone())) {
                    // We had a waker registered for another task. Wake it, so the other task can
                    // reregister itself if it's still interested.
                    //
                    // If two tasks are waiting on the same thing concurrently, this will cause them
                    // to wake each other in a loop fighting over this WakerRegistration. This wastes
                    // CPU but things will still work.
                    //
                    // If the user wants to have two tasks waiting on the same thing they should use
                    // a more appropriate primitive that can store multiple wakers.
                    old_waker.wake()
                }
            }
        }
    }

    /// Wake the registered waker, if any.
    pub fn wake(&mut self) {
        if let Some(w) = self.waker.take() {
            w.wake()
        }
    }
}

/// Utility struct to register and wake a waker.
pub struct AtomicWaker {
    waker: Mutex<CriticalSectionRawMutex, Cell<Option<Waker>>>,
}

impl AtomicWaker {
    pub const fn new() -> Self {
        Self {
            waker: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(None)),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        critical_section::with(|cs| {
            let cell = self.waker.borrow(cs);
            cell.set(match cell.replace(None) {
                Some(w2) if (w2.will_wake(w)) => Some(w2),
                _ => Some(w.clone()),
            })
        })
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        critical_section::with(|cs| {
            let cell = self.waker.borrow(cs);
            if let Some(w) = cell.replace(None) {
                w.wake_by_ref();
                cell.set(Some(w));
            }
        })
    }
}
