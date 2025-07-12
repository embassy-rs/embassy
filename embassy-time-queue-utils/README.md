# embassy-time-queue-utils

This crate contains timer queues to help implementing an [`embassy-time-driver`](https://crates.io/crates/embassy-time-driver).

As a HAL user, you should not need to depend on this crate.

As a HAL implementer, you need to depend on this crate if you want to implement a time driver,
but how you should do so is documented in `embassy-time-driver`.
