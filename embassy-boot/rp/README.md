# embassy-boot-rp

An [Embassy](https://embassy.dev) project.

An adaptation of `embassy-boot` for RP2040.

NOTE: The applications using this bootloader should not link with the `link-rp.x` linker script.

## Features

* Configure bootloader partitions based on linker script.
* Load applications from active partition.

## Minimum supported Rust version (MSRV)

`embassy-boot-rp` requires Rust nightly to compile as it relies on async traits for interacting with the flash peripherals.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
