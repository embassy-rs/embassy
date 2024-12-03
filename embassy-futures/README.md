# embassy-futures

An [Embassy](https://embassy.dev) project.

Utilities for working with futures, compatible with `no_std` and not using `alloc`. Optimized for code size,
ideal for embedded systems.

- Future combinators, like [`join`](join) and [`select`](select)
- Utilities to use `async` without a fully fledged executor: [`block_on`](block_on::block_on) and [`yield_now`](yield_now::yield_now).

## Interoperability

Futures from this crate can run on any executor.
