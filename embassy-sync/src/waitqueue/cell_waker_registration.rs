use core::cell::Cell;
use core::task::Waker;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one.
/// The previous waker will be woken in this case, giving it a chance to reregister itself.
/// Although it is possible to wake multiple tasks this way,
/// this will cause them to wake each other in a loop registering themselves.
///
/// This implementation (unlike [super::WakerRegistration]) implements interior mutability
/// and can perform register/wake operations using only shared reference, so it can be used in
/// cases where source and consumer of async events only have shared reference to this value.
#[derive(Default)]
pub struct CellWakerRegistration {
    waker: Cell<Option<Waker>>,
}

impl CellWakerRegistration {
    /// Create a new `WakerRegistration`.
    pub const fn new() -> Self {
        Self { waker: Cell::new(None) }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    ///
    /// This function accepts `&self` and it can trigger callbacks to external code (inside waker VTable).
    ///
    /// This function is safe to re-enter.
    /// * If re-entrance happens from the Waker::clone(), then calls to this function which happen earlier
    ///   in the stack will win.
    /// * If re-entrance happens from Waker::wake(), then calls to this function which happen deeper in the stack
    ///   will win.
    pub fn register(&self, w: &Waker) {
        // There is no way to get a reference to a value inside `Cell`, so we are using following sequence
        // to work with value inside it:
        // 1. Get value out of `Cell`, leaving `None` there
        // 2. Work with the value
        // 3. Put value back inside Cell
        //
        // It's not guaranteed, but usually compiler is able to optimize this and eliminate Cell::take + Cell::set
        // sequence if no side effects happen in between. In this case, Waker::will_wake() does not have side effects,
        // so there should be no added cost to this sequence in release mode (I confirmed by reviewing assembly).

        let old_waker = self.waker.take();
        let will_wake_same_task = old_waker.as_ref().is_some_and(|w2| w2.will_wake(w));
        self.waker.set(old_waker);

        // Optimization: If both the old and new Wakers wake the same task, we can simply
        // keep the old waker, skipping the clone. (In most executor implementations,
        // cloning a waker is somewhat expensive, comparable to cloning an Arc).
        if will_wake_same_task {
            return;
        }

        // clone the new waker and store it
        if let Some(old_waker) = self.waker.replace(Some(w.clone())) {
            // We had a waker registered for another task. Wake it, so the other task can
            // reregister itself if it's still interested.
            //
            // If two tasks are waiting on the same thing concurrently, this will cause them
            // to wake each other in a loop fighting over this WakerRegistration. This wastes
            // CPU but things will still work.
            //
            // If the user wants to have two tasks waiting on the same thing they should use
            // a more appropriate primitive that can store multiple wakers.
            old_waker.wake();
        }
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        if let Some(w) = self.waker.take() {
            w.wake()
        }
    }

    /// Returns true if a waker is currently registered
    pub fn occupied(&self) -> bool {
        let waker = self.waker.take();
        let is_occupied = waker.is_some();
        self.waker.set(waker);
        is_occupied
    }
}
