# embassy-usb-dfu

An implementation of the USB DFU 1.1 protocol using embassy-boot. It has 2 components depending on which feature is enabled by the user.

* DFU protocol mode, enabled by the `dfu` feature. This mode corresponds to the transfer phase DFU protocol described by the USB IF. It supports DFU_DNLOAD requests if marked by the user, and will automatically reset the chip once a DFU transaction has been completed. It also responds to DFU_GETSTATUS, DFU_GETSTATE, DFU_ABORT, and DFU_CLRSTATUS with no user intervention.
* DFU runtime mode, enabled by the `application feature`. This mode allows users to expose a DFU interface on their USB device, informing the host of the capability to DFU over USB, and allowing the host to reset the device into its bootloader to complete a DFU operation. Supports DFU_GETSTATUS and DFU_DETACH. When detach/reset is seen by the device as described by the standard, will write a new DFU magic number into the bootloader state in flash, and reset the system.
