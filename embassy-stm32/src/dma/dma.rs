use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;

use crate::_generated::DMA_CHANNEL_COUNT;
use crate::interrupt;
use crate::pac;
use crate::pac::dma::{regs, vals};

use super::{Burst, FlowControl, Request, TransferOptions, Word, WordSize};

impl From<WordSize> for vals::Size {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BITS8,
            WordSize::TwoBytes => Self::BITS16,
            WordSize::FourBytes => Self::BITS32,
        }
    }
}

impl From<Burst> for vals::Burst {
    fn from(burst: Burst) -> Self {
        match burst {
            Burst::Single => vals::Burst::SINGLE,
            Burst::Incr4 => vals::Burst::INCR4,
            Burst::Incr8 => vals::Burst::INCR8,
            Burst::Incr16 => vals::Burst::INCR16,
        }
    }
}

impl From<FlowControl> for vals::Pfctrl {
    fn from(flow: FlowControl) -> Self {
        match flow {
            FlowControl::Dma => vals::Pfctrl::DMA,
            FlowControl::Peripheral => vals::Pfctrl::PERIPHERAL,
        }
    }
}

struct ChannelState {
    waker: AtomicWaker,
}

impl ChannelState {
    const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

struct State {
    channels: [ChannelState; DMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const CH: ChannelState = ChannelState::new();
        Self {
            channels: [CH; DMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init() {
    foreach_interrupt! {
        ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
            interrupt::$irq::steal().enable();
        };
    }
    crate::_generated::init_dma();
}

foreach_dma_channel! {
    ($channel_peri:ident, $dma_peri:ident, dma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl crate::dma::sealed::Channel for crate::peripherals::$channel_peri {
            unsafe fn start_write<W: Word>(&mut self, request: Request, buf: *const [W], reg_addr: *mut W, options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    vals::Dir::MEMORYTOPERIPHERAL,
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
                )
            }

            unsafe fn start_write_repeated<W: Word>(&mut self, request: Request, repeated: W, count: usize, reg_addr: *mut W, options: TransferOptions) {
                let buf = [repeated];
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    vals::Dir::MEMORYTOPERIPHERAL,
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

            unsafe fn start_read<W: Word>(&mut self, request: Request, reg_addr: *const W, buf: *mut [W], options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts_mut(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    vals::Dir::PERIPHERALTOMEMORY,
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

            unsafe fn start_double_buffered_read<W: Word>(
                &mut self,
                request: Request,
                reg_addr: *const W,
                buffer0: *mut W,
                buffer1: *mut W,
                buffer_len: usize,
                options: TransferOptions,
            ) {
                low_level_api::start_dbm_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    vals::Dir::PERIPHERALTOMEMORY,
                    reg_addr as *const u32,
                    buffer0 as *mut u32,
                    buffer1 as *mut u32,
                    buffer_len,
                    true,
                    vals::Size::from(W::bits()),
                    options,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_REGS,
                    #[cfg(dmamux)]
                    <Self as super::dmamux::sealed::MuxChannel>::DMAMUX_CH_NUM,
                );
            }

            unsafe fn set_buffer0<W: Word>(&mut self, buffer: *mut W) {
                low_level_api::set_dbm_buffer0(pac::$dma_peri, $channel_num, buffer as *mut u32);
            }

            unsafe fn set_buffer1<W: Word>(&mut self, buffer: *mut W) {
                low_level_api::set_dbm_buffer1(pac::$dma_peri, $channel_num, buffer as *mut u32);
            }

            unsafe fn is_buffer0_accessible(&mut self) -> bool {
                low_level_api::is_buffer0_accessible(pac::$dma_peri, $channel_num)
            }

            fn request_stop(&mut self) {
                unsafe {low_level_api::request_stop(pac::$dma_peri, $channel_num);}
            }

            fn is_running(&self) -> bool {
                unsafe {low_level_api::is_running(pac::$dma_peri, $channel_num)}
            }

            fn remaining_transfers(&mut self) -> u16 {
                unsafe {low_level_api::get_remaining_transfers(pac::$dma_peri, $channel_num)}
            }

            fn set_waker(&mut self, waker: &Waker) {
                unsafe {low_level_api::set_waker($index, waker )}
            }

            fn on_irq() {
                unsafe {
                    low_level_api::on_irq_inner(pac::$dma_peri, $channel_num, $index);
                }
            }
        }
        impl crate::dma::Channel for crate::peripherals::$channel_peri { }
    };
}

mod low_level_api {
    use super::*;

    pub unsafe fn start_transfer(
        dma: pac::dma::Dma,
        channel_number: u8,
        request: Request,
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
        #[cfg(dmamux)]
        super::super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        reset_status(dma, channel_number);

        let ch = dma.st(channel_number as _);
        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(mem_addr as u32);
        ch.ndtr().write_value(regs::Ndtr(mem_len as _));
        ch.cr().write(|w| {
            w.set_dir(dir);
            w.set_msize(data_size);
            w.set_psize(data_size);
            w.set_pl(vals::Pl::VERYHIGH);
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

            w.set_pburst(options.pburst.into());
            w.set_mburst(options.mburst.into());
            w.set_pfctrl(options.flow_ctrl.into());

            w.set_en(true);
        });
    }

    pub unsafe fn start_dbm_transfer(
        dma: pac::dma::Dma,
        channel_number: u8,
        request: Request,
        dir: vals::Dir,
        peri_addr: *const u32,
        mem0_addr: *mut u32,
        mem1_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: vals::Size,
        options: TransferOptions,
        #[cfg(dmamux)] dmamux_regs: pac::dmamux::Dmamux,
        #[cfg(dmamux)] dmamux_ch_num: u8,
    ) {
        #[cfg(dmamux)]
        super::super::dmamux::configure_dmamux(dmamux_regs, dmamux_ch_num, request);

        trace!(
            "Starting DBM transfer with 0: 0x{:x}, 1: 0x{:x}, len: 0x{:x}",
            mem0_addr as u32,
            mem1_addr as u32,
            mem_len
        );

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        reset_status(dma, channel_number);

        let ch = dma.st(channel_number as _);
        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(mem0_addr as u32);
        // configures the second buffer for DBM
        ch.m1ar().write_value(mem1_addr as u32);
        ch.ndtr().write_value(regs::Ndtr(mem_len as _));
        ch.cr().write(|w| {
            w.set_dir(dir);
            w.set_msize(data_size);
            w.set_psize(data_size);
            w.set_pl(vals::Pl::VERYHIGH);
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

            // enable double buffered mode
            w.set_dbm(vals::Dbm::ENABLED);

            w.set_pburst(options.pburst.into());
            w.set_mburst(options.mburst.into());
            w.set_pfctrl(options.flow_ctrl.into());

            w.set_en(true);
        });
    }

    pub unsafe fn set_dbm_buffer0(dma: pac::dma::Dma, channel_number: u8, mem_addr: *mut u32) {
        // get a handle on the channel itself
        let ch = dma.st(channel_number as _);
        // change M0AR to the new address
        ch.m0ar().write_value(mem_addr as _);
    }

    pub unsafe fn set_dbm_buffer1(dma: pac::dma::Dma, channel_number: u8, mem_addr: *mut u32) {
        // get a handle on the channel itself
        let ch = dma.st(channel_number as _);
        // change M1AR to the new address
        ch.m1ar().write_value(mem_addr as _);
    }

    pub unsafe fn is_buffer0_accessible(dma: pac::dma::Dma, channel_number: u8) -> bool {
        // get a handle on the channel itself
        let ch = dma.st(channel_number as _);
        // check the current target register value
        ch.cr().read().ct() == vals::Ct::MEMORY1
    }

    /// Stops the DMA channel.
    pub unsafe fn request_stop(dma: pac::dma::Dma, channel_number: u8) {
        // get a handle on the channel itself
        let ch = dma.st(channel_number as _);

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_tcie(true);
        });

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }

    /// Gets the running status of the channel
    pub unsafe fn is_running(dma: pac::dma::Dma, ch: u8) -> bool {
        // get a handle on the channel itself
        let ch = dma.st(ch as _);
        // Get whether it's enabled (running)
        ch.cr().read().en()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub unsafe fn get_remaining_transfers(dma: pac::dma::Dma, ch: u8) -> u16 {
        // get a handle on the channel itself
        let ch = dma.st(ch as _);
        // read the remaining transfer count. If this is zero, the transfer completed fully.
        ch.ndtr().read().ndt()
    }

    /// Sets the waker for the specified DMA channel
    pub unsafe fn set_waker(state_number: usize, waker: &Waker) {
        STATE.channels[state_number].waker.register(waker);
    }

    pub unsafe fn reset_status(dma: pac::dma::Dma, channel_number: u8) {
        let isrn = channel_number as usize / 4;
        let isrbit = channel_number as usize % 4;

        dma.ifcr(isrn).write(|w| {
            w.set_tcif(isrbit, true);
            w.set_teif(isrbit, true);
        });
    }

    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub unsafe fn on_irq_inner(dma: pac::dma::Dma, channel_num: u8, state_index: u8) {
        let channel_num = channel_num as usize;
        let state_index = state_index as usize;

        let cr = dma.st(channel_num).cr();
        let isr = dma.isr(channel_num / 4).read();

        if isr.teif(channel_num % 4) {
            panic!(
                "DMA: error on DMA@{:08x} channel {}",
                dma.0 as u32, channel_num
            );
        }

        if isr.tcif(channel_num % 4) && cr.read().tcie() {
            if cr.read().dbm() == vals::Dbm::DISABLED {
                cr.write(|_| ()); // Disable channel with the default value.
            } else {
                // for double buffered mode, clear TCIF flag but do not stop the transfer
                dma.ifcr(channel_num / 4)
                    .write(|w| w.set_tcif(channel_num % 4, true));
            }
            STATE.channels[state_index].waker.wake();
        }
    }
}
