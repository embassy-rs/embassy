# embassy-boot

An [Embassy](https://embassy.dev) project.

A lightweight bootloader supporting firmware updates in a power-fail-safe way, with trial boots and rollbacks.

The bootloader can be used either as a library or be flashed directly with the default configuration derived from linker scripts.

By design, the bootloader does not provide any network capabilities. Networking capabilities for fetching new firmware can be provided by the user application, using the bootloader as a library for updating the firmware, or by using the bootloader as a library and adding this capability yourself.

## Hardware support

The bootloader supports different hardware in separate crates:

* `embassy-boot-nrf` - for the nRF microcontrollers.
* `embassy-boot-rp` - for the RP2040 microcontrollers.
* `embassy-boot-stm32` - for the STM32 microcontrollers.

## Minimum supported Rust version (MSRV)

`embassy-boot` is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
