# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Added experimental W6100 driver with disabled MAC filter (does not currently work with it enabled)
- Introduced `SOCKET_INTR_CLR` register which is needed on W6100 and later models (on W5100/W5500 this is shared with `SOCKET_INTR` and the address is the same)

## 0.2.1 - 2025-08-26

## 0.1.1 - 2025-08-14

- First release with changelog.
