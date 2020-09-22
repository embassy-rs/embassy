# Embassy

Embassy is a project to make async/await a first-class option for embedded development.

The `embassy` crate defines some traits.

- `embassy::io`: Traits for byte-stream IO, essentially `no_std` compatible versions of `futures::io`.
- `embassy::flash`: Trait for an async flash device.
- More traits for SPI, I2C, UART async HAL coming soon.

The `embassy-nrf` crate contains implementations for nRF 52 series SoCs.

- `uarte`: UARTE driver implementing `AsyncBufRead` and `AsyncWrite`.
- `qspi`: QSPI driver implementing `Flash`.

Currently Embassy requires a recent nightly, mainly for `generic_associated_types` (for trait funcs returning futures) and `type_alias_impl_trait` (for returning futures implemented with `async{}` blocks). Stable support is a non-goal.

## Why the name?

EMBedded ASYnc.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
