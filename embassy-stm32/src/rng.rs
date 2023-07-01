#![macro_use]

use core::future::poll_fn;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use rand_core::{CryptoRng, RngCore};

use crate::{pac, peripherals, Peripheral};

pub(crate) static RNG_WAKER: AtomicWaker = AtomicWaker::new();

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    SeedError,
    ClockError,
}

pub struct Rng<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Rng<'d, T> {
    pub fn new(inner: impl Peripheral<P = T> + 'd) -> Self {
        T::enable();
        T::reset();
        into_ref!(inner);
        let mut random = Self { _inner: inner };
        random.reset();
        random
    }

    #[cfg(rng_v1)]
    pub fn reset(&mut self) {
        // rng_v2 locks up on seed error, needs reset
        #[cfg(rng_v2)]
        if T::regs().sr().read().seis() {
            T::reset();
        }
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
            reg.set_ie(true);
        });
        T::regs().sr().modify(|reg| {
            reg.set_seis(false);
            reg.set_ceis(false);
        });
        // Reference manual says to discard the first.
        let _ = self.next_u32();
    }

    #[cfg(not(rng_v1))]
    pub fn reset(&mut self) {
        T::regs().cr().modify(|reg| {
            reg.set_rngen(false);
            reg.set_condrst(true);
            // set RNG config "A" according to reference manual
            // this has to be written within the same write access as setting the CONDRST bit
            reg.set_nistc(pac::rng::vals::Nistc::DEFAULT);
            reg.set_rng_config1(pac::rng::vals::RngConfig1::CONFIGA);
            reg.set_rng_config2(pac::rng::vals::RngConfig2::CONFIGA_B);
            reg.set_rng_config3(pac::rng::vals::RngConfig3::CONFIGA);
            reg.set_clkdiv(pac::rng::vals::Clkdiv::NODIV);
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
            reg.set_ie(true);
        });
        // wait for CONDRST to be reset
        while T::regs().cr().read().condrst() {}
    }

    pub async fn async_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        T::regs().cr().modify(|reg| {
            reg.set_rngen(true);
        });

        for chunk in dest.chunks_mut(4) {
            poll_fn(|cx| {
                RNG_WAKER.register(cx.waker());
                T::regs().cr().modify(|reg| {
                    reg.set_ie(true);
                });

                let bits = T::regs().sr().read();

                if bits.drdy() {
                    Poll::Ready(Ok(()))
                } else if bits.seis() {
                    self.reset();
                    Poll::Ready(Err(Error::SeedError))
                } else if bits.ceis() {
                    self.reset();
                    Poll::Ready(Err(Error::ClockError))
                } else {
                    Poll::Pending
                }
            })
            .await?;
            let random_bytes = T::regs().dr().read().to_be_bytes();
            for (dest, src) in chunk.iter_mut().zip(random_bytes.iter()) {
                *dest = *src
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
            for (slot, num) in chunk.iter_mut().zip(rand.to_be_bytes().iter()) {
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

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> pac::rng::Rng;
    }
}

pub trait Instance: sealed::Instance + crate::rcc::RccPeripheral {}

foreach_peripheral!(
    (rng, $inst:ident) => {
        impl Instance for peripherals::$inst {}

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::rng::Rng {
                crate::pac::RNG
            }
        }
    };
);

#[cfg(feature = "rt")]
macro_rules! irq {
    ($irq:ident) => {
        mod rng_irq {
            use crate::interrupt;

            #[interrupt]
            unsafe fn $irq() {
                let bits = $crate::pac::RNG.sr().read();
                if bits.drdy() || bits.seis() || bits.ceis() {
                    $crate::pac::RNG.cr().write(|reg| reg.set_ie(false));
                    $crate::rng::RNG_WAKER.wake();
                }
            }
        }
    };
}

#[cfg(feature = "rt")]
foreach_interrupt!(
    (RNG) => {
        irq!(RNG);
    };

    (RNG_LPUART1) => {
        irq!(RNG_LPUART1);
    };

    (AES_RNG_LPUART1) => {
        irq!(AES_RNG_LPUART1);
    };

    (AES_RNG) => {
        irq!(AES_RNG);
    };

    (HASH_RNG) => {
        irq!(HASH_RNG);
    };
);
