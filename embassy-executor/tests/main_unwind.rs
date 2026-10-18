//! `#[embassy_executor::main]` keeps the executor on the stack of the generated `main` and hands
//! out `&'static` references to it. A panic that unwinds out of `main` would pop that frame while
//! the references are still reachable, e.g. from the panic payload, so the process has to abort
//! instead.
//!
//! The test re-runs its own binary: the child panics inside the main task with a payload whose
//! `Drop` prints a marker, and the parent checks that the child aborted before that `Drop` ran.

#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

use std::process::Command;

use embassy_executor::Spawner;
use embassy_executor::raw::Executor;

const CHILD_ENV: &str = "EMBASSY_EXECUTOR_MAIN_UNWIND_CHILD";
const DROP_MARKER: &str = "main_unwind: panic payload dropped";

struct NoopPender;

impl embassy_executor::pender::Pender for NoopPender {
    fn pend(_context: *mut ()) {}
}

embassy_executor::pender_impl!(NoopPender);

struct TestExecutor {
    inner: Executor,
}

impl TestExecutor {
    fn new() -> Self {
        Self {
            inner: Executor::new(core::ptr::null_mut()),
        }
    }

    fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());
        loop {
            unsafe { self.inner.poll() };
        }
    }
}

/// Stands in for a payload that captured a `Spawner` or other reference into the executor.
struct Payload;

impl Drop for Payload {
    fn drop(&mut self) {
        eprintln!("{DROP_MARKER}");
    }
}

#[embassy_executor::main(executor = "TestExecutor")]
async fn main(_spawner: Spawner) {
    if std::env::var_os(CHILD_ENV).is_some() {
        std::panic::panic_any(Payload);
    }

    // Miri cannot spawn processes.
    if cfg!(miri) {
        std::process::exit(0);
    }

    let output = Command::new(std::env::current_exe().unwrap())
        .env(CHILD_ENV, "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains(DROP_MARKER) || output.status.success() {
        eprintln!("child exited with {} and stderr:\n{stderr}", output.status);
        eprintln!("test main_unwind ... FAILED: the panic unwound out of `main` and dropped its payload");
        std::process::exit(1);
    }

    println!("test main_unwind ... ok");
    std::process::exit(0);
}
