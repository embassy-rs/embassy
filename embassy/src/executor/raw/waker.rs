use core::mem;
use core::ptr::NonNull;
use core::task::{RawWaker, RawWakerVTable, Waker};

use super::TaskHeader;

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn wake(p: *const ()) {
    (*(p as *mut TaskHeader)).enqueue()
}

unsafe fn drop(_: *const ()) {
    // nop
}

pub(crate) unsafe fn from_task(p: NonNull<TaskHeader>) -> Waker {
    Waker::from_raw(RawWaker::new(p.as_ptr() as _, &VTABLE))
}

pub unsafe fn task_from_waker(waker: &Waker) -> NonNull<TaskHeader> {
    let hack: &WakerHack = mem::transmute(waker);
    if hack.vtable != &VTABLE {
        panic!("Found waker not created by the embassy executor. Consider enabling the `executor-agnostic` feature on the `embassy` crate.")
    }
    NonNull::new_unchecked(hack.data as *mut TaskHeader)
}

struct WakerHack {
    data: *const (),
    vtable: &'static RawWakerVTable,
}
