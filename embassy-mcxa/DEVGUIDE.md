# Embassy MCXA Developer's Guide

This document is intended to assist developers of the `embassy-mcxa` crate.

As of 2026-01-29, there is currently no "how to write/maintain a HAL" guide for `embassy`, so we intend to write up and explain why the embassy-mcxa crate was implemented the way it was, and to serve as a reference for people incrementally building out more features in the future. We also hope to "upstream" these docs when possible, to assist with better consistency between embassy HALs in the future.

This document will be written incrementally. If you see something missing: please do one of the following:

* Open an issue or ask in the embassy matrix chat why things are the way they are, or how they should be.
* Open a PR to add the documentation you think is missing.

## The `Cargo.toml`

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

