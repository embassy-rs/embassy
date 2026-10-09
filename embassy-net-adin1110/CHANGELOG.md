# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Added OPEN Alliance TC6 SPI protocol support (ADIN1110 and ADIN2111 in OPEN Alliance SPI mode)
- Added the `packetmeta-id` feature: `PacketMeta::id` carries the ingress and egress port in OPEN Alliance SPI mode
- Added `Runner::port_links` for the port count and per-port link state in OPEN Alliance SPI mode
- Fixed the `defmt` feature failing to build: `AdinError::Spi` is now formatted via `Debug2Format`, so the SPI error type only needs `Debug` rather than `defmt::Format`

## 0.4.0 - 2026-03-10

- Update embassy-net-driver-channel to 0.4.0

## 0.3.1 - 2025-08-26

- First release with changelog.
