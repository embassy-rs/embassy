# embassy-time-queue-driver

This crate contains the driver trait used by the [`embassy-time`](https://crates.io/crates/embassy-time) timer queue.

You should rarely need to use this crate directly. Only use it when implementing your own timer queue.

There is two timer queue implementations, one in `embassy-time` enabled by the `generic-queue` feature, and 
another in `embassy-executor` enabled by the `integrated-timers` feature.
