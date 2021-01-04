use core::mem;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{cell::UnsafeCell, marker::PhantomData};

use crate::interrupt::OwnedInterrupt;

pub struct Store<T>(MaybeUninit<UnsafeCell<T>>);
impl<T> Store<T> {
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    unsafe fn as_mut_ptr(&self) -> *mut T {
        (*self.0.as_ptr()).get()
    }

    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.as_mut_ptr()
    }

    unsafe fn write(&self, val: T) {
        ptr::write(self.as_mut_ptr(), val)
    }

    unsafe fn drop_in_place(&self) {
        ptr::drop_in_place(self.as_mut_ptr())
    }

    unsafe fn read(&self) -> T {
        ptr::read(self.as_mut_ptr())
    }
}
unsafe impl<T> Send for Store<T> {}
unsafe impl<T> Sync for Store<T> {}

pub trait State: Sized {
    type Interrupt: OwnedInterrupt;
    fn on_interrupt(&mut self);
    #[doc(hidden)]
    fn store<'a>() -> &'a Store<Self>;
}

pub struct Registration<P: State> {
    irq: P::Interrupt,
    not_send: PhantomData<*mut P>,
}

impl<P: State> Registration<P> {
    pub fn new(irq: P::Interrupt, state: P) -> Self {
        // safety:
        // - No other PeripheralRegistration can already exist because we have the owned interrupt
        // - therefore, storage is uninitialized
        // - therefore it's safe to overwrite it without dropping the previous contents
        unsafe { P::store().write(state) }

        irq.set_handler(|| {
            // safety:
            // - If a PeripheralRegistration instance exists, P::storage() is initialized.
            // - It's OK to get a &mut to it since the irq is disabled.
            unsafe { P::store().as_mut() }.on_interrupt();
        });

        compiler_fence(Ordering::SeqCst);
        irq.enable();

        Self {
            irq,
            not_send: PhantomData,
        }
    }

    pub fn with<R>(&mut self, f: impl FnOnce(&mut P, &mut P::Interrupt) -> R) -> R {
        self.irq.disable();
        compiler_fence(Ordering::SeqCst);

        // safety:
        // - If a PeripheralRegistration instance exists, P::storage() is initialized.
        // - It's OK to get a &mut to it since the irq is disabled.
        let r = f(unsafe { P::store().as_mut() }, &mut self.irq);

        compiler_fence(Ordering::SeqCst);
        self.irq.enable();

        r
    }

    pub fn free(self) -> (P::Interrupt, P) {
        let irq = unsafe { ptr::read(&self.irq) };
        irq.disable();
        irq.set_handler(|| ());
        mem::forget(self);
        let storage = P::store();
        (irq, unsafe { storage.read() })
    }
}

impl<P: State> Drop for Registration<P> {
    fn drop(&mut self) {
        self.irq.disable();
        self.irq.set_handler(|| ());

        let storage = P::store();
        unsafe { storage.drop_in_place() };
    }
}
