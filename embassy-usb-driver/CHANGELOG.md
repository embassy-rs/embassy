# Changelog for embassy-usb-driver

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add `EndpointOut::read_data()` and `EndpointIn::write_data()` provided methods.
- Upgrade heapless to 0.9, drop support for defmt-03 in favor of defmt (1.x).
- Upgrade embedded-io-async to 0.7.x.

## 0.2.0 - 2025-07-16

- Make USB endpoint allocator methods accept an optional `EndpointAddress`.

## 0.1.1 - 2025-07-15

- Add `embedded_io_async::Error` implementation for `EndpointError` ([#4176](https://github.com/embassy-rs/embassy/pull/4176))
