use core::task::Waker;

use heapless::Vec;

/// Utility struct to register and wake multiple wakers.
/// Queue of wakers with a maximum length of `N`.
/// Intended for waking multiple tasks.
#[derive(Debug)]
pub struct MultiWakerRegistration<const N: usize> {
    wakers: Vec<Waker, N>,
}

impl<const N: usize> MultiWakerRegistration<N> {
    /// Create a new empty instance
    pub const fn new() -> Self {
        Self { wakers: Vec::new() }
    }

    /// Register a waker.
    ///
    /// If the buffer is full, [wakes all the wakers](Self::wake), clears its buffer and registers the waker.
    pub fn register(&mut self, w: &Waker) {
        // If we already have some waker that wakes the same task as `w`, do nothing.
        // This avoids cloning wakers, and avoids unnecessary mass-wakes.
        for w2 in &self.wakers {
            if w.will_wake(w2) {
                return;
            }
        }

        if self.wakers.is_full() {
            // All waker slots were full. It's a bit inefficient, but we can wake everything.
            // Any future that is still active will simply reregister.
            // This won't happen a lot, so it's ok.
            self.wake();
        }

        if self.wakers.push(w.clone()).is_err() && N == 0 {
            // This can't happen unless N=0
            // (Either `wakers` wasn't full, or it was in which case `wake()` empied it)
            panic!("tried to push a waker to a zero-length MultiWakerRegistration")
        }
    }

    /// Wake all registered wakers. This clears the buffer
    pub fn wake(&mut self) {
        for w in self.wakers.drain(..) {
            // Wake it by value, which consumes (drops) it.
            w.wake();
        }
    }
}
