use core::ptr::NonNull;
use core::task::Waker;

use super::{wake_task, TaskHeader, TaskRef};

pub(crate) unsafe fn from_task(p: TaskRef) -> Waker {
    Waker::from_turbo_ptr(NonNull::new_unchecked(p.as_ptr() as _))
}

/// Get a task pointer from a waker.
///
/// This can be used as an optimization in wait queues to store task pointers
/// (1 word) instead of full Wakers (2 words). This saves a bit of RAM and helps
/// avoid dynamic dispatch.
///
/// You can use the returned task pointer to wake the task with [`wake_task`](super::wake_task).
///
/// # Panics
///
/// Panics if the waker is not created by the Embassy executor.
pub fn task_from_waker(waker: &Waker) -> TaskRef {
    let ptr = waker.as_turbo_ptr().as_ptr();

    // safety: our wakers are always created with `TaskRef::as_ptr`
    unsafe { TaskRef::from_ptr(ptr as *const TaskHeader) }
}

#[inline(never)]
#[no_mangle]
fn _turbo_wake(ptr: NonNull<()>) {
    // safety: our wakers are always created with `TaskRef::as_ptr`
    let task = unsafe { TaskRef::from_ptr(ptr.as_ptr() as *const TaskHeader) };
    wake_task(task)
}
