use core::task::Waker;

use super::WakerRegistration;

/// Utility struct to register and wake multiple wakers.
pub struct MultiWakerRegistration<const N: usize> {
    wakers: [WakerRegistration; N],
}

impl<const N: usize> MultiWakerRegistration<N> {
    /// Create a new empty instance
    pub const fn new() -> Self {
        const WAKER: WakerRegistration = WakerRegistration::new();
        Self { wakers: [WAKER; N] }
    }

    /// Register a waker. If the buffer is full the function returns it in the error
    pub fn register<'a>(&mut self, w: &'a Waker) -> Result<(), &'a Waker> {
        if let Some(waker_slot) = self.wakers.iter_mut().find(|waker_slot| !waker_slot.occupied()) {
            waker_slot.register(w);
            Ok(())
        } else {
            Err(w)
        }
    }

    /// Wake all registered wakers. This clears the buffer
    pub fn wake(&mut self) {
        for waker_slot in self.wakers.iter_mut() {
            waker_slot.wake()
        }
    }
}
