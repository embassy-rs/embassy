# Embassy MSPM0 HAL

The embassy-mspm0 HAL aims to provide a safe, idiomatic hardware abstraction layer for all MSPM0 and MSPS003 chips.

* [Documentation](https://docs.embassy.dev/embassy-mspm0/) (**Important:** use docs.embassy.dev rather than docs.rs to see the specific docs for the chip you’re using!)
* [Source](https://github.com/embassy-rs/embassy/tree/main/embassy-mspm0)
* [Examples](https://github.com/embassy-rs/embassy/tree/main/examples)

## Embedded-hal

The `embassy-mspm0` HAL implements the traits from [embedded-hal](https://crates.io/crates/embedded-hal) (1.0) and [embedded-hal-async](https://crates.io/crates/embedded-hal-async), as well as [embedded-io](https://crates.io/crates/embedded-io) and [embedded-io-async](https://crates.io/crates/embedded-io-async).

## A note on feature flag names

Feature flag names for chips do not include temperature rating or distribution format.

Usually chapter 10 of your device's datasheet will explain the device nomenclature and how to decode it. Feature names in embassy-mspm0 only use the following from device nomenclature:
- MCU platform
- Product family
- Device subfamily
- Flash memory
- Package type

This means for a part such as `MSPM0G3507SPMR`, the feature name is `mspm0g3507pm`. This also means that `MSPM0G3507QPMRQ1` uses the feature `mspm0g3507pm`, since the Q1 parts are just qualified variants of the base G3507 with a PM (QFP-64) package.

## Interoperability

This crate can run on any executor.
