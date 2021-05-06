#![macro_use]

//use crate::pac::rng::{regs, Rng};
use crate::pac;
use crate::peripherals;
use crate::interrupt;
use embassy::util::{Unborrow, AtomicWaker};
use embassy_extras::unborrow;
use rand_core::{RngCore, CryptoRng};

use defmt::*;

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

#[interrupt]
unsafe fn RNG() {
    let bits = crate::pac::RNG.sr().read();
    if bits.drdy() || bits.seis() || bits.ceis() {
        crate::pac::RNG.cr().write(|reg| reg.set_ie(false));
        RNG_WAKER.wake();
    }
}

pub struct Random<T: Instance> {
    inner: T,
}

impl<T: Instance> Random<T> {
    pub fn new(inner: impl Unborrow<Target=T>) -> Self {
        unborrow!(inner);
        Self { inner }
    }
}

impl<T: Instance> RngCore for Random<T> {
    fn next_u32(&mut self) -> u32 {
        loop {
            let bits = unsafe { T::regs().sr().read() };
            if bits.drdy() {
                return unsafe{ T::regs().dr().read() }
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
        self.fill_bytes( dest );
        Ok(())
    }
}

impl<T: Instance> CryptoRng for Random<T> { }

use core::future::Future;
use core::marker::PhantomData;
use embassy::traits;
use core::task::{Poll, Context};
use core::pin::Pin;

struct RngInterruptFuture<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> Future for RngInterruptFuture<T> {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RNG_WAKER.register(cx.waker());

        let bits = unsafe { T::regs().sr().read() };

        if bits.drdy() {
            return Poll::Ready(Ok(()));
        } else if bits.seis() {
            unsafe {
                T::regs().sr().modify(|reg| {
                    reg.set_seis(false);
                });
            }
        } else if bits.ceis() {
            unsafe {
                T::regs().sr().modify(|reg| {
                    reg.set_ceis(false);
                });
            }
        }

        Poll::Pending
    }
}

impl<T: Instance> RngInterruptFuture<T> {
    async fn new() -> Result<(), Error> {
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_ie(true);
                //reg.set_rngen(true);
            });
        }

        Self {
            _marker: PhantomData
        }.await
    }
}

pub enum Error {
    SeedError,
    ClockError,
}

impl<T: Instance> traits::rng::Rng for Random<T> {
    type Error = Error;
    type RngFuture<'a> where Self: 'a = impl Future<Output=Result<(), Self::Error>>;

    fn fill_bytes<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a> {
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_rngen(true);
            });
        }

        async move {
            for chunk in dest.chunks_mut(4) {
                RngInterruptFuture::<T>::new().await?;
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

pub trait Instance: sealed::Instance {}

macro_rules! impl_rng {
    ($addr:ident) => {
        impl crate::rng::sealed::Instance for peripherals::RNG {
            fn regs() -> crate::pac::chip::rng::Rng {
                crate::pac::RNG
            }
        }

        impl crate::rng::Instance for peripherals::RNG {}
    };
}
