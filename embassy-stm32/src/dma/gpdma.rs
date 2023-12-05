#![macro_use]

use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::sync::atomic::{fence, AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use super::ringbuffer::{DmaCtrl, OverrunError, ReadableDmaRingBuffer, WritableDmaRingBuffer};
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
    circular_address: [AtomicUsize; GPDMA_CHANNEL_COUNT],
    complete_count: [AtomicUsize; GPDMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const ZERO: AtomicUsize = AtomicUsize::new(0);
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; GPDMA_CHANNEL_COUNT],
            circular_address: [ZERO; GPDMA_CHANNEL_COUNT],
            complete_count: [ZERO; GPDMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init(cs: critical_section::CriticalSection, irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, gpdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, irq_priority);
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

    if sr.htf() {
        //clear the flag for the half transfer complete
        ch.fcr().modify(|w| w.set_htf(true));
        STATE.ch_wakers[index].wake();
    }

    if sr.tcf() {
        //clear the flag for the transfer complete
        ch.fcr().modify(|w| w.set_tcf(true));
        STATE.complete_count[index].fetch_add(1, Ordering::Relaxed);
        STATE.ch_wakers[index].wake();
        return;
    }

    if sr.suspf() {
        ch.fcr().modify(|w| w.set_suspf(true));

        // disable all xxIEs to prevent the irq from firing again.
        ch.cr().modify(|w| {
            w.set_tcie(false);
            w.set_useie(false);
            w.set_dteie(false);
            w.set_suspie(false);
            w.set_htie(false);
        });

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
        ch.cr().modify(|w| {
            w.set_susp(true);
        })
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().ch(self.channel.num());
        let sr = ch.sr().read();
        !sr.tcf() && !sr.suspf()
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

struct DmaCtrlImpl<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
    word_size: WordSize,
}

impl<'a, C: Channel> DmaCtrl for DmaCtrlImpl<'a, C> {
    fn get_remaining_transfers(&self) -> usize {
        let ch = self.channel.regs().ch(self.channel.num());
        (ch.br1().read().bndt() / self.word_size.bytes() as u16) as usize
    }

    fn get_complete_count(&self) -> usize {
        STATE.complete_count[self.channel.index()].load(Ordering::Acquire)
    }

    fn reset_complete_count(&mut self) -> usize {
        STATE.complete_count[self.channel.index()].swap(0, Ordering::AcqRel)
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE.ch_wakers[self.channel.index()].register(waker);
    }
}

struct RingBuffer {}

impl RingBuffer {
    fn configure<'a, W: Word>(
        ch: &pac::gpdma::Channel,
        channel_index: usize,
        request: Request,
        dir: Dir,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        _options: TransferOptions,
    ) {
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let (mem_addr, mem_len) = super::slice_ptr_parts_mut(buffer);

        ch.cr().write(|w| w.set_reset(true));
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF); // clear all irqs

        if mem_addr & 0b11 != 0 {
            panic!("circular address must be 4-byte aligned");
        }

        STATE.circular_address[channel_index].store(mem_addr, Ordering::Release);
        let lli = STATE.circular_address[channel_index].as_ptr() as u32;
        ch.llr().write(|w| {
            match dir {
                Dir::MemoryToPeripheral => w.set_usa(true),
                Dir::PeripheralToMemory => w.set_uda(true),
            }
            // lower 16 bits of the memory address
            w.set_la(((lli >> 2usize) & 0x3fff) as u16);
        });
        ch.lbar().write(|w| {
            // upper 16 bits of the address of lli1
            w.set_lba((lli >> 16usize) as u16);
        });

        let data_size = W::size();
        ch.tr1().write(|w| {
            w.set_sdw(data_size.into());
            w.set_ddw(data_size.into());
            w.set_sinc(dir == Dir::MemoryToPeripheral);
            w.set_dinc(dir == Dir::PeripheralToMemory);
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
    }

    fn clear_irqs(ch: &pac::gpdma::Channel) {
        ch.fcr().modify(|w| {
            w.set_htf(true);
            w.set_tcf(true);
            w.set_suspf(true);
        });
    }

    fn is_running(ch: &pac::gpdma::Channel) -> bool {
        !ch.sr().read().tcf()
    }

    fn request_suspend(ch: &pac::gpdma::Channel) {
        ch.cr().modify(|w| {
            w.set_susp(true);
        });
    }

    async fn suspend(ch: &pac::gpdma::Channel, set_waker: &mut dyn FnMut(&Waker)) {
        use core::sync::atomic::compiler_fence;

        Self::request_suspend(ch);

        //wait until cr.susp reads as true
        poll_fn(|cx| {
            set_waker(cx.waker());

            compiler_fence(Ordering::SeqCst);

            let cr = ch.cr().read();
            if cr.susp() {
                defmt::info!("Ready {}", cr.susp());
                Poll::Ready(())
            } else {
                defmt::info!("still pending {}", cr.susp());
                Poll::Pending
            }
        })
        .await
    }

    fn resume(ch: &pac::gpdma::Channel) {
        Self::clear_irqs(ch);
        ch.cr().modify(|w| {
            w.set_susp(false);
            w.set_en(true);
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);
            w.set_htie(true);
        });
    }
}

pub struct ReadableRingBuffer<'a, C: Channel, W: Word> {
    channel: PeripheralRef<'a, C>,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, C: Channel, W: Word> ReadableRingBuffer<'a, C, W> {
    pub unsafe fn new_read(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut channel, request);

        RingBuffer::configure(
            &channel.regs().ch(channel.num()),
            channel.index(),
            request,
            Dir::PeripheralToMemory,
            peri_addr,
            buffer,
            options,
        );

        Self {
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    pub fn start(&mut self) {
        let ch = &self.channel.regs().ch(self.channel.num());
        RingBuffer::clear_irqs(ch);
        ch.cr().modify(|w| w.set_en(true));
    }

    pub fn request_stop(&mut self, ch: &pac::gpdma::Channel) {
        ch.cr().modify(|w| w.set_en(false));
    }

    pub async fn suspend(&mut self) {
        RingBuffer::suspend(&self.channel.regs().ch(self.channel.num()), &mut |waker| {
            self.set_waker(waker)
        })
        .await
    }

    pub fn resume(&mut self) {
        RingBuffer::resume(&self.channel.regs().ch(self.channel.num()));
    }

    pub fn clear(&mut self) {
        self.ringbuf.clear(&mut DmaCtrlImpl {
            channel: self.channel.reborrow(),
            word_size: W::size(),
        });
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.read(
            &mut DmaCtrlImpl {
                channel: self.channel.reborrow(),
                word_size: W::size(),
            },
            buf,
        )
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, OverrunError> {
        self.ringbuf
            .read_exact(
                &mut DmaCtrlImpl {
                    channel: self.channel.reborrow(),
                    word_size: W::size(),
                },
                buffer,
            )
            .await
    }

    // The capacity of the ringbuffer
    pub const fn cap(&self) -> usize {
        self.ringbuf.cap()
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl {
            channel: self.channel.reborrow(),
            word_size: W::size(),
        }
        .set_waker(waker);
    }

    pub fn is_running(&mut self) -> bool {
        RingBuffer::is_running(&self.channel.regs().ch(self.channel.num()))
    }
}

impl<'a, C: Channel, W: Word> Drop for ReadableRingBuffer<'a, C, W> {
    fn drop(&mut self) {
        RingBuffer::request_suspend(&self.channel.regs().ch(self.channel.num()));
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

pub struct WritableRingBuffer<'a, C: Channel, W: Word> {
    #[allow(dead_code)] //this is only read by the DMA controller
    channel: PeripheralRef<'a, C>,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, C: Channel, W: Word> WritableRingBuffer<'a, C, W> {
    pub unsafe fn new_write(
        channel: impl Peripheral<P = C> + 'a,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut channel, request);

        RingBuffer::configure(
            &channel.regs().ch(channel.num()),
            channel.index(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr,
            buffer,
            options,
        );

        Self {
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    pub fn start(&mut self) {
        self.resume();
    }

    pub async fn suspend(&mut self) {
        RingBuffer::suspend(&self.channel.regs().ch(self.channel.num()), &mut |waker| {
            self.set_waker(waker)
        })
        .await
    }

    pub fn resume(&mut self) {
        RingBuffer::resume(&self.channel.regs().ch(self.channel.num()));
    }

    pub fn request_stop(&mut self) {
        // reads can be stopped by disabling the enable flag
        let ch = &self.channel.regs().ch(self.channel.num());
        ch.cr().modify(|w| w.set_en(false));
    }

    pub fn clear(&mut self) {
        self.ringbuf.clear(&mut DmaCtrlImpl {
            channel: self.channel.reborrow(),
            word_size: W::size(),
        });
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.write(
            &mut DmaCtrlImpl {
                channel: self.channel.reborrow(),
                word_size: W::size(),
            },
            buf,
        )
    }

    pub fn prime(&mut self, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.prime(
            &mut DmaCtrlImpl {
                channel: self.channel.reborrow(),
                word_size: W::size(),
            },
            buf,
        )
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, OverrunError> {
        self.ringbuf
            .write_exact(
                &mut DmaCtrlImpl {
                    channel: self.channel.reborrow(),
                    word_size: W::size(),
                },
                buffer,
            )
            .await
    }

    // The capacity of the ringbuffer
    pub const fn cap(&self) -> usize {
        self.ringbuf.cap()
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl {
            channel: self.channel.reborrow(),
            word_size: W::size(),
        }
        .set_waker(waker);
    }

    pub fn is_running(&mut self) -> bool {
        RingBuffer::is_running(&self.channel.regs().ch(self.channel.num()))
    }
}

impl<'a, C: Channel, W: Word> Drop for WritableRingBuffer<'a, C, W> {
    fn drop(&mut self) {
        RingBuffer::request_suspend(&self.channel.regs().ch(self.channel.num()));
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}
