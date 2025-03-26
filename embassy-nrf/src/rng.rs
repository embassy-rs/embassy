//! Random Number Generator (RNG) driver.

#![macro_use]

use core::cell::{RefCell, RefMut};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::task::Poll;

use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::WakerRegistration;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();

        // Clear the event.
        r.events_valrdy().write_value(0);

        // Mutate the slice within a critical section,
        // so that the future isn't dropped in between us loading the pointer and actually dereferencing it.
        critical_section::with(|cs| {
            let mut state = T::state().borrow_mut(cs);
            // We need to make sure we haven't already filled the whole slice,
            // in case the interrupt fired again before the executor got back to the future.
            if !state.ptr.is_null() && state.ptr != state.end {
                // If the future was dropped, the pointer would have been set to null,
                // so we're still good to mutate the slice.
                // The safety contract of `Rng::new` means that the future can't have been dropped
                // without calling its destructor.
                unsafe {
                    *state.ptr = r.value().read().value();
                    state.ptr = state.ptr.add(1);
                }

                if state.ptr == state.end {
                    state.waker.wake();
                }
            }
        });
    }
}

/// A wrapper around an nRF RNG peripheral.
///
/// It has a non-blocking API, and a blocking api through `rand`.
pub struct Rng<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Creates a new RNG driver from the `RNG` peripheral and interrupt.
    ///
    /// SAFETY: The future returned from `fill_bytes` must not have its lifetime end without running its destructor,
    /// e.g. using `mem::forget`.
    ///
    /// The synchronous API is safe.
    pub fn new(
        rng: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        let this = Self { _peri: rng };

        this.stop();
        this.disable_irq();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        this
    }

    fn stop(&self) {
        T::regs().tasks_stop().write_value(1)
    }

    fn start(&self) {
        T::regs().tasks_start().write_value(1)
    }

    fn enable_irq(&self) {
        T::regs().intenset().write(|w| w.set_valrdy(true));
    }

    fn disable_irq(&self) {
        T::regs().intenclr().write(|w| w.set_valrdy(true));
    }

    /// Enable or disable the RNG's bias correction.
    ///
    /// Bias correction removes any bias towards a '1' or a '0' in the bits generated.
    /// However, this makes the generation of numbers slower.
    ///
    /// Defaults to disabled.
    pub fn set_bias_correction(&self, enable: bool) {
        T::regs().config().write(|w| w.set_dercen(enable))
    }

    /// Fill the buffer with random bytes.
    pub async fn fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.is_empty() {
            return; // Nothing to fill
        }

        let range = dest.as_mut_ptr_range();
        // Even if we've preempted the interrupt, it can't preempt us again,
        // so we don't need to worry about the order we write these in.
        critical_section::with(|cs| {
            let mut state = T::state().borrow_mut(cs);
            state.ptr = range.start;
            state.end = range.end;
        });

        self.enable_irq();
        self.start();

        let on_drop = OnDrop::new(|| {
            self.stop();
            self.disable_irq();

            critical_section::with(|cs| {
                let mut state = T::state().borrow_mut(cs);
                state.ptr = ptr::null_mut();
                state.end = ptr::null_mut();
            });
        });

        poll_fn(|cx| {
            critical_section::with(|cs| {
                let mut s = T::state().borrow_mut(cs);
                s.waker.register(cx.waker());
                if s.ptr == s.end {
                    // We're done.
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        })
        .await;

        // Trigger the teardown
        drop(on_drop);
    }

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        self.start();

        for byte in dest.iter_mut() {
            let regs = T::regs();
            while regs.events_valrdy().read() == 0 {}
            regs.events_valrdy().write_value(0);
            *byte = regs.value().read().value();
        }

        self.stop();
    }
}

impl<'d, T: Instance> Drop for Rng<'d, T> {
    fn drop(&mut self) {
        self.stop();
        critical_section::with(|cs| {
            let mut state = T::state().borrow_mut(cs);
            state.ptr = ptr::null_mut();
            state.end = ptr::null_mut();
        });
    }
}

impl<'d, T: Instance> rand_core::RngCore for Rng<'d, T> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }

    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.blocking_fill_bytes(&mut bytes);
        // We don't care about the endianness, so just use the native one.
        u32::from_ne_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.blocking_fill_bytes(&mut bytes);
        u64::from_ne_bytes(bytes)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, T: Instance> rand_core::CryptoRng for Rng<'d, T> {}

/// Peripheral static state
pub(crate) struct State {
    inner: Mutex<RefCell<InnerState>>,
}

struct InnerState {
    ptr: *mut u8,
    end: *mut u8,
    waker: WakerRegistration,
}

unsafe impl Send for InnerState {}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(InnerState::new())),
        }
    }

    fn borrow_mut<'cs>(&'cs self, cs: CriticalSection<'cs>) -> RefMut<'cs, InnerState> {
        self.inner.borrow(cs).borrow_mut()
    }
}

impl InnerState {
    const fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            end: ptr::null_mut(),
            waker: WakerRegistration::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::rng::Rng;
    fn state() -> &'static State;
}

/// RNG peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_rng {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::rng::SealedInstance for peripherals::$type {
            fn regs() -> crate::pac::rng::Rng {
                pac::$pac_type
            }
            fn state() -> &'static crate::rng::State {
                static STATE: crate::rng::State = crate::rng::State::new();
                &STATE
            }
        }
        impl crate::rng::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
