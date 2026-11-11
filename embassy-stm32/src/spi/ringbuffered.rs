//! Ring-buffered SPI reception: a continuous stream, or one frame per hardware trigger.
//!
//! Both modes receive words into a ring of DMA buffers with no CPU time per word or per frame.
//! The consumer reads words out with [`RingBufferedSpiRx::read`] (copied), or one whole frame
//! at a time in place with [`RingBufferedSpiRx::next_frame`] (no copy).
//!
//! - Slave, continuous: created with [`Spi::into_ring_buffered`]. The master's clock drives
//!   reception; whatever arrives lands in the ring.
//! - Master, triggered: created with [`Spi::into_triggered_ring_buffered`]. Each trigger edge
//!   (an EXTI event, a timer, ...) starts one fixed-size frame, with an NSS pulse if the [`Spi`]
//!   drives NSS. A third DMA channel, gated by the trigger, restarts the SPI without the CPU.
//!
//! ### Notes on 'waiting for bytes'
//!
//! The `read(buf)` (but not `read()`) function
//! may need to wait for bytes to arrive, if the ring buffer does not
//! contain enough bytes to fill the buffer passed by the caller of
//! the function, or is empty.
//!
//! Waiting for bytes operates in one of two modes, depending on
//! the behavior of the sender, the size of the buffer passed to the
//! function:
//!
//! - If the slave is deselected (either rising or falling edge on NSS pin
//! depending on SPI configuration), then any bytes in the ring buffer
//! will be returned. If there are no bytes in the buffer, the check will
//! be repeated each time the NSS deselect edge is detected, so if the
//! sender sends just a single byte, it will be returned once the NSS
//! deselect edge is detected.
//!
//! - If the sender sends continuously, the call will wait until
//! the DMA controller indicates that it has written to either the
//! middle byte or last byte of the ring buffer ('half transfer'
//! or 'transfer complete', respectively). This does not indicate
//! the buffer is half-full or full, though, because the DMA
//! controller does not detect those conditions; it sends an
//! interrupt when those specific buffer addresses have been written.

#[cfg(any(spi_v4, spi_v5, spi_v6))]
use core::cell::Cell;
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
use core::cell::UnsafeCell;
use core::future::{Future, poll_fn};
use core::mem;
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
use core::mem::MaybeUninit;
#[cfg(feature = "exti")]
use core::pin::pin;
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
use core::sync::atomic::fence;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
#[cfg(feature = "exti")]
use embassy_futures::select::select;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use embassy_sync::blocking_mutex::Mutex;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_io_async::ReadReady;

#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
use crate::dma::{
    Channel, ItemConfig, LinearItem, LinkedListItem, LinkedListTransfer, Priority, RunMode, Table, TransferOptions,
    TriggerConfig,
};
use crate::dma::{ReadableRingBuffer, RingBufferError};
use crate::gpio::Flex;
use crate::interrupt;
use crate::interrupt::InterruptExt;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::Async;
use crate::rcc::WakeGuard;
#[cfg(feature = "exti")]
use crate::spi::SlaveSelectPolarity;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use crate::spi::flush_rx_fifo;
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
use crate::spi::mode::Master;
use crate::spi::mode::Slave;
use crate::spi::{
    Config, ConfigError, CsPinType, Error, Info, Regs, RegsExt, Spi, Word, check_error_flags, reconfigure, set_rxdmaen,
};
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use crate::spi::{Instance, regs};
use crate::time::Hertz;

/// Per-instance state of the receiver: what the interrupt handler shares with the consumer.
pub(crate) struct State {
    crc_errors: AtomicU32,
    overruns: AtomicU32,
    /// Where and how to record a bad frame, while the marks are enabled.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    session: Mutex<CriticalSectionRawMutex, Cell<Option<Session>>>,
    /// DMA descriptor storage for the triggered mode's transmit and re-arm channels.
    #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
    tables: Tables,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            crc_errors: AtomicU32::new(0),
            overruns: AtomicU32::new(0),
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            session: Mutex::new(Cell::new(None)),
            #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
            tables: Tables::new(),
        }
    }
}

/// What the interrupt handler needs to mark a bad frame: plain addresses, so it is `Send`.
#[cfg(any(spi_v4, spi_v5, spi_v6))]
#[derive(Clone, Copy)]
struct Session {
    /// Snapshots the ring's write head, in words.
    head: fn(usize) -> usize,
    /// The ring buffer `head` snapshots.
    ring_buf: usize,
    frame_len: usize,
    frames: usize,
    /// The marks: `&[AtomicU32]`, one bit per frame in the ring.
    bad: usize,
}

#[cfg(any(spi_v4, spi_v5, spi_v6))]
impl Session {
    /// CRCE comes after the CRC, when the receive DMA has usually moved on to the next frame.
    /// If it is still inside a frame, that frame is the bad one.
    fn mark_bad_frame(&self) {
        let pos = (self.head)(self.ring_buf);
        let slot = pos / self.frame_len;
        let slot = if pos % self.frame_len == 0 {
            (slot + self.frames - 1) % self.frames
        } else {
            slot % self.frames
        };

        let marks = unsafe { &*(self.bad as *const AtomicU32).add(slot / 32) };
        marks.fetch_or(1 << (slot % 32), Ordering::Relaxed);
    }
}

/// A [`Session::head`] for `W`.
#[cfg(any(spi_v4, spi_v5, spi_v6))]
fn write_head_shim<W: Word>(ring_buf: usize) -> usize {
    // Safety: the session is dropped before the `RingBufferedSpiRx` (its `drop` clears it), so
    // the pointer is valid whenever the handler runs.
    unsafe { &*(ring_buf as *const ReadableRingBuffer<'static, W>) }.write_head()
}

/// The most ring blocks a ring is split into in triggered mode: one linked-list item and one
/// wake-up per block.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
pub const MAX_BLOCKS: usize = 16;

/// DMA descriptor storage for the triggered mode, in the instance's [`State`] so it outlives the
/// stream. The 512-byte alignment (larger than the 496 bytes of tables) keeps every linked list
/// inside one 512-byte block, which no 64 KiB boundary can cross: `Table::offset_address` panics
/// for a list that straddles one.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
#[repr(C, align(512))]
struct Tables {
    /// One item per ring block: the receive ring.
    rx: UnsafeCell<MaybeUninit<Table<LinearItem, MAX_BLOCKS>>>,
    /// One item: `tx_frame`, reloaded each frame.
    tx: UnsafeCell<MaybeUninit<Table<LinearItem, 1>>>,
    /// Three items: re-arm the SPI once per trigger.
    rearm: UnsafeCell<MaybeUninit<Table<LinearItem, 3>>>,
    /// The words the re-arm items copy into `CR1` and `IFCR`. The DMA reads them.
    rearm_values: UnsafeCell<[u32; 4]>,
}

#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
impl Tables {
    const fn new() -> Self {
        Self {
            rx: UnsafeCell::new(MaybeUninit::uninit()),
            tx: UnsafeCell::new(MaybeUninit::uninit()),
            rearm: UnsafeCell::new(MaybeUninit::uninit()),
            rearm_values: UnsafeCell::new([0; 4]),
        }
    }
}

// Safety: the tables are only written while the SPI is owned by a single `RingBufferedSpiRx`
// (the constructor consumes the `Spi`), and then only read by the DMA.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
unsafe impl Sync for Tables {}

/// All SPI event flags. Re-enabling the SPI clears neither TXTF nor SUSP, and TXTF blocks the
/// transmit DMA request.
#[cfg(any(spi_v4, spi_v5, spi_v6))]
fn all_flags() -> regs::Ifcr {
    let mut clear = regs::Ifcr(0);
    clear.set_eotc(true);
    clear.set_txtfc(true);
    clear.set_suspc(true);
    clear.set_udrc(true);
    clear.set_ovrc(true);
    clear.set_crcec(true);
    clear.set_tifrec(true);
    clear.set_modfc(true);
    clear
}

/// An SPI instance with an interrupt of its own.
#[cfg(any(spi_v4, spi_v5, spi_v6))]
pub trait InterruptInstance: Instance {
    /// The SPI's interrupt.
    type Interrupt: Interrupt;
}

#[cfg(any(spi_v4, spi_v5, spi_v6))]
foreach_interrupt!(
    ($inst:ident, spi, $block:ident, GLOBAL, $irq:ident) => {
        impl InterruptInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);

/// SPI interrupt handler for [`RingBufferedSpiRx`]: counts CRC failures and overruns.
#[cfg(any(spi_v4, spi_v5, spi_v6))]
pub struct InterruptHandler<T: InterruptInstance> {
    _phantom: core::marker::PhantomData<T>,
}

#[cfg(any(spi_v4, spi_v5, spi_v6))]
impl<T: InterruptInstance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let state = &T::state().ring;

        let regs = T::info().regs;
        let sr = regs.sr().read();
        if sr.crce() {
            state.crc_errors.fetch_add(1, Ordering::Relaxed);
            if let Some(session) = state.session.lock(Cell::get) {
                session.mark_bad_frame();
            }
        }
        if sr.ovr() {
            state.overruns.fetch_add(1, Ordering::Relaxed);
        }

        // Clear everything, so no other flag keeps the interrupt firing.
        regs.ifcr().write_value(all_flags());
    }
}

/// The triggered mode's DMA channels and teardown bookkeeping.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
struct Triggered<'d> {
    /// `tx_frame` reloaded each frame.
    tx: LinkedListTransfer<'d>,
    /// Re-arms the SPI on each trigger, gated by the first item.
    rearm: LinkedListTransfer<'d>,
    /// Poll budget for one frame, for [`wait_frame_end`].
    frame_polls: u32,
}

/// Rx-only ring-buffered SPI driver
///
/// Created with [`Spi::into_ring_buffered`] (slave) or [`Spi::into_triggered_ring_buffered`]
/// (master, one frame per trigger).
pub struct RingBufferedSpiRx<'d, W: Word> {
    info: &'static Info,
    kernel_clock: Hertz,
    _wake_guard: WakeGuard,
    _sck: Option<Flex<'d>>,
    _mosi: Option<Flex<'d>>,
    _miso: Option<Flex<'d>>,
    /// Kept to hold the pin; read by the NSS wait (`exti`).
    #[allow(dead_code)]
    nss: CsPinType<'d>,
    ring_buf: ReadableRingBuffer<'d, W>,
    /// The ring's first word, for the frame slots of the CRC validity marks.
    #[cfg_attr(not(any(spi_v4, spi_v5, spi_v6)), allow(dead_code))]
    ring_addr: usize,
    /// Words per frame ([`set_frame_len`](Self::set_frame_len), or the triggered frame length);
    /// `None` when unset.
    frame_len: Option<usize>,
    /// The instance state behind the interrupt handler, when one is bound.
    state: Option<&'static State>,
    /// The SPI's interrupt, when its handler is enabled.
    irq: Option<interrupt::Interrupt>,
    /// Per-frame CRC validity marks, one bit per frame in the ring
    /// ([`enable_frame_marks`](Self::enable_frame_marks)).
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    marks: Option<&'d mut [AtomicU32]>,
    #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
    triggered: Option<Triggered<'d>>,
}

impl<'d, W: Word> SetConfig for RingBufferedSpiRx<'d, W> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

#[cfg(feature = "exti")]
impl<'d> Spi<'d, Async, Slave> {
    /// Turn the `Spi` into a buffered spi which can continuously receive in the background
    /// without the possibility of losing bytes. The `dma_buf` is a buffer registered to the
    /// DMA controller, and must be large enough to prevent overflows.
    pub fn into_ring_buffered<W: Word>(mut self, dma_buf: &'d mut [W]) -> RingBufferedSpiRx<'d, W> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        self.set_word_size(W::CONFIG);

        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let rx_dma = self.rx_dma.as_mut().unwrap();
        let request = rx_dma.request;
        let rx_dma = unsafe { rx_dma.channel.clone_unchecked() };

        let tx_dma = unsafe { self.tx_dma.as_ref().map(|x| x.clone_unchecked()) };

        let info = self.info;
        let kernel_clock = self.kernel_clock;
        let ring_addr = dma_buf.as_ptr() as usize;
        let ring_buf = unsafe { ReadableRingBuffer::new(rx_dma, request, info.regs.rx_ptr::<W>(), dma_buf, opts) };
        let sck = unsafe { self._sck.as_ref().map(|x| x.clone_unchecked()) };
        let mosi = unsafe { self._mosi.as_ref().map(|x| x.clone_unchecked()) };
        let miso = unsafe { self._miso.as_ref().map(|x| x.clone_unchecked()) };
        let nss = unsafe { self.nss.clone_unchecked() };

        let wake_guard = self.info.rcc.wake_guard();

        // Don't disable the clock
        mem::forget(self);

        // Drop (cleanup) peripherals that are no longer needed.
        // Miso pin is kept to maintain its state set by user.
        drop(tx_dma);

        RingBufferedSpiRx {
            info,
            kernel_clock,
            _wake_guard: wake_guard,
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            nss,
            ring_buf,
            ring_addr,
            frame_len: None,
            state: None,
            irq: None,
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            marks: None,
            #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
            triggered: None,
        }
    }
}

#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
impl<'d> Spi<'d, Async, Master> {
    /// Receive one frame per trigger edge into a ring, without CPU time per frame.
    ///
    /// Each edge of `trigger` (an EXTI event, a timer, ...) starts one frame of `tx_frame.len()`
    /// words, with an NSS pulse if this [`Spi`] drives NSS ([`Spi::new_with_nss`]). A third DMA
    /// channel, gated by the trigger, restarts the SPI; the SPI's own channels receive into
    /// `ring` and send `tx_frame` again each frame.
    ///
    /// - `_irq`: [`InterruptHandler`] bound to the SPI's interrupt.
    /// - `rearm`: a third channel of the same DMA controller, consumed.
    /// - `frames_per_block`: frames per ring block, the consumer's wake-up granularity: the ring
    ///   hands frames out faster, but a waiting [`RingBufferedSpiRx::next_frame`] is woken once
    ///   per block. Blocks are also the DMA's linked-list items: at most [`MAX_BLOCKS`] blocks,
    ///   at most 65535 bytes each.
    /// - `tx_frame`: sent each frame; its length is the frame length, at most 65535 words.
    /// - `ring`: whole blocks of frames ([`frame_len`](RingBufferedSpiRx::frame_len) words per
    ///   frame), at least two frames.
    ///
    /// The SPI's own configuration ([`Config::crc`] and friends) applies to the frames. The
    /// trigger source is set up separately (e.g. [`ExtiInput::enable_event`]). The frames are
    /// read in place with [`RingBufferedSpiRx::next_frame`], or copied out with
    /// [`RingBufferedSpiRx::read`].
    ///
    /// [`ExtiInput::enable_event`]: crate::exti::ExtiInput::enable_event
    pub fn into_triggered_ring_buffered<T: InterruptInstance, W: Word>(
        mut self,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rearm: Channel<'d>,
        trigger: TriggerConfig,
        frames_per_block: usize,
        ring: &'d mut [W],
        tx_frame: &'d [W],
    ) -> RingBufferedSpiRx<'d, W> {
        assert!(core::ptr::eq(self.info, T::info()), "the SPI is not instance T");

        let frame_len = tx_frame.len();
        assert!(
            frame_len > 0 && frame_len <= u16::MAX as usize,
            "frame length out of range"
        );
        assert!(frames_per_block > 0, "frames per block out of range");
        let block_len = frames_per_block * frame_len;
        assert!(
            !ring.is_empty() && ring.len() % block_len == 0 && ring.len() / block_len <= MAX_BLOCKS,
            "the ring must be at most MAX_BLOCKS blocks of frames_per_block frames"
        );
        assert!(
            block_len * W::size().bytes() <= 0xFFFF,
            "a block may not be larger than 65535 bytes"
        );

        let regs = self.info.regs;
        let info = self.info;
        let kernel_clock = self.kernel_clock;

        // SPI: the frame length. The `Config` already programmed the CRC.
        regs.cr1().modify(|w| w.set_spe(false));
        self.set_word_size(W::CONFIG);
        regs.ifcr().write_value(all_flags());
        regs.cr2().modify(|w| w.set_tsize(frame_len as u16));

        // The three register states the re-arm chain writes back, per frame.
        let mut base = regs.cr1().read();
        base.set_spe(false);
        base.set_cstart(false);
        let mut enabled = base;
        enabled.set_spe(true);
        let mut started = enabled;
        started.set_cstart(true);

        let cfg1 = regs.cfg1().read();
        let crc_bits = if cfg1.crcen() { u32::from(cfg1.crcsize()) + 1 } else { 0 };
        let frame_bits = frame_len as u32 * (u32::from(cfg1.dsize()) + 1) + crc_bits;
        let divider = 2u32 << cfg1.mbr().to_bits();
        // Holds for a core up to 16x faster than the SPI kernel clock.
        let frame_polls = frame_bits * divider * 16 + 1000;

        let state = &T::state().ring;
        state.crc_errors.store(0, Ordering::Relaxed);
        state.overruns.store(0, Ordering::Relaxed);

        // Volatile: only the DMA reads these.
        unsafe {
            state
                .tables
                .rearm_values
                .get()
                .write_volatile([base.0, all_flags().0, enabled.0, started.0]);
        }

        // Safety: we forget the struct before this function returns.
        let rx_dma = self.rx_dma.as_mut().expect("the SPI has no receive DMA channel");
        let tx_dma = self.tx_dma.as_mut().expect("the SPI has no transmit DMA channel");
        let (rx_request, tx_request) = (rx_dma.request, tx_dma.request);
        let rx_channel = unsafe { rx_dma.channel.clone_unchecked() };
        let tx_channel = unsafe { tx_dma.channel.clone_unchecked() };

        let sck = unsafe { self._sck.as_ref().map(|x| x.clone_unchecked()) };
        let mosi = unsafe { self._mosi.as_ref().map(|x| x.clone_unchecked()) };
        let miso = unsafe { self._miso.as_ref().map(|x| x.clone_unchecked()) };
        let nss = unsafe { self.nss.clone_unchecked() };
        let wake_guard = info.rcc.wake_guard();
        mem::forget(self);

        let ring_addr = ring.as_ptr() as usize;
        // Receive: the ring, one circular item per block (one wake-up per block).
        let mut rx_options = TransferOptions::default();
        rx_options.priority = Priority::VeryHigh;
        let mut ring_buf = unsafe {
            ReadableRingBuffer::new_blocked(
                state.tables.rx.get(),
                ring.len() / block_len,
                rx_channel,
                rx_request,
                regs.rx_ptr::<W>(),
                ring,
                rx_options,
            )
        };
        ring_buf.set_alignment(frame_len);
        regs.cfg1().modify(|w| w.set_rxdmaen(true));
        ring_buf.start();

        // Transmit: `tx_frame` once per frame.
        let mut quiet = TransferOptions::default();
        quiet.half_transfer_ir = false;
        quiet.complete_transfer_ir = false;

        let tx_table = unsafe {
            install(
                state.tables.tx.get(),
                Table::new([LinearItem::new_write::<W, W>(
                    tx_request,
                    tx_frame,
                    regs.tx_ptr::<W>(),
                    ItemConfig::default(),
                )]),
            )
        };
        let tx = unsafe { tx_channel.into_linked_list(tx_table, quiet) };
        regs.cfg1().modify(|w| w.set_txdmaen(true));

        regs.ier().write(|w| {
            w.set_crceie(true);
            w.set_ovrie(true);
            w.set_modfie(true);
            w.set_tifreie(true);
        });
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        // Re-arm: CR1 (SPE off), IFCR, CR1 (SPE, CSTART); only the first item waits for the
        // trigger. IFCR is needed: only TXTFC clears TXTF.
        let cr1 = regs.cr1().as_ptr() as *mut u32;
        let rearm_table = unsafe {
            let values: &[u32; 4] = &*state.tables.rearm_values.get();
            let item = |words: core::ops::Range<usize>, dest: *mut u32, triggered: bool| {
                let mut config = ItemConfig::default();
                config.trigger = triggered.then_some(trigger);
                let src = &values[words];
                // To one register: only the source advances.
                LinearItem::new_copy(src.as_ptr(), dest, src.len(), true, false, config)
            };

            install(
                state.tables.rearm.get(),
                Table::new([
                    item(0..1, cr1, true),
                    item(1..2, regs.ifcr().as_ptr() as *mut u32, false),
                    item(2..4, cr1, false),
                ]),
            )
        };
        let rearm = unsafe { rearm.into_linked_list(rearm_table, quiet) };

        RingBufferedSpiRx {
            info,
            kernel_clock,
            _wake_guard: wake_guard,
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            nss,
            ring_buf,
            ring_addr,
            frame_len: Some(frame_len),
            state: Some(state),
            irq: Some(T::Interrupt::IRQ),
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            marks: None,
            triggered: Some(Triggered { tx, rearm, frame_polls }),
        }
    }
}

/// Move `table` into `slot`, link it circularly, and return it.
///
/// # Safety
///
/// `slot` must outlive the DMA transfer and not be aliased.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
unsafe fn install<T: LinkedListItem, const M: usize>(
    slot: *mut MaybeUninit<Table<T, M>>,
    table: Table<T, M>,
) -> &'static Table<T, M> {
    let table = (*slot).write(table);
    table.link(RunMode::Circular);
    // "Preceding reads and writes cannot be moved past subsequent writes."
    fence(Ordering::SeqCst);
    (*slot).assume_init_ref()
}

/// Wait, bounded, for the frame in flight. Polls EOT: `CSTART` can stay set after it.
#[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
fn wait_frame_end(regs: Regs, frame_polls: u32) {
    let mut polls = frame_polls;
    while regs.cr1().read().spe() && !regs.sr().read().eot() && polls > 0 {
        polls -= 1;
    }
}

impl<'d, W: Word> RingBufferedSpiRx<'d, W> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        #[cfg(gpio_v2)]
        super::set_speed(&mut self._sck, &mut self._mosi, config.gpio_speed);

        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if self.triggered.is_some() {
            return self.set_config_triggered(config);
        }

        reconfigure::<Slave>(self.info, self.kernel_clock, config)
    }

    /// Reconfigure while the triggered stream is stopped, then refresh the register values the
    /// re-arm chain writes: they snapshot `CR1`, which the new CRC settings change.
    #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
    fn set_config_triggered(&mut self, config: &Config) -> Result<(), ConfigError> {
        let regs = self.info.regs;
        let (state, t) = (self.state.unwrap(), self.triggered.as_mut().unwrap());

        t.rearm.request_pause();
        while t.rearm.is_running() {}
        wait_frame_end(regs, t.frame_polls);

        let result = reconfigure::<Master>(self.info, self.kernel_clock, config);

        let mut base = regs.cr1().read();
        base.set_spe(false);
        base.set_cstart(false);
        let mut enabled = base;
        enabled.set_spe(true);
        let mut started = enabled;
        started.set_cstart(true);
        // Volatile: only the DMA reads these.
        unsafe {
            state
                .tables
                .rearm_values
                .get()
                .write_volatile([base.0, all_flags().0, enabled.0, started.0]);
        }

        t.rearm.request_resume();
        result
    }

    /// Start (or resume) reception.
    ///
    /// Note: This is also done automatically by the read functions if
    /// required.
    pub fn start(&mut self) {
        compiler_fence(Ordering::SeqCst);
        self.ring_buf.start();

        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if let Some(t) = self.triggered.as_mut() {
            t.tx.request_resume();
            t.rearm.request_resume();
            return;
        }

        set_rxdmaen(self.info.regs, true);

        self.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
    }

    /// Stop reception. The frame in flight completes first.
    ///
    /// In triggered mode the DMA channels are paused and [`start`](Self::start) resumes them;
    /// otherwise the SPI is disabled.
    pub fn stop(&mut self) {
        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if let Some(t) = self.triggered.as_mut() {
            // Stop the trigger channel first, so no frame starts while stopping.
            t.rearm.request_pause();
            wait_frame_end(self.info.regs, t.frame_polls);
            t.tx.request_pause();
            self.ring_buf.stop();

            compiler_fence(Ordering::SeqCst);
            return;
        }

        self.ring_buf.stop();

        set_rxdmaen(self.info.regs, false);

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        compiler_fence(Ordering::SeqCst);
    }

    /// Discards all data currently in the ring buffer.
    /// Returns error that were detected during background reception,
    /// and were not yet consumed by calling read functions.
    /// After calling this function ring buffer is empty and has no errors.
    pub fn clear(&mut self) -> Result<(), Error> {
        let sr = self.info.regs.sr().read();
        clear_spi_errors(self.info.regs);
        // clear ring buffer after clearing errors, otherwise errors
        // that occur between could leave garbage and be unnoticed
        self.ring_buf.clear();
        check_error_flags(sr, true)
    }

    /// (Re-)start DMA and SPI if it is not running (has not been started yet or has failed), and
    /// check for errors in status register. Error flags are checked/cleared first.
    fn start_or_check_errors(&mut self) -> Result<(), Error> {
        if self.irq.is_some() {
            // The interrupt handler owns the error flags: they are counted (and the frames
            // marked) instead of returned, and clearing them here could eat a pending mark.
            if !self.ring_buf.is_running() {
                self.start();
            }
            return Ok(());
        }

        let r = self.info.regs;

        let sr = r.sr().read();
        clear_spi_errors(r);
        check_error_flags(sr, true)?;

        if !self.ring_buf.is_running() {
            self.start();
        }
        Ok(())
    }

    /// Read bytes that are available in the ring buffer.
    ///
    /// Background reception is started if necessary (if `start()` had
    /// not previously been called, or if an error was detected which
    /// caused background reception to be stopped).
    ///
    /// Background reception is terminated when an error is returned.
    /// It must be started again by calling `start()` or by
    /// calling a read function again.
    pub async fn read(&mut self, buf: &mut [W]) -> Result<usize, Error> {
        self.start_or_check_errors()?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.stop();
                    return Err(Error::Overrun);
                }
            }

            self.wait_for_data().await;
        }
    }

    /// Wait for the DMA to write somewhere new in the ring.
    async fn wait_for_data(&mut self) {
        #[cfg(feature = "exti")]
        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if self.triggered.is_some() {
            compiler_fence(Ordering::SeqCst);
            return Self::dma_wait(&mut self.ring_buf).await;
        }

        #[cfg(feature = "exti")]
        return self.wait_for_data_or_nss_deselect_edge().await;
        #[cfg(not(feature = "exti"))]
        {
            compiler_fence(Ordering::SeqCst);
            Self::dma_wait(&mut self.ring_buf).await
        }
    }

    /// A future that completes when the DMA controller indicates it has written to the ring
    /// buffer's middle byte, or last byte.
    fn dma_wait<'a>(ring_buf: &'a mut ReadableRingBuffer<'d, W>) -> impl Future<Output = ()> + 'a {
        let mut dma_init = false;
        poll_fn(move |cx| {
            ring_buf.set_waker(cx.waker());

            let status = match dma_init {
                false => Poll::Pending,
                true => Poll::Ready(()),
            };

            dma_init = true;
            status
        })
    }

    /// Wait for NSS deselect edge or dma half-full or full.
    ///
    /// NSS deselect edge is:
    /// spi_v1, spi_v2, spi_v3 - not configurable, always rising edge
    /// spi_v4, spi_v5, spi_v6 - configurable, matches slave select polarity
    #[cfg(feature = "exti")]
    async fn wait_for_data_or_nss_deselect_edge(&mut self) {
        compiler_fence(Ordering::SeqCst);

        // Future which completes when the DMA controller indicates it
        // has written to the ring buffer's middle byte, or last byte
        let dma = Self::dma_wait(&mut self.ring_buf);

        // Future which completes when NSS deselect edge is detected
        let exti = pin!(self.nss.wait_for_edge(SlaveSelectPolarity::from_regs(self.info.regs)));

        select(exti, dma).await;
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `blocking_read()`.
    pub fn blocking_read(&mut self, buf: &mut [W]) -> Result<usize, Error> {
        self.start_or_check_errors()?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.stop();

                    return Err(Error::Overrun);
                }
            }
        }
    }

    /// Return whether the DMA ring buffer contains data, so that a read would not wait.
    pub fn read_ready(&mut self) -> Result<bool, Error> {
        let len = self.ring_buf.len().map_err(|e| match e {
            RingBufferError::Overrun => Error::Overrun,
        })?;
        Ok(len > 0)
    }

    /// Words per frame ([`set_frame_len`](Self::set_frame_len), or the triggered frame length);
    /// `None` when unset.
    pub fn frame_len(&self) -> Option<usize> {
        self.frame_len
    }

    /// Set the frame length for [`next_frame`](Self::next_frame) when receiving continuously.
    /// In triggered mode the frames are `tx_frame.len()` words and this only checks.
    ///
    /// The ring must hold at least two whole frames. Frames then stay aligned across overruns,
    /// and [`next_frame`](Self::next_frame) hands out exactly `frame_len` words at a time.
    pub fn set_frame_len(&mut self, frame_len: usize) {
        if let Some(len) = self.frame_len {
            assert_eq!(len, frame_len, "the frame length is fixed in triggered mode");
            return;
        }
        self.ring_buf.set_alignment(frame_len);
        self.frame_len = Some(frame_len);
    }

    /// Record each frame's CRC verdict, handed out by [`next_frame`](Self::next_frame) as
    /// [`Frame::is_valid`].
    ///
    /// With the [`Config::crc`] enabled, the SPI checks each frame's CRC and the interrupt
    /// handler marks the failed ones. Frames are `frame_len` words (the frame length from
    /// [`set_frame_len`](Self::set_frame_len) or the triggered frame length), counted from the
    /// ring's start; a slave's frames must line up with the master's.
    ///
    /// - `_irq`: [`InterruptHandler`] bound to the SPI's interrupt, which records the marks.
    /// - `marks`: one bit per frame in the ring: at least `(frames in the ring + 31) / 32` words.
    ///   Bits are cleared as their frames are read.
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    pub fn enable_frame_marks<T: InterruptInstance>(
        &mut self,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        marks: &'d mut [AtomicU32],
    ) {
        assert!(core::ptr::eq(self.info, T::info()), "the SPI is not instance T");
        assert!(
            self.info.regs.cfg1().read().crcen(),
            "frame marks need a CRC (Config::crc)"
        );

        let frame_len = self.frame_len.expect("frame length not configured (set_frame_len)");
        assert!(frame_len <= u16::MAX as usize, "frame length out of range");
        let frames = self.ring_buf.capacity() / frame_len;
        assert!(frames > 0, "the ring must hold a frame");
        assert!(
            marks.len() * 32 >= frames,
            "marks must hold one bit per frame in the ring"
        );

        for word in marks.iter() {
            word.store(0, Ordering::Relaxed);
        }

        // The frame count: CRC is checked per `TSIZE` frame.
        self.info.regs.cr2().modify(|w| w.set_tsize(frame_len as u16));

        let session = Session {
            head: write_head_shim::<W>,
            ring_buf: &self.ring_buf as *const _ as usize,
            frame_len,
            frames,
            bad: marks.as_ptr() as usize,
        };
        self.state = Some(&T::state().ring);
        T::state().ring.session.lock(|s| s.set(Some(session)));

        self.info.regs.ier().write(|w| {
            w.set_crceie(true);
            w.set_ovrie(true);
            w.set_modfie(true);
            w.set_tifreie(true);
        });
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        self.irq = Some(T::Interrupt::IRQ);

        self.marks = Some(marks);
    }

    /// Whether the frame at `words` passed its CRC (always true without
    /// [`enable_frame_marks`](Self::enable_frame_marks)).
    #[cfg_attr(not(any(spi_v4, spi_v5, spi_v6)), allow(unused_variables))]
    fn frame_is_valid(&self, words: *const W) -> bool {
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        if let Some(marks) = self.marks.as_deref() {
            let frame_len = self.frame_len.unwrap();
            let frames = self.ring_buf.capacity() / frame_len;
            let pos = (words as usize - self.ring_addr) / W::size().bytes();
            let slot = (pos / frame_len) % frames;
            let bit = 1u32 << (slot % 32);
            let was = marks[slot / 32].fetch_and(!bit, Ordering::Relaxed);
            return was & bit == 0;
        }

        true
    }

    /// The next whole frame, read in place: no copy.
    ///
    /// Waits until a frame has arrived in the ring and hands out its words, borrowed from the
    /// ring. They stay valid until the DMA laps the read pointer, which this (like `read`)
    /// reports as [`Error::Overrun`].
    ///
    /// Needs [`frame_len`](Self::frame_len): fixed in triggered mode, or set with
    /// [`set_frame_len`](Self::set_frame_len). A waiting call is woken at the ring's block
    /// granularity: once per `frames_per_block` frames in triggered mode.
    pub async fn next_frame(&mut self) -> Result<Frame<'_, W>, Error> {
        let frame_len = self.frame_len.expect("frame length not configured (set_frame_len)");
        self.start_or_check_errors()?;

        // With marks enabled, stay one frame behind the write head: the interrupt handler
        // records a frame's verdict when its CRC finishes, at the end of the frame, so a frame
        // is only handed out once its verdict has landed.
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let lag = if self.marks.is_some() { frame_len } else { 0 };
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        let lag = 0;

        loop {
            // How much is ready, without consuming it.
            match self.ring_buf.len() {
                Err(_) => {
                    self.stop();
                    return Err(Error::Overrun);
                }
                Ok(ready) if ready >= frame_len + lag => {}
                Ok(_) => {
                    self.wait_for_data().await;
                    continue;
                }
            }

            // As a raw pointer and length, so the ring is not borrowed while handling the
            // other outcomes.
            let attempt = self
                .ring_buf
                .read_in_place(frame_len)
                .map(|words| (words.as_ptr(), words.len()));
            match attempt {
                Ok((ptr, len)) if len > 0 => {
                    // Safety: the words are in the ring's buffer; `&mut self` keeps them
                    // exclusive until the `Frame` is dropped, and the DMA can only reach them
                    // on an overrun, which is reported as `Error::Overrun`.
                    let words = unsafe { core::slice::from_raw_parts(ptr, len) };
                    return Ok(Frame {
                        valid: self.frame_is_valid(ptr),
                        words,
                    });
                }
                _ => continue,
            }
        }
    }

    /// Whether a trigger was lost because the previous one was still being served. Clears it.
    /// Always false when receiving continuously.
    pub fn take_missed_trigger(&mut self) -> bool {
        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if let Some(t) = self.triggered.as_mut() {
            let missed = t.rearm.trigger_overrun();
            if missed {
                t.rearm.clear_trigger_overrun();
            }
            return missed;
        }

        false
    }

    /// Frames that failed their CRC since the stream started. Needs the interrupt handler.
    pub fn crc_errors(&self) -> u32 {
        self.state.map_or(0, |s| s.crc_errors.load(Ordering::Relaxed))
    }

    /// SPI receive overruns since the stream started. Needs the interrupt handler.
    pub fn overruns(&self) -> u32 {
        self.state.map_or(0, |s| s.overruns.load(Ordering::Relaxed))
    }
}

/// One frame of received words, read in place from the ring.
///
/// Handed out by [`RingBufferedSpiRx::next_frame`].
pub struct Frame<'a, W> {
    words: &'a [W],
    valid: bool,
}

impl<'a, W> Frame<'a, W> {
    /// The frame's words, in the DMA ring.
    pub fn words(&self) -> &'a [W] {
        self.words
    }

    /// Whether the frame passed its CRC (always true without
    /// [`RingBufferedSpiRx::enable_frame_marks`]).
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Words in the frame.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Whether the frame has no words.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

impl<W: Word> Drop for RingBufferedSpiRx<'_, W> {
    fn drop(&mut self) {
        #[cfg(all(any(spi_v4, spi_v5, spi_v6), any(gpdma, lpdma)))]
        if let Some(t) = self.triggered.as_mut() {
            // Stop the trigger channel first.
            t.rearm.request_reset();
            wait_frame_end(self.info.regs, t.frame_polls);

            // RM disable order: DMA channels, SPI, DMA enables.
            t.tx.request_reset();
            self.ring_buf.stop();
            self.info.regs.cfg1().modify(|w| {
                w.set_rxdmaen(false);
                w.set_txdmaen(false);
            });
            self.info.regs.cr1().modify(|w| w.set_spe(false));
            self.info.regs.ifcr().write_value(all_flags());
            self.info.regs.cr2().modify(|w| w.set_tsize(0));
            // The transfers reset their channels when dropped.
            self.disable_handler();
            self.info.rcc.disable();
            return;
        }

        self.stop();
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        self.info.regs.cr2().modify(|w| w.set_tsize(0));
        self.disable_handler();
        self.info.rcc.disable();
    }
}

impl<'d, W: Word> RingBufferedSpiRx<'d, W> {
    /// Stop marking frames: clear the session before the marks go away, and disable the
    /// interrupt.
    fn disable_handler(&mut self) {
        if self.irq.is_none() {
            return;
        }

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        {
            self.info.regs.ier().write(|w| w.0 = 0);
            if let Some(state) = self.state {
                state.session.lock(|s| s.set(None));
            }
        }
        if let Some(irq) = self.irq {
            irq.disable();
        }
    }
}

impl<W: Word> embedded_io_async::ErrorType for RingBufferedSpiRx<'_, W> {
    type Error = Error;
}

impl embedded_io_async::Read for RingBufferedSpiRx<'_, u8> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl<W: Word> ReadReady for RingBufferedSpiRx<'_, W> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        RingBufferedSpiRx::read_ready(self)
    }
}

/// Clear sticky SPI error flags and flush any stale RX FIFO data.
///
/// On SPI v4/v5/v6 (H7), OVR/UDR/MODF/CRCE/TIFRE are cleared via IFCR.
/// On older SPI, OVR is cleared by reading SR then DR.
fn clear_spi_errors(r: Regs) {
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    {
        // Write 1s to all flag-clear bits in IFCR
        r.ifcr().write(|w| w.0 = 0xffff_ffff);
        flush_rx_fifo(r);
    }
    #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
    {
        // OVR is cleared by reading SR then DR.
        // MODF is cleared by reading SR then writing CR1.
        let sr = r.sr().read();
        if sr.modf() {
            r.cr1().modify(|w| w.set_spe(false));
            r.cr1().modify(|w| w.set_spe(true));
        }
        #[cfg(not(spi_v3))]
        let _ = r.dr().read();
        #[cfg(spi_v3)]
        let _ = r.dr16().read();
    }
}
