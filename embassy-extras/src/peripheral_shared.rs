use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;
use core::ptr;

use embassy::interrupt::{Interrupt, InterruptExt};

/// A version of `PeripheralState` without the `'static` bound,
/// for cases where the compiler can't statically make sure
/// that `on_interrupt` doesn't reference anything which might be invalidated.
///
/// # Safety
/// When types implementing this trait are used with `Peripheral`,
/// no fields referenced by `on_interrupt`'s lifetimes must end without first calling `Drop` on the `Peripheral`.
pub unsafe trait PeripheralStateUnchecked: Sync {
    type Interrupt: Interrupt;
    fn on_interrupt(&self);
}

/// A type which can be used as state with `Peripheral`.
///
/// It needs to be `Sync` because references are shared between the 'thread' which owns the `Peripheral` and the interrupt.
///
/// It also requires `'static`, because although `Pin` guarantees that the memory of the state won't be invalidated,
/// it doesn't guarantee that the lifetime will last.
pub trait PeripheralState: Sync + 'static {
    type Interrupt: Interrupt;
    fn on_interrupt(&self);
}

pub struct Peripheral<S: PeripheralStateUnchecked> {
    state: S,

    irq_setup_done: bool,
    irq: S::Interrupt,

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

impl<S: PeripheralStateUnchecked> Peripheral<S> {
    pub fn new(irq: S::Interrupt, state: S) -> Self {
        Self {
            irq,
            irq_setup_done: false,

            state,
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
        this.irq.set_handler(|p| {
            // We need to be in a critical section so that no one can preempt us
            // and drop the state after we check whether `p.is_null()`.
            critical_section::with(|_| {
                if p.is_null() {
                    // The state was dropped, so we can't operate on it.
                    return;
                }
                let state = unsafe { &*(p as *const S) };
                state.on_interrupt();
            });
        });
        this.irq
            .set_handler_context((&this.state) as *const _ as *mut ());
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn state(self: Pin<&mut Self>) -> &S {
        &self.into_ref().get_ref().state
    }
}

impl<S: PeripheralStateUnchecked> Drop for Peripheral<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
        // Set the context to null so that the interrupt will know we're dropped
        // if we pre-empted it before it entered a critical section.
        self.irq.set_handler_context(ptr::null_mut());
    }
}
