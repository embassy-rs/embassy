use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem;
use core::task::Poll;

use super::raw;

/// Token to spawn a newly-created task in an executor.
///
/// When calling a task function (like `#[embassy_executor::task] async fn my_task() { ... }`), the returned
/// value is a `SpawnToken` that represents an instance of the task, ready to spawn. You must
/// then spawn it into an executor, typically with [`Spawner::spawn()`].
///
/// The generic parameter `S` determines whether the task can be spawned in executors
/// in other threads or not. If `S: Send`, it can, which allows spawning it into a [`SendSpawner`].
/// If not, it can't, so it can only be spawned into the current thread's executor, with [`Spawner`].
///
/// # Panics
///
/// Dropping a SpawnToken instance panics. You may not "abort" spawning a task in this way.
/// Once you've invoked a task function and obtained a SpawnToken, you *must* spawn it.
#[must_use = "Calling a task function does nothing on its own. You must spawn the returned SpawnToken, typically with Spawner::spawn()"]
pub struct SpawnToken<S> {
    raw_task: Option<raw::TaskRef>,
    phantom: PhantomData<*mut S>,
}

impl<S> SpawnToken<S> {
    pub(crate) unsafe fn new(raw_task: raw::TaskRef) -> Self {
        Self {
            raw_task: Some(raw_task),
            phantom: PhantomData,
        }
    }

    /// Return a SpawnToken that represents a failed spawn.
    pub fn new_failed() -> Self {
        Self {
            raw_task: None,
            phantom: PhantomData,
        }
    }
}

impl<S> Drop for SpawnToken<S> {
    fn drop(&mut self) {
        // TODO deallocate the task instead.
        panic!("SpawnToken instances may not be dropped. You must pass them to Spawner::spawn()")
    }
}

/// Error returned when spawning a task.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpawnError {
    /// Too many instances of this task are already running.
    ///
    /// By default, a task marked with `#[embassy_executor::task]` can only have one instance
    /// running at a time. You may allow multiple instances to run in parallel with
    /// `#[embassy_executor::task(pool_size = 4)]`, at the cost of higher RAM usage.
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
    pub(crate) fn new(executor: &'static raw::Executor) -> Self {
        Self {
            executor,
            not_send: PhantomData,
        }
    }

    /// Get a Spawner for the current executor.
    ///
    /// This function is `async` just to get access to the current async
    /// context. It returns instantly, it does not block/yield.
    ///
    /// # Panics
    ///
    /// Panics if the current executor is not an Embassy executor.
    pub async fn for_current_executor() -> Self {
        poll_fn(|cx| {
            let task = raw::task_from_waker(cx.waker());
            let executor = unsafe { task.header().executor.get().unwrap_unchecked() };
            let executor = unsafe { raw::Executor::wrap(executor) };
            Poll::Ready(Self::new(executor))
        })
        .await
    }

    /// Spawn a task into an executor.
    ///
    /// You obtain the `token` by calling a task function (i.e. one marked with `#[embassy_executor::task]`).
    pub fn spawn<S>(&self, token: SpawnToken<S>) -> Result<(), SpawnError> {
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

    // Used by the `embassy_macros::main!` macro to throw an error when spawn
    // fails. This is here to allow conditional use of `defmt::unwrap!`
    // without introducing a `defmt` feature in the `embassy_macros` package,
    // which would require use of `-Z namespaced-features`.
    /// Spawn a task into an executor, panicking on failure.
    ///
    /// # Panics
    ///
    /// Panics if the spawning fails.
    pub fn must_spawn<S>(&self, token: SpawnToken<S>) {
        unwrap!(self.spawn(token));
    }

    /// Convert this Spawner to a SendSpawner. This allows you to send the
    /// spawner to other threads, but the spawner loses the ability to spawn
    /// non-Send tasks.
    pub fn make_send(&self) -> SendSpawner {
        SendSpawner::new(&self.executor.inner)
    }
}

/// Handle to spawn tasks into an executor from any thread.
///
/// This Spawner can be used from any thread (it is Send), but it can
/// only spawn Send tasks. The reason for this is spawning is effectively
/// "sending" the tasks to the executor thread.
///
/// If you want to spawn non-Send tasks, use [Spawner].
#[derive(Copy, Clone)]
pub struct SendSpawner {
    executor: &'static raw::SyncExecutor,
}

impl SendSpawner {
    pub(crate) fn new(executor: &'static raw::SyncExecutor) -> Self {
        Self { executor }
    }

    /// Get a Spawner for the current executor.
    ///
    /// This function is `async` just to get access to the current async
    /// context. It returns instantly, it does not block/yield.
    ///
    /// # Panics
    ///
    /// Panics if the current executor is not an Embassy executor.
    pub async fn for_current_executor() -> Self {
        poll_fn(|cx| {
            let task = raw::task_from_waker(cx.waker());
            let executor = unsafe { task.header().executor.get().unwrap_unchecked() };
            Poll::Ready(Self::new(executor))
        })
        .await
    }

    /// Spawn a task into an executor.
    ///
    /// You obtain the `token` by calling a task function (i.e. one marked with `#[embassy_executor::task]`).
    pub fn spawn<S: Send>(&self, token: SpawnToken<S>) -> Result<(), SpawnError> {
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

    /// Spawn a task into an executor, panicking on failure.
    ///
    /// # Panics
    ///
    /// Panics if the spawning fails.
    pub fn must_spawn<S: Send>(&self, token: SpawnToken<S>) {
        unwrap!(self.spawn(token));
    }
}
