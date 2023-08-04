#![macro_use]

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{fence, Ordering};
use core::task::{Context, Poll, Waker};

use atomic_polyfill::AtomicUsize;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use super::ringbuffer::{DmaCtrl, OverrunError, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use super::word::{Word, WordSize};
use super::Dir;
use crate::_generated::BDMA_CHANNEL_COUNT;
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::Priority;
use crate::pac;
use crate::pac::bdma::{regs, vals};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Enable circular DMA
    pub circular: bool,
    /// Enable half transfer interrupt
    pub half_transfer_ir: bool,
    /// Enable transfer complete interrupt
    pub complete_transfer_ir: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            circular: false,
            half_transfer_ir: false,
            complete_transfer_ir: true,
        }
    }
}

impl From<WordSize> for vals::Size {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BITS8,
            WordSize::TwoBytes => Self::BITS16,
            WordSize::FourBytes => Self::BITS32,
        }
    }
}

impl From<Dir> for vals::Dir {
    fn from(raw: Dir) -> Self {
        match raw {
            Dir::MemoryToPeripheral => Self::FROMMEMORY,
            Dir::PeripheralToMemory => Self::FROMPERIPHERAL,
        }
    }
}

struct State {
    ch_wakers: [AtomicWaker; BDMA_CHANNEL_COUNT],
    complete_count: [AtomicUsize; BDMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const ZERO: AtomicUsize = AtomicUsize::new(0);
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; BDMA_CHANNEL_COUNT],
            complete_count: [ZERO; BDMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init(irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority(irq_priority);
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_bdma();
}

foreach_dma_channel! {
    ($channel_peri:ident, BDMA1, bdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        // BDMA1 in H7 doesn't use DMAMUX, which breaks
    };
    ($channel_peri:ident, $dma_peri:ident, bdma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl sealed::Channel for crate::peripherals::$channel_peri {
            fn regs(&self) -> pac::bdma::Dma {
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
pub(crate) unsafe fn on_irq_inner(dma: pac::bdma::Dma, channel_num: usize, index: usize) {
    let isr = dma.isr().read();
    let cr = dma.ch(channel_num).cr();

    if isr.teif(channel_num) {
        panic!("DMA: error on BDMA@{:08x} channel {}", dma.as_ptr() as u32, channel_num);
    }

    if isr.htif(channel_num) && cr.read().htie() {
        // Acknowledge half transfer complete interrupt
        dma.ifcr().write(|w| w.set_htif(channel_num, true));
    } else if isr.tcif(channel_num) && cr.read().tcie() {
        // Acknowledge transfer complete interrupt
        dma.ifcr().write(|w| w.set_tcif(channel_num, true));
        STATE.complete_count[index].fetch_add(1, Ordering::Release);
    } else {
        return;
    }

    STATE.ch_wakers[index].wake();
}

#[cfg(any(bdma_v2, dmamux))]
pub type Request = u8;
#[cfg(not(any(bdma_v2, dmamux)))]
pub type Request = ();

#[cfg(dmamux)]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static + super::dmamux::MuxChannel {}
#[cfg(not(dmamux))]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static {}

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        fn regs(&self) -> pac::bdma::Dma;
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
        _request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        options: TransferOptions,
    ) -> Self {
        let ch = channel.regs().ch(channel.num());

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        #[cfg(bdma_v2)]
        critical_section::with(|_| channel.regs().cselr().modify(|w| w.set_cs(channel.num(), _request)));

        let mut this = Self { channel };
        this.clear_irqs();
        STATE.complete_count[this.channel.index()].store(0, Ordering::Release);

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        ch.par().write_value(peri_addr as u32);
        ch.mar().write_value(mem_addr as u32);
        ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
        ch.cr().write(|w| {
            w.set_psize(data_size.into());
            w.set_msize(data_size.into());
            if incr_mem {
                w.set_minc(vals::Inc::ENABLED);
            } else {
                w.set_minc(vals::Inc::DISABLED);
            }
            w.set_dir(dir.into());
            w.set_teie(true);
            w.set_tcie(options.complete_transfer_ir);
            w.set_htie(options.half_transfer_ir);
            if options.circular {
                w.set_circ(vals::Circ::ENABLED);
                debug!("Setting circular mode");
            } else {
                w.set_circ(vals::Circ::DISABLED);
            }
            w.set_pl(vals::Pl::VERYHIGH);
            w.set_en(true);
        });

        this
    }

    fn clear_irqs(&mut self) {
        self.channel.regs().ifcr().write(|w| {
            w.set_tcif(self.channel.num(), true);
            w.set_teif(self.channel.num(), true);
        });
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().ch(self.channel.num());
        let en = ch.cr().read().en();
        let circular = ch.cr().read().circ() == vals::Circ::ENABLED;
        let tcif = STATE.complete_count[self.channel.index()].load(Ordering::Acquire) != 0;
        en && (circular || !tcif)
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.ndtr().read().ndt()
    }

    pub fn blocking_wait(mut self) {
        while self.is_running() {}
        self.request_stop();

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

// ==============================

struct DmaCtrlImpl<'a, C: Channel>(PeripheralRef<'a, C>);

impl<'a, C: Channel> DmaCtrl for DmaCtrlImpl<'a, C> {
    fn get_remaining_transfers(&self) -> usize {
        let ch = self.0.regs().ch(self.0.num());
        ch.ndtr().read().ndt() as usize
    }

    fn get_complete_count(&self) -> usize {
        STATE.complete_count[self.0.index()].load(Ordering::Acquire)
    }

    fn reset_complete_count(&mut self) -> usize {
        STATE.complete_count[self.0.index()].swap(0, Ordering::AcqRel)
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE.ch_wakers[self.0.index()].register(waker);
    }
}

pub struct ReadableRingBuffer<'a, C: Channel, W: Word> {
    cr: regs::Cr,
    channel: PeripheralRef<'a, C>,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, C: Channel, W: Word> ReadableRingBuffer<'a, C, W> {
    pub unsafe fn new_read(
        channel: impl Peripheral<P = C> + 'a,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        _options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let len = buffer.len();
        assert!(len > 0 && len <= 0xFFFF);

        let dir = Dir::PeripheralToMemory;
        let data_size = W::size();

        let channel_number = channel.num();
        let dma = channel.regs();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        #[cfg(bdma_v2)]
        critical_section::with(|_| channel.regs().cselr().modify(|w| w.set_cs(channel.num(), _request)));

        let mut w = regs::Cr(0);
        w.set_psize(data_size.into());
        w.set_msize(data_size.into());
        w.set_minc(vals::Inc::ENABLED);
        w.set_dir(dir.into());
        w.set_teie(true);
        w.set_htie(true);
        w.set_tcie(true);
        w.set_circ(vals::Circ::ENABLED);
        w.set_pl(vals::Pl::VERYHIGH);
        w.set_en(true);

        let buffer_ptr = buffer.as_mut_ptr();
        let mut this = Self {
            channel,
            cr: w,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        };
        this.clear_irqs();

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        let ch = dma.ch(channel_number);
        ch.par().write_value(peri_addr as u32);
        ch.mar().write_value(buffer_ptr as u32);
        ch.ndtr().write(|w| w.set_ndt(len as u16));

        this
    }

    pub fn start(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.cr().write_value(self.cr)
    }

    pub fn clear(&mut self) {
        self.ringbuf.clear(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.read(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
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
            .read_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// The capacity of the ringbuffer.
    pub const fn cap(&self) -> usize {
        self.ringbuf.cap()
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    fn clear_irqs(&mut self) {
        let dma = self.channel.regs();
        dma.ifcr().write(|w| {
            w.set_htif(self.channel.num(), true);
            w.set_tcif(self.channel.num(), true);
            w.set_teif(self.channel.num(), true);
        });
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        // If the channel is enabled and transfer is not completed, we need to perform
        // two separate write access to the CR register to disable the channel.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_htie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.cr().read().en()
    }
}

impl<'a, C: Channel, W: Word> Drop for ReadableRingBuffer<'a, C, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

pub struct WritableRingBuffer<'a, C: Channel, W: Word> {
    cr: regs::Cr,
    channel: PeripheralRef<'a, C>,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, C: Channel, W: Word> WritableRingBuffer<'a, C, W> {
    pub unsafe fn new_write(
        channel: impl Peripheral<P = C> + 'a,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        _options: TransferOptions,
    ) -> Self {
        into_ref!(channel);

        let len = buffer.len();
        assert!(len > 0 && len <= 0xFFFF);

        let dir = Dir::MemoryToPeripheral;
        let data_size = W::size();

        let channel_number = channel.num();
        let dma = channel.regs();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        #[cfg(bdma_v2)]
        critical_section::with(|_| channel.regs().cselr().modify(|w| w.set_cs(channel.num(), _request)));

        let mut w = regs::Cr(0);
        w.set_psize(data_size.into());
        w.set_msize(data_size.into());
        w.set_minc(vals::Inc::ENABLED);
        w.set_dir(dir.into());
        w.set_teie(true);
        w.set_htie(true);
        w.set_tcie(true);
        w.set_circ(vals::Circ::ENABLED);
        w.set_pl(vals::Pl::VERYHIGH);
        w.set_en(true);

        let buffer_ptr = buffer.as_mut_ptr();
        let mut this = Self {
            channel,
            cr: w,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        };
        this.clear_irqs();

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        let ch = dma.ch(channel_number);
        ch.par().write_value(peri_addr as u32);
        ch.mar().write_value(buffer_ptr as u32);
        ch.ndtr().write(|w| w.set_ndt(len as u16));

        this
    }

    pub fn start(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.cr().write_value(self.cr)
    }

    pub fn clear(&mut self) {
        self.ringbuf.clear(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Write elements to the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.write(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, OverrunError> {
        self.ringbuf
            .write_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// The capacity of the ringbuffer.
    pub const fn cap(&self) -> usize {
        self.ringbuf.cap()
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    fn clear_irqs(&mut self) {
        let dma = self.channel.regs();
        dma.ifcr().write(|w| {
            w.set_htif(self.channel.num(), true);
            w.set_tcif(self.channel.num(), true);
            w.set_teif(self.channel.num(), true);
        });
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().ch(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        // If the channel is enabled and transfer is not completed, we need to perform
        // two separate write access to the CR register to disable the channel.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_htie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().ch(self.channel.num());
        ch.cr().read().en()
    }
}

impl<'a, C: Channel, W: Word> Drop for WritableRingBuffer<'a, C, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}
