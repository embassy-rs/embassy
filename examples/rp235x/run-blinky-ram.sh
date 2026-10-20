#!/bin/sh
set -eu

cargo build --release --bin blinky_ram

elf=target/thumbv8m.main-none-eabihf/release/blinky_ram
sealed="${elf}.sealed.elf"
uf2="${elf}.flash.uf2"

# Add a boot-ROM load map, then package the SRAM image for flash.
picotool seal -t elf "$elf" "$sealed"
picotool uf2 convert "$sealed" "$uf2" --offset 0x10000000 --family rp2350-arm-s --platform rp2350

probe-rs download --chip RP235x --binary-format uf2 --verify "$uf2"
probe-rs reset --chip RP235x
probe-rs attach --chip RP235x "$elf"
