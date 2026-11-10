# embassy-usb

Async USB device and host stack for embedded devices in Rust.

## Features

- Native async.
- Fully lock-free: endpoints are separate objects that can be used independently without needing a central mutex. If the driver supports it, they can even be used from different priority levels.
- Suspend/resume, remote wakeup.
- USB composite devices.
- Ergonomic descriptor builder.
- Ready-to-use implementations for common USB classes (note you can still implement any class yourself outside the crate). See [Class features](#class-features) for the full list and role support.
    - Serial ports (CDC ACM)
    - Ethernet (CDC NCM)
    - Human Interface Devices (HID)
    - MIDI
    - Generic USB Display (GUD)
    - Mass Storage (MSC)
    - Test & Measurement (USBTMC)

The [`host`](https://docs.embassy.dev/embassy-usb/git/default/host/index.html) module provides USB host enumeration,
descriptor parsing, and class drivers for devices including HID, CDC ACM, mass storage, MIDI, and USB Audio Class.

## Class features

Every class driver is behind its own Cargo feature, and each submodule is gated by the
role it belongs to:

- `device` — the device stack (`UsbDevice`, `Builder`, `Handler`, and class `device` modules).
- `host` — the host stack (`host` and class `host` modules).
- `class-all` — every class driver listed below. It is enabled by default.
- `class-<name>` — one class driver, for example `class-cdc-acm` or `class-hid`.
- `embedded-io` — the `embedded-io-async` trait implementations and the `ClassError`
  mapping. Enabled by `class-cdc-acm` and `class-vcp`.

| Feature | Device | Host |
|---------|--------|------|
| `class-bth` | yes | no |
| `class-ccid` | yes | yes |
| `class-cdc-acm` | yes | yes |
| `class-cdc-ncm` | yes | no |
| `class-cmsis-dap-v2` | yes | no |
| `class-dfu` | yes | no |
| `class-gip` | no | yes |
| `class-gud` | yes | no |
| `class-hid` | yes | yes |
| `class-hub` | no | yes |
| `class-midi` | yes | yes |
| `class-msc` | yes | yes |
| `class-uac` | yes | yes |
| `class-usbtmc` | yes | no |
| `class-uvc` | yes | no |
| `class-vcp` | no | yes |
| `class-web-usb` | yes | no |

`class-vcp` also enables `class-cdc-acm`, because the CP210x driver reuses the CDC ACM line
coding types.

The default features are `device`, `host`, and `class-all`, so existing users keep the full
class set. For a minimal build, disable default features and select only what you need:

```toml
[dependencies]
embassy-usb = { version = "0.6", default-features = false, features = ["device", "class-cdc-acm"] }
```

A class that can be built for both roles needs at least one of `device` or `host`; enabling
it without either is a compile error. Device-only and host-only class features enable their
role automatically.

## Adding support for new hardware

To add `embassy-usb` support for new hardware (i.e. a new MCU chip), you have to write a driver that implements
the [`embassy-usb-driver`](https://crates.io/crates/embassy-usb-driver) traits.

Before writing a new driver, you can first verify whether the chip uses a common USB IP. Several widely used USB IPs already have implementations available, such as:

- **Synopsys OTG (dwc2)**: Available at [embassy-usb-synopsys-otg](https://crates.io/crates/embassy-usb-synopsys-otg). This IP is used by vendors like STMicroelectronics, Espressif, and others.
- **Musbmhdrc (musb)**: Available at [musb](https://crates.io/crates/musb). This IP is used by vendors like TI, MediaTek, Puya, and others.

Driver crates should depend only on `embassy-usb-driver`, not on the main `embassy-usb` crate.
This allows existing drivers to continue working for newer `embassy-usb` major versions, without needing an update, if the driver
trait has not had breaking changes.

## Configuration

`embassy-usb` has some configuration settings that are set at compile time, affecting sizes
and counts of buffers.

They can be set in two ways:

- Via Cargo features: enable a feature like `<name>-<value>`. `name` must be in lowercase and
  use dashes instead of underscores. For example. `max-interface-count-3`. Only a selection of values
  is available, check `Cargo.toml` for the list.
- Via environment variables at build time: set the variable named `EMBASSY_USB_<value>`. For example
  `EMBASSY_USB_MAX_INTERFACE_COUNT=3 cargo build`. You can also set them in the `[env]` section of `.cargo/config.toml`.
  Any value can be set, unlike with Cargo features.

Environment variables take precedence over Cargo features. If two Cargo features are enabled for the same setting
with different values, compilation fails.

### `MAX_INTERFACE_COUNT`

Max amount of interfaces that can be created in one device. Default: 4.

## Interoperability

This crate can run on any executor.
