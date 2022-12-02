# Bootloader for RP2040

The bootloader uses `embassy-boot` to interact with the flash.

# Usage

Flash the bootloader

```
cargo flash --release --chip RP2040 --target thumbv6m-none-eabi
```
