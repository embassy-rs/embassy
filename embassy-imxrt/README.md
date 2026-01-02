# Embassy iMXRT HAL

## Introduction

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so
raw register manipulation is not needed.

The Embassy iMXRT HAL targets the NXP iMXRT Family of MCUs. The HAL implements
both blocking and async APIs for many peripherals. The benefit of using the
async APIs is that the HAL takes care of waiting for peripherals to complete
operations in low power mode and handling of interrupts, so that applications
can focus on business logic.

NOTE: The Embassy HALs can be used both for non-async and async operations. For
async, you can choose which runtime you want to use.

For a complete list of available peripherals and features, see the
[embassy-imxrt documentation](https://docs.embassy.dev/embassy-imxrt).

## Hardware support

The `embassy-imxrt` HAL currently supports two main variants of the iMXRT
family:

* MIMXRT685S
  ([examples](https://github.com/OpenDevicePartnership/embassy-imxrt/tree/main/examples/rt685s-evk))
* MIMXRT633s
  ([examples](https://github.com/OpenDevicePartnership/embassy-imxrt/tree/main/examples/rt633))

Several peripherals are supported and tested on both supported chip variants. To
check what's available, make sure to the MCU you're targetting in the top menu
in the [documentation](https://docs.embassy.dev/embassy-imxrt).

## TrustZone support

TrustZone support is yet to be implemented.

## Time driver

If the `time-driver` feature is enabled, the HAL uses the RTC peripheral as a
global time driver for [embassy-time](https://crates.io/crates/embassy-time),
with a tick rate of 32768 Hz.

## Embedded-hal

The `embassy-imxrt` HAL implements the traits from
[embedded-hal](https://crates.io/crates/embedded-hal) (v0.2 and 1.0) and
[embedded-hal-async](https://crates.io/crates/embedded-hal-async), as well as
[embedded-io](https://crates.io/crates/embedded-io) and
[embedded-io-async](https://crates.io/crates/embedded-io-async).

## Interoperability

This crate can run on any executor.

Optionally, some features requiring
[`embassy-time`](https://crates.io/crates/embassy-time) can be activated with
the `time` feature. If you enable it, you must link an `embassy-time` driver in
your project.
