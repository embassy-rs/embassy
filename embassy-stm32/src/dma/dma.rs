use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{fence, AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use super::ringbuffer::{DmaCtrl, DmaRingBuffer, OverrunError};
use super::word::{Word, WordSize};
use super::Dir;
use crate::_generated::DMA_CHANNEL_COUNT;
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::Priority;
use crate::pac::dma::{regs, vals};
use crate::{interrupt, pac};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Peripheral burst transfer configuration
    pub pburst: Burst,
    /// Memory burst transfer configuration
    pub mburst: Burst,
    /// Flow control configuration
    pub flow_ctrl: FlowControl,
    /// FIFO threshold for DMA FIFO mode. If none, direct mode is used.
    pub fifo_threshold: Option<FifoThreshold>,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            pburst: Burst::Single,
            mburst: Burst::Single,
            flow_ctrl: FlowControl::Dma,
            fifo_threshold: None,
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
            Dir::MemoryToPeripheral => Self::MEMORYTOPERIPHERAL,
            Dir::PeripheralToMemory => Self::PERIPHERALTOMEMORY,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Burst {
    /// Single transfer
    Single,
    /// Incremental burst of 4 beats
    Incr4,
    /// Incremental burst of 8 beats
    Incr8,
    /// Incremental burst of 16 beats
    Incr16,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl {
    /// Flow control by DMA
    Dma,
    /// Flow control by peripheral
    Peripheral,
}

impl From<FlowControl> for vals::Pfctrl {
    fn from(flow: FlowControl) -> Self {
        match flow {
            FlowControl::Dma => vals::Pfctrl::DMA,
            FlowControl::Peripheral => vals::Pfctrl::PERIPHERAL,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FifoThreshold {
    /// 1/4 full FIFO
    Quarter,
    /// 1/2 full FIFO
    Half,
    /// 3/4 full FIFO
    ThreeQuarters,
    /// Full FIFO
    Full,
}

impl From<FifoThreshold> for vals::Fth {
    fn from(value: FifoThreshold) -> Self {
        match value {
            FifoThreshold::Quarter => vals::Fth::QUARTER,
            FifoThreshold::Half => vals::Fth::HALF,
            FifoThreshold::ThreeQuarters => vals::Fth::THREEQUARTERS,
            FifoThreshold::Full => vals::Fth::FULL,
        }
    }
}

struct State {
    ch_wakers: [AtomicWaker; DMA_CHANNEL_COUNT],
    complete_count: [AtomicUsize; DMA_CHANNEL_COUNT],
}

impl State {
    const fn new() -> Self {
        const ZERO: AtomicUsize = AtomicUsize::new(0);
        const AW: AtomicWaker = AtomicWaker::new();
        Self {
            ch_wakers: [AW; DMA_CHANNEL_COUNT],
            complete_count: [ZERO; DMA_CHANNEL_COUNT],
        }
    }
}

static STATE: State = State::new();

/// safety: must be called only once
pub(crate) unsafe fn init(irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
            interrupt::typelevel::$irq::set_priority(irq_priority);
            interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_dma();
}

foreach_dma_channel! {
    ($channel_peri:ident, $dma_peri:ident, dma, $channel_num:expr, $index:expr, $dmamux:tt) => {
        impl sealed::Channel for crate::peripherals::$channel_peri {
            fn regs(&self) -> pac::dma::Dma {
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
pub(crate) unsafe fn on_irq_inner(dma: pac::dma::Dma, channel_num: usize, index: usize) {
    let cr = dma.st(channel_num).cr();
    let isr = dma.isr(channel_num / 4).read();

    if isr.teif(channel_num % 4) {
        panic!("DMA: error on DMA@{:08x} channel {}", dma.as_ptr() as u32, channel_num);
    }

    if isr.htif(channel_num % 4) && cr.read().htie() {
        // Acknowledge half transfer complete interrupt
        dma.ifcr(channel_num / 4).write(|w| w.set_htif(channel_num % 4, true));
    } else if isr.tcif(channel_num % 4) && cr.read().tcie() {
        // Acknowledge  transfer complete interrupt
        dma.ifcr(channel_num / 4).write(|w| w.set_tcif(channel_num % 4, true));
        STATE.complete_count[index].fetch_add(1, Ordering::Release);
    } else {
        return;
    }

    STATE.ch_wakers[index].wake();
}

#[cfg(any(dma_v2, dmamux))]
pub type Request = u8;
#[cfg(not(any(dma_v2, dmamux)))]
pub type Request = ();

#[cfg(dmamux)]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static + super::dmamux::MuxChannel {}
#[cfg(not(dmamux))]
pub trait Channel: sealed::Channel + Peripheral<P = Self> + 'static {}

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        fn regs(&self) -> pac::dma::Dma;
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
        let ch = channel.regs().st(channel.num());

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let mut this = Self { channel };
        this.clear_irqs();

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(mem_addr as u32);
        ch.ndtr().write_value(regs::Ndtr(mem_len as _));
        ch.fcr().write(|w| {
            if let Some(fth) = options.fifo_threshold {
                // FIFO mode
                w.set_dmdis(vals::Dmdis::DISABLED);
                w.set_fth(fth.into());
            } else {
                // Direct mode
                w.set_dmdis(vals::Dmdis::ENABLED);
            }
        });
        ch.cr().write(|w| {
            w.set_dir(dir.into());
            w.set_msize(data_size.into());
            w.set_psize(data_size.into());
            w.set_pl(vals::Pl::VERYHIGH);
            w.set_minc(match incr_mem {
                true => vals::Inc::INCREMENTED,
                false => vals::Inc::FIXED,
            });
            w.set_pinc(vals::Inc::FIXED);
            w.set_teie(true);
            w.set_tcie(true);
            #[cfg(dma_v1)]
            w.set_trbuff(true);

            #[cfg(dma_v2)]
            w.set_chsel(_request);

            w.set_pburst(options.pburst.into());
            w.set_mburst(options.mburst.into());
            w.set_pfctrl(options.flow_ctrl.into());

            w.set_en(true);
        });

        this
    }

    fn clear_irqs(&mut self) {
        let isrn = self.channel.num() / 4;
        let isrbit = self.channel.num() % 4;

        self.channel.regs().ifcr(isrn).write(|w| {
            w.set_tcif(isrbit, true);
            w.set_teif(isrbit, true);
        });
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().st(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().st(self.channel.num());
        ch.cr().read().en()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let ch = self.channel.regs().st(self.channel.num());
        ch.ndtr().read().ndt()
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

// ==================================

pub struct DoubleBuffered<'a, C: Channel, W: Word> {
    channel: PeripheralRef<'a, C>,
    _phantom: PhantomData<W>,
}

impl<'a, C: Channel, W: Word> DoubleBuffered<'a, C, W> {
    pub unsafe fn new_read(
        channel: impl Peripheral<P = C> + 'a,
        _request: Request,
        peri_addr: *mut W,
        buf0: *mut W,
        buf1: *mut W,
        len: usize,
        options: TransferOptions,
    ) -> Self {
        into_ref!(channel);
        assert!(len > 0 && len <= 0xFFFF);

        let dir = Dir::PeripheralToMemory;
        let data_size = W::size();

        let channel_number = channel.num();
        let dma = channel.regs();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let mut this = Self {
            channel,
            _phantom: PhantomData,
        };
        this.clear_irqs();

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        let ch = dma.st(channel_number);
        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(buf0 as u32);
        ch.m1ar().write_value(buf1 as u32);
        ch.ndtr().write_value(regs::Ndtr(len as _));
        ch.fcr().write(|w| {
            if let Some(fth) = options.fifo_threshold {
                // FIFO mode
                w.set_dmdis(vals::Dmdis::DISABLED);
                w.set_fth(fth.into());
            } else {
                // Direct mode
                w.set_dmdis(vals::Dmdis::ENABLED);
            }
        });
        ch.cr().write(|w| {
            w.set_dir(dir.into());
            w.set_msize(data_size.into());
            w.set_psize(data_size.into());
            w.set_pl(vals::Pl::VERYHIGH);
            w.set_minc(vals::Inc::INCREMENTED);
            w.set_pinc(vals::Inc::FIXED);
            w.set_teie(true);
            w.set_tcie(true);
            #[cfg(dma_v1)]
            w.set_trbuff(true);

            #[cfg(dma_v2)]
            w.set_chsel(_request);

            w.set_pburst(options.pburst.into());
            w.set_mburst(options.mburst.into());
            w.set_pfctrl(options.flow_ctrl.into());

            w.set_en(true);
        });

        this
    }

    fn clear_irqs(&mut self) {
        let channel_number = self.channel.num();
        let dma = self.channel.regs();
        let isrn = channel_number / 4;
        let isrbit = channel_number % 4;

        dma.ifcr(isrn).write(|w| {
            w.set_htif(isrbit, true);
            w.set_tcif(isrbit, true);
            w.set_teif(isrbit, true);
        });
    }

    pub unsafe fn set_buffer0(&mut self, buffer: *mut W) {
        let ch = self.channel.regs().st(self.channel.num());
        ch.m0ar().write_value(buffer as _);
    }

    pub unsafe fn set_buffer1(&mut self, buffer: *mut W) {
        let ch = self.channel.regs().st(self.channel.num());
        ch.m1ar().write_value(buffer as _);
    }

    pub fn is_buffer0_accessible(&mut self) -> bool {
        let ch = self.channel.regs().st(self.channel.num());
        ch.cr().read().ct() == vals::Ct::MEMORY1
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        STATE.ch_wakers[self.channel.index()].register(waker);
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().st(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().st(self.channel.num());
        ch.cr().read().en()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let ch = self.channel.regs().st(self.channel.num());
        ch.ndtr().read().ndt()
    }
}

impl<'a, C: Channel, W: Word> Drop for DoubleBuffered<'a, C, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

// ==============================

struct DmaCtrlImpl<'a, C: Channel>(PeripheralRef<'a, C>);

impl<'a, C: Channel> DmaCtrl for DmaCtrlImpl<'a, C> {
    fn get_remaining_transfers(&self) -> usize {
        let ch = self.0.regs().st(self.0.num());
        ch.ndtr().read().ndt() as usize
    }

    fn get_complete_count(&self) -> usize {
        STATE.complete_count[self.0.index()].load(Ordering::Acquire)
    }

    fn reset_complete_count(&mut self) -> usize {
        STATE.complete_count[self.0.index()].swap(0, Ordering::AcqRel)
    }
}

pub struct RingBuffer<'a, C: Channel, W: Word> {
    cr: regs::Cr,
    channel: PeripheralRef<'a, C>,
    ringbuf: DmaRingBuffer<'a, W>,
}

impl<'a, C: Channel, W: Word> RingBuffer<'a, C, W> {
    pub unsafe fn new_read(
        channel: impl Peripheral<P = C> + 'a,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        options: TransferOptions,
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

        let mut w = regs::Cr(0);
        w.set_dir(dir.into());
        w.set_msize(data_size.into());
        w.set_psize(data_size.into());
        w.set_pl(vals::Pl::VERYHIGH);
        w.set_minc(vals::Inc::INCREMENTED);
        w.set_pinc(vals::Inc::FIXED);
        w.set_teie(true);
        w.set_htie(true);
        w.set_tcie(true);
        w.set_circ(vals::Circ::ENABLED);
        #[cfg(dma_v1)]
        w.set_trbuff(true);
        #[cfg(dma_v2)]
        w.set_chsel(_request);
        w.set_pburst(options.pburst.into());
        w.set_mburst(options.mburst.into());
        w.set_pfctrl(options.flow_ctrl.into());
        w.set_en(true);

        let buffer_ptr = buffer.as_mut_ptr();
        let mut this = Self {
            channel,
            cr: w,
            ringbuf: DmaRingBuffer::new(buffer),
        };
        this.clear_irqs();

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&mut *this.channel, _request);

        let ch = dma.st(channel_number);
        ch.par().write_value(peri_addr as u32);
        ch.m0ar().write_value(buffer_ptr as u32);
        ch.ndtr().write_value(regs::Ndtr(len as _));
        ch.fcr().write(|w| {
            if let Some(fth) = options.fifo_threshold {
                // FIFO mode
                w.set_dmdis(vals::Dmdis::DISABLED);
                w.set_fth(fth.into());
            } else {
                // Direct mode
                w.set_dmdis(vals::Dmdis::ENABLED);
            }
        });

        this
    }

    pub fn start(&mut self) {
        let ch = self.channel.regs().st(self.channel.num());
        ch.cr().write_value(self.cr);
    }

    pub fn clear(&mut self) {
        self.ringbuf.clear(DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Read bytes from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the bytes were read, then there will be some bytes in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the bytes remaining after the read
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), OverrunError> {
        self.ringbuf.read(DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    // The capacity of the ringbuffer
    pub fn cap(&self) -> usize {
        self.ringbuf.cap()
    }

    pub fn set_waker(&mut self, waker: &Waker) {
        STATE.ch_wakers[self.channel.index()].register(waker);
    }

    fn clear_irqs(&mut self) {
        let channel_number = self.channel.num();
        let dma = self.channel.regs();
        let isrn = channel_number / 4;
        let isrbit = channel_number % 4;

        dma.ifcr(isrn).write(|w| {
            w.set_htif(isrbit, true);
            w.set_tcif(isrbit, true);
            w.set_teif(isrbit, true);
        });
    }

    pub fn request_stop(&mut self) {
        let ch = self.channel.regs().st(self.channel.num());

        // Disable the channel. Keep the IEs enabled so the irqs still fire.
        ch.cr().write(|w| {
            w.set_teie(true);
            w.set_htie(true);
            w.set_tcie(true);
        });
    }

    pub fn is_running(&mut self) -> bool {
        let ch = self.channel.regs().st(self.channel.num());
        ch.cr().read().en()
    }
}

impl<'a, C: Channel, W: Word> Drop for RingBuffer<'a, C, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}
