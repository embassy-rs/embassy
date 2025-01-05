# Changelog for embassy-embedded-hal

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
