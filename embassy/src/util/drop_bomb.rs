use crate::fmt::panic;
use core::mem;

/// An explosive ordinance that panics if it is improperly disposed of.
///
/// This is to forbid dropping futures, when there is absolutely no other choice.
///
/// To correctly dispose of this device, call the [defuse](struct.DropBomb.html#method.defuse)
/// method before this object is dropped.
pub struct DropBomb {
    _private: (),
}

impl DropBomb {
    pub fn new() -> Self {
        Self { _private: () }
    }

    // Diffuses the bomb, rendering it safe to drop.
    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        panic!("boom")
    }
}
