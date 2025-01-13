# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.7.0 - 2025-01-02

- Performance optimizations.
- Remove feature `integrated-timers`. Starting with `embassy-time-driver` v0.2, `embassy-time` v0.4 the timer queue is now part of the time driver, so it's no longer the executor's responsibility. Therefore, `embassy-executor` no longer provides an `embassy-time-queue-driver` implementation.
- Added the possibility for timer driver implementations to store arbitrary data in task headers. This can be used to make a timer queue intrusive list, similar to the previous `integrated-timers` feature. Payload size is controlled by the `timer-item-payload-size-X` features.
- Added `TaskRef::executor` to obtain a reference to a task's executor

## 0.6.3 - 2024-11-12

- Building with the `nightly` feature now works with the Xtensa Rust compiler 1.82.
- Compare vtable address instead of contents. Saves 44 bytes of flash on cortex-m.

## 0.6.2 - 2024-11-06

- The `nightly` feature no longer requires `nightly-2024-09-06` or newer.

## 0.6.1 - 2024-10-21

- Soundness fix: Deny using `impl Trait` in task arguments. This was previously accidentally allowed when not using the `nightly` feature,
  and could cause out of bounds memory accesses if spawning the same task mulitple times with different underlying types
  for the `impl Trait`. Affected versions are 0.4.x, 0.5.0 and 0.6.0, which have been yanked.
- Add an architecture-agnostic executor that spins waiting for tasks to run, enabled with the `arch-spin` feature.
- Update for breaking change in the nightly waker_getters API. The `nightly` feature now requires `nightly-2024-09-06` or newer.
- Improve macro error messages.

## 0.6.0 - 2024-08-05

- Add collapse_debuginfo to fmt.rs macros.
- initial support for AVR
- use nightly waker_getters APIs

## 0.5.1 - 2024-10-21

- Soundness fix: Deny using `impl Trait` in task arguments. This was previously accidentally allowed when not using the `nightly` feature,
  and could cause out of bounds memory accesses if spawning the same task mulitple times with different underlying types
  for the `impl Trait`. Affected versions are 0.4.x, 0.5.0 and 0.6.0, which have been yanked.

## 0.5.0 - 2024-01-11

- Updated to `embassy-time-driver 0.1`, `embassy-time-queue-driver 0.1`, compatible with `embassy-time v0.3` and higher.

## 0.4.0 - 2023-12-05

- Removed `arch-xtensa`. Use the executor provided by the HAL crate (`esp-hal`, `esp32s3-hal`, etc...) instead.
- Added an arena allocator for tasks, allowing using the `main` and `task` macros on Rust 1.75 stable. (it is only used if the `nightly` feature is not enabled. When `nightly` is enabled, `type_alias_impl_trait` is used to statically allocate tasks, as before).

## 0.3.3 - 2023-11-15

- Add `main` macro reexport for Xtensa arch.
- Remove use of `atomic-polyfill`. The executor now has multiple implementations of its internal data structures for cases where the target supports atomics or doesn't.

## 0.3.2 - 2023-11-06

- Use `atomic-polyfill` for `riscv32`
- Removed unused dependencies (static_cell, futures-util)

## 0.3.1 - 2023-11-01

- Fix spurious "Found waker not created by the Embassy executor" error in recent nightlies.

## 0.3.0 - 2023-08-25

- Replaced Pender. Implementations now must define an extern function called `__pender`.
- Made `raw::AvailableTask` public
- Made `SpawnToken::new_failed` public
- You can now use arbitrary expressions to specify `#[task(pool_size = X)]`

## 0.2.1 - 2023-08-10

- Avoid calling `pend()` when waking expired timers
- Properly reset finished task state with `integrated-timers` enabled
- Introduce `InterruptExecutor::spawner()`
- Fix incorrect critical section in Xtensa executor

## 0.2.0 - 2023-04-27

- Replace unnecessary atomics in runqueue
- add Pender, rework Cargo features.
- add support for turbo-wakers.
- Allow TaskStorage to auto-implement `Sync`
- Use AtomicPtr for signal_ctx, removes 1 unsafe.
- Replace unsound critical sections with atomics

## 0.1.1 - 2022-11-23

- Fix features for documentation

## 0.1.0 - 2022-11-23

- First release
