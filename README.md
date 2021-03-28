# Embassy

Embassy is a project to make async/await a first-class option for embedded development. For more information and instructions to
get started, click [here](https://github.com/embassy-rs/embassy/wiki).

## Traits and types

`embassy` provides a set of traits and types specifically designed for `async` usage.

- `embassy::io`: `AsyncBufRead`, `AsyncWrite`. Traits for byte-stream IO, essentially `no_std` compatible versions of `futures::io`.
- `embassy::traits::flash`: Flash device trait.
- `embassy::time`: `Clock` and `Alarm` traits. Std-like `Duration` and `Instant`.
- More traits for SPI, I2C, UART async HAL coming soon.

## Executor

The `embassy::executor` module provides an async/await executor designed for embedded usage.

- No `alloc`, no heap needed. Task futures are statically allocated.
- No "fixed capacity" data structures, executor works with 1 or 1000 tasks without needing config/tuning.
- Integrated timer queue: sleeping is easy, just do `Timer::after(Duration::from_secs(1)).await;`.
- No busy-loop polling: CPU sleeps when there's no work to do, using interrupts or `WFE/SEV`.
- Efficient polling: a wake will only poll the woken task, not all of them.
- Fair: a task can't monopolize CPU time even if it's constantly being woken. All other tasks get a chance to run before a given task gets polled for the second time.
- Creating multiple executor instances is supported, to run tasks with multiple priority levels. This allows higher-priority tasks to preempt lower-priority tasks.

## Utils

`embassy::util` contains some lightweight async/await utilities, mainly helpful for async driver development (signaling a task that an interrupt has occured, for example).

## embassy-nrf

The `embassy-nrf` crate contains implementations for nRF 52 series SoCs.

- `uarte`: UARTE driver implementing `AsyncBufRead` and `AsyncWrite`.
- `qspi`: QSPI driver implementing `Flash`.
- `gpiote`: GPIOTE driver. Allows `await`ing GPIO pin changes. Great for reading buttons or receiving interrupts from external chips.
- `rtc`: RTC driver implementing `Clock` and `Alarm`, for use with `embassy::executor`.

## Running the examples

Examples are for the nRF52840 chip but should be easily adaptable to other nRF52 chips.

GPIO pins are set up for the `nrf52840-dk` board (PCA10056)

- Install `probe-run` with defmt support.

```
cargo install probe-run
```

- Run the example

```
cargo run --bin rtc_async
```

## Minimum supported Rust version (MSRV)

Only recent nighly supported. Nightly is required for:

- `generic_associated_types`: for trait funcs returning futures.
- `type_alias_impl_trait`: for trait funcs returning futures implemented with `async{}` blocks, and for `static-executor`.

Stable support is a non-goal until these features get stabilized.

## Why the name?

EMBedded ASYnc! :)

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
