# Embassy HAL developer's guide and API guidelines

## Naming

Sometimes the same thing has different names depending on vendor, chip family, etc. We follow this algorithm to choose names, in order of decreasing priority.

1. Be consistent within each HAL.
2. Be consistent with the vendor documentation
3. Be consistent with the industry-wide accepted names.

Examples:

- Nordic calls I2C "TWI", probably for trademark reasons. Even though the industry-wide name is "I2C" we call it "TWI" in `embassy-nrf`, because `2` takes priority over `3`.
- We use "new" terms for SPI/I2C roles like "controller/peripheral/target" or "legacy" terms like "master/slave" matching what the vendor uses in their documentation, because `2` takes priority over `3`.
- ST itself is inconsistent in DMA naming. STM32F4, F7, H7 uses "DMA stream" and "DMA channel" to refer to what other STM32 families call "DMA channel" and "DMA request number" respectively. `1` takes priority over `2`, so `embassy-stm32` uses "DMA channel" and "DMA request number" since it's both the most common in STM32 documentation and industry-wide. 

## The `Cargo.toml` file

This section describes the notable components of the `Cargo.toml` package
manifest.

### `package.metadata`

As an embassy crate, we have a couple of embassy-specific metadata sections.

* `package.metadata.embassy`
    * This section is used for determining how to build the crate for embassy's
      CI process.
* `package.metadata.embassy_docs`
    * This section is used for determining how to generate embassy's API docs.
    * See <https://docs.embassy.dev/embassy-mcxa/git/mcx-a256/index.html>.
    * These docs are rebuilt after each PR is merged, with a short debouncing
      period.

### Features

We have a couple of features/kinds of features exposed as part of the crate. For
general features, see the `Cargo.toml` docs for what features are activated by
default, and what these features do.

Notable features/groupings of features are discussed below.

#### `...-as-gpio` features

Some pins can operate EITHER for GPIO/peripheral use, OR for some kind of
dedicated feature, such as SWD/JTAG debugging, external oscillator, etc. Since
it is difficult to expose this conditionally in the `Peripherals` struct
returned by `hal::init()`, we make this a compile-time feature decision. This is
generally reasonable, because when pins are dedicated to a use (or not), this
requires board-level electrical wiring, which is not typically reconfigured at
runtime.

For pins covered by `...-as-gpio` features, they are typically in their
dedicated feature mode at boot. When an `...-as-gpio` feature is active, the
relevant pins will be moved back to the "disabled" state at boot, rather than
remaining in their default dedicated feature state.

For example, the `swd-swo-as-gpio` feature is on by default. When this feature
is NOT enabled, the pin is used as SWO by default. On the FRDM development
board, this causes issues, as this pin is NOT wired up to SWO, and is instead
wired up to the I2C/I3C circuit, preventing normal operation.

## The top level of the crate - `lib.rs`

The `lib.rs` is the top level API of the HAL crate.

### `embassy_hal_internal::peripherals!`

The `embassy_hal_internal::peripherals!` macro is used to create the list of
peripherals available to users of the HAL after calling `hal::init()`. Each item
generates a `Peri<'static, T>`, which is a zero-sized type "token", which is
used to prove exclusive access to a peripheral. These are often referred to as
"singletons", as these tokens can only (safely) be created once. For more
information on how these tokens are used, see the "Peripheral Drivers" section
below.

In this list, we include:

* All hardware peripherals.
* Any "synthetic" peripherals that we also want to exist as a singleton, even if
  they are not a "real" hardware peripheral.

The generated `Peripherals` struct always creates all items, which means it's
not generally possible for functions like `hal::init()` to say "depending on
config, we MIGHT not give you back some pins/peripherals". For this reason, we
make any of these conditionally-returned tokens a crate feature. See the
`Cargo.toml` section above for more details.

### `embassy_hal_internal::interrupt_mod!`

The `embassy_hal_internal::interrupt_mod!` macro is used to generate a number of
helper functions, types, and marker traits for each hardware interrupt signal on
the chip.

All interrupts available for a chip should be listed in this macro.

### The `init` function

This function is also referred to as `hal::init()` in these docs.

This function is typically one of the first functions called by the user. It
takes all configuration values relevant for the lifetime of the firmware,
including:

* The priority level for any "automagically handled" peripheral interrupts.
* Clock and power configuration.

This function then performs important "boot up" work, including:

* Enabling system level clocks and power based on the user configuration
* Enabling and configuring "automagically handled" peripherals (those listed
  above)
* Enabling and configuring the priority of interrupts for "automagically
  handled" peripherals

Finally, when setup is complete, The `init` function returns the `Peripherals`
struct, created by the `embassy_hal_internal::peripherals!` macro, containing
one `Peri<'static, T>` token for each peripheral.

## Peripheral Drivers

The majority of a HAL handles high-level drivers for hardware peripherals of the
chip. These sections discuss "best practices" or "notable oddities" for these
hardware drivers.

### General Guidelines

This section regards patterns that are used for all or most peripheral drivers.
The per-peripheral sections further below assume them and don't repeat them.

#### Sealed traits

Most traits a HAL exposes (`Instance`, `Pin`, `Mode`, pin traits, channel traits, …) are
**sealed**: user code may name them in bounds, but may not implement them. Sealing is what lets the
HAL add methods, add supertraits, or change the internals of a trait without it being a breaking
change, and it is what makes `unsafe` code inside the driver sound — the driver relies on
`T::info()` really pointing at that peripheral's registers, which only the HAL can guarantee.

The pattern is a **public trait with a private supertrait**:

```rust
/// Private half. Not `pub`, so downstream crates cannot name or implement it.
trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// UART instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}
```

Rules:

- The sealed supertrait is named `Sealed<Name>` — `SealedInstance` for `Instance`, `SealedPin` for
  `Pin`, `SealedMode` for `Mode`, `SealedAdcChannel` for `AdcChannel`.
- It is **private** (no visibility modifier, or `pub(crate)` when it has to be reachable from
  another module in the crate), and lives next to the public trait — *not* in a `mod sealed`.
- The public trait carries `#[allow(private_bounds)]`, which is what silences the "private type in
  public interface" lint for the supertrait bound.
- Do not use the other sealing patterns you may have seen elsewhere: no `mod sealed { pub trait Sealed {} }`, no `fn __sealed(&self, _: private::Token)` method, no `#[doc(hidden)]` public trait.

The split is not only about sealing: it is also **where the public/private API boundary of the
trait lives**. Items users are meant to see and call go on the public trait; the internals the
driver needs — register block accessors, `&'static` state, interrupt plumbing — go on the sealed
supertrait, where they are invisible in rustdoc and uncallable from user code:

```rust
pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::uart::Uart;
    fn state() -> &'static State;
}

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}
```

#### Type Erasure and Constructors

In order to prevent "monomorphization bloat", as well as "cognitive overload"
for HAL users, each peripheral driver should strive to MINIMIZE the number of
lifetimes and generics present on the driver. For example, for an I2c peripheral
with two GPIO pins, we DO NOT want:

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

* Use a single lifetime where possible, as our HAL driver will "require" its
  parts for the same amount of time
* Erase ALL peripheral instance generics, instead using runtime storage to store
  which instances are used for a given peripheral.
* Retain a single generic for "Mode", typically `Blocking` or `Async`, where the
  latter is often interrupt-enabled and has async methods, while the former
  doesn't.

This allows us to create a type that looks as follows:

```rust
struct<'a, MODE> I2c { /* ... */ }

type Example = I2C<'a, Async>;
```

In order to retain type safety functionality, we do still use the per-instance
and per-peripheral generics, but ONLY at the constructor. This means that
constructors will end up looking something like:

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

The driver lifetime is named **`'d`** for "Driver".

##### Peripheral ownership: `Peri<'d, T>`

Drivers never take peripheral singletons by value. They take `Peri<'d, T>`, which is a
lifetime-erased borrow of a peripheral singleton, defined in `embassy-hal-internal` and
re-exported from every HAL as `crate::Peri`:

```rust
pub struct Peri<'a, T: PeripheralType> { /* … */ }

impl<'a, T: PeripheralType> Peri<'a, T> {
    pub const unsafe fn new_unchecked(inner: T) -> Self;
    pub const unsafe fn clone_unchecked(&self) -> Peri<'a, T>;
    pub const fn reborrow(&mut self) -> Peri<'_, T>;
    pub fn into<U>(self) -> Peri<'a, U> where T: Into<U>, U: PeripheralType;
}

pub trait PeripheralType: Copy + Sized {}
```

Rules:

- The driver struct is generic over one lifetime, and that lifetime comes from the
  `Peri<'d, _>` arguments to the constructor.
- The concrete peripheral type `T` is a **constructor generic**, not a struct generic.
- Peripherals obtained from `init()` have `'static` lifetime, so `Peri<'static, T>` works
  everywhere `Peri<'d, T>` is expected.

##### `Instance` and pin traits

Every peripheral has a sealed `Instance` trait implemented for the peripheral singletons, plus
one marker trait per pin function:

```rust
/// UART instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// TX pin trait.
pub trait TxPin<T: Instance>: crate::gpio::Pin {}
/// RX pin trait.
pub trait RxPin<T: Instance>: crate::gpio::Pin {}
```

Rules:

- **Seal everything.** `Instance` has a private `SealedInstance` supertrait, so downstream crates
  can't implement it. Same for pin traits when they carry associated data (e.g. an AF number).
  See [Sealed traits](#sealed-traits).
- Pin traits are named after the *signal*, not the register field: `TxPin`, `RxPin`, `CtsPin`,
  `RtsPin`, `SckPin`, `MosiPin`, `MisoPin`, `CsPin`, `SclPin`, `SdaPin`.
- Pin traits are generic over the instance: `TxPin<T>`, so the compiler rejects wrong pin/instance
  combinations.
- Implementations are generated by macros from chip metadata, never hand-written per chip.
- On chips with a full crossbar (e.g. nRF), where any pin works with any peripheral, pin traits are
  unnecessary and constructors take `Peri<'d, impl GpioPin>` directly.

#### Driver Operating Modes (`Blocking`, `Async`)

As described above, a driver should carry a single `Mode` generic rather than
separate `Blocking`/`Async` types. We model the mode as a sealed marker trait:

```rust
trait SealedMode {}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

/// Blocking mode. No interrupt is bound; methods busy-wait.
pub struct Blocking;
impl SealedMode for Blocking {}
impl Mode for Blocking {}

/// Async mode. Completes work in an interrupt or with DMA.
pub struct Async;
impl SealedMode for Async {}
impl Mode for Async {}
```

Guidelines for the mode split:

* **One constructor per mode, funnelling into a shared `new_inner`.**
  `new_blocking` takes no interrupt argument; `new` additionally takes the
  interrupt `Binding` and DMA channels if needed. Each builds the appropriate `Mode` value and calls a
  single private `new_inner` that does all the mode-independent setup (clocks,
  pin mux, register configuration). This keeps the bring-up logic in one place.
* **The `Async` mode value owns the resources that mode needs.** The interrupt
  is bound by the `_irq: impl Binding<...>` argument.
* **Don't gate "is this mode async" on a runtime flag.** The `Async` type
  parameter is what makes the async methods unavailable on a `Blocking` driver
  at compile time. Resist adding an `enum Mode { Blocking, Async }` field; the
  type is the source of truth.

> See [Constructor naming](#constructor-naming) for the canonical `new` /
> `new_blocking` names and [Method naming](#method-naming) for `read` /
> `blocking_read`; it is the *structure* — one constructor per mode funnelling
> into a shared `new_inner` — that this section is prescribing.

Put the mode markers in **one shared module per HAL**, `crate::mode`, rather than repeating
them in each driver module:

```rust
/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    /// Blocking mode.
    pub struct Blocking;
    /// Async mode.
    pub struct Async;

    impl SealedMode for Blocking {}
    impl Mode for Blocking {}
    impl SealedMode for Async {}
    impl Mode for Async {}
}
```

The driver struct is then:

```rust
pub struct Uart<'d, M: Mode> {
    /* no `T: Instance` parameter — the instance is erased into an `Info`/`&'static Regs` field */
}
```

Rules:

- **Erase the instance.** `Instance` is a generic on the *constructor*, not on the struct. This
  keeps the user-facing type name short (`Uart<'d, Async>`, not `Uart<'d, USART1, Async>`) and
  lets users store drivers for different instances in the same variable/collection. The
  constructor stores what it needs (register block pointer, an `Info` struct, interrupt number) in
  the driver.
- Methods that work in both modes go in `impl<'d, M: Mode> Uart<'d, M>`.
- Async-only methods go in `impl<'d> Uart<'d, Async>`.

#### Constructor naming

```rust
impl<'d> Uart<'d, Async> {
    pub fn new<T: Instance, /* dma generics */>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        rx_dma: Peri<'d, impl RxDma<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError>;

    pub fn new_with_rtscts<T: Instance, /* … */>(/* … */) -> Result<Self, ConfigError>;
}

impl<'d> Uart<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError>;

    pub fn new_blocking_with_rtscts<T: Instance>(/* … */) -> Result<Self, ConfigError>;
}
```

Rules:

- The async constructor is **`new`**, not `new_async`. The blocking one is **`new_blocking`**.
- Optional-signal variants append `_with_<signals>`: `new_with_rtscts`,
  `new_blocking_with_rtscts`, `new_with_de`, `new_with_cts`.
- Reduced-function variants append the function: `new_txonly`, `new_rxonly`,
  `new_blocking_txonly`, `new_txonly_nosck`, `new_half_duplex`.
- Argument order is fixed: **peripheral, pins, DMA channels, interrupt binding, buffers, config**.
  Every `Peri<'d, _>` argument comes first, in that order; then the interrupt binding, which
  refers to all of them; then plain arguments. See [Argument order](#argument-order) below.
- **TX comes before RX**, in every group of arguments and in every driver. See
  [TX before RX](#tx-before-rx) below.
- Constructors that can fail on config validation return `Result<Self, ConfigError>`. Ones that
  can't return `Self`. Never panic on a user-supplied `Config` value that the hardware can't
  represent — return an error.

#### Argument order

```
peri, pins…, dma channels…, irq binding, buffers…, config
```

The rule of thumb is: **everything the driver takes ownership of (`Peri<'d, _>`) comes first**
— peripheral, then pins, then DMA channels — followed by the interrupt binding, which is a
proof-of-wiring token for the peripherals just listed, and then plain arguments (ring buffers,
`Config`). `Config` is always last.

Constructors that only use some of the groups just drop the missing ones; the relative order of
the rest does not change:

```rust
// peri, pins, dma, irq, config
pub fn new<T: Instance, TxDma, RxDma>(
    peri: Peri<'d, T>,
    sck: Peri<'d, impl SckPin<T>>,
    mosi: Peri<'d, impl MosiPin<T>>,
    miso: Peri<'d, impl MisoPin<T>>,
    tx_dma: Peri<'d, TxDma>,
    rx_dma: Peri<'d, RxDma>,
    _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    config: Config,
) -> Self;

// no DMA: peri, pins, irq, buffers, config
pub fn new<T: Instance>(
    peri: Peri<'d, T>,
    tx: Peri<'d, impl TxPin<T>>,
    rx: Peri<'d, impl RxPin<T>>,
    _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'd,
    tx_buffer: &'d mut [u8],
    rx_buffer: &'d mut [u8],
    config: Config,
) -> Result<Self, ConfigError>;

// no pins, no DMA: peri, irq, config
pub fn new<T: Instance>(
    peri: Peri<'d, T>,
    _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    config: Config,
) -> Self;
```

#### TX before RX

Wherever a driver takes a **pair of TX/RX things**, the TX one comes first. This
holds across argument groups, across drivers, and across return tuples:

| | TX first | then RX |
| --- | --- | --- |
| UART pins | `tx` | `rx` |
| DMA channels | `tx_dma` | `rx_dma` |
| Ring buffers | `tx_buffer` | `rx_buffer` |
| `split()` result | `UartTx` | `UartRx` |
| Half types / traits | `UartTx`, `TxPin`, `TxDma` | `UartRx`, `RxPin`, `RxDma` |

Pins that are not part of a TX/RX pair keep their conventional position: the
clock or strobe pin leads (`sck` before `mosi`/`miso`, `scl` before `sda`), and
flow-control or auxiliary pins follow the data pins (`cts`, `rts`, `de`, `cs`).

For SPI, `mosi` is the TX pin *of a controller*; the `sck, mosi, miso` order is
used for target/slave constructors too, so the pin list does not change meaning
between the two.

The one deliberate exception is **data buffer arguments of methods**, whose
order is fixed by the `embedded-hal` traits we implement:
`transfer(read, write)` and `write_read(address, write, read)` keep the
upstream order. Do not "fix" those to match this rule — see
[Implementing Upstream Trait Contracts](#implementing-upstream-trait-contracts).

#### Method naming

| Operation | Async | Blocking |
| --- | --- | --- |
| read | `read` | `blocking_read` |
| write | `write` | `blocking_write` |
| flush | `flush` | `blocking_flush` |
| transfer | `transfer` / `transfer_in_place` | `blocking_transfer` / `blocking_transfer_in_place` |
| I2C combined | `write_read` | `blocking_write_read` |
| I2C transaction | `transaction` | `blocking_transaction` |

Rules:

- **Async methods get the plain name; blocking methods get the `blocking_` prefix.** Never
  `read_async` or `async_read`.
- An async method and its blocking counterpart take the same arguments in the same order.
- Getters have no `get_` prefix: `is_high()`, `busy()`, `counter()`, `level()`, `output_level()`,
  `resolution()`. Where a getter and setter pair up, the setter is `set_x()` and the getter is
  plain `x()`.
- Setters use `set_` (`set_config`, `set_baudrate`, `set_level`).

#### Configuration: Defaults and Validation

* **`Default` should be the hardware-nominal / reset configuration, not a
  specific board's tuning.** Board-specific calibration (analog trims,
  crystal-dependent values, etc.) belongs in the example or the user's
  board-support code, not baked into the HAL's `Default`. If `Config::default()`
  encodes one dev board's values, every *other* board silently inherits them.
* **Validate configuration; don't silently mask it.** If a field has a limited
  valid range, check it and return a `BadConfig`-style error (as the clock
  `pre_enable_config` does against `fmax`). Silently truncating a value with
  `& 0xF` turns a user mistake into a hard-to-debug runtime fault.

##### `Config`, `ConfigError` and `SetConfig`

```rust
/// UART config.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    /* … */
}

impl Default for Config {
    fn default() -> Self;
}

/// Config error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    BaudrateTooLow,
    BaudrateTooHigh,
    /* … */
}

impl<'d, M: Mode> SetConfig for Uart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError>;
}
```

Rules:

- The struct is named `Config`, lives in the driver module, is `#[non_exhaustive]`, has all-public
  fields, and implements `Default`.
- Adding a field to a `Config` must not be a breaking change — that's what `#[non_exhaustive]` and
  `Default` are for.
- Drivers whose config can be changed at runtime implement `embassy_embedded_hal::SetConfig` and
  also expose an inherent `set_config(&mut self, config: &Config) -> Result<(), ConfigError>`.
- Frequently-changed single settings may get a dedicated setter in addition
  (`set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError>`).

#### Checking Errors

When checking errors, ensure that ALL errors are cleared before returning.
Otherwise early returns
can lead to "stuck" errors. Instead of this:

```rust
fn check_and_clear_rx_errors(info: &'static Info) -> Result<()> {
    let stat = info.regs().stat().read();
    if stat.or() {
        info.regs().stat().write(|w| w.set_or(true));
        Err(Error::Overrun)
    } else if stat.pf() {
        info.regs().stat().write(|w| w.set_pf(true));
        Err(Error::Parity)
    } else if stat.fe() {
        info.regs().stat().write(|w| w.set_fe(true));
        return Err(Error::Framing);
    } else if stat.nf() {
        info.regs().stat().write(|w| w.set_nf(true));
        return Err(Error::Noise);
    } else {
        Ok(())
    }
}
```

Ensure that all errors are cleared:

```rust
fn check_and_clear_rx_errors(info: &'static Info) -> Result<()> {
    let stat = info.regs().stat().read();

    // Check for overrun first - other error flags are prevented when OR is set
    let or_set = stat.or();
    let pf_set = stat.pf();
    let fe_set = stat.fe();
    let nf_set = stat.nf();

    // Clear all errors before returning
    info.regs().stat().write(|w| {
        w.set_or(or_set);
        w.set_pf(pf_set);
        w.set_fe(fe_set);
        w.set_nf(nf_set);
    });

    // Return error source
    if or_set {
        Err(Error::Overrun)
    } else if pf_set {
        Err(Error::Parity)
    } else if fe_set {
        Err(Error::Framing)
    } else if nf_set {
        Err(Error::Noise)
    } else {
        Ok(())
    }
}
```

#### Error types

When creating `Error` types for each peripheral, consider the following high
level guidance:

##### Splitting up the Error types

Instead of making one top-level `Error` for the entire peripheral, it it often
useful to create multiple error enums. For example, instead of:

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

If the same `Error` type is used, the user may need to `match` on errors that
are "impossible", e.g. a `new()` function returning `Error::Timeout`.

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

The established names for the two most common splits are `Error` for the runtime/I-O error and
`ConfigError` for the configuration-validation error (see
[`Config`, `ConfigError` and `SetConfig`](#config-configerror-and-setconfig)). Use those unless
the driver genuinely needs a finer split.

##### Don't make a `Result` alias

It *used* to be common to see module specific aliases for `Result`s, e.g.:

```rust
pub type Result<T> = Result<T, Error>;
```

However:

* This can lead to confusion for users if they have multiple `Result`s in scope
* It pushes for making "one `Error` per module", which is the opposite of what
  is described above

##### Mark errors as `#[non_exhaustive]`

Unless we are **definitely** sure that we have covered all possible kinds of
errors for a HAL driver, we should mark the `Error` type(s) as
`#[non_exhaustive]`, to prevent making a breaking change when adding a new error
type.

For example:

```rust
#[non_exhaustive]
enum RecvError {
    Timeout,
    TransferTooLarge,
}
```

##### Derives, `Display` and the `embedded-hal` error traits

```rust
/// UART error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    Framing,
    Noise,
    Overrun,
    Parity,
    /* … */
}

impl core::fmt::Display for Error { /* … */ }
impl core::error::Error for Error {}
```

- Always derive `Debug, Clone, Copy, PartialEq, Eq` and `defmt::Format` behind
  `#[cfg_attr(feature = "defmt", …)]`.
- Implement `core::fmt::Display` and `core::error::Error`.
- Implement the relevant `embedded_hal` error trait (`embedded_hal_1::i2c::Error`,
  `embedded_hal_1::spi::Error`, `embedded_io::Error`) so the generic traits can be implemented.

#### Interrupts

```rust
/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt();
}
```

Rules:

- The handler type is `InterruptHandler<T: Instance>` in the driver's module. Drivers with more
  than one interrupt line name them by function: `EventInterruptHandler<T>`,
  `ErrorInterruptHandler<T>`, `BufferedInterruptHandler<T>`.
- Async constructors take the binding by value and ignore it:
  `_irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd`.
  It exists purely so the type system proves the user wired up `bind_interrupts!`.
- Blocking constructors must **not** take an interrupt binding.
- DMA-driven drivers additionally require bindings for the DMA channel interrupts, joined with
  `+`.
- The binding argument goes after the peripheral/pin/DMA arguments and before the plain ones —
  see [Argument order](#argument-order).

#### Prefer PAC Accessors; Never Vendor a Forked PAC

* **Use generated field accessors, not hand-written bit constants.** `nxp-pac`
  generates typed `read()`/`write()`/`modify()` accessors and field setters
  (`w.set_men(true)`, `r.txcount()`). Prefer these over a wall of
  `const FOO: u32 = 1 << n;` plus raw `w.0 = bits`. Hand-rolled constants drift
  from the datasheet, can't be checked by the type system, and frequently
  duplicate something the PAC already exposes. A long block of bit constants
  behind `#![allow(dead_code)]` is a strong signal the PAC should be patched
  instead.
* **If the PAC is missing a register or field, fix the PAC.** Add the register
  block/field upstream in `nxp-pac` and depend on the released revision. A
  driver should describe *behavior*, not re-encode the memory map.
* **Never merge a `Cargo.toml` that points a dependency at a personal fork**
  (e.g. `git = "https://github.com/<user>/nxp-pac"`). Pin only upstream
  revisions. A fork pin is acceptable as a *local, temporary* aid while an
  upstream PAC PR is in review, but it must be resolved to the upstream revision
  before the embassy PR merges.

#### `Drop`

Every driver implements `Drop`, and `Drop` must return the hardware to a state where the
peripheral and its pins can be reused:

```rust
impl<'d, M: Mode> Drop for Uart<'d, M> {
    fn drop(&mut self);
}
```

At minimum: stop ongoing transfers, disable the peripheral, disable its clock, and deconfigure the
pins it took (back to disconnected/analog/reset AF).

#### `split()`

Drivers with independent halves expose:

```rust
impl<'d, M: Mode> Uart<'d, M> {
    /// Split into transmitter and receiver, consuming the driver.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>);
    /// Split by reference, borrowing the driver.
    pub fn split_ref(&mut self) -> (UartTx<'_, M>, UartRx<'_, M>);
}
```

`split_ref` returns **owned halves borrowed for the duration**, not `&mut` to stored fields. This
matters:

- The two forms then have the same shape, differing only in lifetime: `split` yields `'d`,
  `split_ref` yields `'_`. Code and examples move between them by changing one call.
- The halves are values, so they can be moved into `join`ed futures, stored in a struct, or passed
  by value to a function taking `UartTx<'_, M>`. A `&mut UartTx<'d, M>` gets you none of that.
- It doesn't force the driver to physically contain `UartTx`/`UartRx` fields. A driver that keeps
  its state flat can still implement `split_ref` by constructing the halves on the fly; with `&mut`
  returns, the public API dictates the internal layout.

If possible, the halves should be independently constructible too (`UartTx::new`, `UartRx::new_blocking`, …).

#### Implementing Upstream Trait Contracts

Many drivers implement a trait defined elsewhere (`embedded-hal`,
`embedded-hal-async`, `embassy-usb-driver`, `embedded-io`, etc.). When you do,
treat the trait's documentation as a checklist of obligations and verify each
one.

#### `embedded-hal` implementations

##### `embedded-hal` 1.0

Every driver implements the matching `embedded-hal` 1.0 traits, and the async ones when it has an
`Async` mode:

| Driver | Blocking traits | Async traits |
| --- | --- | --- |
| GPIO | `digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin}` | `digital::Wait` |
| UART | `embedded_io::{Read, Write, ReadReady, WriteReady}` | `embedded_io_async::{Read, Write, BufRead}` |
| SPI | `spi::{ErrorType, SpiBus}` | `spi::SpiBus` |
| I2C | `i2c::{ErrorType, I2c}` | `i2c::I2c` |
| PWM | `pwm::{ErrorType, SetDutyCycle}` | — |
| Flash | `embedded_storage::nor_flash::{ReadNorFlash, NorFlash}` | `embedded_storage_async::nor_flash::*` |
| RNG | `rand_core::RngCore` | — |

##### `embedded-hal` 0.2

`embedded-hal` 0.2 is superseded, but a large part of the driver ecosystem still targets it, so
**the blocking 0.2 traits are also required**:

| Driver | Required `embedded_hal_02` traits |
| --- | --- |
| GPIO | `digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin}` |
| UART | `blocking::serial::Write<u8>` |
| SPI | `blocking::spi::{Transfer<u8>, Write<u8>}` |
| I2C | `blocking::i2c::{Read, Write, WriteRead, Transactional}` |

Do **not** implement the `nb`-based 0.2 traits — `serial::Read`, `serial::Write`,
`spi::FullDuplex`, `adc::OneShot`. Now that async exists, `nb` is a dead end.

Make `embedded-hal-02` a plain (non-optional) dependency named
`embedded-hal-02 = { package = "embedded-hal", version = "0.2.6", features = ["unproven"] }`;
the `unproven` feature is what gates `digital::v2`.

#### Documentation and derives

- `#![warn(missing_docs)]` at the crate root; every public item has a doc comment.
- Public data types (`Config`, `Error`, enums, `Level`, `Pull`, …) derive
  `Debug, Clone, Copy, PartialEq, Eq` plus `#[cfg_attr(feature = "defmt", derive(defmt::Format))]`.
- Driver structs do **not** derive `Clone`/`Copy`.

#### Avoid Wildcard/Glob imports

We generally want to avoid the use of wildcard/glob imports, like:

```rust
use super::*;
use other_module::*;
```

This can cause [surprising semver breakage], and make the code harder to read.

[surprising semver breakage]: https://predr.ag/blog/breaking-semver-in-rust-by-adding-private-type-or-import/

### Asynchronous (Interrupt-Driven) Drivers

Async drivers turn a hardware interrupt into a woken future. The pattern is
small and worth following exactly, because the failure modes here — lost
wakeups, futures that hang forever, transfers that keep running after a future
is dropped — are subtle and do not show up in casual testing.

#### The interrupt handler masks and wakes; the future rechecks and rearms

Per-instance async state lives in the `State` struct as a waker, separate from
the read-only `Info` that holds the registers (see
[Per-instance state](#per-instance-state-belongs-in-a-static-info-and-a-static-state)).
We use `embassy_sync::waitqueue::AtomicWaker`:

```rust
pub(crate) struct Info {
    pub(crate) regs: pac::lpi2c::Lpi2c,
}

pub(crate) struct State {
    pub(crate) waker: AtomicWaker,
}
```

`AtomicWaker` holds a single waker and needs no lock, which is what a driver
that has one waiter per peripheral instance wants. When the waker has to live
inside state you already guard with a critical section or a `Mutex<RefCell<…>>`
(a queue's head/tail pointers, a shared buffer), use
`embassy_sync::waitqueue::WakerRegistration` instead — it is the same thing
without the atomics, since the surrounding lock already provides the
synchronization. Do not reach for third-party wait primitives; `embassy-sync` is
the dependency every HAL already has.

The interrupt handler does the minimum: it **disables (masks) the
interrupt-enable bits it is responsible for**, then wakes the waiter. It must
not try to make progress on the transfer itself:

```rust
impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt>
    for InterruptHandler<T>
{
    unsafe fn on_interrupt() {
        if T::info().regs().mier().read().0 != 0 {
            // Mask every source we enabled; the woken future will re-enable
            // exactly the ones it still cares about.
            T::info().regs().mier().write(|w| { /* clear all enable bits */ });
            T::state().waker.wake();
        }
    }
}
```

The future registers its waker **and then re-checks the hardware condition**. If the condition is not yet met, re-arm the interrupt then return `Pending`.

```rust
poll_fn(|cx| {
    self.state.waker.register(cx.waker()); // register first...
    if self.is_tx_fifo_empty_or_error() {  // ...then test the real condition
        Poll::Ready(())
    } else {
        self.enable_tx_ints();                 // re-arm the source the ISR masked
        Poll::Pending
    }
})
.await;
```

* **Register before checking.** Registering the waker first, then testing the
  condition, closes the race where the interrupt fires between the check and the
  registration. Never test the condition, then register — a completion in that
  window is lost and the future sleeps forever. Keep `register(cx.waker())` as
  the first statement in the `poll_fn` closure; do not invert it.
* **Mask in the ISR, re-arm in the future.** For level-triggered sources,
  leaving the enable bit set means the ISR re-fires immediately and forever.
  Masking in the handler and re-enabling in the predicate gives a clean
  hand-off. Do **not** instead paper over a still-asserted level source by
  calling `unpend()` at the end of the handler — that can drop an event that
  re-latched while the handler ran.

#### Wake *all* waiters on teardown / global events

When a single interrupt backs **several independent waiters** (e.g. one waiter
per endpoint or per channel — typically an `[AtomicWaker; N]`, one entry per
channel) and a *global* event invalidates outstanding work (like a bus reset, a `disable()`, the peripheral being torn down) the handler must
wake **every** outstanding waiter, not just the one whose individual completion
happened to arrive. The hardware will often *not* produce a per-operation
completion for transfers it abandoned (flushing a queue does not raise "transfer
done"), so a future waiting only for its own completion will hang. Wake all of
them so each future re-checks state and unwinds (typically returning a
"disabled"/"reset" error). A future must always be able to make progress from
*some* event, not from one specific event that may never come.

#### Cancel safety: a dropped future must not leave hardware running

Any future that arms hardware (starts a transfer, primes a DMA descriptor, kicks
a state machine) **can be dropped before it completes** — via `select`,
`with_timeout`, or simply the caller dropping it. If it leaves the hardware
mid-transaction, the next operation (or the controller's DMA) collides with the
abandoned one. Guard the armed region with `embassy_hal_internal::drop::OnDrop`,
and `defuse` it on the success path:

```rust
self.async_start(address, false).await?;

// If the future is dropped (or we early-return) before this is defused,
// `remediation()` runs and returns the peripheral to a clean state.
let on_drop = OnDrop::new(|| self.remediation());

// ... do the transfer ...

on_drop.defuse(); // reached only on the success path
```

#### Keep blocking waits out of async paths

Busy-waiting on a register bit (`while reg.read().busy() {}`) is acceptable in
`Blocking`-mode methods, and in brief, bounded, one-time bring-up handshakes. It
is **not** acceptable on an async path: on a single-threaded executor it stalls
*every* task (including any watchdog feeder) until the bit changes, and if the
bit never changes — cable pulled, clock stopped, peripheral wedged — the whole
system hangs. On async paths, `await` a waker instead; if you must wait on a bit
as part of setup, keep it bounded and be sure it is guaranteed to terminate.

### Shared Static State and DMA

#### Per-instance state belongs in a `&'static Info` and a `&'static State`

A driver's long-lived per-instance data is split into **two** structs, reached
through two accessors on `SealedInstance`:

* **`Info` — read-only.** Everything that describes the instance and never
  changes: the register block handle, clock/reset identifiers, the interrupt
  number, DMA request lines, capability constants.
* **`State` — read-write.** Everything the driver mutates at runtime: wakers,
  flags, counters, in-progress transfer bookkeeping.

The instance macro creates exactly one of each per peripheral instance:

```rust
impl SealedInstance for crate::peripherals::I2C0 {
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: crate::pac::LPI2C0,
            clock_instance: Lpi2cInstance::Lpi2c0,
        };
        &INFO
    }

    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }
}
```

The split is not cosmetic, it is about where the bytes land:

* `INFO` contains no interior mutability, so the linker can place it in
  **flash** (`.rodata`). It costs no RAM, no matter how big it grows — which is
  what lets `Info` carry generous per-instance tables.
* `STATE` must live in **RAM**. Merging the two would drag the whole `Info` into
  RAM as well: a single `AtomicWaker` field is enough to force it there.

**Make `State::new()` produce all-zero bytes.** An all-zero static goes in
`.bss`, which costs RAM only; a static with any non-zero byte goes in `.data`,
which costs the same RAM *plus* a copy of the initializer in flash *plus*
startup time to copy it across. With one `State` per instance this adds up.
The usual building blocks are already zero — `AtomicWaker::new()`,
`WakerRegistration::new()`, `AtomicBool::new(false)`, `AtomicU32::new(0)`,
`None`, `0`, null pointers — so prefer encodings whose idle/unused value is
zero rather than a non-zero sentinel:

```rust
pub(crate) struct State {
    waker: AtomicWaker,
    // Good: idle == 0, so `State::new()` is all-zero bytes and lands in .bss.
    bytes_transferred: AtomicUsize,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            bytes_transferred: AtomicUsize::new(0),
        }
    }
}
```

If you are unsure which section a static ended up in, check with
`arm-none-eabi-nm`: a `b`/`B` symbol type is `.bss`, `d`/`D` is `.data`.

#### Module-global mutable state must be reset at construction

Some controllers (notably DMA-driven ones with descriptor rings or bounce
buffers) need mutable state that is *module-global* rather than per-instance —
static descriptor arrays, flags, a table of wakers. If the driver can be dropped and
re-created (its `Drop` frees the `Peri`, a later `new()` re-takes it), the
second construction will inherit stale descriptors, latched flags, and
registered wakers from the first.

Therefore: **reset all module-global driver state at the top of construction**
(zero the descriptors/flags/wakers). Do not assume "fresh program start" — assume "this may be the second `new()`".

#### Teardown happens at the layer that owns the resource

If a driver enabled a clock/reset/PHY at construction, the matching teardown
belongs to the *same* handle and layer (and should go back through the `clocks`
subsystem, mirroring `enable_and_reset`). Be careful tearing down shared
resources in the `Drop` of a sub-handle when other handles derived from the same
driver may still be live: drop order between sibling handles is not guaranteed,
so gating a shared clock in one handle's `Drop` while another can still touch
the peripheral leads to faults. Tear down only what that handle exclusively
owns, and tie shared-resource teardown to the last/owning handle.

---

## Per-peripheral API reference

This section gives the expected API shape, driver by driver. Everything in
[General Guidelines](#general-guidelines) applies here and is not repeated.

### GPIO

Module: `crate::gpio`.

#### Types

```rust
/// Digital input or output level.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level { Low, High }

impl From<bool> for Level { fn from(val: bool) -> Self; }
impl From<Level> for bool { fn from(level: Level) -> bool; }

/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull { None, Up, Down }

/// Drive strength / slew / speed. Name and variants are chip-specific:
/// `Speed` (STM32), `OutputDrive` (nRF), `Drive` + `SlewRate` (RP, Microchip),
/// `DriveStrength` + `SlewRate` (MCXA).
```

#### `Input`

```rust
/// GPIO input driver.
pub struct Input<'d> { /* … */ }

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a pin with the provided pull setting.
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self;

    /// Get whether the pin input level is high.
    pub fn is_high(&self) -> bool;
    /// Get whether the pin input level is low.
    pub fn is_low(&self) -> bool;
    /// Get the current pin input level.
    pub fn level(&self) -> Level;

    /// Wait until the pin is high. Returns immediately if already high.
    pub async fn wait_for_high(&mut self);
    /// Wait until the pin is low. Returns immediately if already low.
    pub async fn wait_for_low(&mut self);
    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self);
    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self);
    /// Wait for the pin to undergo any transition.
    pub async fn wait_for_any_edge(&mut self);
}
```

#### `Output`

```rust
/// GPIO output driver.
pub struct Output<'d> { /* … */ }

impl<'d> Output<'d> {
    /// Create a GPIO output driver for a pin with the provided initial output level.
    /// The third parameter is the chip's drive-strength/speed type, if it has one.
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level /*, speed: Speed */) -> Self;

    /// Set the output as high.
    pub fn set_high(&mut self);
    /// Set the output as low.
    pub fn set_low(&mut self);
    /// Set the output level.
    pub fn set_level(&mut self, level: Level);
    /// Toggle the output level.
    pub fn toggle(&mut self);

    /// Is the output pin set as high?
    pub fn output_is_high(&self) -> bool;
    /// Is the output pin set as low?
    pub fn output_is_low(&self) -> bool;
    /// What level output is set to?
    pub fn output_level(&self) -> Level;
}
```

Note the deliberate naming split: `is_high`/`is_low`/`level` read the **pin**,
`output_is_high`/`output_is_low`/`output_level` read the **output register**.
All three output-reading methods share the `output_` prefix, so they sort
together and read consistently; do **not** use `is_set_high`/`is_set_low` (the
`embedded-hal` spelling) for the inherent methods. The `StatefulOutputPin` impl
still provides `is_set_high`/`is_set_low`, since those names are fixed by the
trait.

#### `OutputOpenDrain`

Same surface as `Output`, plus the input-reading and `wait_for_*` methods from `Input` (an
open-drain output can always sense the line):

```rust
/// GPIO output open-drain driver.
pub struct OutputOpenDrain<'d> { /* … */ }

impl<'d> OutputOpenDrain<'d> {
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level /*, speed, pull */) -> Self;

    // Output half
    pub fn set_high(&mut self);
    pub fn set_low(&mut self);
    pub fn set_level(&mut self, level: Level);
    pub fn toggle(&mut self);
    pub fn output_is_high(&self) -> bool;
    pub fn output_is_low(&self) -> bool;
    pub fn output_level(&self) -> Level;

    // Input half
    pub fn is_high(&self) -> bool;
    pub fn is_low(&self) -> bool;
    pub fn level(&self) -> Level;
    pub async fn wait_for_high(&mut self);
    pub async fn wait_for_low(&mut self);
    pub async fn wait_for_rising_edge(&mut self);
    pub async fn wait_for_falling_edge(&mut self);
    pub async fn wait_for_any_edge(&mut self);
}
```

#### `Flex`

`Flex` is the "everything" driver: a pin whose direction can be reconfigured at runtime. `Input`,
`Output` and `OutputOpenDrain` are thin wrappers around it.

```rust
/// GPIO flexible pin driver.
pub struct Flex<'d> { /* … */ }

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`. The pin remains disconnected until configured.
    pub fn new(pin: Peri<'d, impl Pin>) -> Self;

    /// Put the pin into input mode.
    pub fn set_as_input(&mut self /*, pull: Pull */);
    /// Put the pin into push-pull output mode.
    pub fn set_as_output(&mut self /*, speed: Speed */);
    /// Put the pin into disconnected mode.
    pub fn set_as_disconnected(&mut self);

    /// Is the pin configured as an input?
    pub fn is_input(&self) -> bool;
    /// Is the pin configured as an output?
    pub fn is_output(&self) -> bool;
    /// Is the pin disconnected?
    pub fn is_disconnected(&self) -> bool;

    // Same accessors as Input + Output + the wait_for_* family.
}

impl<'d> Drop for Flex<'d> {
    /// Returns the pin to its disconnected/reset state.
    fn drop(&mut self);
}
```

`Drop` is implemented on `Flex` only; `Input`/`Output`/`OutputOpenDrain` contain a `Flex` and
inherit its `Drop`.

#### Pin type erasure

```rust
/// Interface for a peripheral that is a GPIO pin.
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Number of the pin within the port.
    fn pin(&self) -> u8;
    /// Port of the pin.
    fn port(&self) -> Port;
}

/// Type-erased GPIO pin.
pub struct AnyPin { /* … */ }

impl AnyPin {
    /// Unsafely create an `AnyPin` from a pin number.
    ///
    /// # Safety
    /// You must ensure that you're only using one instance of this type at a time.
    pub unsafe fn steal(pin_port: u8) -> Peri<'static, Self>;
}

impl_peripheral!(AnyPin);
impl Pin for AnyPin {}
impl From<peripherals::PIN_X> for AnyPin { /* generated by macro */ }
```

The trait is named **`Pin`**, the erased type **`AnyPin`**, and erasure happens through
`Into<AnyPin>` (so `pin.into()` and `Peri::into()` both work). Every concrete pin peripheral gets
`impl From<peripherals::X> for AnyPin` from the impl macro.

---

### UART

Module: `crate::uart` (or `usart` on STM32, `lpuart` on MCXA). Three driver families:

- **`Uart` / `UartTx` / `UartRx`** — DMA (`Async`) or polling (`Blocking`).
- **`BufferedUart` / `BufferedUartTx` / `BufferedUartRx`** — interrupt-driven with user-supplied
  ring buffers, implements `embedded_io_async`.
- **`RingBufferedUartRx`** — circular-DMA receive, no data loss between reads.

#### `Uart`

```rust
/// Bidirectional UART driver.
pub struct Uart<'d, M: Mode> { /* … */ }
/// Transmitter half.
pub struct UartTx<'d, M: Mode> { /* … */ }
/// Receiver half.
pub struct UartRx<'d, M: Mode> { /* … */ }

impl<'d> Uart<'d, Async> {
    pub fn new<T: Instance, TxDma: /* … */, RxDma: /* … */>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError>;

    pub fn new_with_rtscts<T: Instance, /* … */>(/* … */) -> Result<Self, ConfigError>;
    pub fn new_with_de<T: Instance, /* … */>(/* … */) -> Result<Self, ConfigError>;
    pub fn new_half_duplex<T: Instance, /* … */>(/* … */) -> Result<Self, ConfigError>;

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error>;
    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error>;
    pub async fn flush(&mut self) -> Result<(), Error>;
}

impl<'d> Uart<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError>;

    pub fn new_blocking_with_rtscts<T: Instance>(/* … */) -> Result<Self, ConfigError>;
}

impl<'d, M: Mode> Uart<'d, M> {
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error>;
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error>;
    pub fn blocking_flush(&mut self) -> Result<(), Error>;

    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>);
    pub fn split_ref(&mut self) -> (UartTx<'_, M>, UartRx<'_, M>);

    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError>;
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError>;
    pub fn send_break(&mut self);
    pub fn busy(&self) -> bool;
}
```

Methods that change hardware state take `&mut self`, even when the implementation only needs
`&self` to poke a register. `embassy-stm32` and `embassy-mspm0` currently declare
`set_baudrate(&self, …)` and `send_break(&self)`; that should be `&mut self`.

`UartTx` and `UartRx` mirror this: `UartTx::new` / `new_blocking` / `new_with_cts` /
`new_blocking_with_cts`, `UartRx::new` / `new_blocking` / `new_with_rts` /
`new_blocking_with_rts`, with only the methods that apply to their direction.

#### `BufferedUart`

```rust
/// Interrupt-driven, buffered UART driver.
pub struct BufferedUart<'d> { /* … */ }
pub struct BufferedUartTx<'d> { /* … */ }
pub struct BufferedUartRx<'d> { /* … */ }

impl<'d> BufferedUart<'d> {
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, BufferedInterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError>;

    pub fn new_with_rtscts<T: Instance>(/* … */) -> Result<Self, ConfigError>;

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<usize, Error>;
    pub fn blocking_flush(&mut self) -> Result<(), Error>;

    pub fn split(self) -> (BufferedUartTx<'d>, BufferedUartRx<'d>);
    pub fn split_ref(&mut self) -> (BufferedUartTx<'_>, BufferedUartRx<'_>);
}
```

`BufferedUart` has **no `Mode` generic** — it is always interrupt-driven. Async I/O is exposed via
`embedded_io_async::{Read, Write, BufRead}`, not inherent methods.

Note the buffer argument order: `tx_buffer` before `rx_buffer`, both `&'d mut [u8]` — the same
[TX before RX](#tx-before-rx) rule as the pins and DMA channels.

#### `RingBufferedUartRx`

```rust
impl<'d> UartRx<'d, Async> {
    /// Turn the DMA receiver into a circular-DMA receiver backed by `dma_buf`.
    pub fn into_ring_buffered(self, dma_buf: &'d mut [u8]) -> RingBufferedUartRx<'d>;
}

pub struct RingBufferedUartRx<'d> { /* … */ }

impl<'d> RingBufferedUartRx<'d> {
    pub fn start_uart(&mut self);
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError>;
}
```

---

### SPI

Module: `crate::spi` (`spim`/`spis` on nRF).

```rust
/// SPI error.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error { Framing, Crc, ModeFault, Overrun, /* … */ }

/// SPI config.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub mode: Mode,          // embedded_hal::spi::Mode (CPOL/CPHA)
    pub bit_order: BitOrder,
    pub frequency: Hertz,
    /* … */
}

impl Default for Config { fn default() -> Self; }

/// SPI driver.
pub struct Spi<'d, M: Mode> { /* … */ }

impl<'d> Spi<'d, Async> {
    pub fn new<T: Instance, TxDma, RxDma>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        _irq: impl interrupt::typelevel::Binding</* dma irqs */> + 'd,
        config: Config,
    ) -> Self;

    pub fn new_txonly<T: Instance, TxDma>(/* … */) -> Self;
    pub fn new_rxonly<T: Instance, RxDma>(/* … */) -> Self;
    pub fn new_txonly_nosck<T: Instance, TxDma>(/* … */) -> Self;

    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), Error>;
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error>;
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error>;
}

impl<'d> Spi<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        config: Config,
    ) -> Self;

    pub fn new_blocking_txonly<T: Instance>(/* … */) -> Self;
    pub fn new_blocking_rxonly<T: Instance>(/* … */) -> Self;
    pub fn new_blocking_txonly_nosck<T: Instance>(/* … */) -> Self;
}

impl<'d, M: Mode> Spi<'d, M> {
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error>;
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error>;
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error>;
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error>;
    pub fn blocking_flush(&mut self) -> Result<(), Error>;

    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError>;
    pub fn set_frequency(&mut self, freq: Hertz);
}

impl<'d, M: Mode> Drop for Spi<'d, M> { fn drop(&mut self); }
```

Rules:

- **The driver is a bus, not a device.** No CS pin, no CS handling. Chip-select is the user's
  `Output` pin, combined via
  `embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice` or
  `embedded_hal_bus::spi::ExclusiveDevice`.
- Pin order is `sck, mosi, miso`: clock first, then the data pins TX-before-RX
  (`mosi` is the controller's TX). DMA channels follow as `tx_dma, rx_dma`. See
  [TX before RX](#tx-before-rx).
- `transfer(read, write)` takes the **read buffer first**, matching `embedded_hal::spi::SpiBus`.
  This is the deliberate exception to TX-before-RX: buffer arguments of methods follow the
  `embedded-hal` traits.
- Word-size-generic HALs parameterise the *methods*, not the struct:
  `pub async fn write<W: Word>(&mut self, data: &[W]) -> Result<(), Error>`.
- SPI peripheral/slave mode is a separate driver, NOT a mode. This is because the API needs to be different (for
  example, it must return how many bytes were transferred by the controller/master, which is not a thing in controller/master
  mode because we're the ones choosing the byte count). This document doesn't yet specify the API for peripheral/slave mode,
  but you can look at nRF's `spis` module for inspiration.

---

### I2C

Module: `crate::i2c`. Controller (master) and target (slave) are separate driver types.

#### Controller

```rust
/// I2C error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error { Bus, Arbitration, Nack, Timeout, Crc, Overrun, ZeroLengthTransfer, /* … */ }

/// I2C config.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub frequency: Hertz,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
    /* … */
}

impl Default for Config { fn default() -> Self; }

/// I2C driver.
pub struct I2c<'d, M: Mode> { /* … */ }

impl<'d> I2c<'d, Async> {
    pub fn new<T: Instance, TxDma, RxDma>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self;

    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error>;
    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Error>;
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error>;
    pub async fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error>;
}

impl<'d> I2c<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Self;
}

impl<'d, M: Mode> I2c<'d, M> {
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error>;
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error>;
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error>;
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error>;

    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError>;
}

impl<'d, M: Mode> Drop for I2c<'d, M> { fn drop(&mut self); }
```

Rules:

- Pin order is `scl, sda`: clock first, then data. Neither is a TX/RX pin, but the DMA channels
  are, and they follow as `tx_dma, rx_dma` — see [TX before RX](#tx-before-rx).
- Address argument is `u8` for 7-bit-only hardware. HALs that support 10-bit addressing take
  `impl Into<Address>` with a dedicated `Address` type — not a bare `u16`, which silently
  conflates the two.
- `Operation` is `embedded_hal_1::i2c::Operation`, re-exported; don't define a HAL-local copy.
- The driver is a **bus**; sharing between multiple devices is
  `embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice`'s job.
