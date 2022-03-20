#[cfg(bdma)]
pub(crate) mod bdma;
#[cfg(dma)]
pub(crate) mod dma;
#[cfg(dmamux)]
mod dmamux;

#[cfg(dmamux)]
pub use dmamux::*;

use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::Waker;
use core::task::{Context, Poll};
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::transfers::*;
}

pub(crate) use transfers::*;

#[cfg(any(bdma_v2, dma_v2, dmamux))]
pub type Request = u8;
#[cfg(not(any(bdma_v2, dma_v2, dmamux)))]
pub type Request = ();

pub(crate) mod sealed {
    use super::*;

    pub trait Word {}

    pub trait Channel {
        /// Starts this channel for writing a stream of words.
        ///
        /// Safety:
        /// - `buf` must point to a valid buffer for DMA reading.
        /// - `buf` must be alive for the entire duration of the DMA transfer.
        /// - `reg_addr` must be a valid peripheral register address to write to.
        unsafe fn start_write<W: super::Word>(
            &mut self,
            request: Request,
            buf: *const [W],
            reg_addr: *mut W,
            options: TransferOptions,
        );

        /// Starts this channel for writing a word repeatedly.
        ///
        /// Safety:
        /// - `reg_addr` must be a valid peripheral register address to write to.
        unsafe fn start_write_repeated<W: super::Word>(
            &mut self,
            request: Request,
            repeated: W,
            count: usize,
            reg_addr: *mut W,
            options: TransferOptions,
        );

        /// Starts this channel for reading a stream of words.
        ///
        /// Safety:
        /// - `buf` must point to a valid buffer for DMA writing.
        /// - `buf` must be alive for the entire duration of the DMA transfer.
        /// - `reg_addr` must be a valid peripheral register address to read from.
        unsafe fn start_read<W: super::Word>(
            &mut self,
            request: Request,
            reg_addr: *const W,
            buf: *mut [W],
            options: TransferOptions,
        );

        /// Requests the channel to stop.
        /// NOTE: The channel does not immediately stop, you have to wait
        /// for `is_running() = false`.
        fn request_stop(&mut self);

        /// Returns whether this channel is running or stopped.
        ///
        /// The channel stops running when it either completes or is manually stopped.
        fn is_running(&self) -> bool;

        /// Returns the total number of remaining transfers.
        fn remaining_transfers(&mut self) -> u16;

        /// Sets the waker that is called when this channel stops (either completed or manually stopped)
        fn set_waker(&mut self, waker: &Waker);

        /// This is called when this channel triggers an interrupt.
        /// Note: Because some channels share an interrupt, this function might be
        /// called for a channel that didn't trigger an interrupt.
        fn on_irq();
    }
}

pub enum WordSize {
    OneByte,
    TwoBytes,
    FourBytes,
}
pub trait Word: sealed::Word {
    fn bits() -> WordSize;
}

impl sealed::Word for u8 {}
impl Word for u8 {
    fn bits() -> WordSize {
        WordSize::OneByte
    }
}

impl sealed::Word for u16 {}
impl Word for u16 {
    fn bits() -> WordSize {
        WordSize::TwoBytes
    }
}

impl sealed::Word for u32 {}
impl Word for u32 {
    fn bits() -> WordSize {
        WordSize::FourBytes
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum FlowControl {
    /// Flow control by DMA
    Dma,
    /// Flow control by peripheral
    Peripheral,
}

pub struct TransferOptions {
    /// Peripheral burst transfer configuration
    pub pburst: Burst,
    /// Memory burst transfer configuration
    pub mburst: Burst,
    /// Flow control configuration
    pub flow_ctrl: FlowControl,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            pburst: Burst::Single,
            mburst: Burst::Single,
            flow_ctrl: FlowControl::Dma,
        }
    }
}

mod transfers {
    use super::*;

    #[allow(unused)]
    pub fn read<'a, W: Word>(
        channel: impl Unborrow<Target = impl Channel> + 'a,
        request: Request,
        reg_addr: *mut W,
        buf: &'a mut [W],
    ) -> impl Future<Output = ()> + 'a {
        assert!(buf.len() > 0 && buf.len() <= 0xFFFF);
        unborrow!(channel);

        unsafe { channel.start_read::<W>(request, reg_addr, buf, Default::default()) };

        Transfer::new(channel)
    }

    #[allow(unused)]
    pub fn write<'a, W: Word>(
        channel: impl Unborrow<Target = impl Channel> + 'a,
        request: Request,
        buf: &'a [W],
        reg_addr: *mut W,
    ) -> impl Future<Output = ()> + 'a {
        assert!(buf.len() > 0 && buf.len() <= 0xFFFF);
        unborrow!(channel);

        unsafe { channel.start_write::<W>(request, buf, reg_addr, Default::default()) };

        Transfer::new(channel)
    }

    #[allow(unused)]
    pub fn write_repeated<'a, W: Word>(
        channel: impl Unborrow<Target = impl Channel> + 'a,
        request: Request,
        repeated: W,
        count: usize,
        reg_addr: *mut W,
    ) -> impl Future<Output = ()> + 'a {
        unborrow!(channel);

        unsafe {
            channel.start_write_repeated::<W>(
                request,
                repeated,
                count,
                reg_addr,
                Default::default(),
            )
        };

        Transfer::new(channel)
    }

    pub(crate) struct Transfer<'a, C: Channel> {
        channel: C,
        _phantom: PhantomData<&'a mut C>,
    }

    impl<'a, C: Channel> Transfer<'a, C> {
        pub(crate) fn new(channel: impl Unborrow<Target = C> + 'a) -> Self {
            unborrow!(channel);
            Self {
                channel,
                _phantom: PhantomData,
            }
        }
    }

    impl<'a, C: Channel> Drop for Transfer<'a, C> {
        fn drop(&mut self) {
            self.channel.request_stop();
            while self.channel.is_running() {}
        }
    }

    impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
    impl<'a, C: Channel> Future for Transfer<'a, C> {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.channel.set_waker(cx.waker());
            if self.channel.is_running() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }
}

pub trait Channel: sealed::Channel + Unborrow<Target = Self> + 'static {}

pub struct NoDma;

unsafe impl Unborrow for NoDma {
    type Target = NoDma;

    unsafe fn unborrow(self) -> Self::Target {
        self
    }
}

// safety: must be called only once at startup
pub(crate) unsafe fn init() {
    #[cfg(bdma)]
    bdma::init();
    #[cfg(dma)]
    dma::init();
    #[cfg(dmamux)]
    dmamux::init();
}

// TODO: replace transmutes with core::ptr::metadata once it's stable
#[allow(unused)]
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}

#[allow(unused)]
pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (usize, usize) {
    unsafe { mem::transmute(slice) }
}
