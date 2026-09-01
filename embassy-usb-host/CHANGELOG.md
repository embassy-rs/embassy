# Changelog for embassy-usb-host

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add `midi` class support
- Add `ccid` class support
- Add HID `SET_REPORT` request
- Add `get_string_descriptor` so a STRING request can carry a LANGID
- Add `WritableDescriptor` trait, with implementations for the built-in descriptor types
- Add `StringDescriptor`, `StringDescriptorZero`, `StringDescriptorLossy`, and english language identifiers
- Fix enumeration reliability: retry descriptor reads on error, free the device address on enumeration failure, hold an address already assigned by `SET_ADDRESS`, guard against short/zero-length descriptors, read the HID report descriptor length from the correct offset, fix the length of `DeviceDescriptorPartial`
- Fix hub handling: don't panic on hubs with more ports than `MAX_PORTS`, take a hub port's speed after reset (not before), match multi-TT hubs (not just single-TT), acknowledge hub/port status changes that aren't otherwise handled
- Fix MIDI: skip reserved packets, recognize legacy Yamaha vendor-specific interfaces
- Use the USB spec minimum packet size before the max is known; use `bInterval` for HID interrupt endpoints
- Drop the device-side default features from the host crate

## 0.1.0 - 2026-05-04

- Initial release
