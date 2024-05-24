//! Random Number Generator (RNG)
#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use rand_core::{CryptoRng, RngCore};

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, rcc, Peripheral};

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

/// RNG error
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Seed error.
    SeedError,
    /// Clock error. Double-check the RCC configuration,
    /// see the Reference Manual for details on restrictions
    /// on RNG clocks.
    ClockError,
}

/// RNG interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let bits = T::regs().sr().read();
        if bits.drdy() || bits.seis() || bits.ceis() {
            T::regs().cr().modify(|reg| reg.set_ie(false));
            RNG_WAKER.wake();
        }
    }
}

/// RNG driver.
pub struct Rng<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    /// Create a new RNG driver.
    pub fn new(
        inner: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        into_ref!(inner);
        let mut random = Self { _inner: inner };
        random.reset();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        random
    }

    /// Reset the RNG.
    #[cfg(rng_v1)]
    pub fn reset(&mut self) {
        T::regs().cr().write(|reg| {
            reg.set_rngen(false);
        });
        T::regs().sr().modify(|reg| {
            reg.set_seis(false);
            reg.set_ceis(false);
        });
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
        });
        // Reference manual says to discard the first.
        let _ = self.next_u32();
    }

    /// Reset the RNG.
    #[cfg(not(rng_v1))]
    pub fn reset(&mut self) {
        T::regs().cr().write(|reg| {
            reg.set_condrst(true);
            reg.set_nistc(pac::rng::vals::Nistc::CUSTOM);
            // set RNG config "A" according to reference manual
            // this has to be written within the same write access as setting the CONDRST bit
            reg.set_rng_config1(pac::rng::vals::RngConfig1::CONFIGA);
            reg.set_clkdiv(pac::rng::vals::Clkdiv::NODIV);
            reg.set_rng_config2(pac::rng::vals::RngConfig2::CONFIGA_B);
            reg.set_rng_config3(pac::rng::vals::RngConfig3::CONFIGA);
            reg.set_ced(true);
            reg.set_ie(false);
            reg.set_rngen(true);
        });
        T::regs().cr().modify(|reg| {
            reg.set_ced(false);
        });
        // wait for CONDRST to be set
        while !T::regs().cr().read().condrst() {}
        // magic number must be written immediately before every read or write access to HTCR
        T::regs().htcr().write(|w| w.set_htcfg(pac::rng::vals::Htcfg::MAGIC));
        // write recommended value according to reference manual
        // note: HTCR can only be written during conditioning
        T::regs()
            .htcr()
            .write(|w| w.set_htcfg(pac::rng::vals::Htcfg::RECOMMENDED));
        // finish conditioning
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
            reg.set_condrst(false);
        });
        // wait for CONDRST to be reset
        while T::regs().cr().read().condrst() {}
    }

    /// Try to recover from a seed error.
    pub fn recover_seed_error(&mut self) {
        self.reset();
        // reset should also clear the SEIS flag
        if T::regs().sr().read().seis() {
            warn!("recovering from seed error failed");
            return;
        }
        // wait for SECS to be cleared by RNG
        while T::regs().sr().read().secs() {}
    }

    /// Fill the given slice with random values.
    pub async fn async_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        for chunk in dest.chunks_mut(4) {
            let mut bits = T::regs().sr().read();
            if !bits.seis() && !bits.ceis() && !bits.drdy() {
                // wait for interrupt
                poll_fn(|cx| {
                    // quick check to avoid registration if already done.
                    let bits = T::regs().sr().read();
                    if bits.drdy() || bits.seis() || bits.ceis() {
                        return Poll::Ready(());
                    }
                    RNG_WAKER.register(cx.waker());
                    T::regs().cr().modify(|reg| reg.set_ie(true));
                    // Need to check condition **after** `register` to avoid a race
                    // condition that would result in lost notifications.
                    let bits = T::regs().sr().read();
                    if bits.drdy() || bits.seis() || bits.ceis() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;

                // Re-read the status register after wait.
                bits = T::regs().sr().read()
            }
            if bits.seis() {
                // in case of noise-source or seed error we try to recover here
                // but we must not use the data in DR and we return an error
                // to leave retry-logic to the application
                self.recover_seed_error();
                return Err(Error::SeedError);
            } else if bits.ceis() {
                // clock error detected, DR could still be used but keep it safe,
                // clear the error and abort
                T::regs().sr().modify(|sr| sr.set_ceis(false));
                return Err(Error::ClockError);
            } else if bits.drdy() {
                // DR can be read up to four times until the output buffer is empty
                // DRDY is cleared automatically when that happens
                let random_word = T::regs().dr().read();
                // reference manual: always check if DR is zero
                if random_word == 0 {
                    return Err(Error::SeedError);
                }
                // write bytes to chunk
                for (dest, src) in chunk.iter_mut().zip(random_word.to_ne_bytes().iter()) {
                    *dest = *src
                }
            }
        }

        Ok(())
    }
}

impl<'d, T: Instance> RngCore for Rng<'d, T> {
    fn next_u32(&mut self) -> u32 {
        loop {
            let sr = T::regs().sr().read();
            if sr.seis() | sr.ceis() {
                self.reset();
            } else if sr.drdy() {
                return T::regs().dr().read();
            }
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut rand = self.next_u32() as u64;
        rand |= (self.next_u32() as u64) << 32;
        rand
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let rand = self.next_u32();
            for (slot, num) in chunk.iter_mut().zip(rand.to_ne_bytes().iter()) {
                *slot = *num
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl<'d, T: Instance> CryptoRng for Rng<'d, T> {}

trait SealedInstance {
    fn regs() -> pac::rng::Rng;
}

/// RNG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this RNG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, rng, RNG, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::rng::Rng {
                crate::pac::$inst
            }
        }
    };
);
