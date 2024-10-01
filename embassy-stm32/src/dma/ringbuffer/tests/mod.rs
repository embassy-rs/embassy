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

mod prop_test;
