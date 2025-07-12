//! Direct Memory Access (DMA)
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::vals::DataSize;

use crate::interrupt::InterruptExt;
use crate::pac::dma::vals;
use crate::{interrupt, pac, peripherals};

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
    Transfer::new(ch)
}

/// DMA transfer driver.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a, C: Channel> {
    channel: Peri<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: Peri<'a, C>) -> Self {
        Self { channel }
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

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
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
pub trait Word: SealedWord {
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
