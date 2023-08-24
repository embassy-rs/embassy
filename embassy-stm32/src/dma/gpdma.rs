#![macro_use]

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use super::word::{Word, WordSize};
use super::Dir;
use crate::_generated::GPDMA_CHANNEL_COUNT;
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::Priority;
use crate::pac;
use crate::pac::gpdma::vals;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {}
    }
}

impl From<WordSize> for vals::ChTr1Dw {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BYTE,
            WordSize::TwoBytes => Self::HALFWORD,
            WordSize::FourBytes => Self::WORD,
        }
    }
}

struct State {
    ch_wakers: [AtomicWaker; GPDMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; GPDMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init(irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, gpdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority(irq_priority);
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_gpdma();
}

foreach_dma_channel! {
    ($channel_peri:ident, $dma_peri:ident, gpdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl sealed::Channel for crate::peripherals::$channel_peri {
            fn regs(&self) -> pac::gpdma::Gpdma {
                pac::$dma_peri
            }
            fn num(&self) -> usize {
                $channel_num
            }
            fn index(&self) -> usize {
                $index
            }
            fn on_irq() {
                unsafe { on_irq_inner(pac::$dma_peri, $channel_num, $index) }
            }
        }

        impl Channel for crate::peripherals::$channel_peri {}
    };
}

/// Safety: Must be called with a matching set of parameters for a valid dma channel
pub(crate) unsafe fn on_irq_inner(dma: pac::gpdma::Gpdma, channel_num: usize, index: usize) {
    let ch = dma.ch(channel_num);
    let sr = ch.sr().read();

    if sr.dtef() {
        panic!(
            "DMA: data transfer error on DMA@{:08x} channel {}",
            dma.as_ptr() as u32,
            channel_num
        );
    }
    if sr.usef() {
        panic!(
            "DMA: user settings error on DMA@{:08x} channel {}",
            dma.as_ptr() as u32,
            channel_num
        );
    }

    if sr.suspf() || sr.tcf() {
        // disable all xxIEs to prevent the irq from firing again.
        ch.cr().write(|_| {});

        // Wake the future. It'll look at tcf and see it's set.
        STATE.ch_wakers[index].wake();
    }
}

pub type Request = u8;

#[cfg(dmamux)]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static + super::dmamux::MuxChannel {}
#[cfg(not(dmamux))]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static {}

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        fn regs(&self) -> pac::gpdma::Gpdma;
        fn num(&self) -> usize;
        fn index(&self) -> usize;
        fn on_irq();
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub unsafe fn new_read<W: Word>(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Self {
        Self::new_read_raw(channel, request, peri_addr, buf, options)
    }

    pub unsafe fn new_read_raw<W: Word>(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let (ptr, len) = super::slice_ptr_parts_mut(buf);
        assert!(len > 0 && len <= 0xFFFF);

        Self::new_inner(
            channel,
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            ptr as *mut u32,
            len,
            true,
            W::size(),
            options,
        )
    }

    pub unsafe fn new_write<W: Word>(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        Self::new_write_raw(channel, request, buf, peri_addr, options)
    }

    pub unsafe fn new_write_raw<W: Word>(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        buf: *const [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let (ptr, len) = super::slice_ptr_parts(buf);
        assert!(len > 0 && len <= 0xFFFF);

        Self::new_inner(
            channel,
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            ptr as *mut u32,
            len,
            true,
            W::size(),
            options,
        )
    }

    pub unsafe fn new_write_repeated<W: Word>(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        Self::new_inner(
            channel,
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            repeated as *const W as *mut u32,
            count,
            false,
            W::size(),
            options,
        )
    }

    unsafe fn new_inner(
        channel: PeripheralRef<'a, C>,
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        _options: TransferOptions,
    ) -> Self {
        let ch = channel.regs().ch(channel.num());

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let this = Self { channel };

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, request);

        ch.cr().write(|w| w.set_reset(true));
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF); // clear all irqs
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

        this
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);
        })
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().ch(self.channel.num());
        !ch.sr().read().tcf()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.br1().read().bndt()
    }

    pub fn blocking_wait(mut self) {
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);

        core::mem::forget(self);
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        STATE.ch_wakers[self.channel.index()].register(cx.waker());

        if self.is_running() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
