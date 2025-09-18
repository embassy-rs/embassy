//! double-buffered dma receive helpers
//!
//! overview: set up two dma channels that alternately write into two user-provided buffers.
//! the api exposes a stream-like interface: awaiting `next()` yields the next filled buffer.
//! dropping the yielded buffer guard re-queues that buffer for the next transfer. only rx is supported.

use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::Peri;
use futures_core::stream::Stream;

use crate::dma::{Channel, CHANNEL_COUNT};
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
pub struct RxBuf<'a, 's, C0: Channel, C1: Channel> {
    stream: &'a mut RxStream<'a, 's, C0, C1>,
    which: Which,
    // immutable slice view of completed data
    buf: &'s [u8],
}

impl<'a, 's, C0: Channel, C1: Channel> core::ops::Deref for RxBuf<'a, 's, C0, C1> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl<'a, 's, C0: Channel, C1: Channel> Drop for RxBuf<'a, 's, C0, C1> {
    fn drop(&mut self) {
        // re-queue this buffer. if the stream is already closed, do nothing.
        self.stream.on_buffer_released(self.which);
    }
}

/// double-buffered dma rx stream
pub struct RxStream<'d, 's, C0: Channel, C1: Channel> {
    ch_a: Peri<'d, C0>,
    ch_b: Peri<'d, C1>,

    // uart dr pointer and treq for the peripheral
    from_ptr: *const u32,
    dreq: TreqSel,

    // raw pointers to user buffers
    buf_a_ptr: *mut u8,
    buf_b_ptr: *mut u8,
    len_a: usize,
    len_b: usize,

    // state
    running: Option<Which>,
    in_user: Option<Which>,
    pending_complete: Option<Which>,
    next_to_fill: Which,
    closed: bool,
    overrun: bool,

    // !send by construction; dma regs are not send-safe
    _not_send: PhantomData<*const ()>,
}

impl<'d, 's, C0: Channel, C1: Channel> RxStream<'d, 's, C0, C1> {
    /// create a new rx stream for a peripheral register `from_ptr` and dreq.
    pub fn new(
        ch_a: Peri<'d, C0>,
        ch_b: Peri<'d, C1>,
        from_ptr: *const u32,
        dreq: TreqSel,
        buf_a: &'s mut [u8],
        buf_b: &'s mut [u8],
    ) -> Self {
        let mut s = Self {
            ch_a,
            ch_b,
            from_ptr,
            dreq,
            buf_a_ptr: buf_a.as_mut_ptr(),
            buf_b_ptr: buf_b.as_mut_ptr(),
            len_a: buf_a.len(),
            len_b: buf_b.len(),
            running: None,
            in_user: None,
            pending_complete: None,
            next_to_fill: Which::A,
            closed: false,
            overrun: false,
            _not_send: PhantomData,
        };

        // program both channels, chain to each other. start A only to kick off ping-pong.
        unsafe {
            s.program_channel(Which::A, true);
            s.program_channel(Which::B, false);
        }
        s.running = Some(Which::A);
        s.next_to_fill = Which::B;
        s
    }

    /// async convenience that yields the next filled buffer.
    pub async fn next(&mut self) -> Option<RxBuf<'_, 's, C0, C1>> {
        use embassy_futures::poll_fn;
        poll_fn(|cx| self.poll_next(cx)).await
    }

    /// called by `RxBuf::drop` to release the buffer and (re)start a transfer if possible.
    fn on_buffer_released(&mut self, which: Which) {
        if self.closed {
            return;
        }
        // mark user buffer as free
        if self.in_user == Some(which) {
            self.in_user = None;
        }
        // reprogram the released channel for the next round; do not enable it now.
        unsafe { self.program_channel(which, false) };
    }

    /// poll for next completed buffer.
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<RxBuf<'_, 's, C0, C1>>> {
        if self.closed {
            return Poll::Ready(None);
        }

        // register wakers on both channels. any completion will wake us.
        // safety: using the same waker for both is fine; irq wakes per-channel.
        let a_idx = self.ch_a.number() as usize;
        let b_idx = self.ch_b.number() as usize;
        assert!(a_idx < CHANNEL_COUNT && b_idx < CHANNEL_COUNT);
        super::CHANNEL_WAKERS[a_idx].register(cx.waker());
        super::CHANNEL_WAKERS[b_idx].register(cx.waker());

        // update completion state based on dma busy flags
        if self.pending_complete.is_none() {
            match self.running {
                Some(Which::A) => {
                    if !self.ch_a.regs().ctrl_trig().read().busy() {
                        // detect if this buffer is still in use by user -> overrun
                        if self.in_user == Some(Which::A) {
                            self.overrun = true;
                        }
                        self.pending_complete = Some(Which::A);
                        self.running = None;
                    }
                }
                Some(Which::B) => {
                    if !self.ch_b.regs().ctrl_trig().read().busy() {
                        if self.in_user == Some(Which::B) {
                            self.overrun = true;
                        }
                        self.pending_complete = Some(Which::B);
                        self.running = None;
                    }
                }
                None => {}
            }
        }

        // if we have a completed buffer and none is currently borrowed by user, yield it
        if let Some(which) = self.pending_complete.take() {
            if self.in_user.is_some() {
                // can't yield yet, wait until user drops previous buffer
                self.pending_complete = Some(which);
                return Poll::Pending;
            }

            // the other channel should already be running due to chain. record state.
            let other = match which { Which::A => Which::B, Which::B => Which::A };
            self.running = Some(other);
            self.next_to_fill = which;

            // build guard with immutable slice
            let buf: &'s [u8] = unsafe { self.slice_for(which) };
            self.in_user = Some(which);
            let guard = RxBuf { stream: self, which, buf };
            return Poll::Ready(Some(guard));
        }

        Poll::Pending
    }

    unsafe fn slice_for(&self, which: Which) -> &'s [u8] {
        match which {
            Which::A => core::slice::from_raw_parts(self.buf_a_ptr as *const u8, self.len_a),
            Which::B => core::slice::from_raw_parts(self.buf_b_ptr as *const u8, self.len_b),
        }
    }

    unsafe fn program_channel(&mut self, which: Which, enable: bool) {
        let (ch_this, wptr, len, ch_other_num) = match which {
            Which::A => (self.ch_a.reborrow(), self.buf_a_ptr, self.len_a, self.ch_b.number()),
            Which::B => (self.ch_b.reborrow(), self.buf_b_ptr, self.len_b, self.ch_a.number()),
        };
        let p = ch_this.regs();
        p.read_addr().write_value(self.from_ptr as u32);
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
            w.set_treq_sel(self.dreq);
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

impl<'d, 's, C0: Channel, C1: Channel> Drop for RxStream<'d, 's, C0, C1> {
    fn drop(&mut self) {
        self.closed = true;
        // abort both channels to stop transfers
        unsafe {
            pac::DMA
                .chan_abort()
                .modify(|m| m.set_chan_abort((1 << self.ch_a.number()) | (1 << self.ch_b.number())));
        }
    }
}

impl<'d, 's, C0: Channel, C1: Channel> Stream for RxStream<'d, 's, C0, C1> {
    type Item = RxBuf<'_, 's, C0, C1>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // safety: we never move fields that are not Unpin; we only delegate to inner method
        let this = unsafe { self.get_unchecked_mut() };
        this.poll_next(cx)
    }
}

impl<'d, 's, C0: Channel, C1: Channel> RxStream<'d, 's, C0, C1> {
    /// returns and clears the overrun flag. true means a buffer was overwritten while in use.
    pub fn take_overrun(&mut self) -> bool {
        let o = self.overrun;
        self.overrun = false;
        o
    }
}


