# WIZnet `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for the WIZnet SPI ethernet chips, operating in MACRAW mode.

See [`examples`](https://github.com/embassy-rs/embassy/tree/main/examples/rp) directory for usage examples with the rp2040 [`WIZnet W5500-EVB-Pico`](https://www.wiznet.io/product-item/w5500-evb-pico/) module.

## Supported chips

- W5500
- W5100S

## Interoperability

This crate can run on any executor.

It supports any SPI driver implementing [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).
