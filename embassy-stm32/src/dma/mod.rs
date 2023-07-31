#[cfg(dma)]
pub(crate) mod dma;
#[cfg(dma)]
pub use dma::*;

// stm32h7 has both dma and bdma. In that case, we export dma as "main" dma,
// and bdma as "secondary", under `embassy_stm32::dma::bdma`.
#[cfg(all(bdma, dma))]
pub mod bdma;

#[cfg(all(bdma, not(dma)))]
pub(crate) mod bdma;
#[cfg(all(bdma, not(dma)))]
pub use bdma::*;

#[cfg(gpdma)]
pub(crate) mod gpdma;
#[cfg(gpdma)]
pub use gpdma::*;

#[cfg(dmamux)]
mod dmamux;

pub(crate) mod ringbuffer;
pub mod word;

use core::mem;

use embassy_hal_internal::impl_peripheral;

#[cfg(dmamux)]
pub use self::dmamux::*;
use crate::interrupt::Priority;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Dir {
    MemoryToPeripheral,
    PeripheralToMemory,
}

pub struct NoDma;

impl_peripheral!(NoDma);

// TODO: replace transmutes with core::ptr::metadata once it's stable
#[allow(unused)]
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}

#[allow(unused)]
pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}

// safety: must be called only once at startup
pub(crate) unsafe fn init(
    #[cfg(bdma)] bdma_priority: Priority,
    #[cfg(dma)] dma_priority: Priority,
    #[cfg(gpdma)] gpdma_priority: Priority,
) {
    #[cfg(bdma)]
    bdma::init(bdma_priority);
    #[cfg(dma)]
    dma::init(dma_priority);
    #[cfg(gpdma)]
    gpdma::init(gpdma_priority);
    #[cfg(dmamux)]
    dmamux::init();
}
