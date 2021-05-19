use core::sync::atomic::{compiler_fence, Ordering};

use crate::fmt::assert;
use crate::pac::dma::vals;
use crate::{pac, peripherals};

pub struct Dma<T: Channel> {
    _inner: T,
}

impl<T: Channel> Dma<T> {
    pub fn copy(inner: T, from: &[u32], to: &mut [u32]) {
        assert!(from.len() == to.len());

        unsafe {
            let p = inner.regs();

            p.read_addr().write_value(from.as_ptr() as u32);
            p.write_addr().write_value(to.as_mut_ptr() as u32);
            p.trans_count().write_value(from.len() as u32);

            compiler_fence(Ordering::SeqCst);

            p.ctrl_trig().write(|w| {
                w.set_data_size(vals::DataSize::SIZE_WORD);
                w.set_incr_read(true);
                w.set_incr_write(true);
                w.set_chain_to(inner.number());
                w.set_en(true);
            });

            while p.ctrl_trig().read().busy() {}

            compiler_fence(Ordering::SeqCst);
        }
    }
}

mod sealed {
    use super::*;

    pub trait Channel {
        fn number(&self) -> u8;

        fn regs(&self) -> pac::dma::Channel {
            pac::DMA.ch(self.number() as _)
        }
    }
}

pub trait Channel: sealed::Channel {}

pub struct AnyChannel {
    number: u8,
}

impl Channel for AnyChannel {}
impl sealed::Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($type:ident, $num:expr) => {
        impl Channel for peripherals::$type {}
        impl sealed::Channel for peripherals::$type {
            fn number(&self) -> u8 {
                $num
            }
        }
    };
}

channel!(DMA_CH0, 0);
channel!(DMA_CH1, 1);
channel!(DMA_CH2, 2);
channel!(DMA_CH3, 3);
channel!(DMA_CH4, 4);
channel!(DMA_CH5, 5);
channel!(DMA_CH6, 6);
channel!(DMA_CH7, 7);
channel!(DMA_CH8, 8);
channel!(DMA_CH9, 9);
channel!(DMA_CH10, 10);
channel!(DMA_CH11, 11);
