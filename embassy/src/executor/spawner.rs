use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

use super::raw;

#[must_use = "Calling a task function does nothing on its own. You must pass the returned SpawnToken to Executor::spawn()"]
pub struct SpawnToken<F> {
    raw_task: Option<NonNull<raw::TaskHeader>>,
    phantom: PhantomData<*mut F>,
}

impl<F> SpawnToken<F> {
    pub(crate) unsafe fn new(raw_task: NonNull<raw::TaskHeader>) -> Self {
        Self {
            raw_task: Some(raw_task),
            phantom: PhantomData,
        }
    }

    pub(crate) fn new_failed() -> Self {
        Self {
            raw_task: None,
            phantom: PhantomData,
        }
    }
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
    pub(crate) unsafe fn new(executor: &'static raw::Executor) -> Self {
        Self {
            executor,
            not_send: PhantomData,
        }
    }

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

    /// Used by the `embassy_macros::main!` macro to throw an error when spawn
    /// fails. This is here to allow conditional use of `defmt::unwrap!`
    /// without introducing a `defmt` feature in the `embassy_macros` package,
    /// which would require use of `-Z namespaced-features`.
    pub fn must_spawn<F>(&self, token: SpawnToken<F>) -> () {
        unwrap!(self.spawn(token));
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
