#[cfg(bdma)]
mod bdma;
#[cfg(dma)]
mod dma;
#[cfg(dmamux)]
mod dmamux;

#[cfg(dmamux)]
pub use dmamux::*;

use core::future::Future;
use core::task::Waker;
use embassy::util::Unborrow;

#[cfg(any(bdma_v2, dma_v2, dmamux))]
pub type Request = u8;
#[cfg(not(any(bdma_v2, dma_v2, dmamux)))]
pub type Request = ();

pub(crate) mod sealed {
    pub trait Channel {}
}

pub trait Channel: sealed::Channel {
    type ReadFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    type WriteFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    fn read<'a>(
        &'a mut self,
        request: Request,
        src: *mut u8,
        buf: &'a mut [u8],
    ) -> Self::ReadFuture<'a>;

    fn write<'a>(
        &'a mut self,
        request: Request,
        buf: &'a [u8],
        dst: *mut u8,
    ) -> Self::WriteFuture<'a>;

    fn write_x<'a>(
        &'a mut self,
        request: Request,
        word: &u8,
        num: usize,
        dst: *mut u8,
    ) -> Self::WriteFuture<'a>;

    fn stop<'a>(&'a mut self);

    fn is_stopped<'a>(&self) -> bool;
    fn remaining_transfers<'a>(&'a mut self) -> u16;
    fn set_waker(&mut self, waker: &Waker);
}

pub struct NoDma;

unsafe impl Unborrow for NoDma {
    type Target = NoDma;

    unsafe fn unborrow(self) -> Self::Target {
        self
    }
}

// safety: must be called only once at startup
pub(crate) unsafe fn init() {
    #[cfg(bdma)]
    bdma::init();
    #[cfg(dma)]
    dma::init();
    #[cfg(dmamux)]
    dmamux::init();
}
