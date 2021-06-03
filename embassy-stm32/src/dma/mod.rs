#![macro_use]

#[cfg_attr(dma_v1, path = "v1.rs")]
#[cfg_attr(dma_v2, path = "v2.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;

use crate::pac;
use crate::peripherals;

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        fn num(&self) -> u8;

        fn dma_num(&self) -> u8 {
            self.num() / 8
        }
        fn ch_num(&self) -> u8 {
            self.num() % 8
        }
        fn regs(&self) -> pac::dma::Dma {
            pac::DMA(self.num() as _)
        }
    }
}

pub trait Channel: sealed::Channel + Sized {}

macro_rules! impl_dma_channel {
    ($type:ident, $dma_num:expr, $ch_num:expr) => {
        impl Channel for peripherals::$type {}
        impl sealed::Channel for peripherals::$type {
            #[inline]
            fn num(&self) -> u8 {
                $dma_num * 8 + $ch_num
            }
        }
    };
}

crate::pac::peripherals!(
    (dma,DMA1) => {
        impl_dma_channel!(DMA1_CH0, 0, 0);
        impl_dma_channel!(DMA1_CH1, 0, 1);
        impl_dma_channel!(DMA1_CH2, 0, 2);
        impl_dma_channel!(DMA1_CH3, 0, 3);
        impl_dma_channel!(DMA1_CH4, 0, 4);
        impl_dma_channel!(DMA1_CH5, 0, 5);
        impl_dma_channel!(DMA1_CH6, 0, 6);
        impl_dma_channel!(DMA1_CH7, 0, 7);
    };

    (dma,DMA2) => {
        impl_dma_channel!(DMA2_CH0, 1, 0);
        impl_dma_channel!(DMA2_CH1, 1, 1);
        impl_dma_channel!(DMA2_CH2, 1, 2);
        impl_dma_channel!(DMA2_CH3, 1, 3);
        impl_dma_channel!(DMA2_CH4, 1, 4);
        impl_dma_channel!(DMA2_CH5, 1, 5);
        impl_dma_channel!(DMA2_CH6, 1, 6);
        impl_dma_channel!(DMA2_CH7, 1, 7);
    };
);
