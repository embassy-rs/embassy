# Changelog for embassy-embedded-hal

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Shared I2c busses now impl `Clone`

## 0.5.0 - 2025-08-27

## 0.4.0 - 2025-08-03

- `SpiDevice` cancel safety: always set CS pin to high on drop
- Update `embassy-sync` to v0.7.0

## 0.3.2 - 2025-08-03

- Reverted changes in 0.3.1
- Reexport `SetConfig`, `GetConfig` traits from v0.4.0.

## 0.3.1 - 2025-07-16

YANKED due to embassy-sync upgrade being a breaking change.

- `SpiDevice` cancel safety: always set CS pin to high on drop
- Update `embassy-sync` to v0.7.0

## 0.3.0 - 2025-01-05

- The `std` feature has been removed
- Updated `embassy-time` to v0.4

## 0.2.0 - 2024-08-05

- Add Clone derive to flash Partition in embassy-embedded-hal
- Add support for all word sizes to async shared spi
- Add Copy and 'static constraint to Word type in SPI structs
- Improve flexibility by introducing SPI word size as a generic parameter
- Allow changing Spi/I2cDeviceWithConfig's config at runtime
- Impl `MultiwriteNorFlash` for `BlockingAsync`

## 0.1.0 - 2024-01-10

- First release
