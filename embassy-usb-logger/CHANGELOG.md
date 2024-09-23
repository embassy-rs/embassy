# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.2.0 - 2024-05-20

### Added

- [#2414](https://github.com/embassy-rs/embassy/pull/2414) USB logger can now use an existing USB device (@JomerDev)

### Changed

- Update `embassy-usb` to 0.2.0

### Fixed

- No more data loss at `Pipe` wraparound
- [#2414](https://github.com/embassy-rs/embassy/pull/2414) Messages that are exactly `MAX_PACKET_SIZE` long are no
longer delayed (@JomerDev)

## 0.1.0 - 2024-01-14

- Initial Release
