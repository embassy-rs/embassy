# ESP-Hosted `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for Espressif SoCs running the the ESP-Hosted stack.

See [`examples`](https://github.com/embassy-rs/embassy/tree/main/examples/nrf52840) directory for usage examples with the nRF52840.

## Supported chips

- W5500
- W5100S

## Interoperability

This crate can run on any executor.

It supports any SPI driver implementing [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).


## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
