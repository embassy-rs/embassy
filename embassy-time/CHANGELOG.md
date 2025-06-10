# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0 - 2025-01-02

- `embassy-time-driver` updated from v0.1 to v0.2.
- embassy-time no longer provides an `embassy-time-queue-driver` implementation

## 0.3.2 - 2024-08-05

- Implement with_timeout()/with_deadline() method style call on Future
- Add collapse_debuginfo to fmt.rs macros.

## 0.3.1 - 2024-01-11

- Add with\_deadline convenience function and example
- Implement Clone for Delay
- Make Ticker::next Send+Sync
- Add timestamp features

## 0.3.0 - 2024-01-11

- Update `embedded-hal-async` to `1.0.0`
- Update `embedded-hal v1` to `1.0.0`
- Split the time driver to a separate `embassy-time-driver` crate.

## 0.2.0 - 2023-12-04

- Added tick rates in multiples of 10 kHz
- Remove nightly and unstable-traits features in preparation for 1.75.
- Update heapless to 0.8.

## 0.1.5 - 2023-10-16

- Added `links` key to Cargo.toml, to prevent multiple copies of this crate in the same binary.
  Needed because different copies might get different tick rates, causing
  wrong delays if the time driver is using one copy and user code is using another.
  This is especially common when mixing crates from crates.io and git.

## 0.1.4 - 2023-10-12

- Added more tick rates

## 0.1.3 - 2023-08-28

- Update `embedded-hal-async` to `1.0.0-rc.2`
- Update `embedded-hal v1` to `1.0.0-rc.2`

## 0.1.2 - 2023-07-05

- Update `embedded-hal-async` to `0.2.0-alpha.2`.
- Update `embedded-hal v1` to `1.0.0-alpha.11`. (Note: v0.2 support is kept unchanged).

## 0.1.1 - 2023-04-13

- Update `embedded-hal-async` to `0.2.0-alpha.1` (uses `async fn` in traits).
- Update `embedded-hal v1` to `1.0.0-alpha.10`. (Note: v0.2 support is kept unchanged).
- Remove dep on `embassy-sync`.
- Fix reentrancy issues in the `std` time driver (#1177)
- Add `Duration::from_hz()`.
- impl `From` conversions to/from `core::time::Duration`.
- Add `#[must_use]` to all futures.
- Add inherent `async fn tick()` to `Ticker`, so you can use it directly without the `Stream` trait.
- Add more tick rates.
- impl `Default` for `Signal`
- Remove unnecessary uses of `atomic-polyfill`

## 0.1.0 - 2022-08-26

- First release
