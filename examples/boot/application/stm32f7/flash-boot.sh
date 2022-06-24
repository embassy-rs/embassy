#!/bin/bash
mv ../../bootloader/stm32/memory.x ../../bootloader/stm32/memory-old.x
cp memory-bl.x ../../bootloader/stm32/memory.x

cargo flash --manifest-path ../../bootloader/stm32/Cargo.toml --release --features embassy-stm32/stm32f767zi --chip STM32F767ZITx --target thumbv7em-none-eabihf

rm ../../bootloader/stm32/memory.x
mv ../../bootloader/stm32/memory-old.x ../../bootloader/stm32/memory.x
