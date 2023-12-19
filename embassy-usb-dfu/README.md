# embassy-usb-dfu

An implementation of the USB DFU 1.1 protocol using embassy-boot. It has 2 components depending on which feature is enabled by the user.

* DFU protocol mode, enabled by the `dfu` feature. This mode corresponds to the transfer phase DFU protocol described by the USB IF. It supports DFU_DNLOAD requests if marked by the user, and will automatically reset the chip once a DFU transaction has been completed. It also responds to DFU_GETSTATUS, DFU_GETSTATE, DFU_ABORT, and DFU_CLRSTATUS with no user intervention.
* DFU runtime mode, enabled by the `application feature`. This mode allows users to expose a DFU interface on their USB device, informing the host of the capability to DFU over USB, and allowing the host to reset the device into its bootloader to complete a DFU operation. Supports DFU_GETSTATUS and DFU_DETACH. When detach/reset is seen by the device as described by the standard, will write a new DFU magic number into the bootloader state in flash, and reset the system.

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
