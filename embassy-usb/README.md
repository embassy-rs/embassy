# embassy-usb

Async USB device stack for embedded devices in Rust.

## Features

- Native async.
- Fully lock-free: endpoints are separate objects that can be used independently without needing a central mutex. If the driver supports it, they can even be used from different priority levels.
- Suspend/resume, remote wakeup.
- USB composite devices.
- Ergonomic descriptor builder.
- Ready-to-use implementations for a few USB classes (note you can still implement any class yourself outside the crate).
    - Serial ports (CDC ACM)
    - Ethernet (CDC NCM)
    - Human Interface Devices (HID)
    - MIDI

## Adding support for new hardware

To add `embassy-usb` support for new hardware (i.e. a new MCU chip), you have to write a driver that implements
the [`embassy-usb-driver`](https://crates.io/crates/embassy-usb-driver) traits.

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
