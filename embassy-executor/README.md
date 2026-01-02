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
