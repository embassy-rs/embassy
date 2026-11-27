# embassy-boot-rp

An [Embassy](https://embassy.dev) project.

An adaptation of `embassy-boot` for RP2040 and RP235x.

NOTE: The applications using this bootloader should not link with the `link-rp.x` linker script.

## Features

* Configure bootloader partitions based on linker script.
* Load applications from active partition.

## Direct-XIP

See [embassy-boot](../embassy-boot/README.md#direct-xip) for the state layout, boot preparation and update flow.
