use std::{cell, vec};

use super::*;

#[allow(unused)]
#[derive(PartialEq, Debug)]
enum TestCircularTransferRequest {
    ResetCompleteCount(usize),
    PositionRequest(usize),
}

#[allow(unused)]
struct TestCircularTransfer {
    len: usize,
    requests: cell::RefCell<vec::Vec<TestCircularTransferRequest>>,
}

impl DmaCtrl for TestCircularTransfer {
    fn get_remaining_transfers(&self) -> usize {
        match self.requests.borrow_mut().pop().unwrap() {
            TestCircularTransferRequest::PositionRequest(pos) => {
                let len = self.len;

                assert!(len >= pos);

                len - pos
            }
            _ => unreachable!(),
        }
    }

    fn reset_complete_count(&mut self) -> usize {
        match self.requests.get_mut().pop().unwrap() {
            TestCircularTransferRequest::ResetCompleteCount(complete_count) => complete_count,
            _ => unreachable!(),
        }
    }

    fn set_waker(&mut self, _waker: &Waker) {}
}

impl TestCircularTransfer {
    #[allow(unused)]
    pub fn new(len: usize) -> Self {
        Self {
            requests: cell::RefCell::new(vec![]),
            len,
        }
    }

    #[allow(unused)]
    pub fn setup(&self, mut requests: vec::Vec<TestCircularTransferRequest>) {
        requests.reverse();
        self.requests.replace(requests);
    }
}

const CAP: usize = 16;

#[test]
fn dma_index_as_index_returns_index_mod_cap_by_default() {
    let index = DmaIndex::default();
    assert_eq!(index.as_index(CAP, 0), 0);
    assert_eq!(index.as_index(CAP, 1), 1);
    assert_eq!(index.as_index(CAP, 2), 2);
    assert_eq!(index.as_index(CAP, 3), 3);
    assert_eq!(index.as_index(CAP, 4), 4);
    assert_eq!(index.as_index(CAP, CAP), 0);
    assert_eq!(index.as_index(CAP, CAP + 1), 1);
}

#[test]
fn dma_index_advancing_increases_as_index() {
    let mut index = DmaIndex::default();
    assert_eq!(index.as_index(CAP, 0), 0);
    index.advance(CAP, 1);
    assert_eq!(index.as_index(CAP, 0), 1);
    index.advance(CAP, 1);
    assert_eq!(index.as_index(CAP, 0), 2);
    index.advance(CAP, 1);
    assert_eq!(index.as_index(CAP, 0), 3);
    index.advance(CAP, 1);
    assert_eq!(index.as_index(CAP, 0), 4);
    index.advance(CAP, CAP - 4);
    assert_eq!(index.as_index(CAP, 0), 0);
    index.advance(CAP, 1);
    assert_eq!(index.as_index(CAP, 0), 1);
}

/// Test that after an overrun recovery lands on a misaligned position,
/// subsequent reads skip forward to the next frame boundary.
#[test]
fn alignment_skip_after_overrun() {
    // Buffer of 16 u8s, alignment of 4 (simulating stereo 32-bit I2S frames).
    let mut dma_buf = [0u8; CAP];
    // Fill buffer with recognizable pattern: each frame starts with its frame index.
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    ringbuf.set_alignment(4);

    // Simulate DMA having written 10 samples (2.5 frames) and an overrun
    // that resets the read index to the current DMA write position.
    let mut dma = TestCircularTransfer::new(CAP);

    // reset() calls: reset_complete_count, then dma_sync (reset_complete_count + get_remaining_transfers)
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(5), // DMA at position 5 (misaligned)
    ]);
    ringbuf.reset(&mut dma);

    // read_index is now at position 5 (mid-frame). Advance DMA to position 14.
    let mut read_buf = [0u8; 8];
    dma.setup(vec![
        // read_raw calls len() which calls dma_sync (reset_complete_count + get_remaining_transfers)
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(14),
        // len() is called again after reading
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(14),
    ]);

    let (read, _remaining) = ringbuf.read(&mut dma, &mut read_buf).unwrap();

    // 9 samples available (pos 5..14), alignment skip of 3 (to pos 8),
    // leaves 6 samples, rounded down to nearest frame (4).
    assert_eq!(read, 4);
    // Data should start at buffer position 8 (the next aligned frame boundary).
    assert_eq!(&read_buf[..read], &[8, 9, 10, 11]);
}

/// Test that reads from an already-aligned position are unaffected by alignment setting.
#[test]
fn alignment_no_skip_when_aligned() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    ringbuf.set_alignment(4);

    let mut dma = TestCircularTransfer::new(CAP);

    // Reset with DMA at position 8 (already frame-aligned).
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(8),
    ]);
    ringbuf.reset(&mut dma);

    // Advance DMA to position 16 (= 0 wrapped).
    let mut read_buf = [0u8; 8];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(1),
        TestCircularTransferRequest::PositionRequest(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);

    let (read, _remaining) = ringbuf.read(&mut dma, &mut read_buf).unwrap();

    // All 8 samples readable with no skip needed.
    assert_eq!(read, 8);
    assert_eq!(&read_buf[..read], &[8, 9, 10, 11, 12, 13, 14, 15]);
}

/// Test that a non-frame-aligned user buffer gets rounded down so read_index stays aligned.
#[test]
fn alignment_rounds_down_read_length() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    ringbuf.set_alignment(4);

    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 0 (aligned), DMA at position 12.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);
    ringbuf.reset(&mut dma);

    // User buffer of 5 (not a multiple of 4). Should only read 4.
    let mut read_buf = [0u8; 5];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(12),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(12),
    ]);

    let (read, _remaining) = ringbuf.read(&mut dma, &mut read_buf).unwrap();

    // Rounded down from 5 to 4 to maintain frame alignment.
    assert_eq!(read, 4);
    assert_eq!(&read_buf[..read], &[0, 1, 2, 3]);
}

/// Test that set_alignment panics when buffer length is not a multiple of alignment.
#[test]
#[should_panic(expected = "DMA buffer length must be a multiple of the alignment value")]
fn alignment_rejects_non_multiple() {
    let mut dma_buf = [0u8; CAP]; // CAP = 16
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    ringbuf.set_alignment(3); // 16 % 3 != 0 → panic
}

// ── read_latest tests ──────────────────────────────────────────────────

/// read_latest returns the most recent elements when more data is available
/// than the user buffer can hold.
#[test]
fn read_latest_returns_most_recent_data() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 0.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);
    ringbuf.reset(&mut dma);

    // DMA has written 12 samples (positions 0..12). User buffer holds 4.
    // read_latest should return the 4 most recent: [8, 9, 10, 11].
    let mut read_buf = [0u8; 4];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(12),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 4);
    assert_eq!(&read_buf[..n], &[8, 9, 10, 11]);
}

/// read_latest never errors on overrun — it just resets and returns 0.
#[test]
fn read_latest_recovers_from_overrun() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 0.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);
    ringbuf.reset(&mut dma);

    // Simulate overrun: DMA has wrapped more than once (complete_count=2, pos=5).
    // diff = (2*16 + 5) - 0 = 37 > 16 → overrun.
    let mut read_buf = [0u8; 8];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(2),
        TestCircularTransferRequest::PositionRequest(5),
    ]);

    // Should not panic or error — just returns 0 (reset to catch up).
    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 0);

    // Next call after overrun recovery should return fresh data normally.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(13),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 8);
    assert_eq!(&read_buf[..n], &[5, 6, 7, 8, 9, 10, 11, 12]);
}

/// read_latest returns 0 when no new data is available.
#[test]
fn read_latest_returns_zero_when_empty() {
    let mut dma_buf = [0u8; CAP];
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 4.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(4),
    ]);
    ringbuf.reset(&mut dma);

    // DMA is still at position 4 — nothing new.
    let mut read_buf = [0u8; 8];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(4),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 0);
}

/// read_latest respects alignment: skips to frame boundary, rounds down length.
#[test]
fn read_latest_with_alignment() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    ringbuf.set_alignment(4);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 0 (aligned).
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);
    ringbuf.reset(&mut dma);

    // DMA at position 14. 14 samples available, user buffer holds 6.
    // Tail = 14 % 4 = 2 (partial frame discarded), aligned_available = 12.
    // to_read = min(12, 6) = 6, rounded down to 4.
    // front_skip = 12 - 4 = 8. Read starts at position 8 (aligned): [8, 9, 10, 11].
    let mut read_buf = [0u8; 6];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(14),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 4);
    assert_eq!(&read_buf[..n], &[8, 9, 10, 11]);
}

/// read_latest reads all available data when buffer is large enough.
#[test]
fn read_latest_reads_all_when_buffer_is_large() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 0.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(0),
    ]);
    ringbuf.reset(&mut dma);

    // DMA has written 6 samples. User buffer holds 16. Should read exactly 6.
    let mut read_buf = [0u8; CAP];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(6),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 6);
    assert_eq!(&read_buf[..n], &[0, 1, 2, 3, 4, 5]);
}

/// read_latest handles wrap-around correctly.
#[test]
fn read_latest_wraps_around() {
    let mut dma_buf = [0u8; CAP];
    for i in 0..CAP {
        dma_buf[i] = i as u8;
    }
    let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);
    let mut dma = TestCircularTransfer::new(CAP);

    // Reset at position 12.
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(12),
    ]);
    ringbuf.reset(&mut dma);

    // DMA wraps around: complete_count=1, position=4.
    // Available = (1*16+4) - (0*16+12) = 8.
    // User buffer holds 4 → reads latest 4: positions 0..4 after wrap.
    let mut read_buf = [0u8; 4];
    dma.setup(vec![
        TestCircularTransferRequest::ResetCompleteCount(1),
        TestCircularTransferRequest::PositionRequest(4),
    ]);

    let n = ringbuf.read_latest(&mut dma, &mut read_buf);
    assert_eq!(n, 4);
    // After wrap, positions 0..4 in the buffer hold [0, 1, 2, 3]
    assert_eq!(&read_buf[..n], &[0, 1, 2, 3]);
}

mod prop_test;
