# Changelog for embassy-usb-synopsys-otg

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

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
