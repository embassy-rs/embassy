# STM32 dual-bank flash Bootloader

## Overview

This bootloader leverages `embassy-boot` to interact with the flash. 
This example targets STM32 devices with dual-bank flash memory, with a primary focus on the STM32H747XI series. 
Users must modify the `memory.x` configuration file to match with the memory layout of their specific STM32 device.

Additionally, this example can be extended to utilize external flash memory, such as QSPI, for storing partitions.

## Memory Configuration

In this example's `memory.x` file, various symbols are defined to assist in effective memory management within the bootloader environment.  
For dual-bank STM32 devices, it's crucial to assign these symbols correctly to their respective memory banks. 

### Symbol Definitions

The bootloader's state and active symbols are anchored to the flash origin of **bank 1**:

- `__bootloader_state_start` and `__bootloader_state_end`
- `__bootloader_active_start` and `__bootloader_active_end`

In contrast, the Device Firmware Upgrade (DFU) symbols are aligned with the DFU flash origin in **bank 2**:

- `__bootloader_dfu_start` and `__bootloader_dfu_end`

```rust
__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(**FLASH**);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(**FLASH**);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(**FLASH**);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(**FLASH**);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(**DFU**);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(**DFU**);
```

## Flashing the Bootloader

To flash the bootloader onto your STM32H747XI device, use the following command:

```bash
cargo flash --features embassy-stm32/stm32h747xi-cm7 --release --chip STM32H747XIHx
```
