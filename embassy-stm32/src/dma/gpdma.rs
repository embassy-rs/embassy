#![macro_use]

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
use crate::interrupt::Priority;
use crate::pac;
use crate::pac::gpdma::vals;

pub(crate) struct ChannelInfo {
    pub(crate) dma: pac::gpdma::Gpdma,
    pub(crate) num: usize,
    #[cfg(feature = "_dual-core")]
    pub(crate) irq: pac::Interrupt,
}

/// GPDMA transfer options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TransferOptions {}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {}
    }
}

impl From<WordSize> for vals::Dw {
    fn from(raw: WordSize) -> Self {
        match raw {
            WordSize::OneByte => Self::BYTE,
            WordSize::TwoBytes => Self::HALF_WORD,
            WordSize::FourBytes => Self::WORD,
        }
    }
}

pub(crate) struct ChannelState {
    waker: AtomicWaker,
    circular_address: AtomicUsize,
    complete_count: AtomicUsize,
}

impl ChannelState {
    pub(crate) const NEW: Self = Self {
        waker: AtomicWaker::new(),
        circular_address: AtomicUsize::new(0),
        complete_count: AtomicUsize::new(0),
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init(cs: critical_section::CriticalSection, irq_priority: Priority) {
    foreach_interrupt! {
        ($peri:ident, gpdma, $block:ident, $signal_name:ident, $irq:ident) => {
            crate::interrupt::typelevel::$irq::set_priority_with_cs(cs, irq_priority);
            #[cfg(not(feature = "_dual-core"))]
            crate::interrupt::typelevel::$irq::enable();
        };
    }
    crate::_generated::init_gpdma();
}

impl AnyChannel {
    /// Safety: Must be called with a matching set of parameters for a valid dma channel
    pub(crate) unsafe fn on_irq(&self) {
        let info = self.info();
        #[cfg(feature = "_dual-core")]
        {
            use embassy_hal_internal::interrupt::InterruptExt as _;
            info.irq.enable();
        }

        let state = &STATE[self.id as usize];

        let ch = info.dma.ch(info.num);
        let sr = ch.sr().read();

        if sr.dtef() {
            panic!(
                "DMA: data transfer error on DMA@{:08x} channel {}",
                info.dma.as_ptr() as u32,
                info.num
            );
        }
        if sr.usef() {
            panic!(
                "DMA: user settings error on DMA@{:08x} channel {}",
                info.dma.as_ptr() as u32,
                info.num
            );
        }

        if sr.htf() {
            //clear the flag for the half transfer complete
            ch.fcr().modify(|w| w.set_htf(true));
            state.waker.wake();
        }

        if sr.tcf() {
            //clear the flag for the transfer complete
            ch.fcr().modify(|w| w.set_tcf(true));
            state.complete_count.fetch_add(1, Ordering::Relaxed);
            state.waker.wake();
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
            state.waker.wake();
        }
    }

    fn clear_irqs(&self) {
        let info = self.info();
        let ch = info.dma.ch(info.num);
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF);
    }

    unsafe fn configure(
        &self,
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        mem_size: WordSize,
        dst_size: WordSize,
        _options: super::TransferOptions,
    ) {
        let bytes = mem_len.checked_mul(mem_size.bytes()).expect("BNDT overflow");
        let Ok(bndt) = u16::try_from(bytes) else {
            panic!("DMA transfers may not be larger than 65535 bytes.");
        };

        let info = self.info();
        let ch = info.dma.ch(info.num);

        fence(core::sync::atomic::Ordering::SeqCst);

        ch.cr().write(|w| w.set_reset(true));
        self.clear_irqs();
        ch.llr().write(|_| {}); // no linked list by default

        ch.tr1().write(|w| {
            w.set_sdw(mem_size.into());
            w.set_ddw(dst_size.into());
            w.set_sinc(dir == Dir::MemoryToPeripheral && incr_mem);
            w.set_dinc(dir == Dir::PeripheralToMemory && incr_mem);
        });
        ch.tr2().write(|w| {
            w.set_dreq(match dir {
                Dir::MemoryToPeripheral => vals::Dreq::DESTINATION_PERIPHERAL,
                Dir::PeripheralToMemory => vals::Dreq::SOURCE_PERIPHERAL,
            });
            w.set_reqsel(request);
        });
        ch.tr3().write(|_| {});

        ch.br1().write(|w| w.set_bndt(bndt));

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

    fn start(&self) {
        let info = self.info();
        let ch = info.dma.ch(info.num);
        ch.cr().write(|w| {
            w.set_tcie(true);
            w.set_useie(true);
            w.set_dteie(true);
            w.set_suspie(true);
            w.set_htie(true);
            w.set_en(true);
        });
    }

    fn request_stop(&self) {
        let info = self.info();
        info.dma.ch(info.num).cr().modify(|w| w.set_susp(true));
    }

    fn request_pause(&self) {
        // No distinct pause on GPDMA v1; suspend is the best approximation.
        self.request_stop();
    }

    fn is_running(&self) -> bool {
        let info = self.info();
        let ch = info.dma.ch(info.num);
        let sr = ch.sr().read();
        !sr.tcf() && !sr.suspf()
    }

    #[allow(unused)]
    fn get_remaining_transfers(&self) -> u16 {
        let info = self.info();
        info.dma.ch(info.num).br1().read().bndt()
    }

    fn disable_circular_mode(&self) {
        let info = self.info();
        let ch = info.dma.ch(info.num);
        ch.llr().write(|_| {});
    }

    fn poll_stop(&self) -> core::task::Poll<()> {
        use core::sync::atomic::compiler_fence;
        compiler_fence(core::sync::atomic::Ordering::SeqCst);
        if !self.is_running() {
            core::task::Poll::Ready(())
        } else {
            core::task::Poll::Pending
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
    pub unsafe fn new_read_raw<MW: Word, PW: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut PW,
        buf: *mut [MW],
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel.into(),
            request,
            Dir::PeripheralToMemory,
            peri_addr as *const u32,
            buf as *mut MW as *mut u32,
            buf.len(),
            true,
            PW::size(),
            MW::size(),
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
    pub unsafe fn new_write_repeated<MW: Word, PW: Word>(
        channel: Peri<'a, impl Channel>,
        request: Request,
        repeated: &'a MW,
        count: usize,
        peri_addr: *mut PW,
        options: TransferOptions,
    ) -> Self {
        Self::new_inner(
            channel.into(),
            request,
            Dir::MemoryToPeripheral,
            peri_addr as *const u32,
            repeated as *const MW as *mut u32,
            count,
            false,
            MW::size(),
            PW::size(),
            options,
        )
    }

    unsafe fn new_inner(
        channel: Peri<'a, AnyChannel>,
        request: Request,
        dir: Dir,
        peri_addr: *const u32,
        mem_addr: *mut u32,
        mem_len: usize,
        incr_mem: bool,
        data_size: WordSize,
        dst_size: WordSize,
        _options: TransferOptions,
    ) -> Self {
        // BNDT is specified as bytes, not as number of transfers.
        let Ok(bndt) = (mem_len * data_size.bytes()).try_into() else {
            panic!("DMA transfers may not be larger than 65535 bytes.");
        };

        let info = channel.info();
        let ch = info.dma.ch(info.num);

        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        let this = Self { channel };

        ch.cr().write(|w| w.set_reset(true));
        ch.fcr().write(|w| w.0 = 0xFFFF_FFFF); // clear all irqs
        ch.llr().write(|_| {}); // no linked list
        ch.tr1().write(|w| {
            w.set_sdw(data_size.into());
            w.set_ddw(dst_size.into());
            w.set_sinc(dir == Dir::MemoryToPeripheral && incr_mem);
            w.set_dinc(dir == Dir::PeripheralToMemory && incr_mem);
        });
        ch.tr2().write(|w| {
            w.set_dreq(match dir {
                Dir::MemoryToPeripheral => vals::Dreq::DESTINATION_PERIPHERAL,
                Dir::PeripheralToMemory => vals::Dreq::SOURCE_PERIPHERAL,
            });
            w.set_reqsel(request);
        });
        ch.tr3().write(|_| {}); // no address offsets.
        ch.br1().write(|w| w.set_bndt(bndt));

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

    /// Request the transfer to stop.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        ch.cr().modify(|w| w.set_susp(true))
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        let sr = ch.sr().read();
        !sr.tcf() && !sr.suspf()
    }

    /// Gets the total remaining transfers for the channel
    /// Note: this will be zero for transfers that completed without cancellation.
    pub fn get_remaining_transfers(&self) -> u16 {
        let info = self.channel.info();
        let ch = info.dma.ch(info.num);

        ch.br1().read().bndt()
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
        let state = &STATE[self.channel.id as usize];
        state.waker.register(cx.waker());

        if self.is_running() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

// DMA controller implementation for ring buffer operations
pub(crate) struct DmaCtrlImpl<'a>(&'a Peri<'a, AnyChannel>, WordSize);

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        let ch = self.0.info().dma.ch(self.0.info().num);
        let bndt = ch.br1().read().bndt();
        (bndt / self.1.bytes() as u16) as usize
    }

    fn reset_complete_count(&mut self) -> usize {
        STATE[self.0.id as usize].complete_count.swap(0, Ordering::AcqRel)
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.0.id as usize].waker.register(waker);
    }
}

/// Ring buffer for receiving data using GPDMA circular linked-list mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a GPDMA-backed circular ring buffer for **peripheral-to-memory** transfers.
    ///
    /// The function programs the GPDMA v1 linked‑list pointer (LLI) to the start of `buffer`,
    /// configures the channel for circular operation, and prepares it to generate half/complete
    /// transfer interrupts. Call [`start`](Self::start) to actually begin DMA traffic.
    ///
    /// # Safety
    /// * `peri_addr` **must** point to the peripheral data register that matches `request`.
    /// * `buffer` must be valid for writes for the entire lifetime of the ring buffer and be
    ///   at least 4‑byte aligned (the hardware LLI uses a word‑aligned address).
    /// * The provided `channel` must be a valid GPDMA channel connected to the given `request`.
    /// * Only one DMA transfer/ring buffer may be active on a channel at a time.
    ///
    /// # Notes
    /// This constructor only configures the hardware; it does not start the channel.
    /// Use [`start`](Self::start) to enable the channel after constructing the ring buffer.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        options: super::TransferOptions,
    ) -> Self {
        let channel = channel.into();
        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len();

        let addr = buffer_ptr as usize;
        assert!(addr & 0b11 == 0, "circular address must be 4-byte aligned");
        STATE[channel.id as usize]
            .circular_address
            .store(addr, Ordering::Relaxed);

        let lli = addr as u32;
        let ch = channel.info().dma.ch(channel.info().num);
        ch.llr().write(|w| {
            w.set_usa(true);
            w.set_la(((lli >> 2) & 0x3fff).try_into().unwrap());
        });
        ch.lbar().write(|w| {
            w.set_lba((lli >> 16).try_into().unwrap());
        });

        channel.configure(
            request,
            Dir::PeripheralToMemory,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            true,
            W::size(),
            W::size(),
            options,
        );

        Self {
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Enable and start the configured DMA channel.
    ///
    /// Must be called once after [`new`](Self::new) to begin the circular transfer.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Clear all data tracked by the software ring buffer.
    ///
    /// This resets only the software indices; it does **not** modify the DMA controller state.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(&self.channel, W::size()));
    }

    /// Read elements from the ring buffer into `buf`.
    ///
    /// Returns a tuple `(read, remaining)` where:
    /// * `read` is the number of elements copied into `buf`;
    /// * `remaining` is the number of elements still available to read immediately.
    ///
    /// Errors if the requested region was overwritten by the DMA engine (overrun).
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.ringbuf.read(&mut DmaCtrlImpl(&self.channel, W::size()), buf)
    }

    /// Asynchronously read **exactly** `buffer.len()` elements.
    ///
    /// Returns the number of additional elements available for immediate reading afterwards.
    /// If the DMA source does not advance in multiples of `buffer.len()`, this call may await
    /// until the next half/complete transfer event (N/2 cadence).
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, Error> {
        self.ringbuf
            .read_exact(&mut DmaCtrlImpl(&self.channel, W::size()), buffer)
            .await
    }

    /// Return the current number of elements available to read.
    ///
    /// Errors if the unread region was overwritten by the DMA engine.
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(&self.channel, W::size()))?)
    }

    /// Total capacity (in elements) of the underlying ring buffer.
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Register a waker to be notified when new data arrives.
    ///
    /// Wakes on half‑transfer and transfer‑complete events.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(&self.channel, W::size()).set_waker(waker);
    }

    /// Request the DMA to stop as soon as possible.
    ///
    /// This does not instantly stop the channel; poll [`is_running`](Self::is_running)
    /// or await [`stop`](Self::stop) to observe completion.
    pub fn request_stop(&mut self) {
        self.channel.request_stop();
    }

    /// Request a pause while keeping the existing configuration.
    ///
    /// Call [`start`](Self::start) to resume.
    pub fn request_pause(&mut self) {
        self.channel.request_pause();
    }

    /// Return whether the DMA transfer is still active.
    ///
    /// `false` indicates either normal completion or a prior stop/pause request has taken effect.
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the **buffer is full**.
    ///
    /// Disables circular mode so the controller finishes the current lap and halts when the
    /// write pointer reaches the end of the ring. Intended for streaming inputs (e.g. ADC/SAI).
    /// For UART‑style framed input, prefer [`request_stop`](Self::request_stop).
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

/// Ring buffer for writing data using GPDMA circular linked-list mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        options: super::TransferOptions,
    ) -> Self {
        let channel = channel.into();
        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len();

        let addr = buffer_ptr as usize;
        assert!(addr & 0b11 == 0, "circular address must be 4-byte aligned");
        STATE[channel.id as usize]
            .circular_address
            .store(addr, Ordering::Relaxed);

        let lli = addr as u32;
        let ch = channel.info().dma.ch(channel.info().num);
        ch.llr().write(|w| {
            w.set_usa(true);
            w.set_la(((lli >> 2) & 0x3fff).try_into().unwrap());
        });
        ch.lbar().write(|w| {
            w.set_lba((lli >> 16).try_into().unwrap());
        });

        channel.configure(
            request,
            Dir::MemoryToPeripheral,
            buffer_ptr as *mut u32,
            peri_addr as *mut u32,
            len,
            true,
            W::size(),
            W::size(),
            options,
        );

        Self {
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation. Must be called to begin transfers.
    pub fn start(&mut self) {
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(&self.channel, W::size()));
    }

    /// Write data into the ring buffer immediately.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write_immediate(buf)
    }

    /// Write data into the ring buffer.
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write(&mut DmaCtrlImpl(&self.channel, W::size()), buf)
    }

    /// Write an exact amount of data asynchronously.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        self.ringbuf
            .write_exact(&mut DmaCtrlImpl(&self.channel, W::size()), buffer)
            .await
    }

    /// Wait for any write error (like overrun), returning the failure count.
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ringbuf
            .wait_write_error(&mut DmaCtrlImpl(&self.channel, W::size()))
            .await
    }

    /// Get current length of buffered data.
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(&self.channel, W::size()))?)
    }

    /// Get total ring buffer capacity.
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Assign a waker to be notified when buffer can accept more data.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(&self.channel, W::size()).set_waker(waker);
    }

    /// Request the DMA transfer to stop (graceful).
    pub fn request_stop(&mut self) {
        self.channel.request_stop();
    }

    /// Request a pause in the transfer (configuration preserved).
    pub fn request_pause(&mut self) {
        self.channel.request_pause();
    }

    /// Check if the transfer is still running.
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer asynchronously once the buffer empties.
    pub async fn stop(&mut self) {
        self.channel.disable_circular_mode();
        poll_fn(|cx| {
            DmaCtrlImpl(&self.channel, W::size()).set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for WritableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}
        fence(Ordering::SeqCst);
    }
}
