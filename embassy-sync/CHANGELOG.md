# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add `write_all` as a method to `pipe::Writer`, trait import not strictly necessary anymore.
- `AtomicWaker` is now lockless: `register()` and `wake()` no longer enter a critical section,
  using a small atomic state machine (ported from `futures::task::AtomicWaker`).
- `AtomicWaker`: document that callers must re-register the waker on every `poll()`.  The new
  implementation may consume the stored waker as part of resolving a `register`/`wake` race
  (the consumed waker is always woken, never dropped), so a caller that relied on a one-shot
  registration surviving multiple polls without re-registering will need to re-register. This
  matches the `Future::poll` contract ("on multiple calls to poll, only the `Waker` from the
  most recent `Context` should be scheduled to receive a wakeup") and the pattern used by every
  in-tree waker site, so no in-tree caller is affected; out-of-tree callers that worked by
  accident on the previous lock-based implementation may need to adjust.
- Added `CriticalSectionWaker`, a non-generic convenience wrapper around
  `GenericAtomicWaker<CriticalSectionRawMutex>` for callers that want the previous
  critical-section-based semantics without spelling out the mutex parameter.
- Added `Pipe::try_write_all` method which repeatedly calls `Pipe::try_write` until all
  bytes were written.
- Implement `core::error::Error` for `channel::TryReceiveError` and `channel::TrySendError`.
- Made `Signal::poll_wait` public.

## 0.8.0 - 2026-03-10
- Fix wakers getting dropped by `Signal::reset`
- Remove `Sized` trait bound from `MutexGuard::map`
- Update to `embedded-io-async` 0.7.0
- Fix `Pipe::try_write` docs
- Implement `futures_sink::Sink` for `Channel` and `channel::Sender`

## 0.7.2 - 2025-08-26

- Add `get_mut` to `LazyLock`
- Add more `Debug` impls to `embassy-sync`, particularly on `OnceLock`

## 0.7.0 - 2025-05-28

- Add `remove_if` to `priority_channel::{Receiver, PriorityChannel}`.
- impl `Stream` for `channel::{Receiver, Channel}`.
- Fix channels to wake senders on `clear()`.
  For `Channel`, `PriorityChannel`, `PubSub`, `zerocopy_channel::Channel`.
- Allow `zerocopy_channel::Channel` to auto-implement `Sync`/`Send`.
- Add `must_use` to `MutexGuard`.
- Add a `RwLock`.
- Add `lock_mut` to `blocking_mutex::Mutex`.
- Don't select a critical-section implementation when `std` feature is enabled.
- Improve waker documentation.
- Improve `Signal` and `Watch` documentation.
- Update to defmt 1.0. This remains compatible with latest defmt 0.3.
- Add `peek` method on `channel` and `priority_channel`.
- Add dynamic sender and receiver that are Send + Sync for `channel`.

## 0.6.2 - 2025-01-15

- Add dynamic dispatch variant of `Pipe`.

## 0.6.1 - 2024-11-22

- Add `LazyLock` sync primitive.
- Add `Watch` sync primitive.
- Add `clear`, `len`, `is_empty` and `is_full` functions to `zerocopy_channel`.
- Add `capacity`, `free_capacity`, `clear`, `len`, `is_empty` and `is_full` functions to `channel::{Sender, Receiver}`.
- Add `capacity`, `free_capacity`, `clear`, `len`, `is_empty` and `is_full` functions to `priority_channel::{Sender, Receiver}`.
- Add `GenericAtomicWaker` utility.

## 0.6.0 - 2024-05-29

- Add `capacity`, `free_capacity`, `clear`, `len`, `is_empty` and `is_full` functions to `Channel`.
- Add `capacity`, `free_capacity`, `clear`, `len`, `is_empty` and `is_full` functions to `PriorityChannel`.
- Add `capacity`, `free_capacity`, `clear`, `len`, `is_empty` and `is_full` functions to `PubSubChannel`.
- Made `PubSubBehavior` sealed
  - If you called `.publish_immediate(...)` on the queue directly before, then now call `.immediate_publisher().publish_immediate(...)`
- Add `OnceLock` sync primitive.
- Add constructor for `DynamicChannel`
- Add ready_to_receive functions to `Channel` and `Receiver`.

## 0.5.0 - 2023-12-04

- Add a `PriorityChannel`.
- Remove `nightly` and `unstable-traits` features in preparation for 1.75.
- Upgrade `heapless` to 0.8.
- Upgrade `static-cell` to 2.0.

## 0.4.0 - 2023-10-31

- Re-add `impl_trait_projections`
- switch to `embedded-io 0.6`

## 0.3.0 - 2023-09-14

- switch to `embedded-io 0.5`
- add api for polling channels with context
- standardise fn names on channels
- add zero-copy channel

## 0.2.0 - 2023-04-13

- pubsub: Fix messages not getting popped when the last subscriber that needed them gets dropped.
- pubsub: Move instead of clone messages when the last subscriber pops them.
- pubsub: Pop messages which count is 0 after unsubscribe.
- Update `embedded-io` from `0.3` to `0.4` (uses `async fn` in traits).
- impl `Default` for `WakerRegistration`
- impl `Default` for `Signal`
- Remove unnecessary uses of `atomic-polyfill`
- Add `#[must_use]` to all futures.

## 0.1.0 - 2022-08-26

- First release
