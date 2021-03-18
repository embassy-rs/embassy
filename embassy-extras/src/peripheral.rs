use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};

use embassy::interrupt::{Interrupt, InterruptExt};

pub trait PeripheralState {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

pub struct PeripheralMutex<S: PeripheralState> {
    state: UnsafeCell<S>,

    irq_setup_done: bool,
    irq: S::Interrupt,

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

impl<S: PeripheralState> PeripheralMutex<S> {
    pub fn new(state: S, irq: S::Interrupt) -> Self {
        Self {
            irq,
            irq_setup_done: false,

            state: UnsafeCell::new(state),
            _not_send: PhantomData,
            _pinned: PhantomPinned,
        }
    }

    pub fn register_interrupt(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        if this.irq_setup_done {
            return;
        }

        this.irq.disable();
        compiler_fence(Ordering::SeqCst);

        this.irq.set_handler(|p| {
            // Safety: it's OK to get a &mut to the state, since
            // - We're in the IRQ, no one else can't preempt us
            // - We can't have preempted a with() call because the irq is disabled during it.
            let state = unsafe { &mut *(p as *mut S) };
            state.on_interrupt();
        });
        this.irq
            .set_handler_context((&mut this.state) as *mut _ as *mut ());

        compiler_fence(Ordering::SeqCst);
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut S, &mut S::Interrupt) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };

        this.irq.disable();
        compiler_fence(Ordering::SeqCst);

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *this.state.get() };

        let r = f(state, &mut this.irq);

        compiler_fence(Ordering::SeqCst);
        this.irq.enable();

        r
    }
}

impl<S: PeripheralState> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
    }
}
