# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add support for USB HID Boot Protocol Mode 

## 0.5.1 - 2025-08-26

## 0.5.0 - 2025-07-16

- `UAC1`: unmute by default ([#3992](https://github.com/embassy-rs/embassy/pull/3992))
- `cdc_acm`: `State::new` is now `const` ([#4000](https://github.com/embassy-rs/embassy/pull/4000))
- Add support for CMSIS-DAP v2 USB class ([#4107](https://github.com/embassy-rs/embassy/pull/4107))
- Reduce `UsbDevice` builder logs to `trace` ([#4130](https://github.com/embassy-rs/embassy/pull/4130))
- Implement `embedded-io-async` traits for USB CDC ACM ([#4176](https://github.com/embassy-rs/embassy/pull/4176))
- Update `embassy-sync` to v0.7.0
- Fix CDC ACM BufferedReceiver buffer calculation

## 0.4.0 - 2025-01-15

- Change config defaults to to composite with IADs. This ensures embassy-usb Just Works in more cases when using classes with multiple interfaces, or multiple classes. (breaking change)
    - `composite_with_iads` = `true`
    - `device_class` = `0xEF`
    - `device_sub_class` = `0x02`
    - `device_protocol` = `0x01`
- Add support for USB Audio Class 1.
- Add support for isochronous endpoints.
- Add support for setting the USB version number.
- Add support for device qualifier descriptors.
- Allow `bos_descriptor_buf` to be a zero length if BOS descriptors aren't used.

## 0.3.0 - 2024-08-05

- bump usbd-hid from 0.7.0 to 0.8.1
- Add collapse_debuginfo to fmt.rs macros.
- update embassy-sync dependency

## 0.2.0 - 2024-05-20

- [#2862](https://github.com/embassy-rs/embassy/pull/2862) WebUSB implementation by @chmanie
- Removed dynamically sized `device_descriptor` fields

## 0.1.0 - 2024-01-11

- Initial Release
