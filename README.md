# Embassy

Embassy is a project to make async/await a first-class option for embedded development. For more information and instructions to get started, go to [https://embassy.dev](https://embassy.dev).

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

## HALs

Hardware Absraction Layers with asynchronous behaviors are provided for a variety of platforms.
For example, the `embassy-nrf` crate contains implementations for nRF 52 series SoCs.

- `uarte`: UARTE driver implementing `AsyncBufRead` and `AsyncWrite`.
- `qspi`: QSPI driver implementing `Flash`.
- `gpiote`: GPIOTE driver. Allows `await`ing GPIO pin changes. Great for reading buttons or receiving interrupts from external chips.
- `saadc`: SAADC driver. Provides a full implementation of the one-shot sampling for analog channels.

- `rtc`: RTC driver implementing `Clock` and `Alarm`, for use with `embassy::executor`.

## Examples

Examples are found in the `examples/` folder seperated by the chip manufacturer they are designed to run on. For example:

*   `examples/nrf` are designed to run on the `nrf52840-dk` board (PCA10056) but should be easily adaptable to other nRF52 chips and boards.
*   `examples/rp` are for the RP2040 chip.
*   `examples/stm32` are designed for the STM32F429ZI chip but should be easily adaptable to other STM32F4xx chips.
*   `examples/std` are designed to run locally on your pc.

### Running examples

- Setup git submodules (needed for STM32 examples)
```
git submodule init
git submodule update
```

- Install `probe-run` with defmt support.

```
cargo install probe-run
```

- Change directory to the sample's base directory. For example:

```
cd examples/nrf
```

- Run the example

For example:

```
cargo run --bin blinky
```

## Developing Embassy with Rust Analyzer based editors

The [Rust Analyzer](https://rust-analyzer.github.io/) is used by [Visual Studio Code](https://code.visualstudio.com/)
and others. Given the multiple targets that Embassy serves, there is no Cargo workspace file. Instead, the Rust Analyzer 
must be told of the target project to work with. In the case of Visual Studio Code, 
please refer to the `.vscode/settings.json` file's `rust-analyzer.linkedProjects`setting.

## Minimum supported Rust version (MSRV)

Required nightly version is specified in the `rust-toolchain.toml` file. Nightly is required for:

- `generic_associated_types`: for trait funcs returning futures.
- `type_alias_impl_trait`: for trait funcs returning futures implemented with `async{}` blocks, and for `static-executor`.

Stable support is a non-goal until these features get stabilized.

## Documentation

Embassy documentation is located in the `docs/` folder. The documentation is built in [embassy-book](https://github.com/embassy-rs/embassy-book) and published to [https://embassy.dev](https://embassy.dev) by CI.

## Why the name?

EMBedded ASYnc! :)

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

