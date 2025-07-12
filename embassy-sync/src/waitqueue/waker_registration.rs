use core::mem;
use core::task::Waker;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one.
/// The previous waker will be woken in this case, giving it a chance to reregister itself.
/// Although it is possible to wake multiple tasks this way,
/// this will cause them to wake each other in a loop registering themselves.
#[derive(Debug, Default)]
pub struct WakerRegistration {
    waker: Option<Waker>,
}

impl WakerRegistration {
    /// Create a new `WakerRegistration`.
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

    /// Returns true if a waker is currently registered
    pub fn occupied(&self) -> bool {
        self.waker.is_some()
    }
}
