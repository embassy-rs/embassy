#![macro_use]

use core::future::Future;
use core::sync::atomic::{fence, Ordering};
use core::task::{Poll, Waker};

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::drop::OnDrop;
use futures::future::poll_fn;

use crate::dma::{Channel, Request};
use crate::interrupt;
use crate::pac;
use crate::pac::bdma::vals;
use crate::rcc::sealed::RccPeripheral;

const CH_COUNT: usize = pac::peripheral_count!(bdma) * 8;

struct State {
    ch_wakers: [AtomicWaker; CH_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; CH_COUNT],
        }
    }
}

static STATE: State = State::new();

#[allow(unused)]
pub(crate) unsafe fn do_transfer(
    dma: pac::bdma::Dma,
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

    let ch = dma.ch(channel_number as _);

    // Reset status
    dma.ifcr().write(|w| {
        w.set_tcif(channel_number as _, true);
        w.set_teif(channel_number as _, true);
    });

    let on_drop = OnDrop::new(move || unsafe {
        _stop(dma, channel_number);
    });

    #[cfg(dmamux)]
    super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    #[cfg(bdma_v2)]
    critical_section::with(|_| {
        dma.cselr()
            .modify(|w| w.set_cs(channel_number as _, request))
    });

    // "Preceding reads and writes cannot be moved past subsequent writes."
    fence(Ordering::Release);

    ch.par().write_value(peri_addr as u32);
    ch.mar().write_value(mem_addr as u32);
    ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
    ch.cr().write(|w| {
        w.set_psize(vals::Size::BITS8);
        w.set_msize(vals::Size::BITS8);
        if incr_mem {
            w.set_minc(vals::Inc::ENABLED);
        } else {
            w.set_minc(vals::Inc::DISABLED);
        }
        w.set_dir(dir);
        w.set_teie(true);
        w.set_tcie(true);
        w.set_en(true);
    });

    async move {
        let res = poll_fn(|cx| {
            STATE.ch_wakers[state_number as usize].register(cx.waker());

            let isr = dma.isr().read();

            // TODO handle error
            assert!(!isr.teif(channel_number as _));

            if isr.tcif(channel_number as _) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        drop(on_drop)
    }
}

unsafe fn _stop(dma: pac::bdma::Dma, ch: u8) {
    let ch = dma.ch(ch as _);

    // Disable the channel and interrupts with the default value.
    ch.cr().write(|_| ());

    // Wait for the transfer to complete when it was ongoing.
    while ch.cr().read().en() {}

    // "Subsequent reads and writes cannot be moved ahead of preceding reads."
    fence(Ordering::Acquire);
}

unsafe fn _is_stopped(dma: pac::bdma::Dma, ch: u8) -> bool {
    let ch = dma.ch(ch as _);
    ch.cr().read().en()
}

/// Gets the total remaining transfers for the channel
/// Note: this will be zero for transfers that completed without cancellation.
unsafe fn _get_remaining_transfers(dma: pac::bdma::Dma, ch: u8) -> u16 {
    // get a handle on the channel itself
    let ch = dma.ch(ch as _);
    // read the remaining transfer count. If this is zero, the transfer completed fully.
    ch.ndtr().read().ndt()
}

/// Sets the waker for the specified DMA channel
unsafe fn _set_waker(dma: pac::bdma::Dma, state_number: u8, waker: &Waker) {
    let n = state_number as usize;
    STATE.ch_wakers[n].register(waker);
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
                let dman = dma_num!($dma);

                for chn in 0..crate::pac::dma_channels_count!($dma) {
                    let cr = pac::$dma.ch(chn).cr();
                    if isr.tcif(chn) && cr.read().tcie() {
                        cr.write(|_| ()); // Disable channel interrupts with the default value.
                        let n = dma_num!($dma) * 8 + chn;
                        STATE.ch_wakers[n].wake();
                    }
                }
        };
    }
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    pac::interrupts! {
        ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
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
            type ReadFuture<'a> = impl Future<Output = ()> + 'a;
            type WriteFuture<'a> = impl Future<Output = ()> + 'a;

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
                        vals::Dir::FROMMEMORY,
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
                count: usize,
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
                        word as *const u8 as *mut u8,
                        count,
                        false,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                        #[cfg(dmamux)]
                        <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                    )
                }
            }
            fn stop <'a>(&'a mut self){
                unsafe {_stop(crate::pac::$dma_peri, $channel_num);}
            }
            fn is_stopped<'a>(&'a self) -> bool {
                unsafe {_is_stopped(crate::pac::$dma_peri, $channel_num)}
            }
            fn remaining_transfers<'a>(&'a mut self) -> u16 {
                unsafe {_get_remaining_transfers(crate::pac::$dma_peri, $channel_num)}
            }
            fn set_waker<'a>(&'a mut self, waker: &'a Waker) {
                unsafe {_set_waker(crate::pac::$dma_peri,  $channel_num, waker )}
            }
        }
    };
}

pac::interrupts! {
    ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            on_irq()
        }
    };
}
