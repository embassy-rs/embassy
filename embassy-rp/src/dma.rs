//! Direct Memory Access (DMA)
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::vals::DataSize;

use crate::interrupt::InterruptExt;
use crate::pac::dma::vals;
use crate::{interrupt, pac, peripherals};

#[cfg(feature = "rt")]
#[interrupt]
fn DMA_IRQ_0() {
    let ints0 = pac::DMA.ints0().read().ints0();
    for channel in 0..CHANNEL_COUNT {
        let ctrl_trig = pac::DMA.ch(channel).ctrl_trig().read();
        if ctrl_trig.ahb_error() {
            panic!("DMA: error on DMA_0 channel {}", channel);
        }

        if ints0 & (1 << channel) == (1 << channel) {
            CHANNEL_WAKERS[channel].wake();
        }
    }
    pac::DMA.ints0().write(|w| w.set_ints0(ints0));
}

pub(crate) unsafe fn init() {
    interrupt::DMA_IRQ_0.disable();
    interrupt::DMA_IRQ_0.set_priority(interrupt::Priority::P3);

    pac::DMA.inte0().write(|w| w.set_inte0(0xFFFF));

    interrupt::DMA_IRQ_0.enable();
}

pub unsafe fn read<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const W,
    to: *mut [W],
    dreq: u8,
) -> Transfer<'a, C> {
    let (to_ptr, len) = crate::dma::slice_ptr_parts(to);
    copy_inner(
        ch,
        from as *const u32,
        to_ptr as *mut u32,
        len,
        W::size(),
        false,
        true,
        dreq,
    )
}

pub unsafe fn write<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const [W],
    to: *mut W,
    dreq: u8,
) -> Transfer<'a, C> {
    let (from_ptr, len) = crate::dma::slice_ptr_parts(from);
    copy_inner(
        ch,
        from_ptr as *const u32,
        to as *mut u32,
        len,
        W::size(),
        true,
        false,
        dreq,
    )
}

// static mut so that this is allocated in RAM.
static mut DUMMY: u32 = 0;

pub unsafe fn write_repeated<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    to: *mut W,
    len: usize,
    dreq: u8,
) -> Transfer<'a, C> {
    copy_inner(
        ch,
        &mut DUMMY as *const u32,
        to as *mut u32,
        len,
        W::size(),
        false,
        false,
        dreq,
    )
}

pub unsafe fn copy<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: &[W],
    to: &mut [W],
) -> Transfer<'a, C> {
    let (from_ptr, from_len) = crate::dma::slice_ptr_parts(from);
    let (to_ptr, to_len) = crate::dma::slice_ptr_parts_mut(to);
    assert_eq!(from_len, to_len);
    copy_inner(
        ch,
        from_ptr as *const u32,
        to_ptr as *mut u32,
        from_len,
        W::size(),
        true,
        true,
        vals::TreqSel::PERMANENT.0,
    )
}

fn copy_inner<'a, C: Channel>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const u32,
    to: *mut u32,
    len: usize,
    data_size: DataSize,
    incr_read: bool,
    incr_write: bool,
    dreq: u8,
) -> Transfer<'a, C> {
    into_ref!(ch);

    let p = ch.regs();

    p.read_addr().write_value(from as u32);
    p.write_addr().write_value(to as u32);
    p.trans_count().write_value(len as u32);

    compiler_fence(Ordering::SeqCst);

    p.ctrl_trig().write(|w| {
        // TODO: Add all DREQ options to pac vals::TreqSel, and use
        // `set_treq:sel`
        w.0 = ((dreq as u32) & 0x3f) << 15usize;
        w.set_data_size(data_size);
        w.set_incr_read(incr_read);
        w.set_incr_write(incr_write);
        w.set_chain_to(ch.number());
        w.set_en(true);
    });

    compiler_fence(Ordering::SeqCst);
    Transfer::new(ch)
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: impl Peripheral<P = C> + 'a) -> Self {
        into_ref!(channel);

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

pub(crate) const CHANNEL_COUNT: usize = 12;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [NEW_AW; CHANNEL_COUNT];

mod sealed {
    pub trait Channel {}

    pub trait Word {}
}

pub trait Channel: Peripheral<P = Self> + sealed::Channel + Into<AnyChannel> + Sized + 'static {
    fn number(&self) -> u8;

    fn regs(&self) -> pac::dma::Channel {
        pac::DMA.ch(self.number() as _)
    }

    fn degrade(self) -> AnyChannel {
        AnyChannel { number: self.number() }
    }
}

pub trait Word: sealed::Word {
    fn size() -> vals::DataSize;
}

impl sealed::Word for u8 {}
impl Word for u8 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_BYTE
    }
}

impl sealed::Word for u16 {}
impl Word for u16 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_HALFWORD
    }
}

impl sealed::Word for u32 {}
impl Word for u32 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_WORD
    }
}

pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl sealed::Channel for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }

        impl From<peripherals::$name> for crate::dma::AnyChannel {
            fn from(val: peripherals::$name) -> Self {
                crate::dma::Channel::degrade(val)
            }
        }
    };
}

// TODO: replace transmutes with core::ptr::metadata once it's stable
#[allow(unused)]
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (usize, usize) {
    unsafe { core::mem::transmute(slice) }
}

#[allow(unused)]
pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (usize, usize) {
    unsafe { core::mem::transmute(slice) }
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
