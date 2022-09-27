# embassy-futures

An [Embassy](https://embassy.dev) project.

Utilities for working with futures, compatible with `no_std` and not using `alloc`. Optimized for code size,
ideal for embedded systems.

- Future combinators, like [`join`](join) and [`select`](select)
- Utilities to use `async` without a fully fledged executor: [`block_on`](block_on::block_on) and [`yield_now`](yield_now::yield_now).

## Interoperability

Futures from this crate can run on any executor.

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

