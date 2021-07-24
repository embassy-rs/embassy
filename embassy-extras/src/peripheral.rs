use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;
use core::ptr;

use embassy::interrupt::{Interrupt, InterruptExt};

/// A version of `PeripheralState` without the `'static` bound,
/// for cases where the compiler can't statically make sure
/// that `on_interrupt` doesn't reference anything which might be invalidated.
///
/// # Safety
/// When types implementing this trait are used with `PeripheralMutex`,
/// no fields referenced by `on_interrupt`'s lifetimes must end without first calling `Drop` on the `PeripheralMutex`.
pub unsafe trait PeripheralStateUnchecked: Send {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

/// A type which can be used as state with `PeripheralMutex`.
///
/// It needs to be `Send` because `&mut` references are sent back and forth between the 'thread' which owns the `PeripheralMutex` and the interrupt,
/// and `&mut T` is `Send` where `T: Send`.
///
/// It also requires `'static`, because although `Pin` guarantees that the memory of the state won't be invalidated,
/// it doesn't guarantee that the lifetime will last.
pub trait PeripheralState: Send + 'static {
    type Interrupt: Interrupt;
    fn on_interrupt(&mut self);
}

// SAFETY: `T` has to live for `'static` to implement `PeripheralState`, thus its lifetime cannot end.
unsafe impl<T> PeripheralStateUnchecked for T
where
    T: PeripheralState,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        self.on_interrupt()
    }
}

pub struct PeripheralMutex<S: PeripheralStateUnchecked> {
    state: UnsafeCell<S>,

    irq_setup_done: bool,
    irq: S::Interrupt,

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

impl<S: PeripheralStateUnchecked> PeripheralMutex<S> {
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
        this.irq.set_handler(|p| {
            critical_section::with(|_| {
                if p.is_null() {
                    // The state was dropped, so we can't operate on it.
                    return;
                }
                // Safety: it's OK to get a &mut to the state, since
                // - We're in a critical section, no one can preempt us (and call with())
                // - We can't have preempted a with() call because the irq is disabled during it.
                let state = unsafe { &mut *(p as *mut S) };
                state.on_interrupt();
            })
        });
        this.irq
            .set_handler_context((&mut this.state) as *mut _ as *mut ());
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn with<R>(self: Pin<&mut Self>, f: impl FnOnce(&mut S, &mut S::Interrupt) -> R) -> R {
        let this = unsafe { self.get_unchecked_mut() };

        this.irq.disable();

        // Safety: it's OK to get a &mut to the state, since the irq is disabled.
        let state = unsafe { &mut *this.state.get() };
        let r = f(state, &mut this.irq);

        this.irq.enable();

        r
    }
}

impl<S: PeripheralStateUnchecked> Drop for PeripheralMutex<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
        // Set the context to null so that the interrupt will know we're dropped
        // if we pre-empted it before it entered a critical section.
        self.irq.set_handler_context(ptr::null_mut());
    }
}
