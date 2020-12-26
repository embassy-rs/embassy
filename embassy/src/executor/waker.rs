use core::mem;
use core::task::{RawWaker, RawWakerVTable, Waker};

use super::TaskHeader;

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn wake(p: *const ()) {
    let header = &*task_from_ptr(p);
    header.enqueue();
}

unsafe fn drop(_: *const ()) {
    // nop
}

pub(crate) unsafe fn from_task(p: *mut TaskHeader) -> Waker {
    Waker::from_raw(RawWaker::new(p as _, &VTABLE))
}

pub(crate) unsafe fn task_from_ptr(p: *const ()) -> *mut TaskHeader {
    p as *mut TaskHeader
}

pub(crate) unsafe fn task_from_waker(w: &Waker) -> *mut TaskHeader {
    let w: &WakerHack = mem::transmute(w);
    assert_eq!(w.vtable, &VTABLE);
    task_from_ptr(w.data)
}

struct WakerHack {
    data: *const (),
    vtable: &'static RawWakerVTable,
}
