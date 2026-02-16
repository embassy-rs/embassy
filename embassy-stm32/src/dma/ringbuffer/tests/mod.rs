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

mod prop_test;
