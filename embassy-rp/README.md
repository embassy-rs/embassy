# Embassy RP HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The Embassy RP HAL targets the Raspberry Pi 2040 family of hardware. The HAL implements both blocking and async APIs
for many peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mod and handling interrupts, so that applications can focus on more important matters.

NOTE: The Embassy HALs can be used both for non-async and async operations. For async, you can choose which runtime you want to use.

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
