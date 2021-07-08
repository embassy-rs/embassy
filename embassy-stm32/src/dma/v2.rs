use core::task::Poll;

use crate::dma_traits::{ReadDma, WriteDma};
use atomic_polyfill::{AtomicU8, Ordering};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::AtomicWaker;
use futures::future::poll_fn;

use super::*;
use crate::interrupt;
use crate::pac;
use crate::pac::dma::{regs, vals};

use crate::pac::dma_channels;
use crate::pac::interrupts;
use crate::pac::peripheral_count;
use crate::pac::peripheral_dma_channels;
use crate::pac::peripherals;
use crate::peripherals;

const CH_COUNT: usize = peripheral_count!(DMA) * 8;
const CH_STATUS_NONE: u8 = 0;
const CH_STATUS_COMPLETED: u8 = 1;
const CH_STATUS_ERROR: u8 = 2;

struct State {
    ch_wakers: [AtomicWaker; CH_COUNT],
    ch_status: [AtomicU8; CH_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        const AU: AtomicU8 = AtomicU8::new(CH_STATUS_NONE);
        Self {
            ch_wakers: [AW; CH_COUNT],
            ch_status: [AU; CH_COUNT],
        }
    }
}

static STATE: State = State::new();

#[allow(unused)] // Used by usart/v1.rs which may or may not be enabled
pub(crate) async unsafe fn transfer_p2m(
    ch: &mut impl Channel,
    ch_func: u8,
    src: *const u8,
    dst: &mut [u8],
) {
    let n = ch.num();
    let c = ch.regs();

    // ndtr is max 16 bits.
    assert!(dst.len() <= 0xFFFF);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[n].store(CH_STATUS_NONE, Ordering::Release);

    unsafe {
        c.par().write_value(src as _);
        c.m0ar().write_value(dst.as_ptr() as _);
        c.ndtr().write_value(regs::Ndtr(dst.len() as _));
        c.cr().write(|w| {
            w.set_dir(vals::Dir::PERIPHERALTOMEMORY);
            w.set_msize(vals::Size::BITS8);
            w.set_psize(vals::Size::BITS8);
            w.set_minc(vals::Inc::INCREMENTED);
            w.set_pinc(vals::Inc::FIXED);
            w.set_chsel(ch_func);
            w.set_teie(true);
            w.set_tcie(true);
            w.set_en(true);
        });
    }

    let res = poll_fn(|cx| {
        STATE.ch_wakers[n].register(cx.waker());
        match STATE.ch_status[n].load(Ordering::Acquire) {
            CH_STATUS_NONE => Poll::Pending,
            x => Poll::Ready(x),
        }
    })
    .await;

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

#[allow(unused)] // Used by usart/v1.rs which may or may not be enabled
pub(crate) async unsafe fn transfer_m2p(
    ch: &mut impl Channel,
    ch_func: u8,
    src: &[u8],
    dst: *mut u8,
) {
    let n = ch.num();
    let c = ch.regs();

    // ndtr is max 16 bits.
    assert!(src.len() <= 0xFFFF);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[n].store(CH_STATUS_NONE, Ordering::Release);

    unsafe {
        c.par().write_value(dst as _);
        c.m0ar().write_value(src.as_ptr() as _);
        c.ndtr().write_value(regs::Ndtr(src.len() as _));
        c.cr().write(|w| {
            w.set_dir(vals::Dir::MEMORYTOPERIPHERAL);
            w.set_msize(vals::Size::BITS8);
            w.set_psize(vals::Size::BITS8);
            w.set_minc(vals::Inc::INCREMENTED);
            w.set_pinc(vals::Inc::FIXED);
            w.set_chsel(ch_func);
            w.set_teie(true);
            w.set_tcie(true);
            w.set_en(true);
        });
    }

    let res = poll_fn(|cx| {
        STATE.ch_wakers[n].register(cx.waker());
        match STATE.ch_status[n].load(Ordering::Acquire) {
            CH_STATUS_NONE => {
                let left = c.ndtr().read().ndt();
                Poll::Pending
            }
            x => Poll::Ready(x),
        }
    })
    .await;

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

unsafe fn on_irq() {
    peripherals! {
        (dma, $dma:ident) => {
            for isrn in 0..2 {
                let isr = pac::$dma.isr(isrn).read();
                pac::$dma.ifcr(isrn).write_value(isr);
                let dman = <peripherals::$dma as sealed::Dma>::num() as usize;

                for chn in 0..4 {
                    let n = dman * 8 + isrn * 4 + chn;
                    if isr.teif(chn) {
                        STATE.ch_status[n].store(CH_STATUS_ERROR, Ordering::Relaxed);
                        STATE.ch_wakers[n].wake();
                    } else if isr.tcif(chn) {
                        STATE.ch_status[n].store(CH_STATUS_COMPLETED, Ordering::Relaxed);
                        STATE.ch_wakers[n].wake();
                    }
                }
            }
        };
    }
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    interrupts! {
        (DMA, $irq:ident) => {
            interrupt::$irq::steal().enable();
        };
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Dma {
        fn num() -> u8;
        fn regs() -> &'static pac::dma::Dma;
    }

    pub trait Channel {
        fn dma_regs() -> &'static pac::dma::Dma;

        fn num(&self) -> usize;

        fn ch_num(&self) -> u8;

        fn regs(&self) -> pac::dma::St {
            Self::dma_regs().st(self.ch_num() as _)
        }
    }

    pub trait PeripheralChannel<PERI, OP>: Channel {
        fn request(&self) -> u8;
    }
}

pub trait Dma: sealed::Dma + Sized {}
pub trait Channel: sealed::Channel + Sized {}
pub trait PeripheralChannel<PERI, OP>: sealed::PeripheralChannel<PERI, OP> + Sized {}

macro_rules! impl_dma {
    ($peri:ident, $num:expr) => {
        impl Dma for peripherals::$peri {}
        impl sealed::Dma for peripherals::$peri {
            fn num() -> u8 {
                $num
            }
            fn regs() -> &'static pac::dma::Dma {
                &pac::$peri
            }
        }
    };
}

macro_rules! impl_dma_channel {
    ($channel_peri:ident, $dma_peri:ident, $dma_num:expr, $ch_num:expr) => {
        impl Channel for peripherals::$channel_peri {}
        impl sealed::Channel for peripherals::$channel_peri {
            #[inline]
            fn dma_regs() -> &'static pac::dma::Dma {
                &crate::pac::$dma_peri
            }

            fn num(&self) -> usize {
                ($dma_num * 8) + $ch_num
            }

            fn ch_num(&self) -> u8 {
                $ch_num
            }
        }

        impl<T> WriteDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, M2P>,
            T: 'static,
        {
            type WriteDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, M2P>::request(self);
                unsafe { transfer_m2p(self, request, buf, dst) }
            }
        }

        impl<T> ReadDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, P2M>,
            T: 'static,
        {
            type ReadDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(
                &'a mut self,
                src: *const u8,
                buf: &'a mut [u8],
            ) -> Self::ReadDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, P2M>::request(self);
                unsafe { transfer_p2m(self, request, src, buf) }
            }
        }
    };
}

peripherals! {
    (dma, DMA1) => {
        impl_dma!(DMA1, 0);
        dma_channels! {
            ($channel_peri:ident, DMA1, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMA1, 0, $channel_num);
            };
        }
    };
    (dma, DMA2) => {
        impl_dma!(DMA2, 1);
        dma_channels! {
            ($channel_peri:ident, DMA2, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMA2, 1, $channel_num);
            };
        }
    };
}

interrupts! {
    (DMA, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            on_irq()
        }
    };
}

pub struct P2M;
pub struct M2P;

#[cfg(usart)]
use crate::usart;
peripheral_dma_channels! {
    ($peri:ident, usart, $kind:ident, RX, $channel_peri:ident, $dma_peri:ident, $channel_num:expr, $event_num:expr) => {
        impl usart::RxDma<peripherals::$peri> for peripherals::$channel_peri { }
        impl usart::sealed::RxDma<peripherals::$peri> for peripherals::$channel_peri { }

        impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
            fn request(&self) -> u8 {
                $event_num
            }
        }

        impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }
    };

    ($peri:ident, usart, $kind:ident, TX, $channel_peri:ident, $dma_peri:ident, $channel_num:expr, $event_num:expr) => {
        impl usart::TxDma<peripherals::$peri> for peripherals::$channel_peri { }
        impl usart::sealed::TxDma<peripherals::$peri> for peripherals::$channel_peri { }

        impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
            fn request(&self) -> u8 {
                $event_num
            }
        }

        impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }
    };
}
