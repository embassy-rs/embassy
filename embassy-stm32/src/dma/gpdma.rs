use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;

use crate::_generated::GPDMA_CHANNEL_COUNT;
use crate::interrupt;
use crate::pac;
use crate::pac::gpdma::{vals, Gpdma};

use super::{Request, TransferOptions, Word, WordSize};

impl From<WordSize> for vals::ChTr1Dw {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BYTE,
            WordSize::TwoBytes => Self::HALFWORD,
            WordSize::FourBytes => Self::WORD,
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
    channels: [ChannelState; GPDMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const CH: ChannelState = ChannelState::new();
        Self {
            channels: [CH; GPDMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init() {
    foreach_interrupt! {
        ($peri:ident, gpdma, $block:ident, $signal_name:ident, $irq:ident) => {
            interrupt::$irq::steal().enable();
        };
    }
    crate::_generated::init_gpdma();
}

foreach_dma_channel! {
    ($channel_peri:ident, $dma_peri:ident, gpdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl crate::dma::sealed::Channel for crate::peripherals::$channel_peri {
            unsafe fn start_write<W: Word>(&mut self, request: Request, buf: *const [W], reg_addr: *mut W, options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    low_level_api::Dir::MemoryToPeripheral,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    W::bits(),
                    options,
                )
            }

            unsafe fn start_write_repeated<W: Word>(&mut self, request: Request, repeated: W, count: usize, reg_addr: *mut W, options: TransferOptions) {
                let buf = [repeated];
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    low_level_api::Dir::MemoryToPeripheral,
                    reg_addr as *const u32,
                    buf.as_ptr() as *mut u32,
                    count,
                    false,
                    W::bits(),
                    options,
                )
            }

            unsafe fn start_read<W: Word>(&mut self, request: Request, reg_addr: *const W, buf: *mut [W], options: TransferOptions) {
                let (ptr, len) = super::slice_ptr_parts_mut(buf);
                low_level_api::start_transfer(
                    pac::$dma_peri,
                    $channel_num,
                    request,
                    low_level_api::Dir::PeripheralToMemory,
                    reg_addr as *const u32,
                    ptr as *mut u32,
                    len,
                    true,
                    W::bits(),
                    options,
                );
            }

            unsafe fn start_double_buffered_read<W: Word>(
                &mut self,
                _request: Request,
                _reg_addr: *const W,
                _buffer0: *mut W,
                _buffer1: *mut W,
                _buffer_len: usize,
                _options: TransferOptions,
            ) {
                panic!("Unsafe double buffered mode is unavailable on GPBDMA");
            }

            unsafe fn set_buffer0<W: Word>(&mut self, _buffer: *mut W) {
                panic!("Unsafe double buffered mode is unavailable on GPBDMA");
            }

            unsafe fn set_buffer1<W: Word>(&mut self, _buffer: *mut W) {
                panic!("Unsafe double buffered mode is unavailable on GPBDMA");
            }

            unsafe fn is_buffer0_accessible(&mut self) -> bool {
                panic!("Unsafe double buffered mode is unavailable on GPBDMA");
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

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Dir {
        MemoryToPeripheral,
        PeripheralToMemory,
    }

    pub unsafe fn start_transfer(
        dma: Gpdma,
        channel_number: u8,
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        _options: TransferOptions,
    ) {
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let ch = dma.ch(channel_number as _);
        ch.llr().write(|_| {}); // no linked list
        ch.tr1().write(|w| {
            w.set_sdw(data_size.into());
            w.set_ddw(data_size.into());
            w.set_sinc(dir == Dir::MemoryToPeripheral && incr_mem);
            w.set_dinc(dir == Dir::PeripheralToMemory && incr_mem);
        });
        ch.tr2().write(|w| {
            w.set_dreq(match dir {
                Dir::MemoryToPeripheral => vals::ChTr2Dreq::DESTINATIONPERIPHERAL,
                Dir::PeripheralToMemory => vals::ChTr2Dreq::SOURCEPERIPHERAL,
            });
            w.set_reqsel(request);
        });
        ch.br1().write(|w| {
            // BNDT is specified as bytes, not as number of transfers.
            w.set_bndt((mem_len * data_size.bytes()) as u16)
        });

        match dir {
            Dir::MemoryToPeripheral => {
                ch.sar().write_value(mem_addr as _);
                ch.dar().write_value(peri_addr as _);
            }
            Dir::PeripheralToMemory => {
                ch.sar().write_value(peri_addr as _);
                ch.dar().write_value(mem_addr as _);
            }
        }

        ch.cr().write(|w| {
            // Enable interrupts
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);

            // Start it
            w.set_en(true);
        });
    }

    /// Stops the DMA channel.
    pub unsafe fn request_stop(dma: Gpdma, channel_number: u8) {
        // get a handle on the channel itself
        let ch = dma.ch(channel_number as _);

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);
        });

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }

    /// Gets the running status of the channel
    pub unsafe fn is_running(dma: Gpdma, ch: u8) -> bool {
        let ch = dma.ch(ch as _);
        !ch.sr().read().idlef()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub unsafe fn get_remaining_transfers(dma: Gpdma, ch: u8) -> u16 {
        // get a handle on the channel itself
        let ch = dma.ch(ch as _);
        // read the remaining transfer count. If this is zero, the transfer completed fully.
        ch.br1().read().bndt()
    }

    /// Sets the waker for the specified DMA channel
    pub unsafe fn set_waker(state_number: usize, waker: &Waker) {
        STATE.channels[state_number].waker.register(waker);
    }

    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub unsafe fn on_irq_inner(dma: Gpdma, channel_num: u8, state_index: u8) {
        let channel_num = channel_num as usize;
        let state_index = state_index as usize;

        let ch = dma.ch(channel_num);
        let sr = ch.sr().read();

        if sr.dtef() {
            panic!(
                "DMA: data transfer error on DMA@{:08x} channel {}",
                dma.0 as u32, channel_num
            );
        }
        if sr.usef() {
            panic!(
                "DMA: user settings error on DMA@{:08x} channel {}",
                dma.0 as u32, channel_num
            );
        }

        if sr.suspf() || sr.tcf() {
            ch.cr().write(|w| w.set_reset(true));
            STATE.channels[state_index].waker.wake();
        }
    }
}
