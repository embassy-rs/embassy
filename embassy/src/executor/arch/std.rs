use std::marker::PhantomData;
use std::sync::{Condvar, Mutex};

use super::{raw, Spawner};

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
    signaler: &'static Signaler,
}

impl Executor {
    pub fn new() -> Self {
        let signaler = &*Box::leak(Box::new(Signaler::new()));
        Self {
            inner: raw::Executor::new(
                |p| unsafe {
                    let s = &*(p as *const () as *const Signaler);
                    s.signal()
                },
                signaler as *const _ as _,
            ),
            not_send: PhantomData,
            signaler,
        }
    }

    /// Runs the executor.
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(unsafe { self.inner.spawner() });

        loop {
            unsafe { self.inner.run_queued() };
            self.signaler.wait()
        }
    }
}

struct Signaler {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Signaler {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            signaled = self.condvar.wait(signaled).unwrap();
        }
        *signaled = false;
    }

    fn signal(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        *signaled = true;
        self.condvar.notify_one();
    }
}
