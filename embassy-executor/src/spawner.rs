use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::Ordering;
use core::task::Poll;

use super::raw;
#[cfg(feature = "trace")]
use crate::raw::trace::TaskRefTrace;

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
    pub(crate) raw_task: Option<raw::TaskRef>,
    phantom: PhantomData<*mut S>,
}

impl<S> SpawnToken<S> {
    pub(crate) unsafe fn new(raw_task: raw::TaskRef) -> Self {
        Self {
            raw_task: Some(raw_task),
            phantom: PhantomData,
        }
    }

    /// Returns the task id if available, otherwise 0
    /// This can be used in combination with rtos-trace to match task names with id's
    pub fn id(&self) -> u32 {
        match self.raw_task {
            None => 0,
            Some(t) => t.as_ptr() as u32,
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
#[derive(Copy, Clone)]
pub enum SpawnError {
    /// Too many instances of this task are already running.
    ///
    /// By default, a task marked with `#[embassy_executor::task]` can only have one instance
    /// running at a time. You may allow multiple instances to run in parallel with
    /// `#[embassy_executor::task(pool_size = 4)]`, at the cost of higher RAM usage.
    Busy,
}

impl core::fmt::Debug for SpawnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl core::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SpawnError::Busy => write!(f, "Busy - Too many instances of this task are already running. Check the `pool_size` attribute of the task."),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SpawnError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            SpawnError::Busy => defmt::write!(f, "Busy - Too many instances of this task are already running. Check the `pool_size` attribute of the task."),
        }
    }
}

impl core::error::Error for SpawnError {}

/// Handle to spawn tasks into an executor.
///
/// This Spawner can spawn any task (Send and non-Send ones), but it can
/// only be used in the executor thread (it is not Send itself).
///
/// If you want to spawn tasks from another thread, use [SendSpawner].
#[derive(Copy, Clone)]
pub struct Spawner {
    pub(crate) executor: &'static raw::Executor,
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
    /// Using this method is discouraged due to it being unsafe. Consider the following
    /// alternatives instead:
    ///
    /// - Pass the initial `Spawner` as an argument to tasks. Note that it's `Copy`, so you can
    ///   make as many copies of it as you want.
    /// - Use `SendSpawner::for_current_executor()` instead, which is safe but can only be used
    ///   if task arguments are `Send`.
    ///
    /// The only case where using this method is absolutely required is obtaining the `Spawner`
    /// for an `InterruptExecutor`.
    ///
    /// # Safety
    ///
    /// You must only execute this with an async `Context` created by the Embassy executor.
    /// You must not execute it with manually-created `Context`s.
    ///
    /// # Panics
    ///
    /// Panics if the current executor is not an Embassy executor.
    pub unsafe fn for_current_executor() -> impl Future<Output = Self> {
        poll_fn(|cx| {
            let task = raw::task_from_waker(cx.waker());
            let executor = unsafe {
                task.header()
                    .executor
                    .load(Ordering::Relaxed)
                    .as_ref()
                    .unwrap_unchecked()
            };
            let executor = unsafe { raw::Executor::wrap(executor) };
            Poll::Ready(Self::new(executor))
        })
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

    // Used by the `embassy_executor_macros::main!` macro to throw an error when spawn
    // fails. This is here to allow conditional use of `defmt::unwrap!`
    // without introducing a `defmt` feature in the `embassy_executor_macros` package,
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

    /// Return the unique ID of this Spawner's Executor.
    pub fn executor_id(&self) -> usize {
        self.executor.id()
    }
}

/// Extension trait adding tracing capabilities to the Spawner
///
/// This trait provides an additional method to spawn tasks with an associated name,
/// which can be useful for debugging and tracing purposes.
pub trait SpawnerTraceExt {
    /// Spawns a new task with a specified name.
    ///
    /// # Arguments
    /// * `name` - Static string name to associate with the task
    /// * `token` - Token representing the task to spawn
    ///
    /// # Returns
    /// Result indicating whether the spawn was successful
    fn spawn_named<S>(&self, name: &'static str, token: SpawnToken<S>) -> Result<(), SpawnError>;
}

/// Implementation of the SpawnerTraceExt trait for Spawner when trace is enabled
#[cfg(feature = "trace")]
impl SpawnerTraceExt for Spawner {
    fn spawn_named<S>(&self, name: &'static str, token: SpawnToken<S>) -> Result<(), SpawnError> {
        let task = token.raw_task;
        core::mem::forget(token);

        match task {
            Some(task) => {
                // Set the name and ID when trace is enabled
                task.set_name(Some(name));
                let task_id = task.as_ptr() as u32;
                task.set_id(task_id);

                unsafe { self.executor.spawn(task) };
                Ok(())
            }
            None => Err(SpawnError::Busy),
        }
    }
}

/// Implementation of the SpawnerTraceExt trait for Spawner when trace is disabled
#[cfg(not(feature = "trace"))]
impl SpawnerTraceExt for Spawner {
    fn spawn_named<S>(&self, _name: &'static str, token: SpawnToken<S>) -> Result<(), SpawnError> {
        // When trace is disabled, just forward to regular spawn and ignore the name
        self.spawn(token)
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
    pub fn for_current_executor() -> impl Future<Output = Self> {
        poll_fn(|cx| {
            let task = raw::task_from_waker(cx.waker());
            let executor = unsafe {
                task.header()
                    .executor
                    .load(Ordering::Relaxed)
                    .as_ref()
                    .unwrap_unchecked()
            };
            Poll::Ready(Self::new(executor))
        })
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
