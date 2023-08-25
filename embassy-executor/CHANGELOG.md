# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
