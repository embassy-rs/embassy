use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use embassy_util::waitqueue::AtomicWaker;
use futures::Future;
use pac::dma::vals::DataSize;

use crate::pac::dma::vals;
use crate::{interrupt, pac, peripherals};

#[interrupt]
unsafe fn DMA_IRQ_0() {
    let ints0 = pac::DMA.ints0().read().ints0();

    critical_section::with(|_| {
        for channel in 0..CHANNEL_COUNT {
            if ints0 & (1 << channel) == 1 {
                CHANNEL_WAKERS[channel].wake();
            }
        }
        pac::DMA.ints0().write(|w| w.set_ints0(ints0));
    });
}

pub(crate) fn read<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const W,
    to: *mut [W],
) -> Transfer<'a, C> {
    let (ptr, len) = crate::dma::slice_ptr_parts_mut(to);
    copy(ch, from as *const u32, ptr as *mut u32, len, W::size())
}

pub(crate) fn write<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const [W],
    to: *mut W,
) -> Transfer<'a, C> {
    let (from_ptr, len) = crate::dma::slice_ptr_parts(from);
    copy(ch, from_ptr as *const u32, to as *mut u32, len, W::size())
}

fn copy<'a, C: Channel>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const u32,
    to: *mut u32,
    len: usize,
    data_size: DataSize,
) -> Transfer<'a, C> {
    into_ref!(ch);

    unsafe {
        let p = ch.regs();

        p.read_addr().write_value(from as u32);
        p.write_addr().write_value(to as u32);
        p.trans_count().write_value(len as u32);

        compiler_fence(Ordering::SeqCst);

        p.ctrl_trig().write(|w| {
            w.set_data_size(data_size);
            w.set_incr_read(false);
            w.set_incr_write(true);
            w.set_chain_to(ch.number());
            w.set_en(true);
        });

        compiler_fence(Ordering::SeqCst);
    }
    Transfer::new(ch)
}

pub(crate) struct Transfer<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: impl Peripheral<P = C> + 'a) -> Self {
        into_ref!(channel);

        unsafe {
            let irq = interrupt::DMA_IRQ_0::steal();
            irq.disable();
            irq.set_priority(interrupt::Priority::P6);

            pac::DMA.inte0().write(|w| w.set_inte0(1 << channel.number()));

            irq.enable();
        }

        Self { channel }
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        let p = self.channel.regs();
        unsafe {
            p.ctrl_trig().write(|w| w.set_en(false));
            while p.ctrl_trig().read().busy() {}
        }
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        CHANNEL_WAKERS[self.channel.number() as usize].register(cx.waker());

        if unsafe { self.channel.regs().ctrl_trig().read().en() } {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

const CHANNEL_COUNT: usize = 12;
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
