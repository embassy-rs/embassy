#![macro_use]
use crate::pac::rng::{regs, Rng};
use crate::peripherals;
use embassy::util::Unborrow;
use embassy_extras::unborrow;

pub struct Random<T: Instance> {
    inner: T,
}

impl<T: Instance> Random<T> {
    pub fn new(inner: impl Unborrow<Target = T>) -> Self {
        unborrow!(inner);
        Self { inner }
    }
}

use core::future::Future;
use core::marker::PhantomData;
use embassy::traits::rng::Rng as RngTrait;

impl<T: Instance> RngTrait for Random<T> {
    type Error = ();
    #[rustfmt::skip]
    type RngFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>>;

    fn fill<'a>(&'a mut self, dest: &'a mut [u8]) -> Self::RngFuture<'a> {
        async move { Ok(()) }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> Rng;
    }
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_rng {
    ($inst:ident) => {
        impl crate::rng::sealed::Instance for peripherals::$inst {
            fn regs(&self) -> crate::pac::rng::Rng {
                crate::pac::$inst
            }
        }

        impl crate::rng::Instance for peripherals::$inst {}
    };
}
