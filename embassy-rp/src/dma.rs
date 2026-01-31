//! Direct Memory Access (DMA)
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::{Context, Poll};

use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::vals::DataSize;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::dma::vals;
use crate::{RegExt, interrupt, pac, peripherals};

/// DMA interrupt handler.
pub struct InterruptHandler<T: ChannelInstance> {
    _phantom: PhantomData<T>,
}

impl<T: ChannelInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let channel = T::number() as usize;
        let ctrl_trig = pac::DMA.ch(channel).ctrl_trig().read();
        if ctrl_trig.ahb_error() {
            panic!("DMA: error on DMA_0 channel {}", channel);
        }

        let ints0 = pac::DMA.ints(0).read();
        if ints0 & (1 << channel) != 0 {
            CHANNEL_WAKERS[channel].wake();
        }
        pac::DMA.ints(0).write_value(1 << channel);
    }
}

pub(crate) unsafe fn init() {
    interrupt::DMA_IRQ_0.set_priority(interrupt::Priority::P3);
}

/// DMA channel driver.
pub struct Channel<'d> {
    number: u8,
    phantom: PhantomData<&'d ()>,
}

impl<'d> Channel<'d> {
    /// Create a new DMA channel driver.
    pub fn new<T: ChannelInstance>(
        _ch: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        let number = T::number();

        // Enable interrupt for this channel
        pac::DMA.inte(0).write_set(|v| *v = 1 << number);
        unsafe { T::Interrupt::enable() };

        Self {
            number,
            phantom: PhantomData,
        }
    }

    /// Get the channel number.
    pub(crate) fn number(&self) -> u8 {
        self.number
    }

    /// Get the channel register block.
    pub(crate) fn regs(&self) -> pac::dma::Channel {
        pac::DMA.ch(self.number as _)
    }

    /// Reborrow the channel, allowing it to be used in multiple places.
    pub fn reborrow(&mut self) -> Channel<'_> {
        Channel {
            number: self.number,
            phantom: PhantomData,
        }
    }

    unsafe fn configure(
        &self,
        from: *const u32,
        to: *mut u32,
        len: usize,
        data_size: DataSize,
        incr_read: bool,
        incr_write: bool,
        dreq: vals::TreqSel,
    ) {
        let p = self.regs();

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
            w.set_chain_to(self.number());
            w.set_en(true);
        });

        compiler_fence(Ordering::SeqCst);
    }

    /// DMA read from a peripheral to memory.
    ///
    /// SAFETY: Slice must point to a valid location reachable by DMA.
    pub unsafe fn read<'a, W: Word>(&'a mut self, from: *const W, to: *mut [W], dreq: vals::TreqSel) -> Transfer<'a> {
        self.configure(
            from as *const u32,
            to as *mut W as *mut u32,
            to.len(),
            W::size(),
            false,
            true,
            dreq,
        );
        Transfer::new(self.reborrow())
    }

    /// DMA write from memory to a peripheral.
    ///
    /// SAFETY: Slice must point to a valid location reachable by DMA.
    pub unsafe fn write<'a, W: Word>(&'a mut self, from: *const [W], to: *mut W, dreq: vals::TreqSel) -> Transfer<'a> {
        self.configure(
            from as *const W as *const u32,
            to as *mut u32,
            from.len(),
            W::size(),
            true,
            false,
            dreq,
        );
        Transfer::new(self.reborrow())
    }

    /// DMA repeated write of the same value from memory to a peripheral.
    ///
    /// SAFETY: `to` must point to a valid location reachable by DMA.
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        count: usize,
        to: *mut W,
        dreq: vals::TreqSel,
    ) -> Transfer<'a> {
        // static mut so that this is allocated in RAM.
        static mut DUMMY: u32 = 0;

        self.configure(
            core::ptr::addr_of_mut!(DUMMY) as *const u32,
            to as *mut u32,
            count,
            W::size(),
            false,
            false,
            dreq,
        );
        Transfer::new(self.reborrow())
    }

    /// DMA copy between memory regions.
    ///
    /// SAFETY: Slices must point to locations reachable by DMA.
    pub unsafe fn copy<'a, W: Word>(&'a mut self, from: &[W], to: &mut [W]) -> Transfer<'a> {
        let from_len = from.len();
        let to_len = to.len();
        assert_eq!(from_len, to_len);
        self.configure(
            from.as_ptr() as *const u32,
            to.as_mut_ptr() as *mut u32,
            from_len,
            W::size(),
            true,
            true,
            vals::TreqSel::PERMANENT,
        );
        Transfer::new(self.reborrow())
    }
}

/// DMA transfer driver.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a> {
    channel: Channel<'a>,
}

impl<'a> Transfer<'a> {
    pub(crate) fn new(channel: Channel<'a>) -> Self {
        Self { channel }
    }
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        let p = self.channel.regs();
        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort(1 << self.channel.number()));
        while p.ctrl_trig().read().busy() {}
    }
}

impl<'a> Unpin for Transfer<'a> {}
impl<'a> Future for Transfer<'a> {
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

trait SealedChannelInstance {}
trait SealedWord {}

/// DMA channel instance trait.
#[allow(private_bounds)]
pub trait ChannelInstance: PeripheralType + SealedChannelInstance + Sized + 'static {
    /// The interrupt type for this DMA channel.
    type Interrupt: interrupt::typelevel::Interrupt;

    /// Channel number.
    fn number() -> u8;

    /// Channel registry block.
    fn regs() -> pac::dma::Channel {
        pac::DMA.ch(Self::number() as _)
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

macro_rules! channel {
    ($name:ident, $num:expr, $irq:ident) => {
        impl SealedChannelInstance for peripherals::$name {}
        impl ChannelInstance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$irq;

            fn number() -> u8 {
                $num
            }
        }
    };
}

channel!(DMA_CH0, 0, DMA_IRQ_0);
channel!(DMA_CH1, 1, DMA_IRQ_0);
channel!(DMA_CH2, 2, DMA_IRQ_0);
channel!(DMA_CH3, 3, DMA_IRQ_0);
channel!(DMA_CH4, 4, DMA_IRQ_0);
channel!(DMA_CH5, 5, DMA_IRQ_0);
channel!(DMA_CH6, 6, DMA_IRQ_0);
channel!(DMA_CH7, 7, DMA_IRQ_0);
channel!(DMA_CH8, 8, DMA_IRQ_0);
channel!(DMA_CH9, 9, DMA_IRQ_0);
channel!(DMA_CH10, 10, DMA_IRQ_0);
channel!(DMA_CH11, 11, DMA_IRQ_0);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH12, 12, DMA_IRQ_0);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH13, 13, DMA_IRQ_0);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH14, 14, DMA_IRQ_0);
#[cfg(feature = "_rp235x")]
channel!(DMA_CH15, 15, DMA_IRQ_0);
