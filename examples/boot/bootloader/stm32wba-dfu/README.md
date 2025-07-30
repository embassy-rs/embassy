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
cargo flash --features embassy-stm32/stm32wba65ri --release --chip STM32WBA65RI
```

### 2. Build and Flash Application

Generate your application binary and flash it using DFU:

```
cargo objcopy --release -- -O binary fw.bin
dfu-util -d c0de:cafe -w -D fw.bin
```

### 3. Sign Updates Before Flashing (Optional)

Currently, embassy-usb-dfu only supports a limited implementation of the generic support for ed25519-based update verfication in embassy-boot. This implementation assumes that a signature is simply concatenated to the end of an update binary. For more details, please see https://embassy.dev/book/#_verification and/or refer to the documentation for embassy-boot-dfu.

To sign (and then verify) application updates, you will first need to generate a key pair:

```
signify-openbsd -G -n -p secrets/key.pub -s secrets/key.sec
tail -n1 secrets/key.pub | base64 -d -i - | dd ibs=10 skip=1 > secrets/key.pub.short
```

Then you will need to sign all you binaries with the private key:

```
cargo objcopy --release -- -O binary fw.bin
shasum -a 512 -b fw.bin | head -c128 | xxd -p -r > target/fw-hash.txt
signify-openbsd -S -s secrets/key.sec -m target/fw-hash.txt -x target/fw-hash.sig
cp fw.bin fw-signed.bin
tail -n1 target/fw-hash.sig | base64 -d -i - | dd ibs=10 skip=1 >> fw-signed.bin
dfu-util -d c0de:cafe -w -D fw-signed.bin
```

Finally, as shown in this example with the `verify` feature flag enabled, you then need to embed the public key into your bootloader so that it can verify update signatures.

N.B. Please note that the exact steps above are NOT a good example of how to manage your keys securely. In a production environment, you should take great care to ensure that (at least the private key) is protected and not leaked into your version control system.

## Troubleshooting

- Make sure your device is in DFU mode before flashing
- Verify the USB VID:PID matches your device (c0de:cafe)
- Check USB connections if the device is not detected
- Make sure the transfer size option of `dfu-util` matches the bootloader configuration. By default, `dfu-util` will use the transfer size reported by the device, but you can override it with the `-t` option if needed.
- Make sure `control_buf` size is larger than or equal to the `usb_dfu` `BLOCK_SIZE` parameter (in this example, both are set to 4096 bytes).
