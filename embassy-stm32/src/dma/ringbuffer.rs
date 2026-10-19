use core::future::poll_fn;
use core::sync::atomic::{Ordering, fence};
use core::task::{Poll, Waker};

use crate::dma::word::Word;

pub trait DmaCtrl {
    /// Get the NDTR register value, i.e. the space left in the underlying
    /// buffer until the dma writer wraps.
    fn get_remaining_transfers(&self) -> usize;

    /// Reset the transfer completed counter to 0 and return the value just prior to the reset.
    fn reset_complete_count(&mut self) -> usize;

    /// Set the waker for a running poll_fn
    fn set_waker(&mut self, waker: &Waker);
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Overrun,
    /// the newly read DMA positions don't make sense compared to the previous
    /// ones. This can usually only occur due to wrong Driver implementation, if
    /// the driver author (or the user using raw metapac code) directly resets
    /// the channel for instance.
    DmaUnsynced,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct DmaIndex {
    complete_count: usize,
    pos: usize,
}

impl DmaIndex {
    /// Reset the index to the start of the buffer (position 0, lap 0).
    fn reset(&mut self) {
        self.pos = 0;
        self.complete_count = 0;
    }

    /// Convert the index to a physical buffer offset, applying `offset` additional
    /// steps and wrapping at `cap`.
    fn as_index(&self, cap: usize, offset: usize) -> usize {
        (self.pos + offset) % cap
    }

    /// Synchronise the index against the live DMA hardware state.
    fn dma_sync(&mut self, cap: usize, dma: &mut impl DmaCtrl) {
        // The ordering of the these lines matters.
        // FIRST reset complete count, THEN read remaining transfers.
        //
        // With this order when DMA wraps between these two reads,
        // `laps_completed` will be 0 while `pos` appears to go
        // backwards comparing to the previous value in `self.pos`.
        // This ensures that the approximated position never goes ahead of
        // the real DMA position.
        //
        // If the order was reversed and DMA wrapped between these two reads,
        // `laps_completed` will be 1 while `pos` has a value before wrap.
        // This is not acceptable since the approximated position will jump
        // ahead of the real DMA position.
        let laps_completed = dma.reset_complete_count();
        let pos = cap - dma.get_remaining_transfers();
        // It is also not acceptable for approximated position to go backward, so it
        // must be modified such that it is at least not behind previous position.
        // This also happens when the DMA wrapped but the transfer complete interrupt
        // hasn't run yet. Compare against the previous position including the laps
        // just counted: `self.pos` may still be extended from a previous wrap.
        self.pos = if laps_completed * cap + pos < self.pos {
            // `self.complete_count` cannot be manually incremented here because in
            // next call to this function `laps_completed` will be 1, so it would
            // be applied twice. However `self.pos` can temporarily be extended
            // beyond normal range. The next call will set it back to normal value
            // and increment `self.complete_count` compensating `cap` added here.
            pos + cap
        } else {
            pos
        };
        self.complete_count += laps_completed;
    }

    /// Advance the index by `steps` words, incrementing `complete_count` for
    /// each full lap completed.
    fn advance(&mut self, cap: usize, steps: usize) {
        let next = self.pos + steps;
        self.complete_count += next / cap;
        self.pos = next % cap;
    }

    /// Subtract the smaller `complete_count` from both indices so that the
    /// absolute lap counts stay small. This prevents overflow in [`diff`] after
    /// many laps while leaving the relative difference unchanged.
    fn normalize(lhs: &mut DmaIndex, rhs: &mut DmaIndex) {
        let min_count = lhs.complete_count.min(rhs.complete_count);
        lhs.complete_count -= min_count;
        rhs.complete_count -= min_count;
    }

    /// Return how many words `self` is ahead of `rhs` in linear (unwrapped)
    /// space. A negative result means `self` is behind `rhs`, which indicates
    /// a driver bug or an out-of-band DMA reset.
    fn diff(&self, cap: usize, rhs: &DmaIndex) -> isize {
        self.total(cap) as isize - rhs.total(cap) as isize
    }

    fn total(&self, cap: usize) -> usize {
        self.complete_count * cap + self.pos
    }
}

pub struct ReadableDmaRingBuffer<'a, W: Word> {
    dma_buf: &'a mut [W],
    write_index: DmaIndex,
    read_index: DmaIndex,
    alignment: usize,
}

impl<'a, W: Word> ReadableDmaRingBuffer<'a, W> {
    /// Construct an empty buffer.
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        Self {
            dma_buf,
            write_index: Default::default(),
            read_index: Default::default(),
            alignment: 1,
        }
    }

    /// Set the frame alignment for the ring buffer.
    ///
    /// When set to a value > 1, the ring buffer will automatically discard partial
    /// frames after overrun recovery to maintain alignment. This is critical for
    /// protocols like I2S where each frame consists of multiple DMA transfers
    /// (e.g. 4 half-words for stereo 32-bit I2S) and reading from a mid-frame
    /// position produces garbage data.
    ///
    /// The DMA buffer length must be a multiple of the alignment value.
    pub fn set_alignment(&mut self, alignment: usize) {
        let alignment = alignment.max(1);
        assert!(
            self.cap().is_multiple_of(alignment),
            "DMA buffer length must be a multiple of the alignment value"
        );
        assert!(
            self.cap() >= alignment * 2,
            "DMA buffer must hold at least 2 frames (cap >= 2 * alignment); \
             with only one frame there is no safe read window between HTIF and TCIF"
        );
        self.alignment = alignment;
    }

    /// Reset the ring buffer to its initial state.
    pub fn reset(&mut self, dma: &mut impl DmaCtrl) {
        dma.reset_complete_count();
        self.write_index.reset();
        self.write_index.dma_sync(self.cap(), dma);
        self.read_index = self.write_index;
    }

    /// Get the full ringbuffer capacity.
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Sync against the DMA hardware and return the number of readable samples available.
    pub fn sync_len(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        self.write_index.dma_sync(self.cap(), dma);
        DmaIndex::normalize(&mut self.write_index, &mut self.read_index);

        let diff = self.write_index.diff(self.cap(), &self.read_index);

        if diff < 0 {
            Err(Error::DmaUnsynced)
        } else if diff > self.cap() as isize {
            Err(Error::Overrun)
        } else {
            Ok(diff as usize)
        }
    }

    /// Read elements from the ring buffer.
    ///
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.sync_len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller,
    /// in which case the rinbuffer will automatically reset itself.
    pub fn read(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.read_raw(dma, buf).inspect_err(|_e| {
            self.reset(dma);
        })
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &mut [W]) -> Result<usize, Error> {
        let mut read_data = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            match self.read(dma, &mut buffer[read_data..]) {
                Ok((len, remaining)) => {
                    read_data += len;
                    if read_data == buffer_len {
                        Poll::Ready(Ok(remaining))
                    } else {
                        Poll::Pending
                    }
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    fn read_raw(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), Error> {
        fence(Ordering::Acquire);

        let mut available = self.sync_len(dma)?;

        // Compute alignment skip without advancing read_index yet.
        // DMA always starts at buffer position 0 (frame-aligned) and advances
        // sequentially, so position N is frame-aligned when N % alignment == 0.
        let skip = if self.alignment > 1 {
            let misalignment = self.read_index.pos % self.alignment;
            if misalignment != 0 {
                let skip = self.alignment - misalignment;
                if available >= skip {
                    available -= skip;
                    skip
                } else {
                    return Ok((0, available));
                }
            } else {
                0
            }
        } else {
            0
        };

        let mut readable = available.min(buf.len());
        // Round down to alignment so read_index always lands on a frame boundary.
        if self.alignment > 1 {
            readable -= readable % self.alignment;
        }
        // Snapshot write_index before copying so the returned `remaining` reflects
        // the buffer state at the time of the read rather than requiring a second
        // hardware NDTR access and complete_count reset mid-copy.
        let write_snapshot = self.write_index;

        // Copy data starting skip words ahead of current read_index.
        for i in 0..readable {
            buf[i] = self.read_buf(skip + i);
        }

        // Commit both the alignment skip and the read in one advance, so
        // read_index is never in a partially-advanced state on error.
        self.read_index.advance(self.cap(), skip + readable);

        let remaining = write_snapshot.diff(self.cap(), &self.read_index).max(0) as usize;
        Ok((readable, remaining))
    }

    /// Read the most recent elements from the ring buffer, discarding any older data.
    ///
    /// Returns the number of elements actually read into `buf`. This may be less than
    /// `buf.len()` if fewer samples are available (e.g. the DMA has not yet filled enough
    /// data since the last read).
    ///
    /// Unlike [`read`], this method **never returns an overrun error**. If the DMA has
    /// lapped the read pointer, the read pointer is silently advanced to catch up, and
    /// only the most recent data is returned. This makes it ideal for use cases like ADC
    /// sampling where the consumer only cares about the latest values and old data can
    /// be safely discarded.
    ///
    /// If an `alignment` has been set, the returned count is rounded down to a multiple
    /// of the alignment and reading starts from a frame-aligned position.
    pub fn read_latest(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> usize {
        fence(Ordering::Acquire);

        let available = self.sync_len(dma).unwrap_or_else(|err| {
            self.read_index = self.write_index;
            match err {
                Error::Overrun => {
                    // the entire buffer contains the latest data
                    self.write_index.complete_count += 1;
                    self.cap()
                }
                Error::DmaUnsynced => {
                    #[cfg(feature = "defmt")]
                    defmt::error!("Ring buffer broken invariants detected!");
                    0
                }
            }
        });

        // Respect frame alignment. Because read_latest reads the NEWEST data
        // (skip at the front, read at the tail), reducing to_read moves the
        // start forward. We must compute front_skip explicitly so the read
        // window starts at an aligned buffer position.
        let (to_read, front_skip) = if self.alignment > 1 {
            // Skip any partial frame at the tail of available data, then
            // round down to_read so it fits in buf and lands on a frame boundary.
            let end_pos = self.read_index.as_index(self.cap(), available);
            let aligned_available = available.saturating_sub(end_pos % self.alignment);
            let to_read = aligned_available.min(buf.len());
            let to_read = to_read - to_read % self.alignment;
            (to_read, aligned_available - to_read)
        } else {
            let to_read = available.min(buf.len());
            (to_read, available - to_read)
        };

        // Advance past old data to the aligned start position.
        if front_skip > 0 {
            self.read_index.advance(self.cap(), front_skip);
        }

        for i in 0..to_read {
            buf[i] = self.read_buf(i);
        }

        // Advance past what we read. Trailing partial frame is left in the buffer.
        self.read_index.advance(self.cap(), to_read);

        to_read
    }

    fn read_buf(&self, offset: usize) -> W {
        unsafe { core::ptr::read_volatile(self.dma_buf.as_ptr().add(self.read_index.as_index(self.cap(), offset))) }
    }
}

pub struct WritableDmaRingBuffer<'a, W: Word> {
    dma_buf: &'a mut [W],
    read_index: DmaIndex,
    write_index: DmaIndex,
}

impl<'a, W: Word> WritableDmaRingBuffer<'a, W> {
    /// Construct an empty ring buffer.
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        Self {
            dma_buf,
            read_index: Default::default(),
            write_index: DmaIndex {
                complete_count: 0,
                pos: 0,
            },
        }
    }

    /// Reset the ring buffer to its initial state.
    pub fn reset(&mut self, dma: &mut impl DmaCtrl) {
        _ = dma.reset_complete_count();
        self.read_index.reset();
        self.read_index.dma_sync(self.cap(), dma);
        self.write_index = self.read_index;
    }

    /// Return the current write position (index into the DMA buffer where the next CPU write will go).
    ///
    /// Immediately after an overrun reset, this equals the DMA position recorded at reset time.
    /// Use this to compute frame-alignment padding after a TX write error without the timing
    /// uncertainty of reading the DMA NDTR register externally.
    pub fn write_pos(&self) -> usize {
        self.write_index.pos
    }

    /// Sync against the DMA hardware and return the number of writable samples available.
    pub fn sync_len(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        self.read_index.dma_sync(self.cap(), dma);
        DmaIndex::normalize(&mut self.read_index, &mut self.write_index);

        let diff = self.write_index.diff(self.cap(), &self.read_index);

        if diff < 0 {
            Err(Error::Overrun)
        } else if diff > self.cap() as isize {
            Err(Error::DmaUnsynced)
        } else {
            Ok(self.cap().saturating_sub(diff as usize))
        }
    }

    /// Get the full ringbuffer capacity.
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Append data to the ring buffer.
    /// Returns a tuple of the data written and the remaining write capacity in the buffer.
    /// Error is returned if the portion to be written was previously read by the DMA controller.
    /// In this case, the ringbuffer will automatically reset itself, giving a full buffer worth of
    /// leeway between the write index and the DMA.
    pub fn write(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), Error> {
        self.write_raw(dma, buf).inspect_err(|_e| {
            self.reset(dma);
        })
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            match self.sync_len(dma) {
                Ok(_) => Poll::Pending,
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    /// Write all elements of `buffer` to the ring buffer, waiting asynchronously
    /// if there is insufficient space.
    ///
    /// Wakes each time the DMA raises a half-transfer or transfer-complete interrupt,
    /// so up to one interrupt period of latency may occur between partial writes.
    ///
    /// Returns the remaining write capacity as reported by the final [`write`] call.
    /// This is a snapshot from the last partial write and may be slightly stale.
    ///
    /// Returns [`Error::Overrun`] if the DMA read pointer overtook the write pointer
    /// mid-write. In this case the ring buffer resets itself and any partially
    /// written data is lost.
    #[allow(dead_code)]
    pub async fn write_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &[W]) -> Result<usize, Error> {
        let mut written_len = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            match self.write(dma, &buffer[written_len..buffer_len]) {
                Ok((len, remaining)) => {
                    written_len += len;
                    if written_len == buffer_len {
                        Poll::Ready(Ok(remaining))
                    } else {
                        Poll::Pending
                    }
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    /// Write as many elements as currently fit, returning `(written, remaining_capacity)`.
    /// Does not block; call `write` for automatic reset-on-overrun or `write_exact` to write all.
    fn write_raw(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), Error> {
        fence(Ordering::Release);

        let writable = self.sync_len(dma)?.min(buf.len());
        // Snapshot read_index before the copy so that the returned `remaining`
        // reflects the buffer state at the time of the write rather than
        // requiring a second hardware NDTR access and complete_count reset mid-copy.
        let read_snapshot = self.read_index;

        for i in 0..writable {
            self.write_buf(i, buf[i]);
        }

        self.write_index.advance(self.cap(), writable);

        let remaining = self
            .cap()
            .saturating_sub(self.write_index.diff(self.cap(), &read_snapshot) as usize);
        Ok((writable, remaining))
    }

    fn write_buf(&mut self, offset: usize, value: W) {
        unsafe {
            core::ptr::write_volatile(
                self.dma_buf
                    .as_mut_ptr()
                    .add(self.write_index.as_index(self.cap(), offset)),
                value,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use core::future::Future;
    use core::ops::Range;
    use core::pin::{Pin, pin};
    use core::task::Context;
    use std::boxed::Box;
    use std::collections::VecDeque;
    use std::vec;
    use std::vec::Vec;

    use super::*;

    const CAP: usize = 16;

    /// Simulated circular DMA channel.
    #[derive(Default)]
    struct Dma {
        /// Position in the buffer, i.e. `CAP - NDTR`.
        pos: Cell<usize>,
        /// Transfers done since start.
        total: Cell<usize>,
        /// Laps counted by the transfer-complete interrupt, not yet taken by `reset_complete_count`.
        laps: Cell<usize>,
        /// Laps whose transfer-complete interrupt hasn't run yet.
        pending_laps: Cell<usize>,
        /// Don't run the transfer-complete interrupt until `irq()` is called.
        defer_irq: bool,
        /// At the end of a lap, NDTR reads 0 until the next transfer reloads it.
        lazy_reload: bool,
        /// Transfers to do right after the next register access, i.e. between the two register reads of a sync.
        race: Cell<usize>,
        /// Transfers to do each time an async method polls, one entry per poll.
        polls: VecDeque<usize>,
    }

    impl Dma {
        fn advance(&self, n: usize) {
            for _ in 0..n {
                if self.pos.get() == CAP {
                    self.wrap();
                }
                self.pos.set(self.pos.get() + 1);
                self.total.set(self.total.get() + 1);
                if self.pos.get() == CAP && !self.lazy_reload {
                    self.wrap();
                }
            }
        }

        fn wrap(&self) {
            self.pos.set(0);
            if self.defer_irq {
                self.pending_laps.set(self.pending_laps.get() + 1);
            } else {
                self.laps.set(self.laps.get() + 1);
            }
        }

        /// Run the deferred transfer-complete interrupt.
        fn irq(&self) {
            self.laps.set(self.laps.get() + self.pending_laps.take());
        }
    }

    impl DmaCtrl for Dma {
        fn get_remaining_transfers(&self) -> usize {
            let remaining = CAP - self.pos.get();
            self.advance(self.race.take());
            remaining
        }

        fn reset_complete_count(&mut self) -> usize {
            let laps = self.laps.take();
            self.advance(self.race.take());
            laps
        }

        fn set_waker(&mut self, _waker: &Waker) {
            let n = self.polls.pop_front().unwrap_or(0);
            self.advance(n);
        }
    }

    fn seq(r: Range<usize>) -> Vec<u32> {
        r.map(|i| i as u32).collect()
    }

    fn new_buf() -> &'static mut [u32] {
        Box::leak(Box::new([u32::MAX; CAP]))
    }

    fn poll<F: Future>(f: Pin<&mut F>) -> Poll<F::Output> {
        f.poll(&mut Context::from_waker(Waker::noop()))
    }

    /// Readable ring buffer filled by a simulated DMA. The DMA's `i`th transfer writes the value `i`.
    struct Reader {
        rb: ReadableDmaRingBuffer<'static, u32>,
        dma: Dma,
    }

    impl Reader {
        fn new() -> Self {
            Self::with_dma(Dma::default())
        }

        fn with_dma(dma: Dma) -> Self {
            Self {
                rb: ReadableDmaRingBuffer::new(new_buf()),
                dma,
            }
        }

        fn aligned() -> Self {
            let mut t = Self::new();
            t.rb.set_alignment(4);
            t
        }

        /// Write the data for the DMA's next `n` transfers, without advancing it.
        fn prepare(&mut self, n: usize) {
            for i in 0..n {
                self.rb.dma_buf[(self.dma.pos.get() + i) % CAP] = (self.dma.total.get() + i) as u32;
            }
        }

        /// DMA transfers `n` samples.
        fn dma(&mut self, n: usize) {
            self.prepare(n);
            self.dma.advance(n);
        }

        /// DMA transfers `n` samples between the two register reads of the next sync.
        fn race(&mut self, n: usize) {
            self.prepare(n);
            self.dma.race.set(n);
        }

        fn read(&mut self, len: usize) -> Result<(Vec<u32>, usize), Error> {
            let mut buf = vec![0; len];
            let (n, remaining) = self.rb.read(&mut self.dma, &mut buf)?;
            buf.truncate(n);
            Ok((buf, remaining))
        }

        fn read_latest(&mut self, len: usize) -> Vec<u32> {
            let mut buf = vec![0; len];
            let n = self.rb.read_latest(&mut self.dma, &mut buf);
            buf.truncate(n);
            buf
        }
    }

    /// Writable ring buffer drained by a simulated DMA. The `i`th sample written has the value `i`.
    struct Writer {
        rb: WritableDmaRingBuffer<'static, u32>,
        dma: Dma,
        /// Value of the next sample to write.
        next: usize,
    }

    impl Writer {
        fn new() -> Self {
            Self::with_dma(Dma::default())
        }

        fn with_dma(dma: Dma) -> Self {
            Self {
                rb: WritableDmaRingBuffer::new(new_buf()),
                dma,
                next: 0,
            }
        }

        fn write(&mut self, len: usize) -> Result<(usize, usize), Error> {
            let res = self.rb.write(&mut self.dma, &seq(self.next..self.next + len));
            if let Ok((n, _)) = res {
                self.next += n;
            }
            res
        }

        /// Data the DMA will send in its next `n` transfers.
        fn peek(&self, n: usize) -> Vec<u32> {
            (0..n)
                .map(|i| self.rb.dma_buf[(self.dma.pos.get() + i) % CAP])
                .collect()
        }

        /// DMA transfers `n` samples, returns the data sent.
        fn dma(&mut self, n: usize) -> Vec<u32> {
            let data = self.peek(n);
            self.dma.advance(n);
            data
        }

        /// DMA transfers `n` samples between the two register reads of the next sync, returns the data sent.
        fn race(&mut self, n: usize) -> Vec<u32> {
            let data = self.peek(n);
            self.dma.race.set(n);
            data
        }
    }

    #[test]
    fn read_partial() {
        let mut t = Reader::new();
        assert_eq!(t.read(3), Ok((vec![], 0)));
        t.dma(5);
        assert_eq!(t.read(3), Ok((seq(0..3), 2)));
        assert_eq!(t.read(3), Ok((seq(3..5), 0)));
        assert_eq!(t.read(3), Ok((vec![], 0)));
    }

    #[test]
    fn read_wraps() {
        let mut t = Reader::new();
        t.dma(10);
        assert_eq!(t.read(10), Ok((seq(0..10), 0)));
        t.dma(10);
        assert_eq!(t.read(CAP), Ok((seq(10..20), 0)));
    }

    #[test]
    fn read_full_buffer() {
        for start in 0..CAP {
            let mut t = Reader::new();
            t.dma(start);
            t.read(CAP).unwrap();
            t.dma(CAP);
            assert_eq!(t.read(CAP + 1), Ok((seq(start..start + CAP), 0)));
        }
    }

    #[test]
    fn read_overrun() {
        for start in 0..CAP {
            let mut t = Reader::new();
            t.dma(start);
            t.read(CAP).unwrap();
            t.dma(CAP + 1);
            assert_eq!(t.read(4), Err(Error::Overrun));
            // The buffer reset itself, unread data is dropped.
            assert_eq!(t.read(4), Ok((vec![], 0)));
            t.dma(3);
            let next = start + CAP + 1;
            assert_eq!(t.read(4), Ok((seq(next..next + 3), 0)));
        }
    }

    #[test]
    fn read_overrun_partially_read() {
        let mut t = Reader::new();
        t.dma(10);
        assert_eq!(t.read(4), Ok((seq(0..4), 6)));
        t.dma(10);
        assert_eq!(t.read(2), Ok((seq(4..6), 14)));
        t.dma(3);
        assert_eq!(t.read(4), Err(Error::Overrun));
    }

    #[test]
    fn read_overrun_many_laps() {
        let mut t = Reader::new();
        t.dma(5 * CAP + 3);
        assert_eq!(t.read(4), Err(Error::Overrun));
        t.dma(2);
        assert_eq!(t.read(4), Ok((seq(5 * CAP + 3..5 * CAP + 5), 0)));
    }

    #[test]
    fn read_reset() {
        let mut t = Reader::new();
        t.dma(2 * CAP + 5);
        t.rb.reset(&mut t.dma);
        assert_eq!(t.read(CAP), Ok((vec![], 0)));
        t.dma(CAP);
        assert_eq!(t.read(CAP), Ok((seq(2 * CAP + 5..3 * CAP + 5), 0)));
    }

    #[test]
    fn read_wrap_between_register_reads() {
        let mut t = Reader::new();
        t.dma(11);
        assert_eq!(t.read(11), Ok((seq(0..11), 0)));
        t.race(11);
        assert_eq!(t.read(CAP), Ok((seq(11..22), 0)));
        assert_eq!(t.read(CAP), Ok((vec![], 0)));
        t.dma(2);
        assert_eq!(t.read(CAP), Ok((seq(22..24), 0)));
    }

    #[test]
    fn read_wrap_with_irq_pending() {
        let mut t = Reader::with_dma(Dma {
            defer_irq: true,
            ..Default::default()
        });
        t.dma(11);
        assert_eq!(t.read(11), Ok((seq(0..11), 0)));
        t.dma(11);
        assert_eq!(t.read(CAP), Ok((seq(11..22), 0)));
        t.dma(2);
        assert_eq!(t.read(CAP), Ok((seq(22..24), 0)));
        t.dma.irq();
        assert_eq!(t.read(CAP), Ok((vec![], 0)));
        t.dma(2);
        assert_eq!(t.read(CAP), Ok((seq(24..26), 0)));
    }

    #[test]
    fn read_ndtr_zero() {
        let mut t = Reader::with_dma(Dma {
            lazy_reload: true,
            ..Default::default()
        });
        t.dma(8);
        assert_eq!(t.read(8), Ok((seq(0..8), 0)));
        t.dma(8);
        assert_eq!(t.read(CAP), Ok((seq(8..16), 0)));
        assert_eq!(t.read(CAP), Ok((vec![], 0)));
        t.dma(1);
        assert_eq!(t.read(CAP), Ok((seq(16..17), 0)));
    }

    #[test]
    fn read_two_wraps_with_irq_pending() {
        let mut t = Reader::with_dma(Dma {
            defer_irq: true,
            ..Default::default()
        });
        t.dma(10);
        assert_eq!(t.read(10), Ok((seq(0..10), 0)));
        t.dma(8);
        assert_eq!(t.read(CAP), Ok((seq(10..18), 0)));
        // The first wrap is counted, the second one isn't yet.
        t.dma.irq();
        t.dma(15);
        assert_eq!(t.read(CAP), Ok((seq(18..33), 0)));
        t.dma.irq();
        t.dma(2);
        assert_eq!(t.read(CAP), Ok((seq(33..35), 0)));
    }

    #[test]
    fn read_long_uptime() {
        let mut t = Reader::new();
        t.rb.write_index.complete_count = usize::MAX / CAP;
        t.rb.read_index.complete_count = usize::MAX / CAP;
        for i in 0..4 {
            t.dma(10);
            assert_eq!(t.read(10), Ok((seq(i * 10..i * 10 + 10), 0)));
        }
    }

    /// Stream data through the buffer in uneven chunks, checking every sample arrives in order.
    fn read_stream(mut t: Reader, racy: bool) {
        let mut written = 0;
        let mut read = 0;
        for i in 0..1000 {
            t.dma.irq();
            // Less than a lap per step, so the DMA never laps the last synced position with an interrupt pending.
            let n = (i * 7) % (CAP - (written - read));
            if racy && i % 3 == 0 {
                t.race(n);
            } else {
                t.dma(n);
            }
            written += n;
            let len = (i * 5) % CAP + 1;
            let k = len.min(written - read);
            assert_eq!(
                t.read(len),
                Ok((seq(read..read + k), written - read - k)),
                "step {i}, defer_irq={}, lazy_reload={}, racy={racy}",
                t.dma.defer_irq,
                t.dma.lazy_reload
            );
            read += k;
        }
    }

    #[test]
    fn read_streams() {
        for defer_irq in [false, true] {
            for lazy_reload in [false, true] {
                for racy in [false, true] {
                    let dma = Dma {
                        defer_irq,
                        lazy_reload,
                        ..Default::default()
                    };
                    read_stream(Reader::with_dma(dma), racy);
                }
            }
        }
    }

    #[test]
    fn read_exact() {
        let mut t = Reader::new();
        t.dma(4);
        t.prepare(6);
        t.dma.polls = VecDeque::from([3, 3]);
        let mut buf = [0; 9];
        {
            let Reader { rb, dma } = &mut t;
            let mut fut = pin!(rb.read_exact(dma, &mut buf));
            assert_eq!(poll(fut.as_mut()), Poll::Pending);
            assert_eq!(poll(fut.as_mut()), Poll::Ready(Ok(1)));
        }
        assert_eq!(buf.to_vec(), seq(0..9));
    }

    #[test]
    fn read_exact_overrun() {
        let mut t = Reader::new();
        t.dma(4);
        t.dma.polls = VecDeque::from([0, CAP + 1]);
        let mut buf = [0; 8];
        let Reader { rb, dma } = &mut t;
        let mut fut = pin!(rb.read_exact(dma, &mut buf));
        assert_eq!(poll(fut.as_mut()), Poll::Pending);
        assert_eq!(poll(fut.as_mut()), Poll::Ready(Err(Error::Overrun)));
    }

    #[test]
    fn read_aligned_whole_frames() {
        let mut t = Reader::aligned();
        t.dma(6);
        assert_eq!(t.read(CAP), Ok((seq(0..4), 2)));
        // The partial frame stays until it's complete.
        assert_eq!(t.read(CAP), Ok((vec![], 2)));
        t.dma(6);
        assert_eq!(t.read(5), Ok((seq(4..8), 4)));
    }

    #[test]
    fn read_aligned_after_overrun() {
        let mut t = Reader::aligned();
        t.dma(CAP + 5);
        assert_eq!(t.read(CAP), Err(Error::Overrun));
        // Reading resumes mid-frame, at position 5. The rest of that frame is skipped.
        t.dma(2);
        assert_eq!(t.read(CAP), Ok((vec![], 2)));
        t.dma(7);
        assert_eq!(t.read(CAP), Ok((seq(24..28), 2)));
    }

    #[test]
    #[should_panic(expected = "multiple of the alignment")]
    fn read_alignment_must_divide_capacity() {
        Reader::new().rb.set_alignment(3);
    }

    #[test]
    #[should_panic(expected = "at least 2 frames")]
    fn read_alignment_needs_two_frames() {
        Reader::new().rb.set_alignment(CAP);
    }

    #[test]
    fn read_latest() {
        let mut t = Reader::new();
        assert_eq!(t.read_latest(4), vec![]);
        t.dma(6);
        assert_eq!(t.read_latest(CAP), seq(0..6));
        // Older samples that don't fit are dropped.
        t.dma(12);
        assert_eq!(t.read_latest(4), seq(14..18));
        assert_eq!(t.read_latest(4), vec![]);
    }

    #[test]
    fn read_latest_overrun() {
        let mut t = Reader::new();
        t.dma(2 * CAP + 5);
        assert_eq!(t.read_latest(CAP + 4), seq(CAP + 5..2 * CAP + 5));
        t.dma(2 * CAP + 5);
        assert_eq!(t.read_latest(4), seq(4 * CAP + 6..4 * CAP + 10));
        t.dma(3);
        assert_eq!(t.read_latest(CAP), seq(4 * CAP + 10..4 * CAP + 13));
    }

    #[test]
    fn read_latest_aligned() {
        let mut t = Reader::aligned();
        t.dma(14);
        // Whole frames only. The trailing partial frame is kept for later.
        assert_eq!(t.read_latest(6), seq(8..12));
        assert_eq!(t.read_latest(6), vec![]);
        t.dma(2);
        assert_eq!(t.read_latest(6), seq(12..16));
    }

    #[test]
    fn read_latest_aligned_overrun() {
        let mut t = Reader::aligned();
        t.dma(2 * CAP + 6);
        assert_eq!(t.read_latest(6), seq(2 * CAP..2 * CAP + 4));
        t.dma(6);
        assert_eq!(t.read_latest(6), seq(2 * CAP + 8..2 * CAP + 12));
    }

    #[test]
    fn write_until_full() {
        let mut t = Writer::new();
        assert_eq!(t.write(5), Ok((5, CAP - 5)));
        assert_eq!(t.write(CAP), Ok((CAP - 5, 0)));
        assert_eq!(t.write(1), Ok((0, 0)));
        assert_eq!(t.dma(4), seq(0..4));
        assert_eq!(t.write(CAP), Ok((4, 0)));
        assert_eq!(t.dma(CAP), seq(4..20));
    }

    #[test]
    fn write_drained() {
        for start in 0..CAP {
            let mut t = Writer::new();
            t.write(start).unwrap();
            t.dma(start);
            assert_eq!(t.write(CAP + 1), Ok((CAP, 0)));
            assert_eq!(t.dma(CAP), seq(start..start + CAP));
            // DMA caught up exactly: not an underrun.
            assert_eq!(t.write(CAP + 1), Ok((CAP, 0)));
        }
    }

    #[test]
    fn write_underrun() {
        for start in 0..CAP {
            let mut t = Writer::new();
            t.write(start).unwrap();
            t.dma(start);
            assert_eq!(t.write(4), Ok((4, CAP - 4)));
            t.dma(5);
            assert_eq!(t.write(4), Err(Error::Overrun));
            // The buffer reset itself, writing resumes at the DMA's position.
            assert_eq!(t.write(4), Ok((4, CAP - 4)));
            assert_eq!(t.dma(4), seq(start + 4..start + 8));
        }
    }

    #[test]
    fn write_underrun_many_laps() {
        let mut t = Writer::new();
        assert_eq!(t.write(4), Ok((4, CAP - 4)));
        t.dma(3 * CAP);
        assert_eq!(t.write(4), Err(Error::Overrun));
        assert_eq!(t.write(4), Ok((4, CAP - 4)));
        assert_eq!(t.dma(4), seq(4..8));
    }

    #[test]
    fn write_reset() {
        let mut t = Writer::new();
        assert_eq!(t.write(5), Ok((5, CAP - 5)));
        t.dma(2 * CAP + 5);
        t.rb.reset(&mut t.dma);
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(CAP), seq(5..CAP + 5));
    }

    #[test]
    fn write_pos() {
        let mut t = Writer::new();
        assert_eq!(t.rb.write_pos(), 0);
        t.write(10).unwrap();
        assert_eq!(t.rb.write_pos(), 10);
        t.dma(10);
        t.write(10).unwrap();
        assert_eq!(t.rb.write_pos(), 4);
        t.dma(13);
        assert_eq!(t.write(1), Err(Error::Overrun));
        assert_eq!(t.rb.write_pos(), 7);
    }

    #[test]
    fn write_wrap_between_register_reads() {
        let mut t = Writer::new();
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(11), seq(0..11));
        assert_eq!(t.write(CAP), Ok((11, 0)));
        assert_eq!(t.race(11), seq(11..22));
        assert_eq!(t.write(CAP), Ok((11, 0)));
        assert_eq!(t.dma(CAP), seq(22..38));
    }

    #[test]
    fn write_wrap_with_irq_pending() {
        let mut t = Writer::with_dma(Dma {
            defer_irq: true,
            ..Default::default()
        });
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(11), seq(0..11));
        assert_eq!(t.write(CAP), Ok((11, 0)));
        assert_eq!(t.dma(11), seq(11..22));
        assert_eq!(t.write(CAP), Ok((11, 0)));
        t.dma.irq();
        assert_eq!(t.write(CAP), Ok((0, 0)));
        assert_eq!(t.dma(CAP), seq(22..38));
    }

    #[test]
    fn write_ndtr_zero() {
        let mut t = Writer::with_dma(Dma {
            lazy_reload: true,
            ..Default::default()
        });
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(CAP), seq(0..CAP));
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(CAP), seq(CAP..2 * CAP));
        assert_eq!(t.write(1), Ok((1, CAP - 1)));
    }

    #[test]
    fn write_two_wraps_with_irq_pending() {
        let mut t = Writer::with_dma(Dma {
            defer_irq: true,
            ..Default::default()
        });
        assert_eq!(t.write(CAP), Ok((CAP, 0)));
        assert_eq!(t.dma(10), seq(0..10));
        assert_eq!(t.write(CAP), Ok((10, 0)));
        assert_eq!(t.dma(8), seq(10..18));
        assert_eq!(t.write(CAP), Ok((8, 0)));
        // The first wrap is counted, the second one isn't yet.
        t.dma.irq();
        assert_eq!(t.dma(15), seq(18..33));
        assert_eq!(t.write(CAP), Ok((15, 0)));
        assert_eq!(t.dma(CAP), seq(33..49));
    }

    #[test]
    fn write_long_uptime() {
        let mut t = Writer::new();
        t.rb.write_index.complete_count = usize::MAX / CAP;
        t.rb.read_index.complete_count = usize::MAX / CAP;
        for i in 0..4 {
            assert_eq!(t.write(10), Ok((10, CAP - 10)));
            assert_eq!(t.dma(10), seq(i * 10..i * 10 + 10));
        }
    }

    /// Stream data through the buffer in uneven chunks, checking every sample is sent in order.
    fn write_stream(mut t: Writer, racy: bool) {
        let mut sent = 0;
        for i in 0..1000 {
            t.dma.irq();
            // Less than a lap per step, so the DMA never laps the last synced position with an interrupt pending.
            let n = (i * 7) % ((t.next - sent).min(CAP - 1) + 1);
            let data = if racy && i % 3 == 0 { t.race(n) } else { t.dma(n) };
            assert_eq!(
                data,
                seq(sent..sent + n),
                "step {i}, defer_irq={}, lazy_reload={}, racy={racy}",
                t.dma.defer_irq,
                t.dma.lazy_reload
            );
            sent += n;
            let len = (i * 5) % CAP + 1;
            let free = CAP - (t.next - sent);
            let k = len.min(free);
            assert_eq!(
                t.write(len),
                Ok((k, free - k)),
                "step {i}, defer_irq={}, lazy_reload={}, racy={racy}",
                t.dma.defer_irq,
                t.dma.lazy_reload
            );
        }
    }

    #[test]
    fn write_streams() {
        for defer_irq in [false, true] {
            for lazy_reload in [false, true] {
                for racy in [false, true] {
                    let dma = Dma {
                        defer_irq,
                        lazy_reload,
                        ..Default::default()
                    };
                    write_stream(Writer::with_dma(dma), racy);
                }
            }
        }
    }

    #[test]
    fn write_exact() {
        let mut t = Writer::new();
        t.dma.polls = VecDeque::from([0, 8]);
        let data = seq(0..20);
        {
            let Writer { rb, dma, .. } = &mut t;
            let mut fut = pin!(rb.write_exact(dma, &data));
            assert_eq!(poll(fut.as_mut()), Poll::Pending);
            assert_eq!(poll(fut.as_mut()), Poll::Ready(Ok(4)));
        }
        assert_eq!(t.dma(12), seq(8..20));
    }

    #[test]
    fn wait_write_error() {
        let mut t = Writer::new();
        t.write(4).unwrap();
        t.dma.polls = VecDeque::from([2, 3]);
        let Writer { rb, dma, .. } = &mut t;
        let mut fut = pin!(rb.wait_write_error(dma));
        assert_eq!(poll(fut.as_mut()), Poll::Pending);
        assert_eq!(poll(fut.as_mut()), Poll::Ready(Err(Error::Overrun)));
    }
}
