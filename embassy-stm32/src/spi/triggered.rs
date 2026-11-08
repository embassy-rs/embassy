//! SPI frames started by a hardware trigger, received into a ring of blocks.
//!
//! For devices that signal each sample with a data-ready edge. Each trigger edge (an EXTI event,
//! a timer, ...) starts one frame of `TSIZE` words, with an NSS pulse if the [`Spi`] drives NSS
//! ([`Spi::new_with_nss`]). A third DMA channel, gated by the trigger, restarts the SPI; the
//! SPI's own channels fill the ring and send the same frame each time. No CPU time per frame.
//!
//! With a CRC ([`Config::crc`]), failed frames are marked per block ([`Block::is_valid`]). This
//! needs [`InterruptHandler`] bound to the SPI's interrupt.

use core::cell::Cell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU32, Ordering, fence};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use super::mode::Master;
use super::{CrcConfig, CrcInit, Instance, Regs, RegsExt, Spi, Word, regs, write_crc};
use crate::dma::{
    Channel, ItemConfig, LinearItem, LinkedListItem, LinkedListTransfer, Priority, RunMode, Table, TransferOptions,
    TriggerConfig,
};
use crate::interrupt;
use crate::interrupt::InterruptExt;
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::Async;

/// Maximum frames per block (the bad-frame marks are 64 bits).
pub const MAX_FRAMES_PER_BLOCK: usize = 64;

/// Stream configuration.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    /// The trigger that starts each frame.
    pub trigger: TriggerConfig,
    /// CRC checked on each frame, replacing the [`Spi`]'s while the stream runs. Default: none.
    pub crc: CrcConfig,
    /// CRC initial value.
    pub crc_init: CrcInit,
}

impl Config {
    /// Frames started by `trigger`, without a CRC.
    pub fn new(trigger: TriggerConfig) -> Self {
        Self {
            trigger,
            crc: CrcConfig::Disabled,
            crc_init: CrcInit::Zeros,
        }
    }
}

/// The SPI's CRC registers, restored when the stream ends.
#[derive(Clone, Copy)]
struct SavedCrc {
    crcpoly: u32,
    crcen: bool,
    crcsize: u8,
    cr1: regs::Cr1,
}

impl SavedCrc {
    fn save(regs: Regs) -> Self {
        let cfg1 = regs.cfg1().read();
        Self {
            crcpoly: regs.crcpoly().read().crcpoly(),
            crcen: cfg1.crcen(),
            crcsize: cfg1.crcsize(),
            cr1: regs.cr1().read(),
        }
    }

    /// With SPE clear.
    fn restore(self, regs: Regs) {
        regs.crcpoly().write(|w| w.set_crcpoly(self.crcpoly));
        regs.cfg1().modify(|w| {
            w.set_crcen(self.crcen);
            w.set_crcsize(self.crcsize);
        });
        regs.cr1().modify(|w| {
            w.set_crc33_17(self.cr1.crc33_17());
            w.set_tcrcini(self.cr1.tcrcini());
            w.set_rcrcini(self.cr1.rcrcini());
        });
    }
}

/// Stream error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The consumer fell a ring behind and a block was overwritten.
    Overrun,
    /// The receive channel stopped.
    Stopped,
}

/// DMA descriptors and per-block state. Must outlive the stream, in memory the DMA reaches.
///
/// The alignment keeps a list from straddling 64 KiB for up to 16 blocks.
#[repr(C, align(1024))]
pub struct Descriptors<const N: usize> {
    rx: MaybeUninit<Table<LinearItem, N>>,
    tx: MaybeUninit<Table<LinearItem, 1>>,
    rearm: MaybeUninit<Table<LinearItem, 3>>,
    rearm_values: [u32; 4],
    bad: [[AtomicU32; 2]; N],
}

impl<const N: usize> Descriptors<N> {
    /// Empty descriptors; [`TriggeredFrames::new`] fills them.
    pub const fn new() -> Self {
        Self {
            rx: MaybeUninit::uninit(),
            tx: MaybeUninit::uninit(),
            rearm: MaybeUninit::uninit(),
            rearm_values: [0; 4],
            bad: [const { [const { AtomicU32::new(0) }; 2] }; N],
        }
    }
}

impl<const N: usize> Default for Descriptors<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// What the interrupt handler needs; plain addresses, so it is `Send`.
#[derive(Clone, Copy)]
struct Session {
    /// The receive channel's destination address register.
    dar: usize,
    ring: usize,
    frame_bytes: usize,
    frames_per_block: usize,
    blocks: usize,
    /// `[[AtomicU32; 2]; blocks]`
    bad: usize,
}

impl Session {
    /// CRCE comes after the CRC, when the receive channel has usually moved to the next slot. If it
    /// is still inside a slot, that slot is the bad one.
    fn mark_bad_frame(&self) {
        let frames = self.blocks * self.frames_per_block;
        let dar = unsafe { (self.dar as *const u32).read_volatile() } as usize;
        let written = dar.wrapping_sub(self.ring);
        let slot = written / self.frame_bytes;
        let bad = if written % self.frame_bytes == 0 {
            (slot + frames - 1) % frames
        } else {
            slot % frames
        };

        let (block, bit) = (bad / self.frames_per_block, bad % self.frames_per_block);
        let marks = unsafe { &*(self.bad as *const [AtomicU32; 2]).add(block) };
        marks[bit / 32].fetch_or(1 << (bit % 32), Ordering::Relaxed);
    }
}

/// Per-instance state of a stream.
pub(crate) struct State {
    session: Mutex<CriticalSectionRawMutex, Cell<Option<Session>>>,
    crc_errors: AtomicU32,
    overruns: AtomicU32,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            session: Mutex::new(Cell::new(None)),
            crc_errors: AtomicU32::new(0),
            overruns: AtomicU32::new(0),
        }
    }
}

/// All SPI event flags. Re-enabling the SPI clears neither TXTF nor SUSP, and TXTF blocks the
/// transmit DMA request.
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
pub trait InterruptInstance: Instance {
    /// The SPI's interrupt.
    type Interrupt: Interrupt;
}

foreach_interrupt!(
    ($inst:ident, spi, $block:ident, GLOBAL, $irq:ident) => {
        impl InterruptInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);

/// SPI interrupt handler for [`TriggeredFrames`]: marks CRC failures and counts overruns.
pub struct InterruptHandler<T: InterruptInstance> {
    _phantom: PhantomData<T>,
}

impl<T: InterruptInstance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let state = &T::state().triggered;
        let Some(session) = state.session.lock(Cell::get) else {
            return;
        };

        let regs = T::info().regs;
        let sr = regs.sr().read();
        if sr.crce() {
            state.crc_errors.fetch_add(1, Ordering::Relaxed);
            session.mark_bad_frame();
        }
        if sr.ovr() {
            state.overruns.fetch_add(1, Ordering::Relaxed);
        }

        // Clear everything, so no other flag keeps the interrupt firing.
        regs.ifcr().write_value(all_flags());
    }
}

/// A stream of triggered frames. Dropping it stops the stream and restores the [`Spi`].
pub struct TriggeredFrames<'a, W: Word, const N: usize> {
    regs: Regs,
    state: &'static State,
    irq: interrupt::Interrupt,
    saved_crc: SavedCrc,
    rx: LinkedListTransfer<'a>,
    tx: LinkedListTransfer<'a>,
    rearm: LinkedListTransfer<'a>,
    /// Raw pointer: the DMA writes the ring while the consumer reads it.
    ring: *const W,
    frame_len: usize,
    frames_per_block: usize,
    bad: &'a [[AtomicU32; 2]; N],
    /// Blocks handed out so far.
    next: usize,
    /// Whether block `next - 1` is still borrowed.
    lent: bool,
    /// Poll budget for one frame.
    frame_polls: u32,
    _ring: PhantomData<&'a mut [W]>,
}

/// Move `table` into `slot` and link it circularly.
fn install<'a, const M: usize>(
    slot: &'a mut MaybeUninit<Table<LinearItem, M>>,
    table: Table<LinearItem, M>,
) -> &'a mut Table<LinearItem, M> {
    let table = slot.write(table);
    table.link(RunMode::Circular);
    fence(Ordering::SeqCst);
    table
}

impl<'a, W: Word, const N: usize> TriggeredFrames<'a, W, N> {
    /// Start streaming: every edge of `config.trigger` runs one frame.
    ///
    /// - `spi`: instance `T`, with both DMA channels; its mode and frequency apply to the frames.
    /// - `rearm`: a third channel of the same DMA controller.
    /// - `tx_frame`: sent each frame; its length is the frame length, at most 65535 words.
    /// - `ring`: `N` blocks of whole frames, at most [`MAX_FRAMES_PER_BLOCK`] frames and 65535
    ///   bytes per block.
    ///
    /// The trigger source is set up separately (e.g. [`ExtiInput::enable_event`]). The first
    /// block may hold frames from before the call.
    ///
    /// [`ExtiInput::enable_event`]: crate::exti::ExtiInput::enable_event
    pub fn new<T: InterruptInstance>(
        spi: &'a mut Spi<'_, Async, Master>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        rearm: &'a mut Channel<'_>,
        config: Config,
        descriptors: &'a mut Descriptors<N>,
        ring: &'a mut [W],
        tx_frame: &'a [W],
    ) -> Self {
        assert!(core::ptr::eq(spi.info, T::info()), "the SPI is not instance T");

        let frame_len = tx_frame.len();
        assert!(
            frame_len > 0 && frame_len <= u16::MAX as usize,
            "frame length out of range"
        );
        let block_len = ring.len() / N;
        assert!(
            ring.len() % N == 0 && block_len % frame_len == 0,
            "the ring must be N blocks of whole frames"
        );
        let frames_per_block = block_len / frame_len;
        assert!(
            (1..=MAX_FRAMES_PER_BLOCK).contains(&frames_per_block),
            "1 to MAX_FRAMES_PER_BLOCK frames per block"
        );

        let regs = spi.info.regs;
        let state = &T::state().triggered;

        // SPI: the stream's CRC and the frame length.
        regs.cr1().modify(|w| w.set_spe(false));
        let saved_crc = SavedCrc::save(regs);
        write_crc(regs, config.crc, config.crc_init);
        spi.set_word_size(W::CONFIG);
        regs.ifcr().write_value(all_flags());
        regs.cr2().modify(|w| w.set_tsize(frame_len as u16));

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

        let Descriptors {
            rx: rx_slot,
            tx: tx_slot,
            rearm: rearm_slot,
            rearm_values,
            bad,
        } = descriptors;

        // Volatile: only the DMA reads these.
        unsafe {
            (rearm_values as *mut [u32; 4]).write_volatile([base.0, all_flags().0, enabled.0, started.0]);
        }

        for word in bad.iter().flatten() {
            word.store(0, Ordering::Relaxed);
        }
        state.crc_errors.store(0, Ordering::Relaxed);
        state.overruns.store(0, Ordering::Relaxed);

        let rx_dma = spi.rx_dma.as_mut().expect("the SPI has no receive DMA channel");
        let tx_dma = spi.tx_dma.as_mut().expect("the SPI has no transmit DMA channel");
        let (rx_request, tx_request) = (rx_dma.request, tx_dma.request);

        state.session.lock(|s| {
            s.set(Some(Session {
                dar: rx_dma.channel.destination_register() as usize,
                ring: ring.as_ptr() as usize,
                frame_bytes: frame_len * W::size().bytes(),
                frames_per_block,
                blocks: N,
                bad: bad.as_ptr() as usize,
            }))
        });

        // Receive: one item per block.
        let mut rx_options = TransferOptions::default();
        rx_options.priority = Priority::VeryHigh;
        rx_options.half_transfer_ir = false;
        rx_options.complete_transfer_ir = true;

        let ring_ptr = ring.as_ptr();
        let rx = unsafe {
            let mut blocks = ring.chunks_exact_mut(block_len);
            let table = Table::new(core::array::from_fn(|_| {
                LinearItem::new_read::<W, W>(
                    rx_request,
                    regs.rx_ptr::<W>(),
                    blocks.next().unwrap(),
                    ItemConfig::default(),
                )
            }));
            rx_dma.channel.linked_list(install(rx_slot, table), rx_options)
        };
        regs.cfg1().modify(|w| w.set_rxdmaen(true));

        // Transmit: `tx_frame` once per frame.
        let mut quiet = TransferOptions::default();
        quiet.half_transfer_ir = false;
        quiet.complete_transfer_ir = false;

        let tx = unsafe {
            let table = Table::new([LinearItem::new_write::<W, W>(
                tx_request,
                tx_frame,
                regs.tx_ptr::<W>(),
                ItemConfig::default(),
            )]);
            tx_dma.channel.linked_list(install(tx_slot, table), quiet)
        };
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
        let trigger = config.trigger;
        let rearm = unsafe {
            let values: &[u32; 4] = rearm_values;
            let item = |words: core::ops::Range<usize>, dest: *mut u32, triggered: bool| {
                let mut config = ItemConfig::default();
                config.trigger = triggered.then_some(trigger);
                let src = &values[words];
                // To one register: only the source advances.
                LinearItem::new_copy(src.as_ptr(), dest, src.len(), true, false, config)
            };

            let table = Table::new([
                item(0..1, cr1, true),
                item(1..2, regs.ifcr().as_ptr() as *mut u32, false),
                item(2..4, cr1, false),
            ]);
            rearm.linked_list(install(rearm_slot, table), quiet)
        };

        Self {
            regs,
            state,
            irq: T::Interrupt::IRQ,
            saved_crc,
            rx,
            tx,
            rearm,
            ring: ring_ptr,
            frame_len,
            frames_per_block,
            bad,
            next: 0,
            lent: false,
            frame_polls,
            _ring: PhantomData,
        }
    }

    /// The next block, read in place. [`Error::Overrun`] if the consumer fell a ring behind, also
    /// while it held the previous block.
    pub async fn next_block(&mut self) -> Result<Block<'_, W>, Error> {
        // Block k is overwritten once `filled - k >= N`.
        let overrun = |filled: usize, k: usize| filled.wrapping_sub(k) >= N;
        if core::mem::take(&mut self.lent) && overrun(self.rx.completed_items(), self.next.wrapping_sub(1)) {
            return Err(Error::Overrun);
        }

        let k = self.next;
        let filled = self.rx.wait_completed_items(k).await;
        if filled == k {
            return Err(Error::Stopped);
        }
        // Fell a ring behind while waiting.
        if overrun(filled, k) {
            self.next = filled;
            return Err(Error::Overrun);
        }

        // Let the last frame's CRC finish before collecting the marks.
        self.wait_frame_end();

        let block = k % N;
        let marks = &self.bad[block];
        let invalid =
            u64::from(marks[0].swap(0, Ordering::Relaxed)) | (u64::from(marks[1].swap(0, Ordering::Relaxed)) << 32);

        self.next = k.wrapping_add(1);
        self.lent = true;

        let block_len = self.frames_per_block * self.frame_len;
        // Safe: the DMA is filling a later block, and `&mut self` prevents a second borrow.
        let words = unsafe { core::slice::from_raw_parts(self.ring.add(block * block_len), block_len) };
        Ok(Block {
            words,
            frame_len: self.frame_len,
            invalid,
        })
    }

    /// Whether a trigger was lost because the previous one was still being served. Clears it.
    pub fn take_missed_trigger(&mut self) -> bool {
        let missed = self.rearm.trigger_overrun();
        if missed {
            self.rearm.clear_trigger_overrun();
        }
        missed
    }

    /// Frames that failed their CRC since the stream started.
    pub fn crc_errors(&self) -> u32 {
        self.state.crc_errors.load(Ordering::Relaxed)
    }

    /// SPI receive overruns since the stream started.
    pub fn overruns(&self) -> u32 {
        self.state.overruns.load(Ordering::Relaxed)
    }

    /// Frames per block.
    pub fn frames_per_block(&self) -> usize {
        self.frames_per_block
    }

    /// Wait, bounded, for the frame in flight. Polls EOT: `CSTART` can stay set after it.
    fn wait_frame_end(&self) {
        let mut polls = self.frame_polls;
        while self.regs.cr1().read().spe() && !self.regs.sr().read().eot() && polls > 0 {
            polls -= 1;
        }
    }
}

impl<W: Word, const N: usize> Drop for TriggeredFrames<'_, W, N> {
    fn drop(&mut self) {
        // Stop the trigger channel first.
        self.rearm.request_reset();
        self.wait_frame_end();

        // RM disable order: DMA channels, SPI, DMA enables.
        self.tx.request_reset();
        self.rx.request_reset();
        self.regs.cfg1().modify(|w| {
            w.set_rxdmaen(false);
            w.set_txdmaen(false);
        });
        self.regs.cr1().modify(|w| w.set_spe(false));
        self.regs.ier().write(|w| w.0 = 0);
        self.irq.disable();
        self.state.session.lock(|s| s.set(None));
        self.regs.ifcr().write_value(all_flags());
        self.regs.cr2().modify(|w| w.set_tsize(0));
        self.saved_crc.restore(self.regs);
        // The transfers reset their channels when dropped.
    }
}

/// One block of frames, borrowed from the ring.
pub struct Block<'a, W> {
    words: &'a [W],
    frame_len: usize,
    invalid: u64,
}

impl<'a, W> Block<'a, W> {
    /// Frames in the block.
    pub fn len(&self) -> usize {
        self.words.len() / self.frame_len
    }

    /// Whether the block has no frames.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Frame `i`'s words.
    pub fn frame(&self, i: usize) -> &'a [W] {
        &self.words[i * self.frame_len..(i + 1) * self.frame_len]
    }

    /// Whether frame `i` passed its CRC (always true without a CRC).
    pub fn is_valid(&self, i: usize) -> bool {
        i >= 64 || self.invalid & (1 << i) == 0
    }

    /// The whole block.
    pub fn words(&self) -> &'a [W] {
        self.words
    }
}
