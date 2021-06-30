use core::cell::RefCell;
use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::task::Poll;
use core::task::Waker;

use embassy::interrupt::InterruptExt;
use embassy::traits;
use embassy::util::CriticalSectionMutex;
use embassy::util::OnDrop;
use embassy::util::Unborrow;
use embassy_extras::unborrow;
use futures::future::poll_fn;
use rand_core::RngCore;

use crate::interrupt;
use crate::pac;
use crate::peripherals::RNG;

impl RNG {
    fn regs() -> &'static pac::rng::RegisterBlock {
        unsafe { &*pac::RNG::ptr() }
    }
}

static STATE: CriticalSectionMutex<RefCell<State>> =
    CriticalSectionMutex::new(RefCell::new(State {
        buffer: None,
        waker: None,
        index: 0,
    }));

struct State {
    buffer: Option<NonNull<[u8]>>,
    waker: Option<Waker>,
    index: usize,
}

// SAFETY: `NonNull` is `!Send` because of the possibility of it being aliased.
// However, `buffer` is only used within `on_interrupt`,
// and the original `&mut` passed to `fill_bytes` cannot be used because the safety contract of `Rng::new`
// means that it must still be borrowed by `RngFuture`, and so `rustc` will not let it be accessed.
unsafe impl Send for State {}

/// A wrapper around an nRF RNG peripheral.
///
/// It has a non-blocking API, through `embassy::traits::Rng`, and a blocking api through `rand`.
pub struct Rng<'d> {
    irq: interrupt::RNG,
    phantom: PhantomData<(&'d mut RNG, &'d mut interrupt::RNG)>,
}

impl<'d> Rng<'d> {
    /// Creates a new RNG driver from the `RNG` peripheral and interrupt.
    ///
    /// SAFETY: The future returned from `fill_bytes` must not have its lifetime end without running its destructor,
    /// e.g. using `mem::forget`.
    ///
    /// The synchronous API is safe.
    pub unsafe fn new(
        _rng: impl Unborrow<Target = RNG> + 'd,
        irq: impl Unborrow<Target = interrupt::RNG> + 'd,
    ) -> Self {
        unborrow!(irq);

        let this = Self {
            irq,
            phantom: PhantomData,
        };

        Self::stop();
        this.disable_irq();

        this.irq.set_handler(Self::on_interrupt);
        this.irq.unpend();
        this.irq.enable();

        this
    }

    fn on_interrupt(_: *mut ()) {
        critical_section::with(|cs| {
            let mut state = STATE.borrow(cs).borrow_mut();
            // SAFETY: the safety requirements on `Rng::new` make sure that the original `&mut`'s lifetime is still valid,
            // meaning it can't be aliased and is a valid pointer.
            let buffer = unsafe { state.buffer.unwrap().as_mut() };
            buffer[state.index] = RNG::regs().value.read().value().bits();
            state.index += 1;
            if state.index == buffer.len() {
                // Stop the RNG within the interrupt so that it doesn't get triggered again on the way to waking the future.
                Self::stop();
                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            }
            RNG::regs().events_valrdy.reset();
        });
    }

    fn stop() {
        RNG::regs().tasks_stop.write(|w| unsafe { w.bits(1) })
    }

    fn start(&self) {
        RNG::regs().tasks_start.write(|w| unsafe { w.bits(1) })
    }

    fn enable_irq(&self) {
        RNG::regs().intenset.write(|w| w.valrdy().set());
    }

    fn disable_irq(&self) {
        RNG::regs().intenclr.write(|w| w.valrdy().clear());
    }

    /// Enable or disable the RNG's bias correction.
    ///
    /// Bias correction removes any bias towards a '1' or a '0' in the bits generated.
    /// However, this makes the generation of numbers slower.
    ///
    /// Defaults to disabled.
    pub fn bias_correction(&self, enable: bool) {
        RNG::regs().config.write(|w| w.dercen().bit(enable))
    }
}

impl<'d> Drop for Rng<'d> {
    fn drop(&mut self) {
        self.irq.disable()
    }
}

impl<'d> traits::rng::Rng for Rng<'d> {
    type Error = Infallible;

    #[rustfmt::skip] // For some reason rustfmt removes the where clause
    type RngFuture<'a> where 'd: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a> {
        async move {
            critical_section::with(|cs| {
                let mut state = STATE.borrow(cs).borrow_mut();
                state.buffer = Some(dest.into());
            });

            self.enable_irq();
            self.start();

            let on_drop = OnDrop::new(|| {
                Self::stop();
                self.disable_irq();
            });

            poll_fn(|cx| {
                critical_section::with(|cs| {
                    let mut state = STATE.borrow(cs).borrow_mut();
                    state.waker = Some(cx.waker().clone());
                    // SAFETY: see safety message in interrupt handler.
                    // Also, both here and in the interrupt handler, we're in a critical section,
                    // so they can't interfere with each other.
                    let buffer = unsafe { state.buffer.unwrap().as_ref() };

                    if state.index == buffer.len() {
                        // Reset the state for next time
                        state.buffer = None;
                        state.index = 0;
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
            })
            .await;

            // Trigger the teardown
            drop(on_drop);

            Ok(())
        }
    }
}

impl<'d> RngCore for Rng<'d> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.start();

        for byte in dest.iter_mut() {
            let regs = RNG::regs();
            while regs.events_valrdy.read().bits() == 0 {}
            regs.events_valrdy.reset();
            *byte = regs.value.read().value().bits();
        }

        Self::stop();
    }

    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        // We don't care about the endianness, so just use the native one.
        u32::from_ne_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.fill_bytes(&mut bytes);
        u64::from_ne_bytes(bytes)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// TODO: Should `Rng` implement `CryptoRng`? It's 'suitable for cryptographic purposes' according to the specification.
