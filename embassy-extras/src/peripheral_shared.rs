use core::cell::UnsafeCell;
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

use embassy::interrupt::{Interrupt, InterruptExt};

pub trait PeripheralState {
    type Interrupt: Interrupt;
    fn on_interrupt(&self);
}

pub struct Peripheral<S: PeripheralState> {
    state: UnsafeCell<S>,

    irq_setup_done: bool,
    irq: S::Interrupt,

    _not_send: PhantomData<*mut ()>,
    _pinned: PhantomPinned,
}

impl<S: PeripheralState> Peripheral<S> {
    pub fn new(irq: S::Interrupt, state: S) -> Self {
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
            let state = unsafe { &*(p as *const S) };
            state.on_interrupt();
        });
        this.irq
            .set_handler_context((&this.state) as *const _ as *mut ());
        this.irq.enable();

        this.irq_setup_done = true;
    }

    pub fn state(self: Pin<&mut Self>) -> &S {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { &*this.state.get() }
    }
}

impl<S: PeripheralState> Drop for Peripheral<S> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.remove_handler();
    }
}
