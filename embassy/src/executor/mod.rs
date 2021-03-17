pub use embassy_macros::task;

use core::marker::PhantomData;
use core::ptr::NonNull;
use core::{mem, ptr};

pub mod raw;
mod run_queue;
pub(crate) mod timer;
mod timer_queue;
mod util;
mod waker;

use crate::fmt::panic;
use crate::interrupt::{Interrupt, InterruptExt};
use crate::time::Alarm;

#[must_use = "Calling a task function does nothing on its own. You must pass the returned SpawnToken to Executor::spawn()"]
pub struct SpawnToken<F> {
    raw_task: Option<NonNull<raw::TaskHeader>>,
    phantom: PhantomData<*mut F>,
}

impl<F> Drop for SpawnToken<F> {
    fn drop(&mut self) {
        // TODO deallocate the task instead.
        panic!("SpawnToken instances may not be dropped. You must pass them to Executor::spawn()")
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpawnError {
    Busy,
}

/// Handle to spawn tasks into an executor.
///
/// This Spawner can spawn any task (Send and non-Send ones), but it can
/// only be used in the executor thread (it is not Send itself).
///
/// If you want to spawn tasks from another thread, use [SendSpawner].
#[derive(Copy, Clone)]
pub struct Spawner {
    executor: &'static raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Spawner {
    pub fn spawn<F>(&self, token: SpawnToken<F>) -> Result<(), SpawnError> {
        let task = token.raw_task;
        mem::forget(token);

        match task {
            Some(task) => {
                unsafe { self.executor.spawn(task) };
                Ok(())
            }
            None => Err(SpawnError::Busy),
        }
    }

    /// Convert this Spawner to a SendSpawner. This allows you to send the
    /// spawner to other threads, but the spawner loses the ability to spawn
    /// non-Send tasks.
    pub fn make_send(&self) -> SendSpawner {
        SendSpawner {
            executor: self.executor,
            not_send: PhantomData,
        }
    }
}

/// Handle to spawn tasks into an executor from any thread.
///
/// This Spawner can be used from any thread (it implements Send and Sync, so after  any task (Send and non-Send ones), but it can
/// only be used in the executor thread (it is not Send itself).
///
/// If you want to spawn tasks from another thread, use [SendSpawner].
#[derive(Copy, Clone)]
pub struct SendSpawner {
    executor: &'static raw::Executor,
    not_send: PhantomData<*mut ()>,
}

unsafe impl Send for SendSpawner {}
unsafe impl Sync for SendSpawner {}

/// Handle to spawn tasks to an executor.
///
/// This Spawner can spawn any task (Send and non-Send ones), but it can
/// only be used in the executor thread (it is not Send itself).
///
/// If you want to spawn tasks from another thread, use [SendSpawner].
impl SendSpawner {
    pub fn spawn<F: Send>(&self, token: SpawnToken<F>) -> Result<(), SpawnError> {
        let header = token.raw_task;
        mem::forget(token);

        match header {
            Some(header) => {
                unsafe { self.executor.spawn(header) };
                Ok(())
            }
            None => Err(SpawnError::Busy),
        }
    }
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            inner: raw::Executor::new(|_| cortex_m::asm::sev(), ptr::null_mut()),
            not_send: PhantomData,
        }
    }

    pub fn set_alarm(&mut self, alarm: &'static dyn Alarm) {
        self.inner.set_alarm(alarm);
    }

    /// Runs the executor.
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(unsafe { self.inner.spawner() });

        loop {
            unsafe { self.inner.run_queued() };
            cortex_m::asm::wfe();
        }
    }
}

fn pend_by_number(n: u16) {
    #[derive(Clone, Copy)]
    struct N(u16);
    unsafe impl cortex_m::interrupt::InterruptNumber for N {
        fn number(self) -> u16 {
            self.0
        }
    }
    cortex_m::peripheral::NVIC::pend(N(n))
}

pub struct InterruptExecutor<I: Interrupt> {
    irq: I,
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl<I: Interrupt> InterruptExecutor<I> {
    pub fn new(irq: I) -> Self {
        let ctx = irq.number() as *mut ();
        Self {
            irq,
            inner: raw::Executor::new(|ctx| pend_by_number(ctx as u16), ctx),
            not_send: PhantomData,
        }
    }

    pub fn set_alarm(&mut self, alarm: &'static dyn Alarm) {
        self.inner.set_alarm(alarm);
    }

    /// Start the executor.
    ///
    /// `init` is called in the interrupt context, then the interrupt is
    /// configured to run the executor.
    pub fn start(&'static mut self, init: impl FnOnce(Spawner) + Send) {
        self.irq.disable();

        init(unsafe { self.inner.spawner() });

        self.irq.set_handler(|ctx| unsafe {
            let executor = &*(ctx as *const raw::Executor);
            executor.run_queued();
        });
        self.irq.set_handler_context(&self.inner as *const _ as _);
        self.irq.enable();
    }
}
