use std::task::Waker;

use proptest::prop_oneof;
use proptest::strategy::{self, BoxedStrategy, Strategy as _};
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest, prop_state_machine};

use super::*;

const CAP: usize = 128;

#[derive(Debug, Default)]
struct DmaMock {
    pos: usize,
    wraps: usize,
}

impl DmaMock {
    pub fn advance(&mut self, steps: usize) {
        let next = self.pos + steps;
        self.pos = next % CAP;
        self.wraps += next / CAP;
    }
}

impl DmaCtrl for DmaMock {
    fn get_remaining_transfers(&self) -> usize {
        CAP - self.pos
    }

    fn reset_complete_count(&mut self) -> usize {
        core::mem::replace(&mut self.wraps, 0)
    }

    fn set_waker(&mut self, _waker: &Waker) {}
}

#[derive(Debug, Clone)]
enum Status {
    Available(usize),
    Failed,
}

impl Status {
    pub fn new(capacity: usize) -> Self {
        Self::Available(capacity)
    }
}

mod reader;
mod writer;
