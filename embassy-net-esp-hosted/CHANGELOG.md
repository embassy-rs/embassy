# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- `Interface` and `SpiInterface` have been moved to `crate::iface` and `crate::iface::spi` respectively.
- Added `iface::hd_spi` module containing a half-duplex SPI `Interface` implementation.
- Added `iface::sdio` module containing an SDIO `Interface` implementation.
- Added `Interface::init` to initialize or re-initialize the transport.
- Changed: The `Interface::transfer` method now takes a packet length.
- Added `Control::scan` to retrieve visible networks. Note that `esp-hosted-fg` can only retrieve up to 8 entries.
- `embassy_net_esp_hosted::new` now returns an `HostedResources` struct with named fields.
- Added Bluetooth (BLE) support via the new `bluetooth` feature. `new` returns a `bluetooth::BtDriver` implementing `bt_hci::transport::Transport`, exposing the ESP coprocessor's HCI controller over the ESP-Hosted HCI interface.
- Added `FwVersion` struct and `Control::get_fw_version`.
- Fixed compilation error with the `log` feature.
- `Interface::transfer` now exchanges the buffer in place.
- The SPI interface now waits for the handshake pin to be low before returning.
- `Control::connect` now waits for a `StationConnectedToAP` event before marking the link up; a successful connect ioctl alone no longer sets `LinkState::Up`. Disconnect event during a pending connect returns `Error::Failed` with the firmware disconnect reason.
- Support for both `esp-hosted-fg` and `esp-hosted-mcu`. Use the `esp_hosted_fg` and `esp_hosted_mcu` feature flags to enable support of a specific version. Enable both to support both versions, in which case the version is determined by the firmware at runtime.

## 0.3.0 - 2026-03-10

- Add an `Interface` trait to allow using other interface transports.
- Switch to `micropb` for protobuf.
- Update protos to latest `esp-hosted-fg`.
- Add support for OTA firmware updates.
- Update embassy-net-driver-channel to 0.4.0
- Update embassy-sync to 0.8.0

## 0.2.1 - 2025-08-26

- First release with changelog.
