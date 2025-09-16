# embassy-executor-time-queue

This crate defines the timer queue item that embassy-executor provides, and a way to access it, for
executor-integrated timer queues. The crate decouples the release cycle of embassy-executor from
that of the queue implementations'.

As a HAL implementer, you only need to depend on this crate if you want to implement executor-integrated
timer queues yourself, without using [`embassy-time-queue-utils`](https://crates.io/crates/embassy-time-queue-utils).

As a HAL user, you should not need to depend on this crate.
