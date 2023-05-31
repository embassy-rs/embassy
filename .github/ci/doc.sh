#!/bin/bash
## on push branch=main

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export BUILDER_THREADS=6
export BUILDER_COMPRESS=true

docserver-builder -i ./embassy-stm32 -o crates/embassy-stm32/git.zup
docserver-builder -i ./embassy-boot/boot -o crates/embassy-boot/git.zup
docserver-builder -i ./embassy-boot/nrf -o crates/embassy-boot-nrf/git.zup
docserver-builder -i ./embassy-boot/rp -o crates/embassy-boot-rp/git.zup
docserver-builder -i ./embassy-boot/stm32 -o crates/embassy-boot-stm32/git.zup
docserver-builder -i ./embassy-cortex-m -o crates/embassy-cortex-m/git.zup
docserver-builder -i ./embassy-embedded-hal -o crates/embassy-embedded-hal/git.zup
docserver-builder -i ./embassy-executor -o crates/embassy-executor/git.zup
docserver-builder -i ./embassy-futures -o crates/embassy-futures/git.zup
docserver-builder -i ./embassy-lora -o crates/embassy-lora/git.zup
docserver-builder -i ./embassy-net -o crates/embassy-net/git.zup
docserver-builder -i ./embassy-net-driver -o crates/embassy-net-driver/git.zup
docserver-builder -i ./embassy-net-driver-channel -o crates/embassy-net-driver-channel/git.zup
docserver-builder -i ./embassy-nrf -o crates/embassy-nrf/git.zup
docserver-builder -i ./embassy-rp -o crates/embassy-rp/git.zup
docserver-builder -i ./embassy-sync -o crates/embassy-sync/git.zup
docserver-builder -i ./embassy-time -o crates/embassy-time/git.zup
docserver-builder -i ./embassy-usb -o crates/embassy-usb/git.zup
docserver-builder -i ./embassy-usb-driver -o crates/embassy-usb-driver/git.zup
docserver-builder -i ./embassy-usb-logger -o crates/embassy-usb-logger/git.zup
docserver-builder -i ./cyw43 -o crates/cyw43/git.zup
docserver-builder -i ./cyw43-pio -o crates/cyw43-pio/git.zup
docserver-builder -i ./embassy-net-w5500 -o crates/embassy-net-w5500/git.zup

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl -n embassy get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp crates $POD:/data
