use crate::pac::rng_v1::{regs, Rng};
use crate::{peripherals, pac};
use embassy::util::{Unborrow, AtomicWaker};
use embassy_extras::unborrow;

pub struct Random<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> Random<T> {
    pub fn new(inner: impl Unborrow<Target=T>) -> Self {
        unborrow!(inner);
        Self {
            _marker: PhantomData
        }
    }
}

use embassy::traits::rng::Rng as RngTrait;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};
use core::pin::Pin;

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

pub unsafe fn on_irq<T:Instance>() {
    let bits = T::regs().sr().read();
    if bits.drdy() || bits.seis() || bits.ceis() {
        T::regs().cr().write( |reg| reg.set_ie(false));
        RNG_WAKER.wake();
    }
}

struct RngInterruptFuture<T:Instance> {
    _marker: PhantomData<T>,
}

impl<T:Instance> Future for RngInterruptFuture<T> {
    type Output = Result<(),Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let bits = unsafe { T::regs().sr().read() };

        if bits.drdy() {
            Poll::Ready(Ok(()))
        } else if bits.seis() {
            Poll::Ready(Err(Error::SeedError))
        } else if bits.ceis() {
            Poll::Ready(Err(Error::ClockError))
        } else {
            RNG_WAKER.register(cx.waker());
            unsafe { T::regs().cr().write( |reg| reg.set_ie(true)) };
            Poll::Pending
        }
    }
}

impl<T:Instance> RngInterruptFuture<T> {
    async fn new() -> Result<(), Error> {
        Self {
            _marker: PhantomData
        }.await
    }
}

pub enum Error {
    SeedError,
    ClockError,
}

impl<T:Instance> RngTrait for Random<T> {
    type Error = Error;
    type RngFuture<'a> where Self: 'a = impl Future<Output=Result<(), Self::Error>>;

    fn fill<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a> {
        async move {
            for chunk in dest.chunks_mut(4) {
                RngInterruptFuture::<T>::new().await?;
                let random_bytes = unsafe { T::regs().dr().read() }.to_be_bytes();
                for ( dest, src ) in chunk.iter_mut().zip(random_bytes.iter()) {
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
        fn regs() -> Rng;
    }
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_rng {
    ($addr:expr) => {
        impl crate::rng::sealed::Instance for peripherals::RNG {
            fn regs() -> crate::pac::rng_v1::Rng {
                crate::pac::rng_v1::Rng($addr as _)
            }
        }

        impl crate::rng::Instance for peripherals::RNG {}
    }
}