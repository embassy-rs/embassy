use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{cell::UnsafeCell, marker::PhantomData};

use crate::fmt::*;
use crate::interrupt::OwnedInterrupt;

pub trait PeripheralState {
    fn on_interrupt(&mut self);
}

pub struct PeripheralMutex<I: OwnedInterrupt, S: PeripheralState> {
    inner: Option<(I, UnsafeCell<S>)>,
    not_send: PhantomData<*mut ()>,
}

impl<I: OwnedInterrupt, S: PeripheralState> PeripheralMutex<I, S> {
    pub fn new(irq: I, state: S) -> Self {
        Self {
            inner: Some((irq, UnsafeCell::new(state))),
            not_send: PhantomData,
        }
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut I, &mut S) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };
        let (irq, state) = unwrap!(this.inner.as_mut());

        irq.disable();
        compiler_fence(Ordering::SeqCst);

        irq.set_handler(
            |p| {
                // Safety: it's OK to get a &mut to the state, since
                // - We're in the IRQ, no one else can't preempt us
                // - We can't have preempted a with() call because the irq is disabled during it.
                let state = unsafe { &mut *(p as *mut S) };
                state.on_interrupt();
            },
            state.get() as *mut (),
        );

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *state.get() };

        let r = f(irq, state);

        compiler_fence(Ordering::SeqCst);
        irq.enable();

        r
    }

    pub fn free(self: Pin<&mut Self>) -> (I, S) {
        let this = unsafe { self.get_unchecked_mut() };
        let (irq, state) = unwrap!(this.inner.take());
        irq.disable();
        irq.remove_handler();
        (irq, state.into_inner())
    }
}

impl<I: OwnedInterrupt, S: PeripheralState> Drop for PeripheralMutex<I, S> {
    fn drop(&mut self) {
        if let Some((irq, state)) = &mut self.inner {
            irq.disable();
            irq.remove_handler();
        }
    }
}
