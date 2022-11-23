# embassy-boot-nrf

An [Embassy](https://embassy.dev) project.

An adaptation of `embassy-boot` for nRF. 

## Features

* Load applications with our without the softdevice.
* Configure bootloader partitions based on linker script.
* Using watchdog timer to detect application failure.


## Minimum supported Rust version (MSRV)

`embassy-boot-nrf` requires Rust nightly to compile as it relies on async traits for interacting with the flash peripherals.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
