#!/usr/bin/env bash
set -e

ELF_PATH="$1"
BIN_PATH="${ELF_PATH}.bin"

# 1. Convert the ELF binary to .bin format
arm-none-eabi-objcopy -O binary "$ELF_PATH" "$BIN_PATH"

# 2. Flash the .bin file using svl.py
python3 tools/svl.py /dev/ttyUSB0 -f "$BIN_PATH"
