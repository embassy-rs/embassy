# embassy-usb-dfu

An implementation of the USB DFU 1.1 protocol using embassy-boot. It has 2 components depending on which feature is enabled by the user.

* DFU protocol mode, enabled by the `dfu` feature. This mode corresponds to the transfer phase DFU protocol described by the USB IF. It supports DFU_DNLOAD requests if marked by the user, and will automatically reset the chip once a DFU transaction has been completed. It also responds to DFU_GETSTATUS, DFU_GETSTATE, DFU_ABORT, and DFU_CLRSTATUS with no user intervention.
* DFU runtime mode, enabled by the `application feature`. This mode allows users to expose a DFU interface on their USB device, informing the host of the capability to DFU over USB, and allowing the host to reset the device into its bootloader to complete a DFU operation. Supports DFU_GETSTATUS and DFU_DETACH. When detach/reset is seen by the device as described by the standard, will write a new DFU magic number into the bootloader state in flash, and reset the system.

## Verification

Embassy-boot provides functionality to verify that an update binary has been correctly signed using ed25519 as described in https://embassy.dev/book/#_verification. Even though the linked procedure describes the signature being concatenated to the end of the update binary, embassy-boot does not force this and is flexible in terms of how the signature for a binary is distributed. The current implementation in embassy-usb-dfu does however assume that the signature is 64 bytes long and concatenated to the end of the update binary since this is the simplest way to make it work with the usb-dfu mechanism. I.e. embassy-usb-dfu does not currently offer the same flexibility as embassy-boot.

To enable verification, you need to enable either the `ed25519-dalek` or the `ed25519-salty` feature with `ed25519-salty` being recommended.
