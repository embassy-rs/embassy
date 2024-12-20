# embassy-time-queue-utils

This crate contains timer queues to help implementing an [`embassy-time-driver`](https://crates.io/crates/embassy-time-driver).

As a HAL user, you should only depend on this crate if your application does not use
`embassy-executor` and your HAL does not configure a generic queue by itself.

As a HAL implementer, you need to depend on this crate if you want to implement a time driver,
but how you should do so is documented in `embassy-time-driver`.
