#!/bin/sh
set -eu

cargo build --release --bin blinky_ram --features blinky-ram

elf=target/thumbv6m-none-eabi/release/blinky_ram
boot2="${elf}.boot2.bin"
app="${elf}.app.bin"
image="${elf}.flash.bin"

arm-none-eabi-objcopy --dump-section .boot2="$boot2" "$elf"
arm-none-eabi-objcopy -O binary --remove-section=.boot2 "$elf" "$app"
if [ "$(wc -c < "$boot2")" -ne 256 ]; then
    echo "boot2 must be exactly 256 bytes" >&2
    exit 1
fi
cat "$boot2" "$app" > "$image"

probe-rs download --chip RP2040 --binary-format bin --base-address 0x10000000 --verify "$image"
probe-rs reset --chip RP2040
probe-rs attach --chip RP2040 "$elf"
