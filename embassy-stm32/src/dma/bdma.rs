#![macro_use]

use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;

use crate::dma::Request;
use crate::interrupt;
use crate::pac;
use crate::pac::bdma::vals;
use crate::rcc::sealed::RccPeripheral;

use super::{Word, WordSize};

impl From<WordSize> for vals::Size {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BITS8,
            WordSize::TwoBytes => Self::BITS16,
            WordSize::FourBytes => Self::BITS32,
        }
    }
}

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

                for chn in 0..pac::dma_channels_count!($dma) {
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
        impl crate::dma::sealed::Channel for crate::peripherals::$channel_peri {

            unsafe fn start_write<W: Word>(&mut self, request: Request, buf: *const[W], reg_addr: *mut W) {
                let (ptr, len) = super::slice_ptr_parts(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    request,
                    vals::Dir::FROMMEMORY,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    vals::Size::from(W::bits()),
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                );
            }


            unsafe fn start_write_repeated<W: Word>(&mut self, request: Request, repeated: W, count: usize, reg_addr: *mut W) {
                let buf = [repeated];
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    request,
                    vals::Dir::FROMMEMORY,
                    reg_addr as *const u32,
                    buf.as_ptr() as *mut u32,
                    count,
                    false,
                    vals::Size::from(W::bits()),
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                )
            }

            unsafe fn start_read<W: Word>(&mut self, request: Request, reg_addr: *const W, buf: *mut [W]) {
                let (ptr, len) = super::slice_ptr_parts_mut(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    request,
                    vals::Dir::FROMPERIPHERAL,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    vals::Size::from(W::bits()),
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                );
            }

            fn request_stop(&mut self){
                unsafe {low_level_api::request_stop(pac::$dma_peri, $channel_num);}
            }

            fn is_running(&self) -> bool {
                unsafe {low_level_api::is_running(pac::$dma_peri, $channel_num)}
            }
            fn remaining_transfers(&mut self) -> u16 {
                unsafe {low_level_api::get_remaining_transfers(pac::$dma_peri, $channel_num)}
            }

            fn set_waker(&mut self, waker: &Waker) {
                unsafe {low_level_api::set_waker(dma_num!($dma_peri) * 8 + $channel_num, waker )}
            }
        }

        impl crate::dma::Channel for crate::peripherals::$channel_peri {}
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

mod low_level_api {
    use super::*;

    pub unsafe fn start_transfer(
        dma: pac::bdma::Dma,
        channel_number: u8,
        #[cfg(any(bdma_v2, dmamux))] request: Request,
        dir: vals::Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: vals::Size,
        #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
        #[cfg(dmamux)] dmamux_ch_num: u8,
    ) {
        let ch = dma.ch(channel_number as _);

        reset_status(dma, channel_number);

        #[cfg(dmamux)]
        super::super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

        #[cfg(bdma_v2)]
        critical_section::with(|_| {
            dma.cselr()
                .modify(|w| w.set_cs(channel_number as _, request))
        });

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        ch.par().write_value(peri_addr as u32);
        ch.mar().write_value(mem_addr as u32);
        ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
        ch.cr().write(|w| {
            w.set_psize(data_size);
            w.set_msize(data_size);
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
    }

    pub unsafe fn request_stop(dma: pac::bdma::Dma, channel_number: u8) {
        reset_status(dma, channel_number);

        let ch = dma.ch(channel_number as _);

        // Disable the channel and interrupts with the default value.
        ch.cr().write(|_| ());

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }

    pub unsafe fn is_running(dma: pac::bdma::Dma, ch: u8) -> bool {
        let ch = dma.ch(ch as _);
        ch.cr().read().en()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub unsafe fn get_remaining_transfers(dma: pac::bdma::Dma, ch: u8) -> u16 {
        // get a handle on the channel itself
        let ch = dma.ch(ch as _);
        // read the remaining transfer count. If this is zero, the transfer completed fully.
        ch.ndtr().read().ndt()
    }

    /// Sets the waker for the specified DMA channel
    pub unsafe fn set_waker(state_number: usize, waker: &Waker) {
        STATE.ch_wakers[state_number].register(waker);
    }

    pub unsafe fn reset_status(dma: pac::bdma::Dma, channel_number: u8) {
        dma.ifcr().write(|w| {
            w.set_tcif(channel_number as _, true);
            w.set_teif(channel_number as _, true);
        });
    }
}
