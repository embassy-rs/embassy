# Bootloader for STM32

This bootloader implementation uses `embassy-boot` and `embassy-usb-dfu` to manage firmware updates and interact with the flash memory on STM32WB55 devices.

## Prerequisites

- Rust toolchain with `cargo` installed
- `cargo-flash` for flashing the bootloader
- `dfu-util` for firmware updates
- `cargo-binutils` for binary generation

## Usage

### 1. Flash the Bootloader

First, flash the bootloader to your device:

```
cargo flash --features embassy-stm32/stm32wb55rg --release --chip STM32WB55RGVx
```

### 2. Build and Flash Application

Generate your application binary and flash it using DFU:

```
cargo objcopy --release -- -O binary fw.bin
dfu-util -d c0de:cafe -w -D fw.bin
```

## Troubleshooting

- Make sure your device is in DFU mode before flashing
- Verify the USB VID:PID matches your device (c0de:cafe)
- Check USB connections if the device is not detected
- Make sure the transfer size option of `dfu-util` matches the bootloader configuration. By default, `dfu-util` will use the transfer size reported by the device, but you can override it with the `-t` option if needed.
- Make sure `control_buf` size is larger than or equal to the `usb_dfu` `BLOCK_SIZE` parameter (in this example, both are set to 4096 bytes).
