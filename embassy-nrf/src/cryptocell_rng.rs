//! Random Number Generator (RNG) driver.

#![macro_use]

use core::cell::{RefCell, RefMut};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::task::Poll;

use critical_section::{CriticalSection, Mutex};
#[cfg(feature = "_nrf5340-app")]
use embassy_futures::{select::select, yield_now};
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::WakerRegistration;

use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, pac};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();

        // Clear the event.
        r.rng_icr().write(|w| w.set_ehr_valid_clear(true));
        pac::CC_HOST_RGF.icr().write(|w| w.set_rng_clear(true));

        // Mutate the slice within a critical section,
        // so that the future isn't dropped in between us loading the pointer and actually dereferencing it.
        critical_section::with(|cs| {
            let mut state = T::state().borrow_mut(cs);
            // We need to make sure we haven't already filled the whole slice,
            // in case the interrupt fired again before the executor got back to the future.
            if !state.ptr.is_null() && state.ptr != state.end {
                // If the future was dropped, the pointer would have been set to null,
                // so we're still good to mutate the slice.
                // The safety contract of `CcRng::new` means that the future can't have been dropped
                // without calling its destructor.

                for i in 0..6 {
                    let bytes = r.ehr_data(i).read().to_ne_bytes();
                    for b in bytes {
                        unsafe {
                            *state.ptr = b;
                            state.ptr = state.ptr.add(1);
                        }

                        if state.ptr == state.end {
                            state.waker.wake();
                            return;
                        }
                    }
                }
            }
        });
    }
}

/// A wrapper around an nRF CryptoCell RNG peripheral.
///
/// It has a non-blocking API, and a blocking api through `rand`.
pub struct CcRng<'d, M: Mode> {
    r: pac::cc_rng::CcRng,
    state: &'static State,
    _phantom: PhantomData<(&'d (), M)>,
}

impl<'d> CcRng<'d, Blocking> {
    /// Creates a new RNG driver from the `CC_RNG` peripheral and interrupt.
    ///
    /// SAFETY: The future returned from `fill_bytes` must not have its lifetime end without running its destructor,
    /// e.g. using `mem::forget`.
    ///
    /// The synchronous API is safe.
    pub fn new_blocking<T: Instance>(_rng: Peri<'d, T>) -> Self {
        let this = Self {
            r: T::regs(),
            state: T::state(),
            _phantom: PhantomData,
        };

        this.stop();

        this
    }
}

impl<'d> CcRng<'d, Async> {
    /// Creates a new RNG driver from the `CC_RNG` peripheral and interrupt.
    ///
    /// SAFETY: The future returned from `fill_bytes` must not have its lifetime end without running its destructor,
    /// e.g. using `mem::forget`.
    ///
    /// The synchronous API is safe.
    pub fn new<T: Instance>(
        _rng: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        let this = Self {
            r: T::regs(),
            state: T::state(),
            _phantom: PhantomData,
        };

        this.disable_irq();
        this.stop();

        T::Interrupt::unpend();

        unsafe { T::Interrupt::enable() };

        this
    }

    fn enable_irq(&self) {
        pac::CC_HOST_RGF
            .imr()
            .modify(|w| w.set_rng_mask(pac::cc_host_rgf::vals::RngMask::IRQENABLE));
        self.r
            .rng_imr()
            .modify(|w| w.set_ehr_valid_mask(pac::cc_rng::vals::EhrValidMask::IRQENABLE));
    }

    fn disable_irq(&self) {
        self.r.rng_icr().write(|w| w.set_ehr_valid_clear(true));
        pac::CC_HOST_RGF.icr().write(|w| w.set_rng_clear(true));
        self.r
            .rng_imr()
            .modify(|w| w.set_ehr_valid_mask(pac::cc_rng::vals::EhrValidMask::IRQDISABLE));
        pac::CC_HOST_RGF
            .imr()
            .modify(|w| w.set_rng_mask(pac::cc_host_rgf::vals::RngMask::IRQDISABLE));
    }

    /// Fill the buffer with random bytes.
    pub async fn fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.is_empty() {
            return; // Nothing to fill
        }

        let range = dest.as_mut_ptr_range();

        let state = self.state;
        // Even if we've preempted the interrupt, it can't preempt us again,
        // so we don't need to worry about the order we write these in.
        critical_section::with(|cs| {
            let mut state = state.borrow_mut(cs);
            state.ptr = range.start;
            state.end = range.end;
        });

        // In self.start() there are calls to set_enable() that resets the interrupt mask,
        // self.enable_irq() needs to be called after self.start().
        self.start();

        self.enable_irq();

        let on_drop = OnDrop::new(|| {
            self.disable_irq();
            self.stop();

            critical_section::with(|cs| {
                let mut state = state.borrow_mut(cs);
                state.ptr = ptr::null_mut();
                state.end = ptr::null_mut();
            });
        });

        let fill_future = poll_fn(|cx| {
            critical_section::with(|cs| {
                let mut s = state.borrow_mut(cs);
                s.waker.register(cx.waker());
                if s.ptr == s.end {
                    // We're done.
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        });

        // nrf5340 needs to be reading from the CryptoCell in order to receive an interrupt from it.
        #[cfg(feature = "_nrf5340-app")]
        let _ = select(fill_future, async {
            loop {
                let _ = pac::CRYPTOCELL.enable().read().enable();
                yield_now().await;
            }
        })
        .await;
        #[cfg(not(feature = "_nrf5340-app"))]
        fill_future.await;

        // Trigger the teardown
        drop(on_drop);
    }
}

impl<'d, M: Mode> CcRng<'d, M> {
    fn start(&self) {
        // FIXME: CRYPTOCELL is never disabled.
        if !pac::CRYPTOCELL.enable().read().enable() {
            pac::CRYPTOCELL.enable().write(|w| w.set_enable(true));
        }

        self.r.rng_clk().write(|w| w.set_enable(true));
        self.r.rng_sw_reset().write(|w| w.set_reset(true));

        // Wait for calibration
        // ROSC1 (ring oscillator lenght) chosen arbitrarly, can be later exposed as configuration.
        loop {
            self.r.rng_clk().write(|w| w.set_enable(true));
            self.r.sample_cnt().write_value(pac::FICR.trng90b().rosc1().read());
            if self.r.sample_cnt().read() == pac::FICR.trng90b().rosc1().read() {
                break;
            };
        }
        self.r
            .trng_config()
            .modify(|w| w.set_rosc_len(pac::cc_rng::vals::TrngConfigRoscLen::ROSC1));
        self.r.noise_source().modify(|w| w.set_enable(true));
    }

    fn stop(&self) {
        self.r.noise_source().modify(|w| w.set_enable(false));

        self.r.rng_clk().write(|w| w.set_enable(false));
    }

    /// Fill the buffer with random bytes, blocking version.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        self.start();
        self.inner_fill_bytes(dest);
        self.stop();
    }

    // inner function so we can use `return` to end all the loops
    fn inner_fill_bytes(&mut self, dest: &mut [u8]) {
        let mut index = 0;
        while index < dest.len() {
            while !self.r.rng_isr().read().ehr_valid_int() {}
            self.r.rng_icr().write(|w| w.set_ehr_valid_clear(true));

            for i in 0..6 {
                let bytes = self.r.ehr_data(i).read().to_ne_bytes();
                for b in bytes {
                    dest[index] = b;
                    index += 1;

                    if index >= dest.len() {
                        return;
                    }
                }
            }
        }
    }

    /// Generate a random u32
    pub fn blocking_next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.blocking_fill_bytes(&mut bytes);
        // We don't care about the endianness, so just use the native one.
        u32::from_ne_bytes(bytes)
    }

    /// Generate a random u64
    pub fn blocking_next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.blocking_fill_bytes(&mut bytes);
        u64::from_ne_bytes(bytes)
    }
}

impl<'d, M: Mode> Drop for CcRng<'d, M> {
    fn drop(&mut self) {
        self.stop();
        critical_section::with(|cs| {
            let mut state = self.state.borrow_mut(cs);
            state.ptr = ptr::null_mut();
            state.end = ptr::null_mut();
        });
    }
}

impl<'d, M: Mode> rand_core_06::RngCore for CcRng<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, M: Mode> rand_core_06::CryptoRng for CcRng<'d, M> {}

impl<'d, M: Mode> rand_core_09::RngCore for CcRng<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
}

impl<'d, M: Mode> rand_core_09::CryptoRng for CcRng<'d, M> {}

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
    fn regs() -> pac::cc_rng::CcRng;
    fn state() -> &'static State;
}

/// RNG peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_ccrng {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::cryptocell_rng::SealedInstance for peripherals::$type {
            fn regs() -> pac::cc_rng::CcRng {
                pac::$pac_type
            }
            fn state() -> &'static crate::cryptocell_rng::State {
                static STATE: crate::cryptocell_rng::State = crate::cryptocell_rng::State::new();
                &STATE
            }
        }
        impl crate::cryptocell_rng::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
