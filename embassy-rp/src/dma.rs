//! Direct Memory Access (DMA)

use core::cmp::min;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::vals::DataSize;

use crate::interrupt::InterruptExt;
use crate::pac::dma::vals;
use crate::{interrupt, pac, peripherals, RegExt};

#[cfg(feature = "rt")]
#[interrupt]
fn DMA_IRQ_0() {
    let ints0 = pac::DMA.ints(0).read();
    for channel in 0..CHANNEL_COUNT {
        let ctrl_trig = pac::DMA.ch(channel).ctrl_trig().read();
        if ctrl_trig.ahb_error() {
            panic!("DMA: error on DMA_0 channel {}", channel);
        }

        if ints0 & (1 << channel) == (1 << channel) {
            CHANNEL_WAKERS[channel].wake();
        }
    }
    pac::DMA.ints(0).write_value(ints0);
}

pub(crate) unsafe fn init() {
    interrupt::DMA_IRQ_0.disable();
    interrupt::DMA_IRQ_0.set_priority(interrupt::Priority::P3);

    pac::DMA.inte(0).write_value(0xFFFF);

    interrupt::DMA_IRQ_0.enable();
}

/// DMA read.
///
/// SAFETY: Slice must point to a valid location reachable by DMA.
pub unsafe fn read<'a, C: Channel, W: Word>(
    ch: Peri<'a, C>,
    from: *const W,
    to: *mut [W],
    dreq: vals::TreqSel,
) -> Transfer<'a, C> {
    copy_inner(
        ch,
        from as *const u32,
        to as *mut W as *mut u32,
        to.len(),
        W::size(),
        false,
        true,
        dreq,
    )
}

/// DMA write.
///
/// SAFETY: Slice must point to a valid location reachable by DMA.
pub unsafe fn write<'a, C: Channel, W: Word>(
    ch: Peri<'a, C>,
    from: *const [W],
    to: *mut W,
    dreq: vals::TreqSel,
) -> Transfer<'a, C> {
    copy_inner(
        ch,
        from as *const W as *const u32,
        to as *mut u32,
        from.len(),
        W::size(),
        true,
        false,
        dreq,
    )
}

// static mut so that this is allocated in RAM.
static mut DUMMY: u32 = 0;

/// DMA repeated write.
///
/// SAFETY: Slice must point to a valid location reachable by DMA.
pub unsafe fn write_repeated<'a, C: Channel, W: Word>(
    ch: Peri<'a, C>,
    to: *mut W,
    len: usize,
    dreq: vals::TreqSel,
) -> Transfer<'a, C> {
    copy_inner(
        ch,
        core::ptr::addr_of_mut!(DUMMY) as *const u32,
        to as *mut u32,
        len,
        W::size(),
        false,
        false,
        dreq,
    )
}

/// DMA copy between slices.
///
/// SAFETY: Slices must point to locations reachable by DMA.
pub unsafe fn copy<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: &[W], to: &mut [W]) -> Transfer<'a, C> {
    let from_len = from.len();
    let to_len = to.len();
    assert_eq!(from_len, to_len);
    copy_inner(
        ch,
        from.as_ptr() as *const u32,
        to.as_mut_ptr() as *mut u32,
        from_len,
        W::size(),
        true,
        true,
        vals::TreqSel::PERMANENT,
    )
}

fn copy_inner<'a, C: Channel>(
    ch: Peri<'a, C>,
    from: *const u32,
    to: *mut u32,
    len: usize,
    data_size: DataSize,
    incr_read: bool,
    incr_write: bool,
    dreq: vals::TreqSel,
) -> Transfer<'a, C> {
    let p = ch.regs();

    p.read_addr().write_value(from as u32);
    p.write_addr().write_value(to as u32);
    #[cfg(feature = "rp2040")]
    p.trans_count().write(|w| {
        *w = len as u32;
    });
    #[cfg(feature = "_rp235x")]
    p.trans_count().write(|w| {
        w.set_mode(0.into());
        w.set_count(len as u32);
    });

    compiler_fence(Ordering::SeqCst);

    p.ctrl_trig().write(|w| {
        w.set_treq_sel(dreq);
        w.set_data_size(data_size);
        w.set_incr_read(incr_read);
        w.set_incr_write(incr_write);
        w.set_chain_to(ch.number());
        w.set_en(true);
    });

    compiler_fence(Ordering::SeqCst);
    Transfer::new_inner(ch)
}

/// DMA Target
pub(crate) trait Target<W: Word> {
    /// size of address wrap region (refer to rp2040 datasheet)
    const RING_SIZE: u8 = 0;

    /// If 1, the read address increments with each transfer. If 0, each
    /// read is directed to the same, initial address.
    const INCR_ADDR: bool = false;

    const TREQ_SEL: u8 = vals::TreqSel::PERMANENT as u8;

    /// return max transfer count
    fn transfer_count(&self) -> u32 {
        u32::MAX
    }
}

const fn resolve_treq(a: u8, b: u8) -> u8 {
    if a == vals::TreqSel::PERMANENT as u8 || a == b {
        b
    } else if b == vals::TreqSel::PERMANENT as u8 {
        a
    } else {
        core::panic!("TREQ of two DMA target doesn't match")
    }
}

/// DMA Target Utils
pub trait TargetExt<W: Word>: Sized {
    /// Read from DMA target
    fn dma_read_with_transfer_count<'a, C: Channel + 'a, T: TargetMut<W> + 'a>(
        self,
        ch: Peri<'a, C>,
        to: T,
        transfer_count: u32,
    ) -> (Transfer<'a, C>, u32)
    where
        Self: TargetRef<W> + 'a,
    {
        let treq = vals::TreqSel::from(const { resolve_treq(Self::TREQ_SEL, T::TREQ_SEL) });
        Transfer::new_with_transfer_count(ch, self, to, transfer_count, treq)
    }

    /// Write to DMA target
    fn dma_write_with_transfer_count<'a, C: Channel + 'a, T: TargetRef<W> + 'a>(
        self,
        ch: Peri<'a, C>,
        from: T,
        transfer_count: u32,
    ) -> (Transfer<'a, C>, u32)
    where
        Self: TargetMut<W> + 'a,
    {
        let treq = vals::TreqSel::from(const { resolve_treq(Self::TREQ_SEL, T::TREQ_SEL) });
        Transfer::new_with_transfer_count(ch, from, self, transfer_count, treq)
    }
}

impl<W: Word, T: Target<W> + Sized> TargetExt<W> for T {}

/// immutable dma target address
#[allow(private_bounds)]
pub trait TargetRef<W: Word>: Target<W> {
    /// return address
    fn as_ptr(&self) -> *const W;
}

/// mutable dma target address
#[allow(private_bounds)]
pub trait TargetMut<W: Word>: Target<W> {
    /// return mutable address
    fn as_mut_ptr(&mut self) -> *mut W;
}

impl<W: Word> Target<W> for &W {}

impl<W: Word> Target<W> for &mut W {}

impl<W: Word> TargetRef<W> for &W {
    fn as_ptr(&self) -> *const W {
        *self
    }
}

impl<W: Word> TargetRef<W> for &mut W {
    fn as_ptr(&self) -> *const W {
        *self
    }
}

impl<W: Word> TargetMut<W> for &mut W {
    fn as_mut_ptr(&mut self) -> *mut W {
        *self
    }
}

const fn ring_size<T: Sized>() -> u8 {
    let t_size = size_of::<T>();
    let mut i = 1;
    while i <= 0b1111 {
        if 1 << i == t_size {
            return i;
        }
        i += 1;
    }
    core::panic!("byte size must be power of two");
}

/// wrapper for dma read/write address, will make it repeat slice
/// byte size of type must be power of two
#[derive(Debug)]
pub struct Ring<T>(T);

impl<W: Word, const N: usize> Target<W> for Ring<&[W; N]> {
    const RING_SIZE: u8 = ring_size::<[W; N]>();

    const INCR_ADDR: bool = true;
}

impl<W: Word, const N: usize> Target<W> for Ring<&mut [W; N]> {
    const RING_SIZE: u8 = ring_size::<[W; N]>();
    const INCR_ADDR: bool = true;
}

impl<W: Word, const N: usize> TargetRef<W> for Ring<&[W; N]> {
    fn as_ptr(&self) -> *const W {
        &self.0[0]
    }
}

impl<W: Word, const N: usize> TargetMut<W> for Ring<&mut [W; N]> {
    fn as_mut_ptr(&mut self) -> *mut W {
        &mut self.0[0]
    }
}

impl<W: Word> Target<W> for &[W] {
    const RING_SIZE: u8 = 0;

    const INCR_ADDR: bool = true;

    fn transfer_count(&self) -> u32 {
        self.len() as u32
    }
}

impl<W: Word> Target<W> for &mut [W] {
    const RING_SIZE: u8 = 0;

    const INCR_ADDR: bool = true;

    fn transfer_count(&self) -> u32 {
        self.len() as u32
    }
}

impl<W: Word> TargetRef<W> for &[W] {
    fn as_ptr(&self) -> *const W {
        &self[0]
    }
}

impl<W: Word> TargetMut<W> for &mut [W] {
    fn as_mut_ptr(&mut self) -> *mut W {
        &mut self[0]
    }
}

/// DMA transfer driver.
/// dropping this will abort transfer
pub struct Transfer<'a, C: Channel> {
    channel: Peri<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new_inner(channel: Peri<'a, C>) -> Self {
        Self { channel }
    }

    /// start DMA with transfer count
    pub fn new_with_transfer_count<W: Word, F: TargetRef<W> + 'a, T: TargetMut<W> + 'a>(
        ch: Peri<'a, C>,
        from: F,
        mut to: T,
        // receive transfer count to remind dma can only be finite
        mut transfer_count: u32,
        dreq: vals::TreqSel,
    ) -> (Self, u32) {
        let (ring_size, ring_sel) = const {
            if F::RING_SIZE * T::RING_SIZE != 0 {
                core::panic!("read or write can't be set ring at the same time");
            }
            (F::RING_SIZE + T::RING_SIZE, T::RING_SIZE != 0)
        };
        transfer_count = min(transfer_count, from.transfer_count());
        transfer_count = min(transfer_count, to.transfer_count());

        let p = ch.regs();

        p.read_addr().write_value(from.as_ptr() as u32);
        p.write_addr().write_value(to.as_mut_ptr() as u32);
        #[cfg(feature = "rp2040")]
        p.trans_count().write(|w| {
            *w = transfer_count;
        });
        #[cfg(feature = "_rp235x")]
        p.trans_count().write(|w| {
            w.set_mode(0.into());
            w.set_count(transfer_count);
        });

        compiler_fence(Ordering::SeqCst);

        p.ctrl_trig().write(|w| {
            w.set_treq_sel(dreq);
            w.set_data_size(W::size());
            w.set_incr_read(F::INCR_ADDR);
            w.set_incr_write(T::INCR_ADDR);
            w.set_ring_size(ring_size);
            w.set_ring_sel(ring_sel);
            w.set_chain_to(ch.number());
            w.set_en(true);
        });

        compiler_fence(Ordering::SeqCst);

        (Self::new_inner(ch), transfer_count)
    }

    /// wait until transfer finishes (not busy)
    /// this will never be ready if transfer is disabled before finishing
    pub fn wait(&mut self) -> TransferFuture<'_, C> {
        TransferFuture {
            channel: self.channel.reborrow(),
        }
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`abort`](Self::abort).
    pub fn busy(&self) -> bool {
        self.channel.regs().ctrl_trig().read().busy()
    }

    /// Enable transfer
    ///
    /// Unless disabled manually, transfer is enabled by default
    pub fn enable(&mut self) {
        self.channel.regs().ctrl_trig().write_set(|w| w.set_en(true));
    }

    /// Disable transfer
    ///
    /// Effectively pauses transfer, It will not affect busy state
    pub fn disable(&mut self) {
        self.channel.regs().ctrl_trig().write_clear(|w| w.set_en(true));
    }

    /// Check if transfer is enabled
    ///
    /// Unless disabled manually, transfer is enabled by default
    pub fn is_enabled(&self) -> bool {
        self.channel.regs().ctrl_trig().read().en()
    }

    /// Returns amount of transfers left before finishing
    pub fn transfer_count(&self) -> u32 {
        self.channel.regs().trans_count().read()
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        let p = self.channel.regs();
        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort(1 << self.channel.number()));
        while p.ctrl_trig().read().busy() {}
    }
}

/// dropping this will not affect the state of transfer
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TransferFuture<'a, C: Channel> {
    // while TransferFuture only polls busy state of dma
    // since only one Waker can be registered,
    // we prevent others from registering Waker by having exclusive access
    channel: Peri<'a, C>,
}
impl<'a, C: Channel> Unpin for TransferFuture<'a, C> {}
impl<'a, C: Channel> Future for TransferFuture<'a, C> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        CHANNEL_WAKERS[self.channel.number() as usize].register(cx.waker());

        if self.channel.regs().ctrl_trig().read().busy() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

#[cfg(feature = "rp2040")]
pub(crate) const CHANNEL_COUNT: usize = 12;
#[cfg(feature = "_rp235x")]
pub(crate) const CHANNEL_COUNT: usize = 16;
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [const { AtomicWaker::new() }; CHANNEL_COUNT];

trait SealedChannel {}
trait SealedWord {}

/// DMA channel interface.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Into<AnyChannel> + Sized + 'static {
    /// Channel number.
    fn number(&self) -> u8;

    /// Channel registry block.
    fn regs(&self) -> pac::dma::Channel {
        pac::DMA.ch(self.number() as _)
    }
}

/// DMA word.
#[allow(private_bounds)]
pub trait Word: SealedWord + Sized {
    /// Word size.
    fn size() -> vals::DataSize;
}

impl SealedWord for u8 {}
impl Word for u8 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_BYTE
    }
}

impl SealedWord for u16 {}
impl Word for u16 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_HALFWORD
    }
}

impl SealedWord for u32 {}
impl Word for u32 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_WORD
    }
}

/// Type erased DMA channel.
pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl SealedChannel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl SealedChannel for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }

        impl From<peripherals::$name> for crate::dma::AnyChannel {
            fn from(val: peripherals::$name) -> Self {
                Self { number: val.number() }
            }
        }
    };
}

channel!(DMA_CH0, 0);
channel!(DMA_CH1, 1);
channel!(DMA_CH2, 2);
channel!(DMA_CH3, 3);
channel!(DMA_CH4, 4);
channel!(DMA_CH5, 5);
channel!(DMA_CH6, 6);
channel!(DMA_CH7, 7);
channel!(DMA_CH8, 8);
channel!(DMA_CH9, 9);
channel!(DMA_CH10, 10);
channel!(DMA_CH11, 11);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH12, 12);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH13, 13);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH14, 14);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH15, 15);
