use core::array::IntoIter;
use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};

use crate::fmt::{assert, *};
use crate::interrupt::Interrupt;

pub trait PeripheralState {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Life {
    Ready,
    Created,
    Freed,
}

pub struct PeripheralMutex<S: PeripheralState> {
    life: Life,

    state: MaybeUninit<UnsafeCell<S>>, // Init if life != Freed
    irq: MaybeUninit<S::Interrupt>,    // Init if life != Freed

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

impl<S: PeripheralState> PeripheralMutex<S> {
    pub fn new(state: S, irq: S::Interrupt) -> Self {
        Self {
            life: Life::Created,
            state: MaybeUninit::new(UnsafeCell::new(state)),
            irq: MaybeUninit::new(irq),
            _not_send: PhantomData,
            _pinned: PhantomPinned,
        }
    }

    /// safety: self must be pinned.
    unsafe fn setup(&mut self) {
        assert!(self.life == Life::Created);

        let irq = &mut *self.irq.as_mut_ptr();
        irq.disable();
        compiler_fence(Ordering::SeqCst);

        irq.set_handler(|p| {
            // Safety: it's OK to get a &mut to the state, since
            // - We're in the IRQ, no one else can't preempt us
            // - We can't have preempted a with() call because the irq is disabled during it.
            let state = &mut *(p as *mut S);
            state.on_interrupt();
        });
        irq.set_handler_context(self.state.as_mut_ptr() as *mut ());

        compiler_fence(Ordering::SeqCst);
        irq.enable();

        self.life = Life::Ready;
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut S, &mut S::Interrupt) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };
        if this.life != Life::Ready {
            unsafe { this.setup() }
        }

        let irq = unsafe { &mut *this.irq.as_mut_ptr() };

        irq.disable();
        compiler_fence(Ordering::SeqCst);

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *(*this.state.as_ptr()).get() };

        let r = f(state, irq);

        compiler_fence(Ordering::SeqCst);
        irq.enable();

        r
    }

    pub fn try_free(self: Pin<&mut Self>) -> Option<(S, S::Interrupt)> {
        let this = unsafe { self.get_unchecked_mut() };

        if this.life != Life::Freed {
            return None;
        }

        unsafe { &mut *this.irq.as_mut_ptr() }.disable();
        compiler_fence(Ordering::SeqCst);

        this.life = Life::Freed;

        let state = unsafe { this.state.as_ptr().read().into_inner() };
        let irq = unsafe { this.irq.as_ptr().read() };
        Some((state, irq))
    }

    pub fn free(self: Pin<&mut Self>) -> (S, S::Interrupt) {
        unwrap!(self.try_free())
    }
}

impl<S: PeripheralState> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        if self.life != Life::Freed {
            let irq = unsafe { &mut *self.irq.as_mut_ptr() };
            irq.disable();
            irq.remove_handler();
        }
    }
}
