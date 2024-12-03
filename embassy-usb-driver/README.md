# embassy-usb-driver

This crate contains the driver traits for [`embassy-usb`]. HAL/BSP crates can implement these
traits to add support for using `embassy-usb` for a given chip/platform.

The traits are kept in a separate crate so that breaking changes in the higher-level [`embassy-usb`]
APIs don't cause a semver-major bump of this crate. This allows existing HALs/BSPs to be used
with the newer `embassy-usb` without needing updates.

If you're writing an application using USB, you should depend on the main [`embassy-usb`] crate
instead of this one.

[`embassy-usb`]: https://crates.io/crates/embassy-usb

## Interoperability

This crate can run on any executor.
