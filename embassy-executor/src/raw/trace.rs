#![allow(unused)]
use crate::raw::{SyncExecutor, TaskRef};

#[cfg(not(feature = "rtos-trace"))]
extern "Rust" {
    fn _embassy_trace_task_new(executor_id: u32, task_id: u32);
    fn _embassy_trace_task_exec_begin(executor_id: u32, task_id: u32);
    fn _embassy_trace_task_exec_end(excutor_id: u32, task_id: u32);
    fn _embassy_trace_task_ready_begin(executor_id: u32, task_id: u32);
    fn _embassy_trace_executor_idle(executor_id: u32);
}

#[inline]
pub(crate) fn task_new(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(not(feature = "rtos-trace"))]
    unsafe {
        _embassy_trace_task_new(executor as *const _ as u32, task.as_ptr() as u32)
    }

    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_new(task.as_ptr() as u32);
}

#[inline]
pub(crate) fn task_ready_begin(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(not(feature = "rtos-trace"))]
    unsafe {
        _embassy_trace_task_ready_begin(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_ready_begin(task.as_ptr() as u32);
}

#[inline]
pub(crate) fn task_exec_begin(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(not(feature = "rtos-trace"))]
    unsafe {
        _embassy_trace_task_exec_begin(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_exec_begin(task.as_ptr() as u32);
}

#[inline]
pub(crate) fn task_exec_end(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(not(feature = "rtos-trace"))]
    unsafe {
        _embassy_trace_task_exec_end(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_exec_end();
}

#[inline]
pub(crate) fn executor_idle(executor: &SyncExecutor) {
    #[cfg(not(feature = "rtos-trace"))]
    unsafe {
        _embassy_trace_executor_idle(executor as *const _ as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::system_idle();
}

#[cfg(feature = "rtos-trace")]
impl rtos_trace::RtosTraceOSCallbacks for crate::raw::SyncExecutor {
    fn task_list() {
        // We don't know what tasks exist, so we can't send them.
    }
    fn time() -> u64 {
        const fn gcd(a: u64, b: u64) -> u64 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }

        const GCD_1M: u64 = gcd(embassy_time_driver::TICK_HZ, 1_000_000);
        embassy_time_driver::now() * (1_000_000 / GCD_1M) / (embassy_time_driver::TICK_HZ / GCD_1M)
    }
}

#[cfg(feature = "rtos-trace")]
rtos_trace::global_os_callbacks! {SyncExecutor}
