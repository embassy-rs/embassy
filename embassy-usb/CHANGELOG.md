# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Add USB host support in the `host` module, migrated from `embassy-usb-host` 0.1.0: enumeration, hubs, descriptor parsing, and HID, CDC ACM, CP210x, MSC, MIDI, UAC, CCID and GIP class drivers
- Add `block-device-driver` feature, implementing `BlockDevice` for the host MSC class
- **Breaking:** organize classes as `class::<name>::{device, host}`. Device implementations move to `class::<name>::device` (e.g. `class::cdc_acm::CdcAcmClass` is now `class::cdc_acm::device::CdcAcmClass`, `class::uac1` is now `class::uac::device`); items shared by both sides stay at `class::<name>` (e.g. `cdc_acm::LineCoding`, `hid::ReportId`, `midi::MidiPacket`, `uac::terminal_type::TerminalType`). Host drivers move from `embassy_usb_host::class::<name>` to `embassy_usb::class::<name>::host`
- Deprecate `embassy-usb-host`: it is superseded by this crate and is no longer maintained. Migrate to `embassy_usb::host` and `embassy_usb::class::<name>::host`
- Add `InterfaceAltBuilder::endpoint_bulk_in_double_buffered()` and `endpoint_bulk_out_double_buffered()`
- `CDC-ACM`: Add `CdcAcmClass::new_double_buffered()`
- Bump usbd-hid from 0.9.0 to 0.10.0
- `UAC1`: Add audio source
- `UAC1`: BREAKING: `AudioSource::new(builder, state, config)` replaces the positional arguments and the manual handler registration; `AudioSource` now has `stream` and `control_monitor` (with `muted()`), `Volume` moved to `class::uac::device`. Internally `Speaker` and `AudioSource` share one implementation of the descriptors and control handler
- `UAC1`: BREAKING: `Speaker::new(builder, state, config)` takes `speaker::Config` and returns `Self` with the parts inside instead of a tuple
- `UAC1`: Source and speaker configurations default to no Feature Unit. Set `Config::feature_unit` to a vector of `FeatureUnitControls` flags, with one entry for the master and each audio channel, to enable mute or volume controls
- `CDC-NCM`: Handle `SetEthernetPacketFilter` and advertise it in `bmNetworkCapabilities`, which also works around a macOS bug that intermittently left the data interface disabled
- `MIDI`: Allow sender-only or receiver-only configuration
- `MIDI`: Change constructor to take a configuration struct instead of discrete arguments
- `MIDI`: Add extra fields to endpoint descriptors
- `MIDI`: Add optional names for the jacks
- `MIDI`: Add packet encoding and decoding
- Make `InterfaceAltBuilder::endpoint_in` and `InterfaceAltBuilder::endpoint_out` public
- Fix various typos in comments and internal variable names
- Add USB Mass Storage Class (MSC) implementation (Bulk-Only Transport + SCSI transparent commands)
- `MSC`: Add `SectorCache`, optimizing read-erase-write cycles in large block devices (flash), and `BlockDeviceAdapter` for `block_device_driver::BlockDevice` (feature `block-device-driver`)
- `GUD`: Add Generic USB Display support.
- USB host: debounce hub port connections and cleanup addresses after hub removal
- USB host: call `UsbHostAllocator::device_removed` for each address freed on device removal

## 0.6.0 - 2026-03-10

- Add support for USB HID Boot Protocol Mode
- Bump usbd-hid from 0.8.1 to 0.9.0
- Fix a bug where CDC ACM BufferedReceiver repeats data when its future is dropped
- Expose `dtr()` and `rts()` on `cdc_acm::ControlChanged`
- Add standalone DFU class implementation
- Add method to signal firmware error in DFU
- Allow `dfu_mode::Handler::start` to return a `Result` (fail gracefully)
- Fix bug in USB DFU transition
- Fix DFU GetStatus handler
- Upgrade embassy-sync to 0.8.0
- Upgrade embassy-net-driver-channel to 0.4.0

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
