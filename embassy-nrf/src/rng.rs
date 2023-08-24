//! Random Number Generator (RNG) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let s = T::state();
        let r = T::regs();

        // Clear the event.
        r.events_valrdy.reset();

        // Mutate the slice within a critical section,
        // so that the future isn't dropped in between us loading the pointer and actually dereferencing it.
        let (ptr, end) = critical_section::with(|_| {
            let ptr = s.ptr.load(Ordering::Relaxed);
            // We need to make sure we haven't already filled the whole slice,
            // in case the interrupt fired again before the executor got back to the future.
            let end = s.end.load(Ordering::Relaxed);
            if !ptr.is_null() && ptr != end {
                // If the future was dropped, the pointer would have been set to null,
                // so we're still good to mutate the slice.
                // The safety contract of `Rng::new` means that the future can't have been dropped
                // without calling its destructor.
                unsafe {
                    *ptr = r.value.read().value().bits();
                }
            }
            (ptr, end)
        });

        if ptr.is_null() || ptr == end {
            // If the future was dropped, there's nothing to do.
            // If `ptr == end`, we were called by mistake, so return.
            return;
        }

        let new_ptr = unsafe { ptr.add(1) };
        match s
            .ptr
            .compare_exchange(ptr, new_ptr, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => {
                let end = s.end.load(Ordering::Relaxed);
                // It doesn't matter if `end` was changed under our feet, because then this will just be false.
                if new_ptr == end {
                    s.waker.wake();
                }
            }
            Err(_) => {
                // If the future was dropped or finished, there's no point trying to wake it.
                // It will have already stopped the RNG, so there's no need to do that either.
            }
        }
    }
}

/// A wrapper around an nRF RNG peripheral.
///
/// It has a non-blocking API, and a blocking api through `rand`.
pub struct Rng<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Creates a new RNG driver from the `RNG` peripheral and interrupt.
    ///
    /// SAFETY: The future returned from `fill_bytes` must not have its lifetime end without running its destructor,
    /// e.g. using `mem::forget`.
    ///
    /// The synchronous API is safe.
    pub fn new(
        rng: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(rng);

        let this = Self { _peri: rng };

        this.stop();
        this.disable_irq();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        this
    }

    fn stop(&self) {
        T::regs().tasks_stop.write(|w| unsafe { w.bits(1) })
    }

    fn start(&self) {
        T::regs().tasks_start.write(|w| unsafe { w.bits(1) })
    }

    fn enable_irq(&self) {
        T::regs().intenset.write(|w| w.valrdy().set());
    }

    fn disable_irq(&self) {
        T::regs().intenclr.write(|w| w.valrdy().clear());
    }

    /// Enable or disable the RNG's bias correction.
    ///
    /// Bias correction removes any bias towards a '1' or a '0' in the bits generated.
    /// However, this makes the generation of numbers slower.
    ///
    /// Defaults to disabled.
    pub fn set_bias_correction(&self, enable: bool) {
        T::regs().config.write(|w| w.dercen().bit(enable))
    }

    /// Fill the buffer with random bytes.
    pub async fn fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.len() == 0 {
            return; // Nothing to fill
        }

        let s = T::state();

        let range = dest.as_mut_ptr_range();
        // Even if we've preempted the interrupt, it can't preempt us again,
        // so we don't need to worry about the order we write these in.
        s.ptr.store(range.start, Ordering::Relaxed);
        s.end.store(range.end, Ordering::Relaxed);

        self.enable_irq();
        self.start();

        let on_drop = OnDrop::new(|| {
            self.stop();
            self.disable_irq();

            // The interrupt is now disabled and can't preempt us anymore, so the order doesn't matter here.
            s.ptr.store(ptr::null_mut(), Ordering::Relaxed);
            s.end.store(ptr::null_mut(), Ordering::Relaxed);
        });

        poll_fn(|cx| {
            s.waker.register(cx.waker());

            // The interrupt will never modify `end`, so load it first and then get the most up-to-date `ptr`.
            let end = s.end.load(Ordering::Relaxed);
            let ptr = s.ptr.load(Ordering::Relaxed);

            if ptr == end {
                // We're done.
                Poll::Ready(())
            } else {
                Poll::Pending
            }
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
            while regs.events_valrdy.read().bits() == 0 {}
            regs.events_valrdy.reset();
            *byte = regs.value.read().value().bits();
        }

        self.stop();
    }
}

impl<'d, T: Instance> Drop for Rng<'d, T> {
    fn drop(&mut self) {
        self.stop();
        let s = T::state();
        s.ptr.store(ptr::null_mut(), Ordering::Relaxed);
        s.end.store(ptr::null_mut(), Ordering::Relaxed);
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

pub(crate) mod sealed {
    use super::*;

    /// Peripheral static state
    pub struct State {
        pub ptr: AtomicPtr<u8>,
        pub end: AtomicPtr<u8>,
        pub waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                ptr: AtomicPtr::new(ptr::null_mut()),
                end: AtomicPtr::new(ptr::null_mut()),
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static crate::pac::rng::RegisterBlock;
        fn state() -> &'static State;
    }
}

/// RNG peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_rng {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::rng::sealed::Instance for peripherals::$type {
            fn regs() -> &'static crate::pac::rng::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::rng::sealed::State {
                static STATE: crate::rng::sealed::State = crate::rng::sealed::State::new();
                &STATE
            }
        }
        impl crate::rng::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
