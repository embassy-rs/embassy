# Embassy nRF HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The Embassy nRF HAL targets the Nordic Semiconductor nRF family of hardware. The HAL implements both blocking and async APIs
for many peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mode and handling interrupts, so that applications can focus on more important matters.

NOTE: The Embassy HALs can be used both for non-async and async operations. For async, you can choose which runtime you want to use.

For a complete list of available peripherals and features, see the [embassy-nrf documentation](https://docs.embassy.dev/embassy-nrf).

## Hardware support

The `embassy-nrf` HAL supports most variants of the nRF family:

* nRF51 ([examples](https://github.com/embassy-rs/embassy/tree/main/examples/nrf51))
* nRF52 ([examples](https://github.com/embassy-rs/embassy/tree/main/examples/nrf52840))
* nRF53 ([examples](https://github.com/embassy-rs/embassy/tree/main/examples/nrf5340))
* nRF91 ([examples](https://github.com/embassy-rs/embassy/tree/main/examples/nrf9160))

Most peripherals are supported, but can vary between chip families. To check what's available, make sure to pick the MCU you're targeting in the top menu in the [documentation](https://docs.embassy.dev/embassy-nrf).

For MCUs with TrustZone support, both Secure (S) and Non-Secure (NS) modes are supported. Running in Secure mode
allows running Rust code without a SPM or TF-M binary, saving flash space and simplifying development.

## Time driver

If the `time-driver-rtc1` feature is enabled, the HAL uses the RTC peripheral as a global time driver for [embassy-time](https://crates.io/crates/embassy-time), with a tick rate of 32768 Hz.

## Embedded-hal

The `embassy-nrf` HAL implements the traits from [embedded-hal](https://crates.io/crates/embedded-hal) (v0.2 and 1.0) and [embedded-hal-async](https://crates.io/crates/embedded-hal-async), as well as [embedded-io](https://crates.io/crates/embedded-io) and [embedded-io-async](https://crates.io/crates/embedded-io-async).

## Interoperability

This crate can run on any executor.

Optionally, some features requiring [`embassy-time`](https://crates.io/crates/embassy-time) can be activated with the `time` feature. If you enable it,
you must link an `embassy-time` driver in your project.

## EasyDMA considerations

On nRF chips, peripherals can use the so called EasyDMA feature to offload the task of interacting
with peripherals. It takes care of sending/receiving data over a variety of bus protocols (TWI/I2C, UART, SPI).
However, EasyDMA requires the buffers used to transmit and receive data to reside in RAM. Unfortunately, Rust
slices will not always do so. The following example using the SPI peripheral shows a common situation where this might happen:

```rust,ignore
// As we pass a slice to the function whose contents will not ever change,
// the compiler writes it into the flash and thus the pointer to it will
// reference static memory. Since EasyDMA requires slices to reside in RAM,
// this function call will fail.
let result = spim.write_from_ram(&[1, 2, 3]);
assert_eq!(result, Err(Error::BufferNotInRAM));

// The data is still static and located in flash. However, since we are assigning
// it to a variable, the compiler will load it into memory. Passing a reference to the
// variable will yield a pointer that references dynamic memory, thus making EasyDMA happy.
// This function call succeeds.
let data = [1, 2, 3];
let result = spim.write_from_ram(&data);
assert!(result.is_ok());
```

Each peripheral struct which uses EasyDMA ([`Spim`](spim::Spim), [`Uarte`](uarte::Uarte), [`Twim`](twim::Twim)) has two variants of their mutating functions:
- Functions with the suffix (e.g. [`write_from_ram`](spim::Spim::write_from_ram), [`transfer_from_ram`](spim::Spim::transfer_from_ram)) will return an error if the passed slice does not reside in RAM.
- Functions without the suffix (e.g. [`write`](spim::Spim::write), [`transfer`](spim::Spim::transfer)) will check whether the data is in RAM and copy it into memory prior to transmission.

Since copying incurs a overhead, you are given the option to choose from `_from_ram` variants which will
fail and notify you, or the more convenient versions without the suffix which are potentially a little bit
more inefficient. Be aware that this overhead is not only in terms of instruction count but also in terms of memory usage
as the methods without the suffix will be allocating a statically sized buffer (up to 512 bytes for the nRF52840).

Note that the methods that read data like [`read`](spim::Spim::read) and [`transfer_in_place`](spim::Spim::transfer_in_place) do not have the corresponding `_from_ram` variants as
mutable slices always reside in RAM.
