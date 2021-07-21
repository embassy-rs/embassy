use core::future::Future;
use core::task::Poll;

use atomic_polyfill::{AtomicU8, Ordering};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::{AtomicWaker, OnDrop};
use futures::future::poll_fn;

use crate::interrupt;
use crate::pac;
use crate::pac::dma::{regs, vals};
use crate::rcc::sealed::RccPeripheral;

use super::{Channel, Request};

const CH_COUNT: usize = pac::peripheral_count!(DMA) * 8;
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

//async unsafe fn do_transfer(ch: &mut impl Channel, ch_func: u8, src: *const u8, dst: &mut [u8]) {

#[allow(unused)]
pub(crate) unsafe fn do_transfer(
    dma: pac::dma::Dma,
    channel_number: u8,
    state_number: u8,
    request: Request,
    dir: vals::Dir,
    peri_addr: *const u8,
    mem_addr: *mut u8,
    mem_len: usize,
    incr_mem: bool,
    #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
    #[cfg(dmamux)] dmamux_ch_num: u8,
) -> impl Future<Output = ()> {
    // ndtr is max 16 bits.
    assert!(mem_len <= 0xFFFF);

    // Reset status
    // Generate a DMB here to flush the store buffer (M7) before enabling the DMA
    STATE.ch_status[state_number as usize].store(CH_STATUS_NONE, Ordering::Release);

    let ch = dma.st(channel_number as _);

    let on_drop = OnDrop::new(move || unsafe {
        ch.cr().modify(|w| {
            w.set_tcie(false);
            w.set_teie(false);
            w.set_en(false);
        });
        while ch.cr().read().en() {}

        // Disabling the DMA mid transfer might cause some flags to be set, clear them all for the
        // next transfer
        dma.ifcr(channel_number as usize / 4).write(|w| {
            w.set_tcif(channel_number as usize % 4, true);
            w.set_teif(channel_number as usize % 4, true);
        });
    });

    #[cfg(dmamux)]
    super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    unsafe {
        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(mem_addr as u32);
        ch.ndtr().write_value(regs::Ndtr(mem_len as _));
        ch.cr().write(|w| {
            w.set_dir(dir);
            w.set_msize(vals::Size::BITS8);
            w.set_psize(vals::Size::BITS8);
            if incr_mem {
                w.set_minc(vals::Inc::INCREMENTED);
            } else {
                w.set_minc(vals::Inc::FIXED);
            }
            w.set_pinc(vals::Inc::FIXED);
            w.set_teie(true);
            w.set_tcie(true);
            #[cfg(dma_v1)]
            w.set_trbuff(true);

            #[cfg(dma_v2)]
            w.set_chsel(request);

            w.set_en(true);
        });
    }

    async move {
        let res = poll_fn(|cx| {
            let n = state_number as usize;
            STATE.ch_wakers[n].register(cx.waker());
            match STATE.ch_status[n].load(Ordering::Acquire) {
                CH_STATUS_NONE => Poll::Pending,
                x => Poll::Ready(x),
            }
        })
        .await;

        // TODO handle error
        assert!(res == CH_STATUS_COMPLETED);
        drop(on_drop)
    }
}

macro_rules! dma_num {
    (DMA1) => {
        0
    };
    (DMA2) => {
        1
    };
}

unsafe fn on_irq() {
    pac::peripherals! {
        (dma, $dma:ident) => {
            for isrn in 0..2 {
                let isr = pac::$dma.isr(isrn).read();
                pac::$dma.ifcr(isrn).write_value(isr);
                let dman = dma_num!($dma);

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
    pac::interrupts! {
        (DMA, $irq:ident) => {
            interrupt::$irq::steal().enable();
        };
    }
    pac::peripherals! {
        (dma, $peri:ident) => {
            crate::peripherals::$peri::enable();
        };
    }
}

pac::dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, dma, $channel_num:expr, $dmamux:tt) => {
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
                        vals::Dir::PERIPHERALTOMEMORY,
                        src,
                        buf.as_mut_ptr(),
                        buf.len(),
                        true,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
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
                        vals::Dir::MEMORYTOPERIPHERAL,
                        dst,
                        buf.as_ptr() as *mut u8,
                        buf.len(),
                        true,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                    )
                }
            }

            fn write_x<'a>(
                &'a mut self,
                request: Request,
                word: &u8,
                num: usize,
                dst: *mut u8,
            ) -> Self::WriteFuture<'a> {
                unsafe {
                    do_transfer(
                        crate::pac::$dma_peri,
                        $channel_num,
                        (dma_num!($dma_peri) * 8) + $channel_num,
                        request,
                        vals::Dir::MEMORYTOPERIPHERAL,
                        dst,
                        word as *const u8 as *mut u8,
                        num,
                        false,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                    )
                }
            }
        }
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
