use core::task::{RawWaker, RawWakerVTable, Waker};

use super::{wake_task, TaskHeader, TaskRef};

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn wake(p: *const ()) {
    wake_task(TaskRef::from_ptr(p as *const TaskHeader))
}

unsafe fn drop(_: *const ()) {
    // nop
}

pub(crate) unsafe fn from_task(p: TaskRef) -> Waker {
    Waker::from_raw(RawWaker::new(p.as_ptr() as _, &VTABLE))
}

/// Get a task pointer from a waker.
///
/// This can be used as an optimization in wait queues to store task pointers
/// (1 word) instead of full Wakers (2 words). This saves a bit of RAM and helps
/// avoid dynamic dispatch.
///
/// You can use the returned task pointer to wake the task with [`wake_task`].
///
/// # Panics
///
/// Panics if the waker is not created by the Embassy executor.
pub fn task_from_waker(waker: &Waker) -> TaskRef {
    // make sure to compare vtable addresses. Doing `==` on the references
    // will compare the contents, which is slower.
    if waker.vtable() as *const _ != &VTABLE as *const _ {
        panic!("Found waker not created by the Embassy executor. `embassy_time::Timer` only works with the Embassy executor.")
    }
    // safety: our wakers are always created with `TaskRef::as_ptr`
    unsafe { TaskRef::from_ptr(waker.data() as *const TaskHeader) }
}
