use core::cell::Cell;
use core::task::Waker;

/// Utility struct to register and wake a waker.
pub struct CriticalSectionWaker {
    waker: critical_section::Mutex<Cell<Option<Waker>>>,
}

impl CriticalSectionWaker {
    /// Create a new `CriticalSectionWaker`.
    pub const fn new() -> Self {
        Self {
            waker: critical_section::Mutex::new(Cell::new(None)),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        critical_section::with(|cs| {
            let cell = self.waker.borrow(cs);
            cell.set(match cell.replace(None) {
                Some(w2) if w2.will_wake(w) => Some(w2),
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
