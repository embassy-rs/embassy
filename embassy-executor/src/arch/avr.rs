#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-avr`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;

    pub use embassy_executor_macros::main_avr as main;
    use portable_atomic::{AtomicBool, Ordering};

    use crate::{raw, Spawner};

    static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    #[export_name = "__pender"]
    fn __pender(_context: *mut ()) {
        SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
    }

    /// avr Executor
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(core::ptr::null_mut()),
                not_send: PhantomData,
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
                unsafe {
                    avr_device::interrupt::disable();
                    if !SIGNAL_WORK_THREAD_MODE.swap(false, Ordering::SeqCst) {
                        avr_device::interrupt::enable();
                        avr_device::asm::sleep();
                    } else {
                        avr_device::interrupt::enable();
                        self.inner.poll();
                    }
                }
            }
        }
    }
}
