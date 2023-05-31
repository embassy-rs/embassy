#!/bin/bash
## on push branch=main

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export BUILDER_THREADS=6

docserver-builder ./embassy-boot/boot crates/embassy-boot/git.zup
docserver-builder ./embassy-boot/nrf crates/embassy-boot-nrf/git.zup
docserver-builder ./embassy-boot/rp crates/embassy-boot-rp/git.zup
docserver-builder ./embassy-boot/stm32 crates/embassy-boot-stm32/git.zup
docserver-builder ./embassy-cortex-m crates/embassy-cortex-m/git.zup
docserver-builder ./embassy-embedded-hal crates/embassy-embedded-hal/git.zup
docserver-builder ./embassy-executor crates/embassy-executor/git.zup
docserver-builder ./embassy-futures crates/embassy-futures/git.zup
docserver-builder ./embassy-lora crates/embassy-lora/git.zup
docserver-builder ./embassy-net crates/embassy-net/git.zup
docserver-builder ./embassy-net-driver crates/embassy-net-driver/git.zup
docserver-builder ./embassy-net-driver-channel crates/embassy-net-driver-channel/git.zup
docserver-builder ./embassy-nrf crates/embassy-nrf/git.zup
docserver-builder ./embassy-rp crates/embassy-rp/git.zup
docserver-builder ./embassy-sync crates/embassy-sync/git.zup
docserver-builder ./embassy-time crates/embassy-time/git.zup
docserver-builder ./embassy-usb crates/embassy-usb/git.zup
docserver-builder ./embassy-usb-driver crates/embassy-usb-driver/git.zup
docserver-builder ./embassy-usb-logger crates/embassy-usb-logger/git.zup
#docserver-builder ./embassy-stm32 crates/embassy-stm32/git.zup

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl -n embassy get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp crates $POD:/data
