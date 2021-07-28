use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

use embassy::interrupt::{Interrupt, InterruptExt};

use crate::peripheral::can_be_preempted;

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
        if can_be_preempted(&irq) {
            panic!("`Peripheral` cannot be created in an interrupt with higher priority than the interrupt it wraps");
        }

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
            // The state can't have been dropped, otherwise the interrupt would have been disabled.
            // We checked in `new` that the thread owning the `Peripheral` can't preempt the interrupt,
            // so someone can't have preempted us before this point and dropped the `Peripheral`.
            let state = unsafe { &*(p as *const S) };
            state.on_interrupt();
        });
        this.irq
            .set_handler_context((&this.state) as *const _ as *mut ());
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn state(self: Pin<&mut Self>) -> &S {
        &self.into_ref().get_ref().state
    }

    /// Returns whether the wrapped interrupt is currently in a pending state.
    pub fn is_pending(&self) -> bool {
        self.irq.is_pending()
    }

    /// Forces the wrapped interrupt into a pending state.
    pub fn pend(&self) {
        self.irq.pend()
    }

    /// Forces the wrapped interrupt out of a pending state.
    pub fn unpend(&self) {
        self.irq.unpend()
    }

    /// Gets the priority of the wrapped interrupt.
    pub fn priority(&self) -> <S::Interrupt as Interrupt>::Priority {
        self.irq.get_priority()
    }
}

impl<S: PeripheralStateUnchecked> Drop for Peripheral<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
    }
}
