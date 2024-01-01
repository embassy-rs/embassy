#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

use std::boxed::Box;
use std::future::poll_fn;
use std::sync::{Arc, Mutex};
use std::task::Poll;

use embassy_executor::raw::Executor;
use embassy_executor::task;

#[export_name = "__pender"]
fn __pender(context: *mut ()) {
    unsafe {
        let trace = &*(context as *const Trace);
        trace.push("pend");
    }
}

#[derive(Clone)]
struct Trace {
    trace: Arc<Mutex<Vec<&'static str>>>,
}

impl Trace {
    fn new() -> Self {
        Self {
            trace: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn push(&self, value: &'static str) {
        self.trace.lock().unwrap().push(value)
    }

    fn get(&self) -> Vec<&'static str> {
        self.trace.lock().unwrap().clone()
    }
}

fn setup() -> (&'static Executor, Trace) {
    let trace = Trace::new();
    let context = Box::leak(Box::new(trace.clone())) as *mut _ as *mut ();
    let executor = &*Box::leak(Box::new(Executor::new(context)));
    (executor, trace)
}

#[test]
fn executor_noop() {
    let (executor, trace) = setup();
    unsafe { executor.poll() };
    assert!(trace.get().is_empty())
}

#[test]
fn executor_task() {
    #[task]
    async fn task1(trace: Trace) {
        trace.push("poll task1")
    }

    let (executor, trace) = setup();
    executor.spawner().spawn(task1(trace.clone())).unwrap();

    unsafe { executor.poll() };
    unsafe { executor.poll() };

    assert_eq!(
        trace.get(),
        &[
            "pend",       // spawning a task pends the executor
            "poll task1", // poll only once.
        ]
    )
}

#[test]
fn executor_task_self_wake() {
    #[task]
    async fn task1(trace: Trace) {
        poll_fn(|cx| {
            trace.push("poll task1");
            cx.waker().wake_by_ref();
            Poll::Pending
        })
        .await
    }

    let (executor, trace) = setup();
    executor.spawner().spawn(task1(trace.clone())).unwrap();

    unsafe { executor.poll() };
    unsafe { executor.poll() };

    assert_eq!(
        trace.get(),
        &[
            "pend",       // spawning a task pends the executor
            "poll task1", //
            "pend",       // task self-wakes
            "poll task1", //
            "pend",       // task self-wakes
        ]
    )
}

#[test]
fn executor_task_self_wake_twice() {
    #[task]
    async fn task1(trace: Trace) {
        poll_fn(|cx| {
            trace.push("poll task1");
            cx.waker().wake_by_ref();
            trace.push("poll task1 wake 2");
            cx.waker().wake_by_ref();
            Poll::Pending
        })
        .await
    }

    let (executor, trace) = setup();
    executor.spawner().spawn(task1(trace.clone())).unwrap();

    unsafe { executor.poll() };
    unsafe { executor.poll() };

    assert_eq!(
        trace.get(),
        &[
            "pend",              // spawning a task pends the executor
            "poll task1",        //
            "pend",              // task self-wakes
            "poll task1 wake 2", // task self-wakes again, shouldn't pend
            "poll task1",        //
            "pend",              // task self-wakes
            "poll task1 wake 2", // task self-wakes again, shouldn't pend
        ]
    )
}

#[test]
fn executor_task_cfg_args() {
    // simulate cfg'ing away argument c
    #[task]
    async fn task1(a: u32, b: u32, #[cfg(any())] c: u32) {
        let (_, _) = (a, b);
    }

    #[task]
    async fn task2(a: u32, b: u32, #[cfg(all())] c: u32) {
        let (_, _, _) = (a, b, c);
    }
}
