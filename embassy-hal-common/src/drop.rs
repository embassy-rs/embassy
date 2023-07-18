use core::mem;
use core::mem::MaybeUninit;

#[must_use = "to delay the drop handler invokation to the end of the scope"]
pub struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self { f: MaybeUninit::new(f) }
    }

    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}

/// An explosive ordinance that panics if it is improperly disposed of.
///
/// This is to forbid dropping futures, when there is absolutely no other choice.
///
/// To correctly dispose of this device, call the [defuse](struct.DropBomb.html#method.defuse)
/// method before this object is dropped.
#[must_use = "to delay the drop bomb invokation to the end of the scope"]
pub struct DropBomb {
    _private: (),
}

impl DropBomb {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Defuses the bomb, rendering it safe to drop.
    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        panic!("boom")
    }
}
