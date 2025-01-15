# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.4.0 - 2025-01-15

- Update `embassy-usb` to 0.4.0

(skipped v0.3.0 to align version numbers with `embassy-usb`)

## 0.2.0 - 2024-05-20

- Update `embassy-usb` to 0.2.0
- Add support for using an existing USB device ([#2414](https://github.com/embassy-rs/embassy/pull/2414), @JomerDev)
- Fix data loss at `Pipe` wraparound
- Messages that are exactly `MAX_PACKET_SIZE` long are no longer delayed ([#2414](https://github.com/embassy-rs/embassy/pull/2414), @JomerDev)

## 0.1.0 - 2024-01-14

- Initial Release
