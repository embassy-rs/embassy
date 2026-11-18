# embassy-time-driver

This crate contains the driver trait necessary for adding [`embassy-time`](https://crates.io/crates/embassy-time) support
for a new hardware platform.

If you want to *use* `embassy-time` with already made drivers, you should depend on the main `embassy-time` crate, not on this crate.

If you are writing a driver, you  should depend only on this crate, not on the main `embassy-time` crate.
This will allow your driver to continue working for newer `embassy-time` major versions, without needing an update,
if the driver trait has not had breaking changes.

## How it works

`embassy-time` is backed by a global "time driver" specified at build time.
Only one driver can be active in a program.

All methods and structs transparently call into the active driver. This makes it
possible for libraries to use `embassy-time` in a driver-agnostic way without
requiring generic parameters.

## Selecting the correct tick rate

In order to select the tick rate that the driver is compiled for, set the `embassy_time_driver_tick_hz` cfg to the desired value. For instance,
an RTC running at 32_768 Hz would set `RUSTFLAGS='--cfg embassy_time_driver_tick_hz="32_768"'`. This can also be configured via [`.cargo/config.toml`][0].

[0]: https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags