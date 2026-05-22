# Changelog for embassy-usb-synopsys-otg

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

## 0.4.0 - 2026-05-15

- **Breaking:** type-erased `State`/`HostState`, non-generic `OtgInstance`/`OtgHostInstance`, endpoint allocation in `State`, const generics removed from device/host drivers. Static state now needs to be constructed as `StateStorage::new()`/`HostStateStorage::new()` and then `as_state()`/`as_host_state()` must be called to obtain the state reference.
- **Breaking:** `OtgInstance::endpoint_count`, `OtgHostInstance::channel_count` have been removed.
- **Breaking:** `endpoint_count`, and `channel_count` parameters have been removed from the interrupt handler functions.
- Allow using 16 host channels

## 0.3.3 - 2026-05-04

- New feature: "host" for embassy-usb-host support
- Implemented remote wakeup support

## 0.3.2 - 2026-03-10

- Disabling an OUT endpoint no longer flushes the TX FIFO of the corresponding IN endpoint
- Upgrade embassy-sync to 0.8.0

## 0.3.1 - 2025-08-26

- Improve receive performance, more efficient copy from FIFO

## 0.3.0 - 2025-07-22

- Bump `embassy-usb-driver` to v0.2.0

## 0.2.0 - 2024-12-06

- Fix corruption in CONTROL OUT transfers (and remove `quirk_setup_late_cnak`)
- Fix build with `defmt` enabled
- Add USBPHYC clock configuration for H7RS series
- Add support for ISO endpoints
- Add support for a full-speed ULPI mode
- Add OTG core DMA address registers
- Ensure endpoint allocation fails when `endpoint_count < MAX_EP_COUNT`.
- New configuration option: `xcvrdly` (transceiver delay).
- `EpState` now implements `Send` and `Sync`.
- The default value of `vbus_detection` is now `false`.

## 0.1.0 - 2024-04-30

Initial release.
