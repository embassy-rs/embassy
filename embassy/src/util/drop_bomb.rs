use crate::fmt::panic;
use core::mem;

pub struct DropBomb {
    _private: (),
}

impl DropBomb {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        panic!("boom")
    }
}
