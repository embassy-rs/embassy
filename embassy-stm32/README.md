# Embassy STM32 HAL

The embassy-stm32 HAL aims to provide a safe, idiomatic hardware abstraction layer for all STM32 families. The HAL implements both blocking and async APIs for many peripherals. Where appropriate, traits from both blocking and asynchronous versions of [embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/) v0.2 and v1.0 are implemented, as well as serial traits from embedded-io\[-async].

* [embassy-stm32 on crates.io](https://crates.io/crates/embassy-stm32)
* [Documentation](https://docs.embassy.dev/embassy-stm32/) (**Important:** use docs.embassy.dev rather than docs.rs to see the specific docs for the chip you’re using!)
* [Source](https://github.com/embassy-rs/embassy/tree/main/embassy-stm32)
* [Examples](https://github.com/embassy-rs/embassy/tree/main/examples)

## embassy-stm32 supports all STM32 chip families

STM32 microcontrollers come in many families and flavors, and supporting all of them is a big undertaking. Embassy takes advantage of the fact that the STM32 peripheral versions are shared across chip families. For example, instead of re-implementing the SPI peripheral for every STM32 chip family, embassy has a single SPI implementation that depends on code-generated register types that are identical for STM32 families with the same version of a given peripheral.

In practice, this works as follows:

1. You tell the compiler which chip you’re using with a feature flag
1. The stm32-metapac module generates register types for that chip at compile time, based on data from the stm32-data module
1. The embassy-stm32 HAL picks the correct implementation each peripheral based on automatically-generated feature flags, and applies any other tweaks which are required for the HAL to work on that chip

Be aware that, while embassy-stm32 strives to consistently support all peripherals across all chips, this approach can lead to slightly different APIs and capabilities being available on different families. Check the [documentation](https://docs.embassy.dev/embassy-stm32/) for the specific chip you’re using to confirm exactly what’s available.

## Minimum supported Rust version (MSRV)
Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## embassy-time Timer Driver
If the `time` feature is enabled, embassy-stm32 provides a timer driver for use with [embassy-time](https://docs.embassy.dev/embassy-time/). You can pick which hardware timer is used for this internally via the `time-driver-*` features, or let embassy pick with `time-driver-any`.

embassy-time has a default tick rate of 1MHz, which is fast enough to cause problems with the 16-bit timers currently supported by the embassy-stm32 time driver (specifically, if a critical section delays an IRQ by more than 32ms). To avoid this, it’s recommended to pick a lower tick rate. 32.768kHz is a reasonable default for many purposes.