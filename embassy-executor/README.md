# embassy-executor

An async/await executor designed for embedded usage.

- No `alloc`, no heap needed.
- Tasks are statically allocated. Each task gets its own `static`, with the exact size to hold the task (or multiple instances of it, if using `pool_size`) calculated automatically at compile time. If tasks don't fit in RAM, this is detected at compile time by the linker. Runtime panics due to running out of memory are not possible.
- No "fixed capacity" data structures, executor works with 1 or 1000 tasks without needing config/tuning.
- Integrated timer queue: sleeping is easy, just do `Timer::after_secs(1).await;`.
- No busy-loop polling: CPU sleeps when there's no work to do, using interrupts or `WFE/SEV`.
- Efficient polling: a wake will only poll the woken task, not all of them.
- Fair: a task can't monopolize CPU time even if it's constantly being woken. All other tasks get a chance to run before a given task gets polled for the second time.
- Creating multiple executor instances is supported, to run tasks with multiple priority levels. This allows higher-priority tasks to preempt lower-priority tasks.

## Platforms

The executor requires a "platform" to be defined to work. A platform defines the following things:

- The main loop, which typically consists of an infinite loop of polling the executor then sleeping the current thread/core in a platform-specific way.
- A "pender" callback, which must cause the executor's thread/core to exit sleep so the executor gets polled again. This is called when a task running in the executor is woken.

The `embassy-executor` crate ships with support for some commonly used platforms, see the crate's feature documentation.

Chip-specific executor platform implementations are maintained in their respective HALs:

- `embassy-rp`: multicore support. Enabled with the `executor-interrupt` or `executor-thread` features.
- `embassy-stm32`: automatic low-power sleep support. Enabled with the `executor-interrupt` or `executor-thread` features.
- `embassy-mcxa`: automatic low-power sleep support. Enabled with the `executor-platform` feature.
- `esp-rtos`: ESP32 RTOS support, multicore support. Enabled with the `embassy` feature.

To use the executor, you must provide exactly one platform implementation, either from this crate, a HAL crate, or a custom one.

## Implementing a custom platform

To implement your own custom platform, e.g. on top of an RTOS, do the following:

1. define the `__pender` callback.

```rust,ignore
#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
    // `context` is the argument passed to `raw::Executor::new`. Here we're using it
    // to pass a handle to the RTOS task but you can use it for anything.
    my_rtos::notify_task(context as _);
}
```

2. Wrap the `raw::Executor` into your own `Executor` struct that defines the main loop.

```rust,ignore
pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(my_rtos::task_get_current() as _),
            not_send: PhantomData,
        }
    }

    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            unsafe { self.inner.poll() }
            my_rtos::task_wait_for_notification();
        }
    }
}
```