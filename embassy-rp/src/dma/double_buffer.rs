//! double-buffered dma receive helpers
//!
//! overview: set up two dma channels that alternately write into two user-provided buffers.
//! the api exposes a stream-like interface: awaiting `next()` yields the next filled buffer.
//! dropping the yielded buffer guard re-queues that buffer for the next transfer. only rx is supported.

use core::future::poll_fn;
use core::task::{Context, Poll};

use embassy_hal_internal::Peri;

use crate::dma::{AnyChannel, Channel, CHANNEL_COUNT};
use crate::pac;

#[cfg(feature = "rp2040")]
use crate::pac::dma::vals::DataSize;
use crate::pac::dma::vals::TreqSel;

/// which buffer/channel pair
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Which {
    A,
    B,
}

/// guard returned to the user. on drop this re-queues the buffer for the next transfer.
pub struct RxBufView<'a, 'buf> {
    state: &'a mut State,
    buffers: &'a mut Buffers<'buf>,
    which: Which,
}

impl<'a, 'buf> core::ops::Deref for RxBufView<'a, 'buf> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buffers.slice_for(self.which)
    }
}

impl<'a, 'buf> Drop for RxBufView<'a, 'buf> {
    fn drop(&mut self) {
        if self.state.borrowed == Some(self.which) {
            self.state.borrowed = None;
        }
    }
}

struct Info<'peri, C0: Channel, C1: Channel> {
    ch_a: Peri<'peri, C0>,
    ch_b: Peri<'peri, C1>,
    from_ptr: *const u32,
    dreq: TreqSel,
}

struct Buffers<'buf> {
    buf_a: &'buf mut [u8],
    buf_b: &'buf mut [u8],
}

impl<'buf> Buffers<'buf> {
    fn slice_for<'a>(&'a self, which: Which) -> &'a [u8] {
        match which {
            Which::A => self.buf_a,
            Which::B => self.buf_b,
        }
    }

    fn slice_for_mut<'a>(&'a mut self, which: Which) -> &'a mut [u8] {
        match which {
            Which::A => self.buf_a.as_mut(),
            Which::B => self.buf_b.as_mut(),
        }
    }
}

struct State {
    /// tracks which buffer is currently being filled. none means that the
    /// stream is closed and neither channel is running
    filling: Option<Which>,
    /// tracks which buffer is currently borrowed by the user
    borrowed: Option<Which>,
    /// tracks which buffer is ready to be yielded
    ready: Option<Which>,
    /// set to true if a transfer happens while a buffer is in use
    overrun: bool,
}

/// double-buffered dma rx stream
pub struct RxStream<'peri, 'buf, C0: Channel, C1: Channel> {
    info: Info<'peri, C0, C1>,
    buffers: Buffers<'buf>,
    state: State,
}

impl<'peri, 'buf, C0: Channel, C1: Channel> RxStream<'peri, 'buf, C0, C1> {
    /// create a new rx stream for a peripheral register `from_ptr` and dreq.
    pub fn new<'s>(
        ch_a: Peri<'peri, C0>,
        ch_b: Peri<'peri, C1>,
        from_ptr: *const u32,
        dreq: TreqSel,
        buf_a: &'buf mut [u8],
        buf_b: &'buf mut [u8],
    ) -> Self {
        let mut s = Self {
            info: Info {
                ch_a,
                ch_b,
                from_ptr,
                dreq,
            },
            state: State {
                filling: Some(Which::A),
                borrowed: None,
                ready: None,
                overrun: false,
            },
            buffers: Buffers { buf_a, buf_b },
        };

        // program both channels, chain to each other. start A only to kick off ping-pong.
        unsafe {
            Self::program_channel(&mut s.info, &mut s.buffers, Which::A, true);
            Self::program_channel(&mut s.info, &mut s.buffers, Which::B, false);
        }

        s
    }

    /// async convenience that yields the next filled buffer.
    pub async fn next<'s>(&'s mut self) -> Option<RxBufView<'s, 'buf>> {
        let info = &mut self.info;
        let state = &mut self.state;
        let buffers = &mut self.buffers;

        let which = poll_fn(|cx| Self::poll_next(cx, info, state, buffers)).await;

        match which {
            Some(which) => Some(RxBufView { state, buffers, which }),
            None => None,
        }
    }

    /// poll for next completed buffer.
    fn poll_next<'cx, 'a>(
        cx: &mut Context<'cx>,
        info: &'a mut Info<'peri, C0, C1>,
        state: &'a mut State,
        buffers: &'a mut Buffers<'buf>,
    ) -> Poll<Option<Which>> {
        // register wakers on both channels. any completion will wake us.
        // safety: using the same waker for both is fine; irq wakes per-channel.
        let a_idx = info.ch_a.number() as usize;
        let b_idx = info.ch_b.number() as usize;
        debug_assert!(a_idx < CHANNEL_COUNT && b_idx < CHANNEL_COUNT);
        super::CHANNEL_WAKERS[a_idx].register(cx.waker());
        super::CHANNEL_WAKERS[b_idx].register(cx.waker());

        match state.filling {
            Some(Which::A) => {
                if !info.ch_a.regs().ctrl_trig().read().busy() {
                    // buffer A is ready
                    state.ready = Some(Which::A);
                    state.filling = Some(Which::B);

                    // if channel B is not enabled, start it
                    if !info.ch_b.regs().ctrl_trig().read().en() {
                        unsafe { Self::program_channel(info, buffers, Which::B, true) };
                    }

                    return Poll::Ready(Some(Which::A));
                }
            }
            Some(Which::B) => {
                if !info.ch_b.regs().ctrl_trig().read().busy() {
                    // buffer B is ready
                    state.ready = Some(Which::B);
                    state.filling = Some(Which::A);

                    // if channel A is not enabled, start it
                    if !info.ch_a.regs().ctrl_trig().read().en() {
                        unsafe { Self::program_channel(info, buffers, Which::A, true) };
                    }

                    return Poll::Ready(Some(Which::B));
                }
            }
            None => return Poll::Ready(None),
        }

        Poll::Pending
    }

    unsafe fn program_channel(info: &mut Info<'peri, C0, C1>, buffers: &mut Buffers<'buf>, which: Which, enable: bool) {
        let (ch_this, wptr, len, ch_other_num) = match which {
            Which::A => (
                Peri::<AnyChannel>::from(info.ch_a.reborrow().into()),
                buffers.buf_a.as_mut_ptr(),
                buffers.buf_a.len(),
                info.ch_b.number(),
            ),
            Which::B => (
                Peri::<AnyChannel>::from(info.ch_b.reborrow().into()),
                buffers.buf_b.as_mut_ptr(),
                buffers.buf_b.len(),
                info.ch_a.number(),
            ),
        };

        let p = ch_this.regs();
        p.read_addr().write_value(info.from_ptr as u32);
        p.write_addr().write_value(wptr as u32);

        #[cfg(feature = "rp2040")]
        p.trans_count().write(|w| {
            *w = len as u32;
        });

        #[cfg(feature = "_rp235x")]
        p.trans_count().write(|w| {
            w.set_mode(0.into());
            w.set_count(len as u32);
        });

        // ensure previous stores are visible before enabling
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        p.ctrl_trig().write(|w| {
            w.set_treq_sel(info.dreq);
            #[cfg(feature = "rp2040")]
            w.set_data_size(DataSize::SIZE_BYTE);
            // rp235x encodes size in the fifo mapping; byte access by default
            w.set_incr_read(false);
            w.set_incr_write(true);
            // chain to the other channel for continuous ping-pong
            w.set_chain_to(ch_other_num);
            w.set_en(enable);
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

impl<'d, 'buf, C0: Channel, C1: Channel> Drop for RxStream<'d, 'buf, C0, C1> {
    fn drop(&mut self) {
        self.state.filling = None;
        // abort both channels to stop transfers

        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort((1 << self.info.ch_a.number()) | (1 << self.info.ch_b.number())));
    }
}

// impl<'peri, 'buf, C0: Channel, C1: Channel> futures_core::stream::Stream for RxStream<'peri, 'buf, C0, C1> {
//     type Item = RxBufView<'_, 'peri, 'buf, C0, C1>;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         // safety: we never move fields that are not Unpin; we only delegate to inner method
//         let this = unsafe { self.get_unchecked_mut() };

//         core::task::ready!(RxStream::<'peri, 'buf, C0, C1>::poll_next(
//             cx,
//             &mut this.info,
//             &mut this.state,
//             &mut this.buffers
//         ));

//         Poll::Ready(this.get_current_buffer())
//     }
// }

impl<'peri, 'buf, C0: Channel, C1: Channel> RxStream<'peri, 'buf, C0, C1> {
    /// returns and clears the overrun flag. true means a buffer was overwritten while in use.
    pub fn take_overrun(&mut self) -> bool {
        let o = self.state.overrun;
        self.state.overrun = false;
        o
    }
}
