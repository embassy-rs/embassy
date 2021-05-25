#![macro_use]

#[cfg_attr(dma_v1, path = "v1.rs")]
#[cfg_attr(dma_v2, path = "v2.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;

use crate::pac;

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
        impl crate::dma::Channel for peripherals::$type {}
        impl crate::dma::sealed::Channel for peripherals::$type {
            #[inline]
            fn num(&self) -> u8 {
                $dma_num * 8 + $ch_num
            }
        }
    };
}
