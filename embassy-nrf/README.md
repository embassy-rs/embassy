# Embassy nRF HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The Embassy nRF HAL targets the Nordic Semiconductor nRF family of hardware. The HAL implements both blocking and async APIs
for many peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mod and handling interrupts, so that applications can focus on more important matters.

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

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

