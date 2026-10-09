#!/usr/bin/env bash
# Downloads the ST-signed STM32WB5x coprocessor binaries from the STM32CubeWB
# repository, locked to a fixed revision so the `fus_update` example always
# embeds the same, known-good images. Re-run this script to update the
# binaries, and bump REV to a newer vetted STM32CubeWB commit.
set -euo pipefail
cd "$(dirname "$0")"

REV=7c5aa7dcd2c0abe787f922bb06a6213c724d08e4
BASE="https://raw.githubusercontent.com/STMicroelectronics/STM32CubeWB/$REV/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x"

for f in stm32wb5x_FUS_fw.bin stm32wb5x_FUS_fw_1_2_0.bin stm32wb5x_FUS_fw_for_fus_0_5_3.bin stm32wb5x_BLE_Mac_802_15_4_fw.bin stm32wb5x_BLE_HCILayer_fw.bin; do
    echo "Downloading $f"
    curl -fLO "$BASE/$f"
done
