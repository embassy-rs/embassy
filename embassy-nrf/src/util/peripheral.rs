use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{cell::UnsafeCell, marker::PhantomData};

use crate::fmt::*;
use crate::interrupt::OwnedInterrupt;

pub trait PeripheralState {
    type Interrupt: OwnedInterrupt;
    fn on_interrupt(&mut self);
}

pub struct PeripheralMutex<S: PeripheralState> {
    inner: Option<(UnsafeCell<S>, S::Interrupt)>,
    not_send: PhantomData<*mut ()>,
}

impl<S: PeripheralState> PeripheralMutex<S> {
    pub fn new(state: S, irq: S::Interrupt) -> Self {
        Self {
            inner: Some((UnsafeCell::new(state), irq)),
            not_send: PhantomData,
        }
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut S, &mut S::Interrupt) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };
        let (state, irq) = unwrap!(this.inner.as_mut());

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

        let r = f(state, irq);

        compiler_fence(Ordering::SeqCst);
        irq.enable();

        r
    }

    pub fn try_free(self: Pin<&mut Self>) -> Option<(S, S::Interrupt)> {
        let this = unsafe { self.get_unchecked_mut() };
        this.inner.take().map(|(state, irq)| {
            irq.disable();
            irq.remove_handler();
            (state.into_inner(), irq)
        })
    }

    pub fn free(self: Pin<&mut Self>) -> (S, S::Interrupt) {
        unwrap!(self.try_free())
    }
}

impl<S: PeripheralState> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        if let Some((_state, irq)) = &mut self.inner {
            irq.disable();
            irq.remove_handler();
        }
    }
}
