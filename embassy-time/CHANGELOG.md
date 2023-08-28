# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.3 - 2023-08-28

- Update `embedded-hal-async` to `1.0.0-rc.1`
- Update `embedded-hal v1` to `1.0.0-rc.1`

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
