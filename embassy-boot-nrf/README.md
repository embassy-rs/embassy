# embassy-boot-nrf

An [Embassy](https://embassy.dev) project.

An adaptation of `embassy-boot` for nRF.

## Features

- Load applications with or without the softdevice.
- Configure bootloader partitions based on linker script.
- Using watchdog timer to detect application failure.

## Working with a SoftDevice

When a SoftDevice is present, it handles starting the bootloader and the application as needed.

The SoftDevice architecture supports the bootloader via a configurable base address, referred to as `BOOTLOADERADDR`, in the application flash region. This address can be specified either:

1. At the `MBR_BOOTLOADER_ADDR` location in flash memory (defined in `nrf_mbr.h`), or
2. In the `UICR.NRFFW[0]` register.

The `UICR.NRFFW[0]` register is used only if `MBR_BOOTLOADER_ADDR` has its default value of `0xFFFFFFFF`. This bootloader relies on the latter approach.

In the `memory.x` linker script, there is a section `.uicr_bootloader_start_address` (origin `0x10001014`, length `0x4`) that stores the `BOOTLOADERADDR` value.
Ensure that `__bootloader_start` is set to the origin address of the bootloader partition.

When a bootloader is present, the SoftDevice forwards interrupts to it and executes the bootloader reset handler, defined in the bootloader's vector table at `BOOTLOADERADDR`.

Once the bootloader loads the application, the SoftDevice initiates the Application Reset Handler, defined in the application’s vector table at APP_CODE_BASE hardcoded in the SoftDevice.
The active partition's origin **must** match the `APP_CODE_BASE` value hardcoded within the SoftDevice. This value can be found in the release notes for each SoftDevice version.
