use core::sync::atomic::{AtomicBool, Ordering};

use crate::{Executor, Spawner};

/// Test runner that will be used by the #[test] macro (only supported for the `arch-std`)
pub struct TestRunner {
    inner: Executor,
    done: AtomicBool,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self {
            inner: Executor::new(),
            done: AtomicBool::new(false),
        }
    }
}

impl TestRunner {
    /// Call the closure with a spawner that can be used to spawn tasks.
    pub fn initialize(&'static self, init: impl FnOnce(Spawner)) {
        init(self.inner.spawner());
    }

    /// Run the executor until the test is done
    pub fn run_until_done(&'static self) {
        self.inner.run_until(|| self.done.load(Ordering::SeqCst));
    }

    /// Mark the test as done
    pub fn done(&'static self) {
        self.done.store(true, Ordering::SeqCst);
    }
}
