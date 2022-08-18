use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_common::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use futures::Future;

use crate::pac::dma::vals;
use crate::{pac, peripherals};

pub fn copy<'a, C: Channel, W: Word>(ch: impl Peripheral<P = C> + 'a, from: &[W], to: &mut [W]) -> Transfer<'a, C> {
    assert!(from.len() == to.len());

    into_ref!(ch);

    unsafe {
        let p = ch.regs();

        p.read_addr().write_value(from.as_ptr() as u32);
        p.write_addr().write_value(to.as_mut_ptr() as u32);
        p.trans_count().write_value(from.len() as u32);

        compiler_fence(Ordering::SeqCst);

        p.ctrl_trig().write(|w| {
            w.set_data_size(W::size());
            w.set_incr_read(true);
            w.set_incr_write(true);
            w.set_chain_to(ch.number());
            w.set_en(true);
        });

        // FIXME:
        while p.ctrl_trig().read().busy() {}

        compiler_fence(Ordering::SeqCst);
    }
    Transfer::new(ch)
}

pub(crate) struct Transfer<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: impl Peripheral<P = C> + 'a) -> Self {
        into_ref!(channel);
        Self { channel }
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        // self.channel.request_stop();
        // while self.channel.is_running() {}
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // self.channel.set_waker(cx.waker());
        // if self.channel.is_running() {
        //     Poll::Pending
        // } else {
            Poll::Ready(())
        // }
    }
}

pub struct NoDma;

impl_peripheral!(NoDma);

mod sealed {
    pub trait Channel {}

    pub trait Word {}
}

pub trait Channel: Peripheral<P = Self> + sealed::Channel + Into<AnyChannel> + Sized + 'static {
    fn number(&self) -> u8;

    fn regs(&self) -> pac::dma::Channel {
        pac::DMA.ch(self.number() as _)
    }

    fn degrade(self) -> AnyChannel {
        AnyChannel {
            number: self.number(),
        }
    }
}

pub trait Word: sealed::Word {
    fn size() -> vals::DataSize;
}

impl sealed::Word for u8 {}
impl Word for u8 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_BYTE
    }
}

impl sealed::Word for u16 {}
impl Word for u16 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_HALFWORD
    }
}

impl sealed::Word for u32 {}
impl Word for u32 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_WORD
    }
}

pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl sealed::Channel for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }

        impl From<peripherals::$name> for crate::dma::AnyChannel {
            fn from(val: peripherals::$name) -> Self {
                crate::dma::Channel::degrade(val)
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
