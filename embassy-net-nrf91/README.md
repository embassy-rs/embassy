# nRF91 `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) driver for Nordic nRF91-series cellular modems.

See the [`examples`](https://github.com/embassy-rs/embassy/tree/main/examples/nrf9160) directory for usage examples with the nRF9160.

## Interoperability

This crate can run on any executor.

## Features

By default the nrf91 crate supports NRF9160 and similar. With newer generation of chips use feature `nrf9151` to support the new data sizes.
