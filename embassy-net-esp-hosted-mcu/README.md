# ESP-Hosted MCU `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for Espressif SoCs running the [ESP-Hosted MCU](https://github.com/espressif/esp-hosted-mcu) stack.

## Interoperability

This crate can run on any executor.

It supports any SPI driver implementing [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).
