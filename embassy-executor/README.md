# embassy-executor

An async/await executor designed for embedded usage.

- No `alloc`, no heap needed.
- With nightly Rust, task futures can be fully statically allocated.
- No "fixed capacity" data structures, executor works with 1 or 1000 tasks without needing config/tuning.
- Integrated timer queue: sleeping is easy, just do `Timer::after_secs(1).await;`.
- No busy-loop polling: CPU sleeps when there's no work to do, using interrupts or `WFE/SEV`.
- Efficient polling: a wake will only poll the woken task, not all of them.
- Fair: a task can't monopolize CPU time even if it's constantly being woken. All other tasks get a chance to run before a given task gets polled for the second time.
- Creating multiple executor instances is supported, to run tasks with multiple priority levels. This allows higher-priority tasks to preempt lower-priority tasks.

## Task arena

When the `nightly` Cargo feature is not enabled, `embassy-executor` allocates tasks out of an arena (a very simple bump allocator).

If the task arena gets full, the program will panic at runtime. To guarantee this doesn't happen, you must set the size to the sum of sizes of all tasks.

Tasks are allocated from the arena when spawned for the first time. If the task exists, the allocation is not released to the arena, but can be reused to spawn the task again. For multiple-instance tasks (like `#[embassy_executor::task(pool_size = 4)]`), the first spawn will allocate memory for all instances. This is done for performance and to increase predictability (for example, spawning at least 1 instance of every task at boot guarantees an immediate panic if the arena is too small, while allocating instances on-demand could delay the panic to only when the program is under load).

The arena size can be configured in two ways:

- Via Cargo features: enable a Cargo feature like `task-arena-size-8192`. Only a selection of values
  is available, see [Task Area Sizes](#task-arena-size) for reference.
- Via environment variables at build time: set the variable named `EMBASSY_EXECUTOR_TASK_ARENA_SIZE`. For example
  `EMBASSY_EXECUTOR_TASK_ARENA_SIZE=4321 cargo build`. You can also set them in the `[env]` section of `.cargo/config.toml`.
  Any value can be set, unlike with Cargo features.

Environment variables take precedence over Cargo features. If two Cargo features are enabled for the same setting
with different values, compilation fails.

## Statically allocating tasks

When using nightly Rust, enable the `nightly` Cargo feature. This will make `embassy-executor` use the `type_alias_impl_trait` feature to allocate all tasks in `static`s. Each task gets its own `static`, with the exact size to hold the task (or multiple instances of it, if using `pool_size`) calculated automatically at compile time. If tasks don't fit in RAM, this is detected at compile time by the linker. Runtime panics due to running out of memory are not possible.

The configured arena size is ignored, no arena is used at all.
