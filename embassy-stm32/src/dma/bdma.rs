#![macro_use]

use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;

use crate::_generated::BDMA_CHANNEL_COUNT;
use crate::dma::Request;
use crate::pac;
use crate::pac::bdma::vals;

use super::{TransferOptions, Word, WordSize};

impl From<WordSize> for vals::Size {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BITS8,
            WordSize::TwoBytes => Self::BITS16,
            WordSize::FourBytes => Self::BITS32,
        }
    }
}

struct State {
    ch_wakers: [AtomicWaker; BDMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; BDMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init() {
    foreach_interrupt! {
        ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::$irq::steal().enable();
        };
    }
    crate::_generated::init_bdma();
}

foreach_dma_channel! {
    ($channel_peri:ident, BDMA1, bdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        // BDMA1 in H7 doesn't use DMAMUX, which breaks
    };
    ($channel_peri:ident, $dma_peri:ident, bdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl crate::dma::sealed::Channel for crate::peripherals::$channel_peri {

            unsafe fn start_write<W: Word>(&mut self, _request: Request, buf: *const[W], reg_addr: *mut W, options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    _request,
                    vals::Dir::FROMMEMORY,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    vals::Size::from(W::bits()),
                    options,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                );
            }


            unsafe fn start_write_repeated<W: Word>(&mut self, _request: Request, repeated: W, count: usize, reg_addr: *mut W, options: TransferOptions) {
                let buf = [repeated];
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    _request,
                    vals::Dir::FROMMEMORY,
                    reg_addr as *const u32,
                    buf.as_ptr() as *mut u32,
                    count,
                    false,
                    vals::Size::from(W::bits()),
                    options,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                )
            }

            unsafe fn start_read<W: Word>(&mut self, _request: Request, reg_addr: *const W, buf: *mut [W], options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts_mut(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    #[cfg(any(bdma_v2, dmamux))]
                    _request,
                    vals::Dir::FROMPERIPHERAL,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    vals::Size::from(W::bits()),
                    options,
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
                unsafe { low_level_api::set_waker($index, waker) }
            }

            fn on_irq() {
                unsafe {
                    low_level_api::on_irq_inner(pac::$dma_peri, $channel_num, $index);
                }
            }
        }

        impl crate::dma::Channel for crate::peripherals::$channel_peri {}
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
        options: TransferOptions,
        #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
        #[cfg(dmamux)] dmamux_ch_num: u8,
    ) {
        assert!(
            options.mburst == crate::dma::Burst::Single,
            "Burst mode not supported"
        );
        assert!(
            options.pburst == crate::dma::Burst::Single,
            "Burst mode not supported"
        );
        assert!(
            options.flow_ctrl == crate::dma::FlowControl::Dma,
            "Peripheral flow control not supported"
        );

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

    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub unsafe fn on_irq_inner(dma: pac::bdma::Dma, channel_num: u8, index: u8) {
        let channel_num = channel_num as usize;
        let index = index as usize;

        let isr = dma.isr().read();
        let cr = dma.ch(channel_num).cr();

        if isr.teif(channel_num) {
            panic!(
                "DMA: error on BDMA@{:08x} channel {}",
                dma.0 as u32, channel_num
            );
        }
        if isr.tcif(channel_num) && cr.read().tcie() {
            cr.write(|_| ()); // Disable channel interrupts with the default value.
            STATE.ch_wakers[index].wake();
        }
    }
}
