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

use core::mem;

use embassy_cortex_m::interrupt::Priority;
use embassy_hal_common::impl_peripheral;

#[cfg(dmamux)]
pub use self::dmamux::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Dir {
    MemoryToPeripheral,
    PeripheralToMemory,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WordSize {
    OneByte,
    TwoBytes,
    FourBytes,
}

impl WordSize {
    pub fn bytes(&self) -> usize {
        match self {
            Self::OneByte => 1,
            Self::TwoBytes => 2,
            Self::FourBytes => 4,
        }
    }
}

mod word_sealed {
    pub trait Word {}
}

pub trait Word: word_sealed::Word {
    fn bits() -> WordSize;
}

impl word_sealed::Word for u8 {}
impl Word for u8 {
    fn bits() -> WordSize {
        WordSize::OneByte
    }
}

impl word_sealed::Word for u16 {}
impl Word for u16 {
    fn bits() -> WordSize {
        WordSize::TwoBytes
    }
}

impl word_sealed::Word for u32 {}
impl Word for u32 {
    fn bits() -> WordSize {
        WordSize::FourBytes
    }
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
