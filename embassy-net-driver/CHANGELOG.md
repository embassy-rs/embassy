# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 - 2023-10-18

- Added support for IEEE 802.15.4 mediums.
- Added `Driver::hardware_address()`, `HardwareAddress`.
- Removed `Medium` enum. The medium is deduced out of the hardware address.
- Removed `Driver::ethernet_address()`. Replacement is `hardware_address()`.

## 0.1.0 - 2023-06-29

- First release
