#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-std`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use std::marker::PhantomData;
    use std::sync::{Condvar, Mutex};

    pub use embassy_executor_macros::main_std as main;

    use crate::{Spawner, raw};

    #[unsafe(export_name = "__pender")]
    fn __pender(context: *mut ()) {
        let signaler: &'static Signaler = unsafe { std::mem::transmute(context) };
        signaler.signal()
    }

    /// Single-threaded std-based executor.
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
        signaler: &'static Signaler,
        #[cfg(feature = "idle-hook")]
        idle_hook: fn(&Executor),
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            let signaler = Box::leak(Box::new(Signaler::new()));
            Self {
                inner: raw::Executor::new(signaler as *mut Signaler as *mut ()),
                not_send: PhantomData,
                signaler,
                #[cfg(feature = "idle-hook")]
                idle_hook: Executor::default_idle,
            }
        }

        /// Add idle-hook to Executor instance.
        #[cfg(feature = "idle-hook")]
        pub fn with_idle_hook(mut self, idle_hook: fn(&Executor)) -> Self {
            self.idle_hook = idle_hook;
            self
        }

        /// Put Executor into default idle state.
        ///
        /// This function might also be called from the application's context,
        /// e.g. from a custom idle-hook.
        #[inline(always)]
        pub fn default_idle(&self) {
            self.signaler.wait();
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
        /// After all tasks have been polled, this function enters an idle state by either calling
        /// the inlined [`Executor::default_idle`] function or a custom idle-hook.
        ///
        /// This function never returns.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe { self.inner.poll() };

                #[cfg(feature = "idle-hook")]
                (self.idle_hook)(self);
                #[cfg(not(feature = "idle-hook"))]
                self.default_idle();
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
