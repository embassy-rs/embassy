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

/// Get a task pointer from a waker.
///
/// This can used as an optimization in wait queues to store task pointers
/// (1 word) instead of full Wakers (2 words). This saves a bit of RAM and helps
/// avoid dynamic dispatch.
///
/// You can use the returned task pointer to wake the task with [`wake_task`](super::wake_task).
///
/// # Panics
///
/// Panics if the waker is not created by the Embassy executor.
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
