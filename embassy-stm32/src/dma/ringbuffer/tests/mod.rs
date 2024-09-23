use std::{cell, vec};

use super::*;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
enum TestCircularTransferRequest {
    ResetCompleteCount(usize),
    PositionRequest(usize),
}

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
    pub fn new(len: usize) -> Self {
        Self {
            requests: cell::RefCell::new(vec![]),
            len,
        }
    }

    pub fn setup(&self, mut requests: vec::Vec<TestCircularTransferRequest>) {
        requests.reverse();
        self.requests.replace(requests);
    }
}

const CAP: usize = 16;

#[test]
fn dma_index_dma_sync_syncs_position_to_last_read_if_sync_takes_place_on_same_dma_cycle() {
    let mut dma = TestCircularTransfer::new(CAP);
    dma.setup(vec![
        TestCircularTransferRequest::PositionRequest(4),
        TestCircularTransferRequest::ResetCompleteCount(0),
        TestCircularTransferRequest::PositionRequest(7),
    ]);
    let mut index = DmaIndex::default();
    index.dma_sync(CAP, &mut dma);
    assert_eq!(index.complete_count, 0);
    assert_eq!(index.pos, 7);
}

#[test]
fn dma_index_dma_sync_updates_complete_count_properly_if_sync_takes_place_on_same_dma_cycle() {
    let mut dma = TestCircularTransfer::new(CAP);
    dma.setup(vec![
        TestCircularTransferRequest::PositionRequest(4),
        TestCircularTransferRequest::ResetCompleteCount(2),
        TestCircularTransferRequest::PositionRequest(7),
    ]);
    let mut index = DmaIndex::default();
    index.complete_count = 1;
    index.dma_sync(CAP, &mut dma);
    assert_eq!(index.complete_count, 3);
    assert_eq!(index.pos, 7);
}

#[test]
fn dma_index_dma_sync_syncs_to_last_position_if_reads_occur_on_different_dma_cycles() {
    let mut dma = TestCircularTransfer::new(CAP);
    dma.setup(vec![
        TestCircularTransferRequest::PositionRequest(10),
        TestCircularTransferRequest::ResetCompleteCount(1),
        TestCircularTransferRequest::PositionRequest(5),
        TestCircularTransferRequest::ResetCompleteCount(0),
    ]);
    let mut index = DmaIndex::default();
    index.dma_sync(CAP, &mut dma);
    assert_eq!(index.complete_count, 1);
    assert_eq!(index.pos, 5);
}

#[test]
fn dma_index_dma_sync_detects_new_cycle_if_later_position_is_less_than_first_and_first_complete_count_occurs_on_first_cycle(
) {
    let mut dma = TestCircularTransfer::new(CAP);
    dma.setup(vec![
        TestCircularTransferRequest::PositionRequest(10),
        TestCircularTransferRequest::ResetCompleteCount(1),
        TestCircularTransferRequest::PositionRequest(5),
        TestCircularTransferRequest::ResetCompleteCount(1),
    ]);
    let mut index = DmaIndex::default();
    index.complete_count = 1;
    index.dma_sync(CAP, &mut dma);
    assert_eq!(index.complete_count, 3);
    assert_eq!(index.pos, 5);
}

#[test]
fn dma_index_dma_sync_detects_new_cycle_if_later_position_is_less_than_first_and_first_complete_count_occurs_on_later_cycle(
) {
    let mut dma = TestCircularTransfer::new(CAP);
    dma.setup(vec![
        TestCircularTransferRequest::PositionRequest(10),
        TestCircularTransferRequest::ResetCompleteCount(2),
        TestCircularTransferRequest::PositionRequest(5),
        TestCircularTransferRequest::ResetCompleteCount(0),
    ]);
    let mut index = DmaIndex::default();
    index.complete_count = 1;
    index.dma_sync(CAP, &mut dma);
    assert_eq!(index.complete_count, 3);
    assert_eq!(index.pos, 5);
}

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

mod prop_test;
