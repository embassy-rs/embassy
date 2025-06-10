use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::sync::atomic::{fence, AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use super::ringbuffer::{DmaCtrl, Error, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use super::word::{Word, WordSize};
use super::{AnyChannel, Channel, Dir, Request, STATE};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac};

pub(crate) struct ChannelInfo {
    pub(crate) dma: DmaInfo,
    pub(crate) num: usize,
    #[cfg(feature = "_dual-core")]
    pub(crate) irq: pac::Interrupt,
    #[cfg(dmamux)]
    pub(crate) dmamux: super::DmamuxInfo,
}

#[derive(Clone, Copy)]
pub(crate) enum DmaInfo {
    #[cfg(dma)]
    Dma(pac::dma::Dma),
    #[cfg(bdma)]
    Bdma(pac::bdma::Dma),
}

/// DMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {
    /// Peripheral burst transfer configuration
    #[cfg(dma)]
    pub pburst: Burst,
    /// Memory burst transfer configuration
    #[cfg(dma)]
    pub mburst: Burst,
    /// Flow control configuration
    #[cfg(dma)]
    pub flow_ctrl: FlowControl,
    /// FIFO threshold for DMA FIFO mode. If none, direct mode is used.
    #[cfg(dma)]
    pub fifo_threshold: Option<FifoThreshold>,
    /// Request priority level
    pub priority: Priority,
    /// Enable circular DMA
    ///
    /// Note:
    /// If you enable circular mode manually, you may want to build and `.await` the `Transfer` in a separate task.
    /// Since DMA in circular mode need manually stop, `.await` in current task would block the task forever.
    pub circular: bool,
    /// Enable half transfer interrupt
    pub half_transfer_ir: bool,
    /// Enable transfer complete interrupt
    pub complete_transfer_ir: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            #[cfg(dma)]
            pburst: Burst::Single,
            #[cfg(dma)]
            mburst: Burst::Single,
            #[cfg(dma)]
            flow_ctrl: FlowControl::Dma,
            #[cfg(dma)]
            fifo_threshold: None,
            priority: Priority::VeryHigh,
            circular: false,
            half_transfer_ir: false,
            complete_transfer_ir: true,
        }
    }
}

/// DMA request priority
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    /// Low Priority
    Low,
    /// Medium Priority
    Medium,
    /// High Priority
    High,
    /// Very High Priority
    VeryHigh,
}

#[cfg(dma)]
impl From<Priority> for pac::dma::vals::Pl {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => pac::dma::vals::Pl::LOW,
            Priority::Medium => pac::dma::vals::Pl::MEDIUM,
            Priority::High => pac::dma::vals::Pl::HIGH,
            Priority::VeryHigh => pac::dma::vals::Pl::VERY_HIGH,
        }
    }
}

#[cfg(bdma)]
impl From<Priority> for pac::bdma::vals::Pl {
    fn from(value: Priority) -> Self {
        match value {
            Priority::Low => pac::bdma::vals::Pl::LOW,
            Priority::Medium => pac::bdma::vals::Pl::MEDIUM,
            Priority::High => pac::bdma::vals::Pl::HIGH,
            Priority::VeryHigh => pac::bdma::vals::Pl::VERY_HIGH,
        }
    }
}

#[cfg(dma)]
pub use dma_only::*;
#[cfg(dma)]
mod dma_only {
    use pac::dma::vals;

    use super::*;

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
                Dir::MemoryToPeripheral => Self::MEMORY_TO_PERIPHERAL,
                Dir::PeripheralToMemory => Self::PERIPHERAL_TO_MEMORY,
            }
        }
    }

    /// DMA transfer burst setting.
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

    /// DMA flow control setting.
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

    /// DMA FIFO threshold.
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
                FifoThreshold::ThreeQuarters => vals::Fth::THREE_QUARTERS,
                FifoThreshold::Full => vals::Fth::FULL,
            }
        }
    }
}

#[cfg(bdma)]
mod bdma_only {
    use pac::bdma::vals;

    use super::*;

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
                Dir::MemoryToPeripheral => Self::FROM_MEMORY,
                Dir::PeripheralToMemory => Self::FROM_PERIPHERAL,
            }
        }
    }
}

pub(crate) struct ChannelState {
    waker: AtomicWaker,
    complete_count: AtomicUsize,
}

impl ChannelState {
    pub(crate) const NEW: Self = Self {
        waker: AtomicWaker::new(),
        complete_count: AtomicUsize::new(0),
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(
    cs: critical_section::CriticalSection,
    #[cfg(dma)] dma_priority: interrupt::Priority,
    #[cfg(bdma)] bdma_priority: interrupt::Priority,
) {
    foreach_interrupt! {
        ($peri:ident, dma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, dma_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
        ($peri:ident, bdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, bdma_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_dma();
    crate::_generated::init_bdma();
}

impl AnyChannel {
    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub(crate) unsafe fn on_irq(&self) {
        let info = self.info();
        let state = &STATE[self.id as usize];
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let cr = r.st(info.num).cr();
                let isr = r.isr(info.num / 4).read();

                if isr.teif(info.num % 4) {
                    panic!("DMA: error on DMA@{:08x} channel {}", r.as_ptr() as u32, info.num);
                }

                if isr.htif(info.num % 4) && cr.read().htie() {
                    // Acknowledge half transfer complete interrupt
                    r.ifcr(info.num / 4).write(|w| w.set_htif(info.num % 4, true));
                } else if isr.tcif(info.num % 4) && cr.read().tcie() {
                    // Acknowledge  transfer complete interrupt
                    r.ifcr(info.num / 4).write(|w| w.set_tcif(info.num % 4, true));
                    state.complete_count.fetch_add(1, Ordering::Release);
                } else {
                    return;
                }
                state.waker.wake();
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                let isr = r.isr().read();
                let cr = r.ch(info.num).cr();

                if isr.teif(info.num) {
                    panic!("DMA: error on BDMA@{:08x} channel {}", r.as_ptr() as u32, info.num);
                }

                if isr.htif(info.num) && cr.read().htie() {
                    // Acknowledge half transfer complete interrupt
                    r.ifcr().write(|w| w.set_htif(info.num, true));
                } else if isr.tcif(info.num) && cr.read().tcie() {
                    // Acknowledge transfer complete interrupt
                    r.ifcr().write(|w| w.set_tcif(info.num, true));
                    #[cfg(not(armv6m))]
                    state.complete_count.fetch_add(1, Ordering::Release);
                    #[cfg(armv6m)]
                    critical_section::with(|_| {
                        let x = state.complete_count.load(Ordering::Relaxed);
                        state.complete_count.store(x + 1, Ordering::Release);
                    })
                } else {
                    return;
                }

                state.waker.wake();
            }
        }
    }

    unsafe fn configure(
        &self,
        _request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        mem_size: WordSize,
        peripheral_size: WordSize,
        options: TransferOptions,
    ) {
        let info = self.info();
        #[cfg(feature = "_dual-core")]
        {
            use embassy_hal_internal::interrupt::InterruptExt as _;
            info.irq.enable();
        }

        #[cfg(dmamux)]
        super::dmamux::configure_dmamux(&info.dmamux, _request);

        assert!(mem_len > 0 && mem_len <= 0xFFFF);

        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.st(info.num);

                // "Preceding reads and writes cannot be moved past subsequent writes."
                fence(Ordering::SeqCst);

                state.complete_count.store(0, Ordering::Release);
                self.clear_irqs();

                ch.par().write_value(peri_addr as u32);
                ch.m0ar().write_value(mem_addr as u32);
                ch.ndtr().write_value(pac::dma::regs::Ndtr(mem_len as _));
                ch.fcr().write(|w| {
                    if let Some(fth) = options.fifo_threshold {
                        // FIFO mode
                        w.set_dmdis(pac::dma::vals::Dmdis::DISABLED);
                        w.set_fth(fth.into());
                    } else {
                        // Direct mode
                        w.set_dmdis(pac::dma::vals::Dmdis::ENABLED);
                    }
                });
                ch.cr().write(|w| {
                    w.set_dir(dir.into());
                    w.set_msize(mem_size.into());
                    w.set_psize(peripheral_size.into());
                    w.set_pl(options.priority.into());
                    w.set_minc(incr_mem);
                    w.set_pinc(false);
                    w.set_teie(true);
                    w.set_htie(options.half_transfer_ir);
                    w.set_tcie(options.complete_transfer_ir);
                    w.set_circ(options.circular);
                    #[cfg(dma_v1)]
                    w.set_trbuff(true);
                    #[cfg(dma_v2)]
                    w.set_chsel(_request);
                    w.set_pburst(options.pburst.into());
                    w.set_mburst(options.mburst.into());
                    w.set_pfctrl(options.flow_ctrl.into());
                    w.set_en(false); // don't start yet
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                #[cfg(bdma_v2)]
                critical_section::with(|_| r.cselr().modify(|w| w.set_cs(info.num, _request)));

                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.ch(info.num);

                state.complete_count.store(0, Ordering::Release);
                self.clear_irqs();

                ch.par().write_value(peri_addr as u32);
                ch.mar().write_value(mem_addr as u32);
                ch.ndtr().write(|w| w.set_ndt(mem_len as u16));
                ch.cr().write(|w| {
                    w.set_psize(peripheral_size.into());
                    w.set_msize(mem_size.into());
                    w.set_minc(incr_mem);
                    w.set_dir(dir.into());
                    w.set_teie(true);
                    w.set_tcie(options.complete_transfer_ir);
                    w.set_htie(options.half_transfer_ir);
                    w.set_circ(options.circular);
                    w.set_pl(options.priority.into());
                    w.set_en(false); // don't start yet
                });
            }
        }
    }

    fn start(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let ch = r.st(info.num);
                ch.cr().modify(|w| w.set_en(true))
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                let ch = r.ch(info.num);
                ch.cr().modify(|w| w.set_en(true));
            }
        }
    }

    fn clear_irqs(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                let isrn = info.num / 4;
                let isrbit = info.num % 4;

                r.ifcr(isrn).write(|w| {
                    w.set_htif(isrbit, true);
                    w.set_tcif(isrbit, true);
                    w.set_teif(isrbit, true);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                r.ifcr().write(|w| {
                    w.set_htif(info.num, true);
                    w.set_tcif(info.num, true);
                    w.set_teif(info.num, true);
                });
            }
        }
    }

    fn request_stop(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                // Disable the channel. Keep the IEs enabled so the irqs still fire.
                r.st(info.num).cr().write(|w| {
                    w.set_teie(true);
                    w.set_tcie(true);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                // Disable the channel. Keep the IEs enabled so the irqs still fire.
                r.ch(info.num).cr().write(|w| {
                    w.set_teie(true);
                    w.set_tcie(true);
                });
            }
        }
    }

    fn request_pause(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => {
                // Disable the channel without overwriting the existing configuration
                r.st(info.num).cr().modify(|w| {
                    w.set_en(false);
                });
            }
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                // Disable the channel without overwriting the existing configuration
                r.ch(info.num).cr().modify(|w| {
                    w.set_en(false);
                });
            }
        }
    }

    fn is_running(&self) -> bool {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => r.st(info.num).cr().read().en(),
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => {
                let state: &ChannelState = &STATE[self.id as usize];
                let ch = r.ch(info.num);
                let en = ch.cr().read().en();
                let circular = ch.cr().read().circ();
                let tcif = state.complete_count.load(Ordering::Acquire) != 0;
                en && (circular || !tcif)
            }
        }
    }

    fn get_remaining_transfers(&self) -> u16 {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(r) => r.st(info.num).ndtr().read().ndt(),
            #[cfg(bdma)]
            DmaInfo::Bdma(r) => r.ch(info.num).ndtr().read().ndt(),
        }
    }

    fn disable_circular_mode(&self) {
        let info = self.info();
        match self.info().dma {
            #[cfg(dma)]
            DmaInfo::Dma(regs) => regs.st(info.num).cr().modify(|w| {
                w.set_circ(false);
            }),
            #[cfg(bdma)]
            DmaInfo::Bdma(regs) => regs.ch(info.num).cr().modify(|w| {
                w.set_circ(false);
            }),
        }
    }

    fn poll_stop(&self) -> Poll<()> {
        use core::sync::atomic::compiler_fence;
        compiler_fence(Ordering::SeqCst);

        if !self.is_running() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// DMA transfer.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: Peri<'a, AnyChannel>,
}

impl<'a> Transfer<'a> {
    /// Create a new read DMA transfer (peripheral to memory).
    pub unsafe fn new_read<W: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Self {
        Self::new_read_raw(channel, request, peri_addr, buf, options)
    }

    /// Create a new read DMA transfer (peripheral to memory), using raw pointers.
    pub unsafe fn new_read_raw<W: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel.into(),
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut W as *mut u32,
            buf.len(),
            true,
            W::size(),
            W::size(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to peripheral).
    pub unsafe fn new_write<MW: Word, PW: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        buf: &'a [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Self {
        Self::new_write_raw(channel, request, buf, peri_addr, options)
    }

    /// Create a new write DMA transfer (memory to peripheral), using raw pointers.
    pub unsafe fn new_write_raw<MW: Word, PW: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        buf: *const [MW],
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel.into(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            buf as *const MW as *mut u32,
            buf.len(),
            true,
            MW::size(),
            PW::size(),
            options,
        )
    }

    /// Create a new write DMA transfer (memory to peripheral), writing the same value repeatedly.
    pub unsafe fn new_write_repeated<W: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel.into(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            repeated as *const W as *mut u32,
            count,
            false,
            W::size(),
            W::size(),
            options,
        )
    }

    unsafe fn new_inner(
        channel: Peri<'a, AnyChannel>,
        _request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        peripheral_size: WordSize,
        options: TransferOptions,
    ) -> Self {
        assert!(mem_len > 0 && mem_len <= 0xFFFF);

        channel.configure(
            _request,
            dir,
            peri_addr,
            mem_addr,
            mem_len,
            incr_mem,
            data_size,
            peripheral_size,
            options,
        );
        channel.start();
        Self { channel }
    }

    /// Request the transfer to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        self.channel.request_stop()
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        self.channel.get_remaining_transfers()
    }

    /// Blocking wait until the transfer finishes.
    pub fn blocking_wait(mut self) {
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);

        core::mem::forget(self);
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

impl<'a> Unpin for Transfer<'a> {}
impl<'a> Future for Transfer<'a> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state: &ChannelState = &STATE[self.channel.id as usize];

        state.waker.register(cx.waker());

        if self.is_running() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

// ==============================

struct DmaCtrlImpl<'a>(Peri<'a, AnyChannel>);

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        self.0.get_remaining_transfers() as _
    }

    fn reset_complete_count(&mut self) -> usize {
        let state = &STATE[self.0.id as usize];
        #[cfg(not(armv6m))]
        return state.complete_count.swap(0, Ordering::AcqRel);
        #[cfg(armv6m)]
        return critical_section::with(|_| {
            let x = state.complete_count.load(Ordering::Acquire);
            state.complete_count.store(0, Ordering::Release);
            x
        });
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.0.id as usize].waker.register(waker);
    }
}

/// Ringbuffer for receiving data using DMA circular mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let channel: Peri<'a, AnyChannel> = channel.into();

        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len();
        let dir = Dir::PeripheralToMemory;
        let data_size = W::size();

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            true,
            data_size,
            data_size,
            options,
        );

        Self {
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.ringbuf.read(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, Error> {
        self.ringbuf
            .read_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the DMA to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        self.channel.request_stop()
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is full.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when the buffer is full.
    ///
    /// This is designed to be used with streaming input data such as the
    /// I2S/SAI or ADC.
    ///
    /// When using the UART, you probably want `request_stop()`.
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for ReadableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

/// Ringbuffer for writing data using DMA circular mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        _request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let channel: Peri<'a, AnyChannel> = channel.into();

        let len = buffer.len();
        let dir = Dir::MemoryToPeripheral;
        let data_size = W::size();
        let buffer_ptr = buffer.as_mut_ptr();

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            true,
            data_size,
            data_size,
            options,
        );

        Self {
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Write elements directly to the raw buffer.
    /// This can be used to fill the buffer before starting the DMA transfer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write_immediate(buf)
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        self.ringbuf
            .write_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ringbuf
            .wait_write_error(&mut DmaCtrlImpl(self.channel.reborrow()))
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the DMA to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        self.channel.request_stop()
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is empty.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when all available data has been written.
    ///
    /// This is designed to be used with streaming output data such as the
    /// I2S/SAI or DAC.
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for WritableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}
