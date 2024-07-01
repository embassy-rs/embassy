# Embassy RP HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The embassy-rp HAL targets the Raspberry Pi RP2040 microcontroller. The HAL implements both blocking and async APIs
for many peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mode and handling interrupts, so that applications can focus on more important matters.

* [embassy-rp on crates.io](https://crates.io/crates/embassy-rp)
* [Documentation](https://docs.embassy.dev/embassy-rp/)
* [Source](https://github.com/embassy-rs/embassy/tree/main/embassy-rp)
* [Examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin)

## `embassy-time` time driver

If the `time-driver` feature is enabled, the HAL uses the TIMER peripheral as a global time driver for [embassy-time](https://crates.io/crates/embassy-time), with a tick rate of 1MHz.

## Embedded-hal

The `embassy-rp` HAL implements the traits from [embedded-hal](https://crates.io/crates/embedded-hal) (v0.2 and 1.0) and [embedded-hal-async](https://crates.io/crates/embedded-hal-async), as well as [embedded-io](https://crates.io/crates/embedded-io) and [embedded-io-async](https://crates.io/crates/embedded-io-async).

## Interoperability

This crate can run on any executor.

Optionally, some features requiring [`embassy-time`](https://crates.io/crates/embassy-time) can be activated with the `time` feature. If you enable it,
you must link an `embassy-time` driver in your project.
