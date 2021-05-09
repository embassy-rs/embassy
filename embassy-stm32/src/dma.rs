#![macro_use]
use crate::pac::dma;

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        fn regs(&self) -> dma::Dma;
    }
}

pub trait Channel: sealed::Channel + Sized {
    fn num(&self) -> u8;
}

macro_rules! impl_dma_channel {
    ($type:ident, $dma_inst:ident, $num:expr) => {
        impl crate::dma::Channel for peripherals::$type {
            #[inline]
            fn num(&self) -> u8 {
                $num
            }
        }
        impl crate::dma::sealed::Channel for peripherals::$type {
            #[inline]
            fn regs(&self) -> dma::Dma {
                crate::pac::$dma_inst
            }
        }
    };
}
