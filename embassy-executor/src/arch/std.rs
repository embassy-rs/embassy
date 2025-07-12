#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-std`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use std::marker::PhantomData;
    use std::sync::{Condvar, Mutex};

    pub use embassy_executor_macros::main_std as main;

    use crate::{raw, Spawner};

    #[export_name = "__pender"]
    fn __pender(context: *mut ()) {
        let signaler: &'static Signaler = unsafe { std::mem::transmute(context) };
        signaler.signal()
    }

    /// Single-threaded std-based executor.
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
        signaler: &'static Signaler,
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            let signaler = Box::leak(Box::new(Signaler::new()));
            Self {
                inner: raw::Executor::new(signaler as *mut Signaler as *mut ()),
                not_send: PhantomData,
                signaler,
            }
        }

        /// Run the executor.
        ///
        /// The `init` closure is called with a [`Spawner`] that spawns tasks on
        /// this executor. Use it to spawn the initial task(s). After `init` returns,
        /// the executor starts running the tasks.
        ///
        /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
        /// for example by passing it as an argument to the initial tasks.
        ///
        /// This function requires `&'static mut self`. This means you have to store the
        /// Executor instance in a place where it'll live forever and grants you mutable
        /// access. There's a few ways to do this:
        ///
        /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
        /// - a `static mut` (unsafe)
        /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
        ///
        /// This function never returns.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe { self.inner.poll() };
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
}
