# embassy-boot-stm32

An [Embassy](https://embassy.dev) project.

An adaptation of `embassy-boot` for STM32.

## Features

* Configure bootloader partitions based on linker script.
* Load applications from active partition.

## Direct-XIP

Pass a `BlockingDirectXipState` and your slot validation results to `BootLoader::try_prepare_direct_xip`. Pass the selected slot's start address to `BootLoader::load`.

Use `BlockingDirectXipState::<STATE>::BUFFER_SIZE` to size the state object's scratch buffer. See `embassy-boot` for the flash layout and update flow.
