#![macro_use]

use core::future::Future;
use core::task::Poll;
use embassy::traits;
use embassy::util::{AtomicWaker, Unborrow};
use embassy_hal_common::unborrow;
use futures::future::poll_fn;
use rand_core::{CryptoRng, RngCore};

use crate::pac;
use crate::peripherals;

pub(crate) static RNG_WAKER: AtomicWaker = AtomicWaker::new();

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    SeedError,
    ClockError,
}

pub struct Rng<T: Instance> {
    _inner: T,
}

impl<T: Instance> Rng<T> {
    pub fn new(inner: impl Unborrow<Target = T>) -> Self {
        T::enable();
        T::reset();
        unborrow!(inner);
        let mut random = Self { _inner: inner };
        random.reset();
        random
    }

    pub fn reset(&mut self) {
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_rngen(true);
                reg.set_ie(true);
            });
            T::regs().sr().modify(|reg| {
                reg.set_seis(false);
                reg.set_ceis(false);
            });
        }
        // Reference manual says to discard the first.
        let _ = self.next_u32();
    }
}

impl<T: Instance> RngCore for Rng<T> {
    fn next_u32(&mut self) -> u32 {
        loop {
            let bits = unsafe { T::regs().sr().read() };
            if bits.drdy() {
                return unsafe { T::regs().dr().read() };
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

impl<T: Instance> CryptoRng for Rng<T> {}

impl<T: Instance> traits::rng::Rng for Rng<T> {
    type Error = Error;
    #[rustfmt::skip]
    type RngFuture<'a> where Self: 'a = impl Future<Output=Result<(), Self::Error>> + 'a;

    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a> {
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_rngen(true);
            });
        }
        async move {
            for chunk in dest.chunks_mut(4) {
                poll_fn(|cx| {
                    RNG_WAKER.register(cx.waker());
                    unsafe {
                        T::regs().cr().modify(|reg| {
                            reg.set_ie(true);
                        });
                    }

                    let bits = unsafe { T::regs().sr().read() };

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
                let random_bytes = unsafe { T::regs().dr().read() }.to_be_bytes();
                for (dest, src) in chunk.iter_mut().zip(random_bytes.iter()) {
                    *dest = *src
                }
            }
            Ok(())
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> pac::rng::Rng;
    }
}

pub trait Instance: sealed::Instance + crate::rcc::RccPeripheral {}

crate::pac::peripherals!(
    (rng, $inst:ident) => {
        impl Instance for peripherals::$inst {}

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::rng::Rng {
                crate::pac::RNG
            }
        }
    };
);

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

crate::pac::interrupts!(
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
