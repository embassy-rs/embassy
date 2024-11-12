# Bootloader for STM32

The bootloader uses `embassy-boot` to interact with the flash.

# Usage

Flash the bootloader

```
cargo flash --features embassy-stm32/stm32wb55rg --release --chip STM32WB55RGVx
```
