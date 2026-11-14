# Changelog for embassy-usb-driver

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add `IsoEndpointIn`, `IsoInSlot`, `IsoWriteError`, and `iso::IsoSchedule` for scheduled isochronous IN transfers. `IsoEndpointIn::ready()` asynchronously reserves a borrowing `Slot` for the next service interval, and `IsoInSlot::write()` synchronously queues one packet for it. Uncollected packets are dropped rather than sent late, `IsoInSlot::skipped()` reports empty intervals left by a late writer, and `IsoWriteError::TooLate` reports a slot that started before `write()` was called. No fallback default implementations are provided.

- **Breaking:** Merge `ControlPipe` into `Bus`. `Driver::start()` returns just the `Bus`, which has `control_max_packet_size()`, `control_data_out()`, `control_data_in()`, `control_accept()`, `control_reject()` and `control_accept_set_address()`, and `Bus::poll()` returns the new `Event::Setup` for a SETUP packet instead of `ControlPipe::setup()`. The control methods must return immediately when the transfer can no longer complete (reset, suspend, power removed, new SETUP) without consuming the event, so `poll()` still returns it.
- Add `Speed::is_super()` and `Speed::uses_microframes()`.
- **Breaking:** `Driver::alloc_endpoint_in()` and `Driver::alloc_endpoint_out()` take an `EndpointConfig`, which has the endpoint's address, type, max packet size and interval, the number of packet buffers, and the SuperSpeed fields.
- **Breaking:** Add `EndpointInterval`, the polling period of an endpoint, and use it for `EndpointInfo::interval` and `EndpointConfig::interval` instead of a `u8` `bInterval` whose unit depended on the speed and transfer type. It is kept in microframes, so equal periods are equal values, and converts to and from the descriptor `bInterval` with `b_interval()` and `from_b_interval()`. `EndpointInfo::new()` and `EndpointConfig::new()` take an `EndpointInterval`; use `EndpointInterval::NONE` for bulk and control endpoints.
- **Breaking:** Add `EndpointInfo::companion`, the endpoint's `SuperSpeedEndpointCompanionInfo`, and `EndpointInfo::bytes_per_interval`.
- Fixed: `EndpointOut::read_transfer()` now returns when the buffer is full.
- Add `UsbHostAllocator::device_removed()` trait method with default.
- **Breaking:** Add required `Endpoint::set_stalled()` and `Endpoint::is_stalled()` methods, so a class can stall an endpoint without access to the `Bus`.
- **Breaking:** Make `Event` and `EndpointError` `#[non_exhaustive]`.
- **Breaking:** Remove the `embedded_io_async::Error` implementations for `EndpointError`, and the `embedded-io-async` dependency.

## 0.2.2 - 2026-05-28

- **Un-breaking change**: Re-add embedded-io-async 0.6.1 support.

## 0.2.1 - 2026-05-04

- Add `EndpointOut::read_transfer()` and `EndpointIn::write_transfer()` provided methods.
- USB host-side driver traits and types

## 0.2.0 - 2025-07-16

- Make USB endpoint allocator methods accept an optional `EndpointAddress`.

## 0.1.1 - 2025-07-15

- Add `embedded_io_async::Error` implementation for `EndpointError` ([#4176](https://github.com/embassy-rs/embassy/pull/4176))
