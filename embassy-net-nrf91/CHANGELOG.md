# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- changed: update to nrf-pac 0.4.0.
- changed: add `keep_open` config option to keep sockets open across PDN loss/reattach.
- changed: link state follows the PDN deactivated/activated socket status notifications rather than polling attach.
- fixed: reopen the raw socket when the modem closes it, instead of polling for a detach.
- changed: select the number of IP TX buffers with the `tx-buf-count-4` or `tx-buf-count-8` feature (exactly one is required).
- fixed: AT commands and socket requests use a dedicated TX buffer, so they are not blocked when IP packets hold all TX buffers.
- fixed: only one AT command runs at a time.

## 0.2.0 - 2026-03-10

- Signal link state based on link attach to prevent too early transmit confusing LTE modem.
- Upgrade embassy-sync to 0.8.0
- Update embassy-net-driver-channel to 0.4.0

## 0.1.0 - 2025-12-15

- Initial release
