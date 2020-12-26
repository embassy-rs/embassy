use core::task::{RawWaker, RawWakerVTable, Waker};

use super::TaskHeader;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn wake(p: *const ()) {
    let header = &*(p as *const TaskHeader);
    header.enqueue();
}

unsafe fn drop(_: *const ()) {
    // nop
}

pub(crate) unsafe fn from_task(p: *mut TaskHeader) -> Waker {
    Waker::from_raw(RawWaker::new(p as _, &VTABLE))
}
