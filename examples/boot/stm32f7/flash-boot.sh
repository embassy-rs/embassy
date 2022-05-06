#!/bin/bash
mv ../../../embassy-boot/stm32/memory.x ../../../embassy-boot/stm32/memory-old.x
cp memory-bl.x ../../../embassy-boot/stm32/memory.x

cargo flash --manifest-path ../../../embassy-boot/stm32/Cargo.toml --release --features embassy-stm32/stm32f767zi --chip STM32F767ZITx --target thumbv7em-none-eabihf

rm ../../../embassy-boot/stm32/memory.x
mv ../../../embassy-boot/stm32/memory-old.x ../../../embassy-boot/stm32/memory.x
