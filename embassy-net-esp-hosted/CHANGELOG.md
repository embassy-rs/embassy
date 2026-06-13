# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Fixed compilation error with the `log` feature.
- `Interface::transfer` now exchanges the buffer in place.
- The SPI interface now waits for the handshake pin to be low before returning.
- `Control::connect` now waits for a `StationConnectedToAP` event before marking the link up; a successful connect ioctl alone no longer sets `LinkState::Up`. Disconnect event during a pending connect returns `Error::Failed` with the firmware disconnect reason.

## 0.3.0 - 2026-03-10

- Add an `Interface` trait to allow using other interface transports.
- Switch to `micropb` for protobuf.
- Update protos to latest `esp-hosted-fg`.
- Add support for OTA firmware updates.
- Update embassy-net-driver-channel to 0.4.0
- Update embassy-sync to 0.8.0

## 0.2.1 - 2025-08-26

- First release with changelog.
