# ESP-Hosted `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for Espressif SoCs running the [ESP-Hosted-Fg](https://github.com/espressif/esp-hosted) or [ESP-Hosted-Mcu](https://github.com/espressif/esp-hosted-mcu) stack.

See [`examples`](https://github.com/embassy-rs/embassy/tree/main/examples/nrf52840) directory for usage examples with the nRF52840.

## Interoperability

This crate can run on any executor.

It supports any SPI driver implementing [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).
