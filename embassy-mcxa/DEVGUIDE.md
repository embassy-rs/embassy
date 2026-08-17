# Embassy MCXA Developer's Guide

This document is intended to assist developers of the `embassy-mcxa` crate.

As of 2026-01-29, there is currently no "how to write/maintain a HAL" guide for
`embassy`, so we intend to write up and explain why the embassy-mcxa crate was
implemented the way it was, and to serve as a reference for people incrementally
building out more features in the future. We also hope to "upstream" these docs
when possible, to assist with better consistency among embassy HALs in the
future.

This document will be written incrementally. If you see something missing:
please do one of the following:

* Open an issue in the embassy github
* Ask in the embassy matrix chat
* Open a PR to add the documentation you think is missing

## FRDM Usage Tips

### Recovering from a too-sleepy firmware

If you have an example that is configured to the DeepSleep state, it will sever
the debugger connection once it enters deep sleep. This can mean it will be hard
to re-flash since the debugging core is disabled.

To recover from this state, you can use the ISP mode, which triggers the ROM
bootloader:

1. Hold the "ISP" button down
2. Tap and release the "RESET" button
3. Release the "ISP" button
4. Try to flash with probe-rs, the first time will likely fail (I don't know why
   yet, it probably makes the bootloader upset)
5. Try to flash again, the second time will likely work.

You probably want to recover the device by flashing a simple example like the
`blinky` example which doesn't attempt to go to deep sleep.

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

The `lib.rs` is the top level API of the `embassy-mcxa` crate.

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

* The priority level for any "automagically handled" peripheral interrupts,
  including:
    * GPIOs
    * RTC
    * OsTimer (used for `embassy-time-driver` impl)
    * DMA
* The Clock and Power configurations for Active (running and WFE sleep) and Deep
  Sleep modes

This function then performs important "boot up" work, including:

* Enabling system level clocks and power based on the user configuration
* Enabling and configuring "automagically handled" peripherals (those listed
  above)
* Enabling and configuring the priority of interrupts for "automagically
  handled" peripherals

Finally, when setup is complete, The `init` function returns the `Peripherals`
struct, created by the `embassy_hal_internal::peripherals!` macro, containing
one `Peri<'static, T>` token for each peripheral.

## Non-Peripheral Components

Some modules of the HAL do not map 1:1 with the memory mapped peripherals of the
system. These components are discussed here.

### Clocking and Power subsystem

The `clocks` module is responsible for setting up the system clock and power
configuration of the device. This functionality spans across a few peripherals
(`SCG`, `SYSCON`, `VBAT`, `MRCC`, etc.).

See the doc comments of `src/clocks/mod.rs` for more details regarding the
architectural choices of this module.

## Peripheral Drivers

The majority of `embassy-mcxa` handles high-level drivers for hardware
peripherals of the MCXA. These sections discuss "best practices" or "notable
oddities" for these hardware drivers

### General Guidelines

This section regards patterns that are used for all or most peripheral drivers.

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

#### Driver Operating Modes (`Blocking`, `Async`, `Dma`)

As described above, a driver should carry a single `Mode` generic rather than
separate `Blocking`/`Async`/`Dma` types. We model the mode as a sealed marker
trait, with a second sealed sub-trait for the modes that are interrupt- or
DMA-driven:

```rust
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Modes that complete work asynchronously (interrupt- or DMA-driven).
#[allow(private_bounds)]
pub trait AsyncMode: sealed::Sealed + Mode {}

/// Blocking mode. No interrupt is bound; methods busy-wait.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Interrupt-only async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}
impl AsyncMode for Async {}

/// DMA async mode. Owns the DMA channels for the driver's lifetime.
pub struct Dma<'d> {
    tx_dma: DmaChannel<'d>,
    rx_dma: DmaChannel<'d>,
    /* ... */
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}
impl AsyncMode for Dma<'_> {}
```

Guidelines for the mode split (see `src/i2c` for a full example):

* **One constructor per mode, funnelling into a shared `new_inner`.**
  `new_blocking` takes no interrupt or DMA arguments; `new_async` additionally
  takes the interrupt `Binding`; `new_async_with_dma` additionally takes the DMA
  channels. Each builds the appropriate `Mode` value and calls a single private
  `new_inner` that does all the mode-independent setup (clocks, pin mux,
  register configuration). This keeps the bring-up logic in one place.
* **The `Async`/`Dma` mode value owns the resources that mode needs.** The
  interrupt is bound by the `_irq: impl Binding<...>` argument; the `Dma` struct
  owns its `DmaChannel`s so they live exactly as long as the driver.
* **Share the async methods, not just the constructors.** When the `Async` and
  `Dma` paths differ only in *how* a transfer is moved (FIFO-at-a-time vs. a DMA
  descriptor), put the public async methods on `impl<M: AsyncMode>` and dispatch
  the differing inner implementation through a small private trait
  (`AsyncEngine` in `i2c`) implemented once for `Async` and once for `Dma<'_>`.
  Users then get the same `async_read`/`async_write` surface regardless of mode.
* **Don't gate "is this mode async" on a runtime flag.** The `AsyncMode` bound
  is what makes the async methods unavailable on a `Blocking` driver at compile
  time. Resist adding an `enum Mode { Blocking, Async }` field; the type is the
  source of truth.

#### Bringing Up Clocks and Resets

Drivers must **not** hand-roll clock gating, clock-source selection, divider
setup, or reset sequencing by poking `MRCC`, `SPC`, `SCG`, etc. directly. That
logic lives in the `clocks` subsystem and is reached through the `Gate` trait
and the `enable_and_reset` helper. A driver's `SealedInstance` should extend
`Gate` and name its per-peripheral clock config type:

```rust
pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = Lpi2cConfig> {
    fn info() -> &'static Info;
    const CLOCK_INSTANCE: Lpi2cInstance;
    // ...
}

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: interrupt::typelevel::Interrupt;
}
```

Then, in the shared `new_inner`, bring the peripheral up in one call:

```rust
let conf = Lpi2cConfig { power, source, div, instance: T::CLOCK_INSTANCE };

// SAFETY: the `Peri<'d, T>` token proves we exclusively own this peripheral
// and it is not yet in use.
let parts = unsafe {
    enable_and_reset::<T>(&conf).map_err(SetupError::ClockSetup)?
};
```

`enable_and_reset`:

* enables the MRCC clock gate,
* runs the per-peripheral `pre_enable_config` (which selects the clock source,
  programs the divider, **validates the resulting frequency against the
  datasheet `fmax`** for the current power mode, and returns
  `ClockError::BadConfig` if it is out of range),
* pulses the reset line, and
* returns a `PreEnableParts` containing the input `freq` (capture this; you need
  it for baud/timing math) and an optional `WakeGuard`.

Two things to retain for the driver's lifetime:

* **The captured `freq`**, used by timing calculations (and recomputed if
  `set_config` changes speed).
* **The `WakeGuard`** (store it as `_wg: Option<WakeGuard>`). It keeps any
  required clock-source power vote alive; dropping it early can let the source
  power down underneath the peripheral.

The same applies to teardown: clock/reset/power teardown is the `clocks`
subsystem's job, not a driver writing to `MRCC`/`SPC` from its `Drop`. If a
driver genuinely needs a register that the clock subsystem does not yet own (a
PHY PLL, an SPC voltage-delay, etc.), add support to the
`clocks`/`periph_helpers` layer rather than writing to those registers from the
driver — otherwise that policy is now configured in two places that can
disagree.

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

#### Implementing Upstream Trait Contracts

Many drivers implement a trait defined elsewhere (`embedded-hal`,
`embedded-hal-async`, `embassy-usb-driver`, `embedded-io`, etc.). When you do,
treat the trait's documentation as a checklist of obligations and verify each
one — they are exactly the requirements that a single happy-path example will
not exercise. For example, an `embassy-usb-driver` implementation must abort an
in-flight control transfer when a new SETUP packet arrives, reset the endpoint
data toggle on clear-halt, and surface a single packet per `read`; none of those
are exercised by running one HID example on one board. Enumerate the contract
points in code comments or tests so the next reader can see they were
considered, rather than discovering a missing one weeks into review.

#### Avoid Wildcard/Glob imports

We generally want to avoid the use of wildcard/glob imports, like:

```rust
use super::*;
use other_module::*;
```

This can cause [surprising semver breakage], and make the code harder to read.

[surprising semver breakage]: https://predr.ag/blog/breaking-semver-in-rust-by-adding-private-type-or-import/

### Asynchronous (Interrupt-Driven) Drivers

Async drivers turn a hardware interrupt into a woken future. The `embassy-mcxa`
pattern (see `src/i2c/controller.rs`) is small and worth following exactly,
because the failure modes here — lost wakeups, futures that hang forever,
transfers that keep running after a future is dropped — are subtle and do not
show up in casual testing.

#### The interrupt handler masks and wakes; the future re-arms and re-checks

Per-instance async state lives in the `Info` struct, alongside the registers, as
a waker. We use `maitake_sync::WaitCell`:

```rust
pub(crate) struct Info {
    pub(crate) regs: pac::lpi2c::Lpi2c,
    pub(crate) wait_cell: WaitCell,
}
unsafe impl Sync for Info {}
```

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
            T::info().wait_cell().wake();
        }
    }
}
```

The future registers its waker **and then re-checks the hardware condition**,
re-enabling the interrupt source as part of the predicate. `WaitCell::wait_for`
does the register-then-recheck for us:

```rust
self.info
    .wait_cell()
    .wait_for(|| {
        self.enable_tx_ints();            // re-arm the source the ISR masked
        self.is_tx_fifo_empty_or_error()  // ...then test the real condition
    })
    .await?;
```

Why this shape:

* **Register before checking.** Registering the waker first, then testing the
  condition, closes the race where the interrupt fires between the check and the
  registration. Never test the condition, then register — a completion in that
  window is lost and the future sleeps forever. (`WaitCell::wait_for`, and
  equivalently `poll_fn(|cx| { waker.register(cx.waker()); /* check */ })`, both
  enforce this ordering; do not invert it.)
* **Mask in the ISR, re-arm in the future.** For level-triggered sources,
  leaving the enable bit set means the ISR re-fires immediately and forever.
  Masking in the handler and re-enabling in the predicate gives a clean
  hand-off. Do **not** instead paper over a still-asserted level source by
  calling `unpend()` at the end of the handler — that can drop an event that
  re-latched while the handler ran.

#### Wake *all* waiters on teardown / global events

When a single interrupt backs **several independent waiters** (e.g. one waiter
per endpoint or per channel) and a *global* event invalidates outstanding work —
a bus reset, a `disable()`, the peripheral being torn down — the handler must
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

For DMA, the drop handler must also disable the peripheral's DMA request and
quiesce the channel, so the controller stops writing into a buffer the driver is
about to reuse. (Mind the ordering of arming vs. the guard: in `i2c` the
`OnDrop` is installed *after* `async_start`, because `async_start` runs its own
NACK remediation and an earlier guard would double-remediate on an early `?`
return.)

#### Keep blocking waits out of async paths

Busy-waiting on a register bit (`while reg.read().busy() {}`) is acceptable in
`Blocking`-mode methods, and in brief, bounded, one-time bring-up handshakes. It
is **not** acceptable on an async path: on a single-threaded executor it stalls
*every* task (including any watchdog feeder) until the bit changes, and if the
bit never changes — cable pulled, clock stopped, peripheral wedged — the whole
system hangs. On async paths, `await` a waker instead; if you must wait on a bit
as part of setup, keep it bounded and be sure it is guaranteed to terminate.

### Shared Static State and DMA

#### Per-instance state belongs in a `&'static Info`

The normal home for a driver's long-lived state (register handle + waker) is a
per-instance `static INFO`, handed out by `T::info()`. The instance macro
creates exactly one per peripheral instance:

```rust
fn info() -> &'static crate::i2c::Info {
    static INFO: crate::i2c::Info = crate::i2c::Info {
        regs: crate::pac::LPI2C0,
        wait_cell: maitake_sync::WaitCell::new(),
    };
    &INFO
}
```

This keeps the mutable global surface small and tied to the instance the
`Peri<'d, T>` token represents.

#### Module-global mutable state must be reset at construction

Some controllers (notably DMA-driven ones with descriptor rings or bounce
buffers) need mutable state that is *module-global* rather than per-instance —
static descriptor arrays, flags, a table of wakers. Remember that the
`Peri<'d, T>` token only proves exclusive ownership of the **live peripheral**;
it does **not** clear these global side-tables. If the driver can be dropped and
re-created (its `Drop` frees the `Peri`, a later `new()` re-takes it), the
second construction will inherit stale descriptors, latched flags, and
registered wakers from the first.

Therefore: **reset all module-global driver state at the top of construction**
(zero the descriptors/flags/wakers), or gate construction on a one-shot claim.
Do not assume "fresh program start" — assume "this may be the second `new()`".

#### DMA memory and cache coherency

DMA structures (descriptors, bounce buffers) are read and written by hardware
concurrently with the CPU. These are two separate concerns:

* **Ordering** between CPU writes and the controller fetching them is handled
  with a barrier (`cortex_m::asm::dsb()`), as the DMA paths in `i2c` do around
  `enable_request()` and after completion.
* **Cache coherency** is *not* handled by a barrier. The current MCX-A targets
  treat the relevant SRAM as non-cacheable, so a `dsb()` is sufficient — but
  that is an assumption, not a guarantee. If a driver relies on it, make it
  explicit: place the DMA structures in a known non-cacheable region (a
  dedicated linker section) and/or assert the attribute, and keep cache
  clean/invalidate hooks next to the barriers so a future port to a data-cached
  SRAM region cannot silently corrupt transfers. A comment is not enforcement.

#### Teardown happens at the layer that owns the resource

If a driver enabled a clock/reset/PHY at construction, the matching teardown
belongs to the *same* handle and layer (and should go back through the `clocks`
subsystem, mirroring `enable_and_reset`). Be careful tearing down shared
resources in the `Drop` of a sub-handle when other handles derived from the same
driver may still be live: drop order between sibling handles is not guaranteed,
so gating a shared clock in one handle's `Drop` while another can still touch
the peripheral leads to faults. Tear down only what that handle exclusively
owns, and tie shared-resource teardown to the last/owning handle.
