# Embassy MCXA Developer's Guide

This document is intended to assist developers of the `embassy-mcxa` crate.

As of 2026-01-29, there is currently no "how to write/maintain a HAL" guide for `embassy`, so we intend to write up and explain why the embassy-mcxa crate was implemented the way it was, and to serve as a reference for people incrementally building out more features in the future. We also hope to "upstream" these docs when possible, to assist with better consistency among embassy HALs in the future.

This document will be written incrementally. If you see something missing: please do one of the following:

* Open an issue in the embassy github
* Ask in the embassy matrix chat
* Open a PR to add the documentation you think is missing

## The `Cargo.toml` file

This section describes the notable components of the `Cargo.toml` package manifest.

### `package.metadata`

As an embassy crate, we have a couple of embassy-specific metadata sections.

* `package.metadata.embassy`
    * This section is used for determining how to build the crate for embassy's CI process.
* `package.metadata.embassy_docs`
    * This section is used for determining how to generate embassy's API docs.
    * See <https://docs.embassy.dev/embassy-mcxa/git/mcx-a256/index.html>.
    * These docs are rebuilt after each PR is merged, with a short debouncing period.

### Features

We have a couple of features/kinds of features exposed as part of the crate. For general features, see the `Cargo.toml` docs for what features are activated by default, and what these features do.

Notable features/groupings of features are discussed below.

#### `...-as-gpio` features

Some pins can operate EITHER for GPIO/peripheral use, OR for some kind of dedicated feature, such as SWD/JTAG debugging, external oscillator, etc. Since it is difficult to expose this conditionally in the `Peripherals` struct returned by `hal::init()`, we make this a compile-time feature decision. This is generally reasonable, because when pins are dedicated to a use (or not), this requires board-level electrical wiring, which is not typically reconfigured at runtime.

For pins covered by `...-as-gpio` features, they are typically in their dedicated feature mode at boot. When an `...-as-gpio` feature is active, the relevant pins will be moved back to the "disabled" state at boot, rather than remaining in their default dedicated feature state.

For example, the `swd-swo-as-gpio` feature is on by default. When this feature is NOT enabled, the pin is used as SWO by default. On the FRDM development board, this causes issues, as this pin is NOT wired up to SWO, and is instead wired up to the I2C/I3C circuit, preventing normal operation.

## The top level of the crate - `lib.rs`

The `lib.rs` is the top level API of the `embassy-mcxa` crate.

### `embassy_hal_internal::peripherals!`

The `embassy_hal_internal::peripherals!` macro is used to create the list of peripherals available to users of the HAL after calling `hal::init()`. Each item generates a `Peri<'static, T>`, which is a zero-sized type "token", which is used to prove exclusive access to a peripheral. These are often referred to as "singletons", as these tokens can only (safely) be created once. For more information on how these tokens are used, see the "Peripheral Drivers" section below.

In this list, we include:

* All hardware peripherals.
* Any "synthetic" peripherals that we also want to exist as a singleton, even if they are not a "real" hardware peripheral.

The generated `Peripherals` struct always creates all items, which means it's not generally possible for functions like `hal::init()` to say "depending on config, we MIGHT not give you back some pins/peripherals". For this reason, we make any of these conditionally-returned tokens a crate feature. See the `Cargo.toml` section above for more details.

### `embassy_hal_internal::interrupt_mod!`

The `embassy_hal_internal::interrupt_mod!` macro is used to generate a number of helper functions, types, and marker traits for each hardware interrupt signal on the chip.

All interrupts available for a chip should be listed in this macro.

### The `init` function

This function is also referred to as `hal::init()` in these docs.

This function is typically one of the first functions called by the user. It takes all configuration values relevant for the lifetime of the firmware, including:

* The priority level for any "automagically handled" peripheral interrupts, including:
    * GPIOs
    * RTC
    * OsTimer (used for `embassy-time-driver` impl)
    * DMA
* The Clock and Power configurations for Active (running and WFE sleep) and Deep Sleep modes

This function then performs important "boot up" work, including:

* Enabling system level clocks and power based on the user configuration
* Enabling and configuring "automagically handled" peripherals (those listed above)
* Enabling and configuring the priority of interrupts for "automagically handled" peripherals

Finally, when setup is complete, The `init` function returns the `Peripherals` struct, created by the `embassy_hal_internal::peripherals!` macro, containing one `Peri<'static, T>` token for each peripheral.

## Non-Peripheral Components

Some modules of the HAL do not map 1:1 with the memory mapped peripherals of the system. These components are discussed here.

### Clocking and Power subsystem

The `clocks` module is responsible for setting up the system clock and power configuration of the device. This functionality spans across a few peripherals (`SCG`, `SYSCON`, `VBAT`, `MRCC`, etc.).

See the doc comments of `src/clocks/mod.rs` for more details regarding the architectural choices of this module.

## Peripheral Drivers

The majority of `embassy-mcxa` handles high-level drivers for hardware peripherals of the MCXA. These sections discuss "best practices" or "notable oddities" for these hardware drivers

### General Guidelines

This section regards patterns that are used for all or most peripheral drivers.

#### Type Erasure and Constructors

In order to prevent "monomorphization bloat", as well as "cognitive overload" for HAL users, each peripheral driver should strive to MINIMIZE the number of lifetimes and generics present on the driver. For example, for an I2c peripheral with two GPIO pins, we DO NOT want:

```rust
struct<'p, 'c, 'd, P, SCL, SDA, MODE> I2c { /* ... */ }

type Example = I2c<
    'periph,                // lifetimes
    'scl,                   // lifetimes
    'sda,                   // lifetimes
    Peri<'periph, I2C0>,    // peripheral instance generic
    Peri<'scl, P0_2>,       // gpio pin instance generic
    Peri<'sda, P0_3>,       // gpio pin instance generic
    Async,                  // operational mode
>;
```

Instead, we want to:

* Use a single lifetime where possible, as our HAL driver will "require" its parts for the same amount of time
* Erase ALL peripheral instance generics, instead using runtime storage to store which instances are used for a given peripheral.
* Retain a single generic for "Mode", typically `Blocking` or `Async`, where the latter is often interrupt-enabled and has async methods, while the former doesn't.

This allows us to create a type that looks as follows:

```rust
struct<'a, MODE> I2c { /* ... */ }

type Example = I2C<'a, Async>;
```

In order to retain type safety functionality, we do still use the per-instance and per-peripheral generics, but ONLY at the constructor. This means that constructors will end up looking something like:

```rust
impl<'a> I2c<'a, Blocking> {
    pub fn new<T: Instance>(
        peri: Peri<'a, T>,
        scl: Peri<'a, impl SclPin<T>>,
        sda: Peri<'a, impl SdaPin<T>>,
        config: Config,
    ) -> Result<I2c<'a, Blocking>, Error> {
        // get information like references/pointers to the specific
        // instance of the peripherals, or per-instance specific setup
        //
        // Get pointers for this instance of I2C
        let info = T::info();
        // Perform GPIO-specific setup
        scl.setup_scl();
        sda.setup_sda();
        // If we needed to enable interrupts, this is likely bound to the generic
        // instance:
        //
        // T::Interrupt::unpend();

        // ...

        Ok(I2c {
            info, // hold on to for later!
            // ...
        })
    }
}
```

#### Error types

When creating `Error` types for each peripheral, consider the following high level guidance:

##### Splitting up the Error types

Instead of making one top-level `Error` for the entire peripheral, it it often useful to create multiple error enums. For example, instead of:

```rust
enum Error {
    Clocks(ClockError),
    BadConfig,
    Timeout,
    TransferTooLarge,
}

impl Example {
    // Can return `Err(Clocks)` or `Err(BadConfig)`
    pub fn new(config: Config) -> Result<Self, Error> { /* ... */ }

    // Can return `Err(BadConfig)` or `Err(TransferTooLarge)`
    pub fn send_u8s(&mut self, mode: Mode, data: &[u8]) -> Result<(), Error> { /* ... */ }

    // Can return `Err(BadConfig)` or `Err(TransferTooLarge)`
    pub fn send_u16s(&mut self, mode: Mode, data: &[u16]) -> Result<(), Error> { /* ... */ }

    // Can return `Err(Timeout)` or `Err(TransferTooLarge)`
    pub fn recv(&mut self, data: &mut [u8]) -> Result<usize, Error> { /* ... */ }
}
```

If the same `Error` type is used, the user may need to `match` on errors that are "impossible", e.g. a `new()` function returning `Error::Timeout`.

Instead, it might be worth splitting this into *three* errors:

```rust
enum CreateError {
    Clocks(ClockError),
    BadConfig,
}

enum SendError {
    BadConfig,
    TransferTooLarge,
}

enum RecvError {
    Timeout,
    TransferTooLarge,
}

impl Example {
    pub fn new(config: Config) -> Result<Self, CreateError> { /* ... */ }
    pub fn send_u8s(&mut self, mode: Mode, data: &[u8]) -> Result<(), SendError> { /* ... */ }
    pub fn send_u16s(&mut self, mode: Mode, data: &[u16]) -> Result<(), SendError> { /* ... */ }
    pub fn recv(&mut self, data: &mut [u8]) -> Result<usize, RecvError> { /* ... */ }
}
```

##### Don't make a `Result` alias

It *used* to be common to see module specific aliases for `Result`s, e.g.:

```rust
pub type Result<T> = Result<T, Error>;
```

However:

* This can lead to confusion for users if they have multiple `Result`s in scope
* It pushes for making "one `Error` per module", which is the opposite of what is described above

##### Mark errors as `#[non_exhaustive]`

Unless we are **definitely** sure that we have covered all possible kinds of errors for a HAL driver, we should mark the `Error` type(s) as `#[non_exhaustive]`, to prevent making a breaking change when adding a new error type.

For example:

```rust
#[non_exhaustive]
enum RecvError {
    Timeout,
    TransferTooLarge,
}
```

#### Avoid Wildcard/Glob imports

We generally want to avoid the use of wildcard/glob imports, like:

```rust
use super::*;
use other_module::*;
```

This can cause [surprising semver breakage], and make the code harder to read.

[surprising semver breakage]: https://predr.ag/blog/breaking-semver-in-rust-by-adding-private-type-or-import/
