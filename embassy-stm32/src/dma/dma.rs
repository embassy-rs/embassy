use core::future::Future;
use core::sync::atomic::{fence, Ordering};
use core::task::{Poll, Waker};

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::drop::OnDrop;
use futures::future::poll_fn;

use crate::interrupt;
use crate::pac;
use crate::pac::dma::{regs, vals};
use crate::rcc::sealed::RccPeripheral;

use super::{Channel, Request};

const CH_COUNT: usize = pac::peripheral_count!(DMA) * 8;

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
    let isrn = channel_number as usize / 4;
    let isrbit = channel_number as usize % 4;
    dma.ifcr(isrn).write(|w| {
        w.set_tcif(isrbit, true);
        w.set_teif(isrbit, true);
    });

    let ch = dma.st(channel_number as _);

    let on_drop = OnDrop::new(move || unsafe {
        _stop(&dma, channel_number);
    });

    #[cfg(dmamux)]
    super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

    // "Preceding reads and writes cannot be moved past subsequent writes."
    fence(Ordering::Release);

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

            let isr = dma.isr(isrn).read();

            // TODO handle error
            assert!(!isr.teif(isrbit));

            if isr.tcif(isrbit) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        drop(on_drop)
    }
}

/// Stops the DMA channel.
unsafe fn _stop(dma: &pac::dma::Dma, ch: u8) {
    // get a handle on the channel itself
    let ch = dma.st(ch as _);

    // Disable the channel and interrupts with the default value.
    ch.cr().write(|_| ());

    // Wait for the transfer to complete when it was ongoing.
    while ch.cr().read().en() {}

    // "Subsequent reads and writes cannot be moved ahead of preceding reads."
    fence(Ordering::Acquire);
}

/// Gets the running status of the channel
unsafe fn _is_stopped(dma: &pac::dma::Dma, ch: u8) -> bool {
    // get a handle on the channel itself
    let ch = dma.st(ch as _);

    // Wait for the transfer to complete when it was ongoing.
    ch.cr().read().en()
}

/// Gets the total remaining transfers for the channel
/// Note: this will be zero for transfers that completed without cancellation.
unsafe fn _get_remaining_transfers(dma: &pac::dma::Dma, ch: u8) -> u16 {
    // get a handle on the channel itself
    let ch = dma.st(ch as _);
    // read the remaining transfer count. If this is zero, the transfer completed fully.
    ch.ndtr().read().ndt()
}

/// Sets the waker for the specified DMA channel
unsafe fn _set_waker(dma: &pac::dma::Dma, state_number: u8, waker: &Waker) {
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
}

unsafe fn on_irq() {
    pac::peripherals! {
        (dma, $dma:ident) => {
            for isrn in 0..2 {
                let isr = pac::$dma.isr(isrn).read();

                for chn in 0..4 {
                    let cr = pac::$dma.st(isrn * 4 + chn).cr();

                    if isr.tcif(chn) && cr.read().tcie() {
                        cr.write(|_| ()); // Disable channel interrupts with the default value.
                        let n = dma_num!($dma) * 8 + isrn * 4 + chn;
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
        ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
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
            fn stop <'a>(&'a mut self){
                unsafe {_stop(&crate::pac::$dma_peri, $channel_num);}
            }
            fn is_stopped<'a>(&'a self) -> bool {
                unsafe {_is_stopped(&crate::pac::$dma_peri, $channel_num)}
            }
            fn remaining_transfers<'a>(&'a mut self) -> u16 {
                unsafe {_get_remaining_transfers(&crate::pac::$dma_peri, $channel_num)}
            }
            fn set_waker<'a>(&'a mut self, waker: &'a Waker) {
                unsafe {_set_waker(&crate::pac::$dma_peri,  $channel_num, waker )}
            }
        }
    };
}

pac::interrupts! {
    ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            on_irq()
        }
    };
}
