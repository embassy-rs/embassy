# Changelog for embassy-usb-driver

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- **Breaking:** Add `Speed::Super` and `Speed::SuperPlus`, with `Speed::is_super()` and `Speed::uses_microframes()`.
- **Breaking:** `Driver::alloc_endpoint_in()` and `Driver::alloc_endpoint_out()` take an `EndpointConfig`, which has the endpoint's address, type, max packet size and interval, the number of packet buffers, and the SuperSpeed fields.
- **Breaking:** Add `EndpointInfo::max_burst`, the SuperSpeed burst size the endpoint was allocated with.
- Fixed: `EndpointOut::read_transfer()` now returns when the buffer is full.
- Add `ControlPipe::data_out_transfer()` and `ControlPipe::data_in_transfer()` provided methods.
- Add `UsbHostAllocator::device_removed()` trait method with default.
- **Breaking:** Add required `Endpoint::set_stall()` and `Endpoint::is_stalled()` methods, so a class can stall an endpoint without access to the `Bus`.

## 0.2.2 - 2026-05-28

- **Un-breaking change**: Re-add embedded-io-async 0.6.1 support.

## 0.2.1 - 2026-05-04

- Add `EndpointOut::read_transfer()` and `EndpointIn::write_transfer()` provided methods.
- USB host-side driver traits and types

## 0.2.0 - 2025-07-16

- Make USB endpoint allocator methods accept an optional `EndpointAddress`.

## 0.1.1 - 2025-07-15

- Add `embedded_io_async::Error` implementation for `EndpointError` ([#4176](https://github.com/embassy-rs/embassy/pull/4176))
