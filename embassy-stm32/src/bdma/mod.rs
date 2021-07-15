#![macro_use]

use core::future::Future;
use core::task::Poll;

use atomic_polyfill::{AtomicU8, Ordering};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::{AtomicWaker, OnDrop};
use futures::future::poll_fn;

use crate::dma_traits::{ReadDma, WriteDma};
use crate::interrupt;
use crate::pac;
use crate::pac::bdma::vals;

const CH_COUNT: usize = pac::peripheral_count!(bdma) * 8;
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

#[allow(unused)]
pub(crate) async unsafe fn transfer_p2m(
    regs: pac::bdma::Ch,
    state_number: u8,
    src: *const u8,
    dst: &mut [u8],
    #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
    #[cfg(dmamux)] dmamux_ch_num: u8,
    #[cfg(dmamux)] request: u8,
) {
    // ndtr is max 16 bits.
    assert!(dst.len() <= 0xFFFF);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[state_number as usize].store(CH_STATUS_NONE, Ordering::Release);

    let on_drop = OnDrop::new(|| unsafe {
        regs.cr().modify(|w| {
            w.set_tcie(false);
            w.set_teie(false);
            w.set_en(false);
        });
        while regs.cr().read().en() {}
    });

    #[cfg(dmamux)]
    crate::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    regs.par().write_value(src as u32);
    regs.mar().write_value(dst.as_mut_ptr() as u32);
    regs.ndtr().write(|w| w.set_ndt(dst.len() as u16));
    regs.cr().write(|w| {
        w.set_psize(vals::Size::BITS8);
        w.set_msize(vals::Size::BITS8);
        w.set_minc(vals::Inc::ENABLED);
        w.set_teie(true);
        w.set_tcie(true);
        w.set_en(true);
    });

    let res = poll_fn(|cx| {
        STATE.ch_wakers[state_number as usize].register(cx.waker());
        match STATE.ch_status[state_number as usize].load(Ordering::Acquire) {
            CH_STATUS_NONE => Poll::Pending,
            x => Poll::Ready(x),
        }
    })
    .await;

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

#[allow(unused)]
pub(crate) async unsafe fn transfer_m2p(
    regs: pac::bdma::Ch,
    state_number: u8,
    src: &[u8],
    dst: *mut u8,
    #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
    #[cfg(dmamux)] dmamux_ch_num: u8,
    #[cfg(dmamux)] request: u8,
) {
    // ndtr is max 16 bits.
    assert!(src.len() <= 0xFFFF);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[state_number as usize].store(CH_STATUS_NONE, Ordering::Release);

    let on_drop = OnDrop::new(|| unsafe {
        regs.cr().modify(|w| {
            w.set_tcie(false);
            w.set_teie(false);
            w.set_en(false);
        });
        while regs.cr().read().en() {}
    });

    #[cfg(dmamux)]
    crate::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    regs.par().write_value(dst as u32);
    regs.mar().write_value(src.as_ptr() as u32);
    regs.ndtr().write(|w| w.set_ndt(src.len() as u16));
    regs.cr().write(|w| {
        w.set_psize(vals::Size::BITS8);
        w.set_msize(vals::Size::BITS8);
        w.set_minc(vals::Inc::ENABLED);
        w.set_dir(vals::Dir::FROMMEMORY);
        w.set_teie(true);
        w.set_tcie(true);
        w.set_en(true);
    });

    let res = poll_fn(|cx| {
        STATE.ch_wakers[state_number as usize].register(cx.waker());
        match STATE.ch_status[state_number as usize].load(Ordering::Acquire) {
            CH_STATUS_NONE => Poll::Pending,
            x => Poll::Ready(x),
        }
    })
    .await;

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

unsafe fn on_irq() {
    pac::peripherals! {
        (bdma, $dma:ident) => {
                let isr = pac::$dma.isr().read();
                pac::$dma.ifcr().write_value(isr);
                let dman = <crate::peripherals::$dma as sealed::Dma>::NUM as usize;

                for chn in 0..crate::pac::dma_channels_count!($dma) {
                    let n = dman * 8 + chn;
                    if isr.teif(chn) {
                        STATE.ch_status[n].store(CH_STATUS_ERROR, Ordering::Relaxed);
                        STATE.ch_wakers[n].wake();
                    } else if isr.tcif(chn) {
                        STATE.ch_status[n].store(CH_STATUS_COMPLETED, Ordering::Relaxed);
                        STATE.ch_wakers[n].wake();
                    }
                }
        };
    }
}

use crate::rcc::sealed::RccPeripheral;

/// safety: must be called only once
pub(crate) unsafe fn init() {
    pac::interrupts! {
        (DMA, $irq:ident) => {
            crate::interrupt::$irq::steal().enable();
        };
    }
    pac::peripherals! {
        (bdma, $peri:ident) => {
            <crate::peripherals::$peri as RccPeripheral>::enable();
        };
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Dma {
        const NUM: u8;
    }

    pub trait Channel {
        const CH_NUM: u8;
        const STATE_NUM: u8;
        const DMA_REGS: pac::bdma::Dma;

        fn regs(&self) -> pac::bdma::Ch {
            Self::DMA_REGS.ch(Self::CH_NUM as usize)
        }
    }
}

pub trait Dma: sealed::Dma + Sized {}
pub trait Channel: sealed::Channel + Sized {}

macro_rules! impl_dma {
    ($peri:ident) => {
        impl Dma for crate::peripherals::$peri {}
        impl sealed::Dma for crate::peripherals::$peri {
            const NUM: u8 = dma_num!($peri);
        }
    };
}

macro_rules! impl_dma_channel {
    ($channel_peri:ident, $dma_peri:ident, $ch_num:expr) => {
        impl Channel for crate::peripherals::$channel_peri {}
        impl sealed::Channel for crate::peripherals::$channel_peri {
            const CH_NUM: u8 = $ch_num;
            const STATE_NUM: u8 = (dma_num!($dma_peri) * 8) + $ch_num;
            const DMA_REGS: pac::bdma::Dma = crate::pac::$dma_peri;

            //#[inline]
            //fn dma_regs() -> pac::bdma::Dma {
            //crate::pac::$dma_peri
            //}

            //fn state_num(&self) -> usize {
            //(dma_num!($dma_peri) * 8) + $ch_num
            //}
        }

        #[cfg(not(dmamux))]
        impl<T> WriteDma<T> for crate::peripherals::$channel_peri
        where
            T: 'static,
        {
            type WriteDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
            where
                T: 'a,
            {
                use sealed::Channel as _Channel;

                let state_num = Self::STATE_NUM;
                let regs = self.regs();

                unsafe { transfer_m2p(regs, state_num, buf, dst) }
            }
        }

        #[cfg(dmamux)]
        impl<T> WriteDma<T> for crate::peripherals::$channel_peri
        where
            Self: crate::dmamux::sealed::PeripheralChannel<T, crate::dmamux::M2P>,
            T: 'static,
        {
            type WriteDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
            where
                T: 'a,
            {
                use sealed::Channel as _Channel;

                let state_num = Self::STATE_NUM;
                let regs = self.regs();

                use crate::dmamux::sealed::Channel as MuxChannel;
                use crate::dmamux::sealed::PeripheralChannel;
                let dmamux_regs = <crate::peripherals::$channel_peri as MuxChannel>::DMAMUX_REGS;
                let dmamux_ch_num =
                    <crate::peripherals::$channel_peri as MuxChannel>::DMAMUX_CH_NUM;
                let request = <crate::peripherals::$channel_peri as PeripheralChannel<
                    T,
                    crate::dmamux::M2P,
                >>::REQUEST;
                unsafe {
                    transfer_m2p(
                        regs,
                        state_num,
                        buf,
                        dst,
                        dmamux_regs,
                        dmamux_ch_num,
                        request,
                    )
                }
            }
        }

        #[cfg(not(dmamux))]
        impl<T> ReadDma<T> for crate::peripherals::$channel_peri
        where
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
                use sealed::Channel as _Channel;

                let state_num = Self::STATE_NUM;
                let regs = self.regs();
                unsafe { transfer_p2m(regs, state_num, src, buf) }
            }
        }

        #[cfg(dmamux)]
        impl<T> ReadDma<T> for crate::peripherals::$channel_peri
        where
            Self: crate::dmamux::sealed::PeripheralChannel<T, crate::dmamux::P2M>,
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
                use sealed::Channel as _Channel;

                let state_num = Self::STATE_NUM;
                let regs = self.regs();

                use crate::dmamux::sealed::Channel as MuxChannel;
                use crate::dmamux::sealed::PeripheralChannel;
                let dmamux_regs = <crate::peripherals::$channel_peri as MuxChannel>::DMAMUX_REGS;
                let dmamux_ch_num =
                    <crate::peripherals::$channel_peri as MuxChannel>::DMAMUX_CH_NUM;
                let request = <crate::peripherals::$channel_peri as PeripheralChannel<
                    T,
                    crate::dmamux::P2M,
                >>::REQUEST;
                unsafe {
                    transfer_p2m(
                        regs,
                        state_num,
                        src,
                        buf,
                        dmamux_regs,
                        dmamux_ch_num,
                        request,
                    )
                }
            }
        }
    };
}

macro_rules! dma_num {
    (DMA1) => {
        0
    };
    (DMA2) => {
        1
    };
    (BDMA) => {
        0
    };
}
pac::peripherals! {
    (bdma, $peri:ident) => {
        impl_dma!($peri);
    };
}

pac::bdma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl_dma_channel!($channel_peri, $dma_peri, $channel_num);
    };
}

pac::interrupts! {
    (DMA, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            on_irq()
        }
    };
}

#[cfg(usart)]
use crate::usart;

pac::peripherals! {
    (usart, $peri:ident) => {
        impl<T:Channel + crate::dma_traits::WriteDma<crate::peripherals::$peri>> usart::TxDma<crate::peripherals::$peri> for T {}
        impl<T:Channel + crate::dma_traits::WriteDma<crate::peripherals::$peri>> usart::sealed::TxDma<crate::peripherals::$peri> for T {}

        impl<T:Channel + crate::dma_traits::ReadDma<crate::peripherals::$peri>> usart::RxDma<crate::peripherals::$peri> for T {}
        impl<T:Channel + crate::dma_traits::ReadDma<crate::peripherals::$peri>> usart::sealed::RxDma<crate::peripherals::$peri> for T {}
    };

    (uart, $peri:ident) => {
        impl<T:Channel + crate::dma_traits::WriteDma<crate::peripherals::$peri>> usart::TxDma<crate::peripherals::$peri> for T {}
        impl<T:Channel + crate::dma_traits::WriteDma<crate::peripherals::$peri>> usart::sealed::TxDma<crate::peripherals::$peri> for T {}

        impl<T:Channel + crate::dma_traits::ReadDma<crate::peripherals::$peri>> usart::RxDma<crate::peripherals::$peri> for T {}
        impl<T:Channel + crate::dma_traits::ReadDma<crate::peripherals::$peri>> usart::sealed::RxDma<crate::peripherals::$peri> for T {}
    };
}
