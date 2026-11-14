# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Host: support SuperSpeed devices.
- Device: standard Chapter 9 requests follow USB 2.0. Remote wakeup is refused unless the configuration supports it, `GET_STATUS` reports `Config::self_powered`, interface requests stall until the device is configured, device and configuration descriptors reject a non-zero index, `SET_ADDRESS(0)` returns an addressed device to the default state, and `SET_CONFIGURATION` / `SET_INTERFACE` clear endpoint halt
- Add CDC-ECM class (device and host) for Ethernet-over-USB networking
- `VCP`: add CH34x and FTDI host drivers
- Add USB Printer class (device and host) supporting IEEE 1284 device IDs and bidirectional raw print streams
- Vendor: add generic vendor-specific bulk class (`class-vendor`) with WinUSB and WebUSB support
- Add per-class Cargo features (`class-<name>`) and independent `device`/`host` role features. The default features are `device`, `host`, and `class-all`, preserving the previous full-class build
- Make `embassy-net-driver-channel`, `embassy-time`, and `embedded-io-async` optional; they are now pulled only by the features that use them
- **Breaking:** with `default-features = false`, `embassy_usb::device`, `embassy_usb::descriptor`, and `embassy_usb::host` are only available when the `device` or `host` feature is enabled, and each class requires its `class-<name>` feature
- `VCP`: `class-vcp` enables `class-cdc-acm`, because the CP210x driver reuses the CDC ACM line coding types
- Add `host::class::{ClassError, HostClass}` and `host::descriptor::{ClassCode, ClassMatch, InterfaceAssociationDescriptor}`; add `ConfigurationDescriptorChain::{find_interface, find_interfaces, find_iad_group}`
- Add the `embedded-io` feature, which implements `embedded_io_async::Error` for `ClassError`; `class-cdc-acm` and `class-vcp` enable it
- `CDC-ACM`: Host driver implements `HostClass` through the new `CdcAcmMatcher`; `CdcAcmInfo` is now `Copy` and `CdcAcmHost::new_with_info` opens from an already-probed info
- Add `InterfaceHandler` and `Builder::{interface_handler, try_interface_handler}`, with `HandlerFull` for fallible registration; interface- and endpoint-recipient control requests are routed only to the owning handler
- `CDC-ACM`, `HID`, `MSC`: Device classes now register through `interface_handler`
- `CCID`, `BTH`, `CDC-NCM`, `CMSIS-DAP v2`, `GUD`, `USBTMC`, `UVC`: Device classes now register through `interface_handler`
- `HID`, `MSC`, `CCID`, `GIP`: Host drivers implement `HostClass`; add `HidMatcher`, `MscMatcher`, `CcidMatcher`, and `GipMatcher`, and make their info types `Copy`
- Add `MidiMatcher`, `UacMatcher`, `Cp210xMatcher`, and `HubMatcher`
- Add the `test-utils` feature with `MockHost`, `MockState`, loopback pipes, `CDC_ACM_CONFIG`, and `enumeration_info`; add a CDC ACM host loopback test
- Add SuperSpeed (5 Gbit/s) and SuperSpeedPlus (10 Gbit/s) device support
- `HID`: Notify `RequestHandler::reset()` on USB bus reset, allowing handlers to restore default Report protocol mode
- `CDC-NCM`: The NetworkConnection notification names the communication interface in `wIndex`, not the data interface
- `CDC-NCM`: Serialize the NTB parameters and the NTB header field by field in little-endian, instead of copying `repr(packed)` structs; a control buffer shorter than the 28-byte NTB parameters is now refused instead of written past
- `CDC-NCM`: `Receiver::read_packet` drops a received datagram longer than `buf`, with a warning, instead of panicking
- Fix full-speed isochronous `bInterval` encoding to use exponential format ($2^{bInterval-1}$ frames) per USB 2.0 §9.6.6 instead of linear milliseconds.
- Add `EndpointInterval` enum (`Milliseconds`, `Microframes`, `Raw`) and corresponding builder methods on `InterfaceAltBuilder` (`endpoint_in_interval`, `endpoint_out_interval`, `endpoint_interrupt_in_interval`, `endpoint_interrupt_out_interval`, `endpoint_isochronous_in_interval`, `endpoint_isochronous_out_interval`, `endpoint_descriptor_with_interval`, `endpoint_descriptor_raw`). This allows specifying sub-millisecond polling periods for high-speed endpoints and preserving manually specified `bInterval` values without being overwritten.
- Add `Builder::other_speed_config_descriptor()`, allowing high-speed devices to generate and respond to `GET_DESCRIPTOR` for `OTHER_SPEED_CONFIGURATION` descriptors.
- `HID`: Add `GamepadReport`, `ConsumerReport`, and composite multi-report presets (`COMPOSITE_DESCRIPTOR`, `IdKeyboardReport`, `IdMouseReport`, `IdConsumerReport`, `IdGamepadReport`) at `class::hid`
- Add `class::ccid::device`, implementing USB Chip/Smart Card Interface Device (CCID) with 54-byte functional descriptor, APDU transfer chunking, and reader/writer split
- `CCID`: Extract shared CCID constants, commands, responses, and status types into `class::ccid::types`
- Add `class::bth::device`, implementing USB Bluetooth Host Controller Interface (HCI) device class conforming to Bluetooth Core Specification (Commands on EP0, Events on Interrupt IN, ACL data on Bulk IN/OUT)
- Add `class::uvc::device`, a USB Video Class 1.1 camera: MJPEG, YUY2 or NV12 over an isochronous IN endpoint, with probe/commit negotiation of format, frame size and frame interval
- `UAC2`: Add `class::uac::device::uac2::io_box`, a USB Audio Class 2.0 I/O box (line out + line in, explicit feedback, optional feature units with an interrupt endpoint), at full or high speed
- Endpoint `interval_ms` 0 on an interrupt or isochronous endpoint now means every frame: `bInterval` 1 at full speed (was 0, invalid) and every microframe at high speed (was 1 ms)
- Add `Builder::max_speed()`, so classes can size their endpoints for the speed the descriptors are built for
- Add USB host support in the `host` module, migrated from `embassy-usb-host` 0.1.0: enumeration, hubs, descriptor parsing, and HID, CDC ACM, CP210x, MSC, MIDI, UAC, CCID and GIP class drivers
- Add `block-device-driver` feature, implementing `BlockDevice` for the host MSC class
- **Breaking:** organize classes as `class::<name>::{device, host}`. Device implementations move to `class::<name>::device` (e.g. `class::cdc_acm::CdcAcmClass` is now `class::cdc_acm::device::CdcAcmClass`, `class::uac1` is now `class::uac::device`); items shared by both sides stay at `class::<name>` (e.g. `cdc_acm::LineCoding`, `hid::ReportId`, `midi::MidiPacket`, `uac::terminal_type::TerminalType`). Host drivers move from `embassy_usb_host::class::<name>` to `embassy_usb::class::<name>::host`
- Deprecate `embassy-usb-host`: it is superseded by this crate and is no longer maintained. Migrate to `embassy_usb::host` and `embassy_usb::class::<name>::host`
- Add `EndpointOptions`, with the max packet size of a bulk endpoint, the number of packet buffers and the SuperSpeed burst size of an endpoint
- **Breaking:** `InterfaceAltBuilder::endpoint_bulk_in()` and `endpoint_bulk_out()` take an `EndpointOptions` instead of the max packet size. The max packet size is the largest one the device speed allows (64 at full speed, 512 at high speed, 1024 at SuperSpeed), or `EndpointOptions::max_packet_size`
- **Breaking:** `InterfaceAltBuilder::{alloc_endpoint_in, alloc_endpoint_out, endpoint_in, endpoint_out, endpoint_in_interval, endpoint_out_interval}` and the isochronous endpoint methods take an `EndpointOptions`
- **Breaking:** `CDC-ACM`: `CdcAcmClass::new()` takes a `Config`, with the options of the data endpoints, instead of the max packet size
- **Breaking:** `CDC-NCM`, `CDC-ECM`: `CdcNcmClass::new()` and `CdcEcmClass::new()` take a `Config`, with the MAC address and the options of the data endpoints, instead of the MAC address and the max packet size
- **Breaking:** `CMSIS-DAP v2`: `CmsisDapV2Class::new()` takes a `Config`, with whether to add the trace endpoint and the options of the endpoints, instead of the max packet size and `trace`
- **Breaking:** `Vendor`, `MSC`, `Printer`, `USBTMC`, `BTH`, `GUD`, `CCID`: Remove the max packet size of the bulk endpoints from `Config` (`max_packet_size_acl` for `BTH`), and `vendor::Config::new()` and `msc::Config::new()` no longer take it. `Vendor`, `MSC`, `Printer`, `USBTMC`, `BTH` and `GUD` add the options of the bulk endpoints to `Config`
- **Breaking:** Remove the `usbd-hid` feature, and `write_serialize()` from `HidWriter` and `HidReaderWriter`
- `HID`: Add `KeyboardReport` and `MouseReport` at `class::hid`, with report descriptors (`DESCRIPTOR`) and `to_bytes()` for sending them from a device
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
- Add `Endpoint::set_stall()` and `Endpoint::is_stalled()`, so a class driver can stall or unstall its own endpoint. Until now the device stack could only be stalled by the host, via `SET_FEATURE(ENDPOINT_HALT)`
- `USBTMC`: stall Bulk-OUT when a message cannot be parsed, so the host aborts the transfer instead of the device misreading the following bytes; `UsbtmcClass::stall()`, `clear_stall()` and `is_stalled()` are exposed for applications, `check_clear()` recovers the endpoint on `viClear()`, and a message that ends early is now reported as `UsbtmcError::MessageTruncated` instead of a short payload
- Make `InterfaceAltBuilder::endpoint_in` and `InterfaceAltBuilder::endpoint_out` public
- Fix various typos in comments and internal variable names
- Added portable-atomic for targets not supporting core::sync::AtomicU32
- Add USB Mass Storage Class (MSC) implementation (Bulk-Only Transport + SCSI transparent commands)
- `MSC`: Add `SectorCache`, optimizing read-erase-write cycles in large block devices (flash), and `BlockDeviceAdapter` for `block_device_driver::BlockDevice` (feature `block-device-driver`)
- `GUD`: Add Generic USB Display support.
- USB host: debounce hub port connections and cleanup addresses after hub removal
- USB host: call `UsbHostAllocator::device_removed` for each address freed on device removal
- Generate `config.rs` in a deterministic order, so builds of the same source produce the same crate

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
