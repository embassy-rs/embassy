use core::future::Future;
use embassy::util::Unborrow;

pub trait WriteDma<T> {
    type WriteDmaFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
    where
        T: 'a;
}

pub trait ReadDma<T> {
    type ReadDmaFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    fn transfer<'a>(&'a mut self, src: *const u8, buf: &'a mut [u8]) -> Self::ReadDmaFuture<'a>
    where
        T: 'a;
}

pub struct NoDma;

unsafe impl Unborrow for NoDma {
    type Target = NoDma;

    unsafe fn unborrow(self) -> Self::Target {
        self
    }
}
