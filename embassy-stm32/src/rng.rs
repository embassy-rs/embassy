use crate::pac::rng_v1::{regs, Rng};
use crate::{peripherals, pac};
use embassy::util::{Unborrow, AtomicWaker};
use embassy_extras::unborrow;

pub struct Random<T: Instance> {
    inner: T,
}

impl<T: Instance> Random<T> {
    pub fn new(inner: impl Unborrow<Target=T>) -> Self {
        unborrow!(inner);
        Self {
            inner,
        }
    }
}

use embassy::traits::rng::Rng as RngTrait;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};
use core::pin::Pin;

static RNG_WAKER: AtomicWaker = AtomicWaker::new();
const RNG: pac::rng_v1::Rng = pac::rng_v1::Rng( <peripherals::RNG as sealed::Instance>::ADDR as _);

pub unsafe fn on_irq() {
    if is_ready() {
        RNG_WAKER.wake();
    }
}

unsafe fn is_ready() -> bool {
    RNG.sr().read().drdy()
}

unsafe fn is_seed_error() -> bool {
    RNG.sr().read().seis()
}

unsafe fn is_clock_error() -> bool {
    RNG.sr().read().ceis()
}

struct RngInterruptFuture {

}

impl Future for RngInterruptFuture {
    type Output = Result<(),Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RNG_WAKER.register(cx.waker());
        if unsafe{ is_ready() } {
            Poll::Ready(Ok(()))
        } else if unsafe { is_seed_error() } {
            Poll::Ready(Err(Error::SeedError))
        } else if unsafe { is_clock_error() } {
            Poll::Ready(Err(Error::ClockError))
        } else {
            Poll::Pending
        }
    }
}

impl RngInterruptFuture {
    async fn entropy_filled() -> Result<(), Error> {
        RngInterruptFuture { }.await
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
                RngInterruptFuture::entropy_filled().await?;
                let random_bytes = unsafe { self.inner.regs().dr().read() }.to_be_bytes();
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
        const ADDR: u32;
        fn regs(&self) -> Rng;
    }
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_rng {
    ($addr:expr) => {
        impl crate::rng::sealed::Instance for peripherals::RNG {
            const ADDR: u32 = $addr;
            fn regs(&self) -> crate::pac::rng_v1::Rng {
                crate::pac::rng_v1::Rng($addr as _)
            }
        }

        impl crate::rng::Instance for peripherals::RNG {}
    }
}