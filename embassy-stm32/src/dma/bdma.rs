#![macro_use]

use core::future::Future;
use core::task::Poll;

use atomic_polyfill::{AtomicU8, Ordering};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::{AtomicWaker, OnDrop};
use futures::future::poll_fn;

use crate::dma::{Channel, Request};
use crate::interrupt;
use crate::pac;
use crate::pac::bdma::vals;
use crate::rcc::sealed::RccPeripheral;

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
pub(crate) async unsafe fn do_transfer(
    dma: pac::bdma::Dma,
    channel_number: u8,
    state_number: u8,
    request: Request,
    dir: vals::Dir,
    peri_addr: *const u8,
    mem_addr: *mut u8,
    mem_len: usize,
    #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
    #[cfg(dmamux)] dmamux_ch_num: u8,
) {
    // ndtr is max 16 bits.
    assert!(mem_len <= 0xFFFF);

    let ch = dma.ch(channel_number as _);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[state_number as usize].store(CH_STATUS_NONE, Ordering::Release);

    let on_drop = OnDrop::new(|| unsafe {
        ch.cr().modify(|w| {
            w.set_tcie(false);
            w.set_teie(false);
            w.set_en(false);
        });
        while ch.cr().read().en() {}
    });

    #[cfg(dmamux)]
    super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    #[cfg(bdma_v2)]
    critical_section::with(|_| {
        dma.cselr()
            .modify(|w| w.set_cs(channel_number as _, request))
    });

    ch.par().write_value(peri_addr as u32);
    ch.mar().write_value(mem_addr as u32);
    ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
    ch.cr().write(|w| {
        w.set_psize(vals::Size::BITS8);
        w.set_msize(vals::Size::BITS8);
        w.set_minc(vals::Inc::ENABLED);
        w.set_dir(dir);
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

unsafe fn on_irq() {
    pac::peripherals! {
        (bdma, $dma:ident) => {
                let isr = pac::$dma.isr().read();
                pac::$dma.ifcr().write_value(isr);
                let dman = dma_num!($dma);

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

/// safety: must be called only once
pub(crate) unsafe fn init() {
    pac::interrupts! {
        (BDMA, $irq:ident) => {
            crate::interrupt::$irq::steal().enable();
        };
    }
    pac::peripherals! {
        (bdma, $peri:ident) => {
            crate::peripherals::$peri::enable();
        };
    }
}

pac::dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, bdma, $channel_num:expr, $dmamux:tt) => {
        impl crate::dma::sealed::Channel for crate::peripherals::$channel_peri {}
        impl Channel for crate::peripherals::$channel_peri
        {
            type ReadFuture<'a> = impl Future<Output = ()>;
            type WriteFuture<'a> = impl Future<Output = ()>;

            fn read<'a>(
                &'a mut self,
                request: Request,
                src: *mut u8,
                buf: &'a mut [u8],
            ) -> Self::ReadFuture<'a> {
                unsafe {
                    do_transfer(
                        crate::pac::$dma_peri,
                        $channel_num,
                        (dma_num!($dma_peri) * 8) + $channel_num,
                        request,
                        vals::Dir::FROMPERIPHERAL,
                        src,
                        buf.as_mut_ptr(),
                        buf.len(),
                        #[cfg(dmamux)]
                        <Self as super::dmamux::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::MuxChannel>::DMAMUX_CH_NUM,
                    )
                }
            }

            fn write<'a>(
                &'a mut self,
                request: Request,
                buf: &'a [u8],
                dst: *mut u8,
            ) -> Self::WriteFuture<'a> {
                unsafe {
                    do_transfer(
                        crate::pac::$dma_peri,
                        $channel_num,
                        (dma_num!($dma_peri) * 8) + $channel_num,
                        request,
                        vals::Dir::FROMMEMORY,
                        dst,
                        buf.as_ptr() as *mut u8,
                        buf.len(),
                        #[cfg(dmamux)]
                        <Self as super::dmamux::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::MuxChannel>::DMAMUX_CH_NUM,
                    )
                }
            }
        }
    };
}

pac::interrupts! {
    (BDMA, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            on_irq()
        }
    };
}
