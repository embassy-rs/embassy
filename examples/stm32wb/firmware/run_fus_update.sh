#!/usr/bin/env bash
# Build `fus_update` with the given fw_* features and drive a full FUS OTA with
# probe-rs only: download, reset, then stream RTT, automatically reconnecting
# across the resets the upgrader performs. Ctrl-C to quit.
#
# Usage:
#   ./run_fus_update.sh fw_fus fw_hci        # switch to the BLE HCI layer stack
#   ./run_fus_update.sh fw_fus fw_mac_ble    # switch to the BLE + MAC combo stack
set -euo pipefail
cd "$(dirname "$0")/.."

if [ $# -eq 0 ]; then
    echo "usage: $0 <fw feature> [fw feature...]   e.g. $0 fw_fus fw_hci" >&2
    exit 2
fi
FEATURES=$(IFS=,; echo "$*")
ELF=target/thumbv7em-none-eabihf/release/fus_update

cargo build --release --bin fus_update --features "$FEATURES"

probe-rs download --chip STM32WB55RG "$ELF"
probe-rs reset --chip STM32WB55RG

# `probe-rs run` is unreliable on this board (see ../probe-rs-issue.md), so
# stream RTT with `attach`, reconnecting whenever the device resets.
trap 'exit 0' INT
while true; do
    probe-rs attach --chip STM32WB55RG "$ELF" || true
    echo "=== attach dropped (device reset?); reconnecting (Ctrl-C to quit) ==="
    sleep 2
done
