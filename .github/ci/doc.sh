#!/bin/bash
## on push branch=main

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export BUILDER_THREADS=4
export BUILDER_COMPRESS=true

# force rustup to download the toolchain before starting building.
# Otherwise, the docs builder is running multiple instances of cargo rustdoc concurrently.
# They all see the toolchain is not installed and try to install it in parallel
# which makes rustup very sad
rustc --version > /dev/null

docserver-builder -i ./embassy-boot/boot -o webroot/crates/embassy-boot/git.zup
docserver-builder -i ./embassy-boot/nrf -o webroot/crates/embassy-boot-nrf/git.zup
docserver-builder -i ./embassy-boot/rp -o webroot/crates/embassy-boot-rp/git.zup
docserver-builder -i ./embassy-boot/stm32 -o webroot/crates/embassy-boot-stm32/git.zup
docserver-builder -i ./embassy-embedded-hal -o webroot/crates/embassy-embedded-hal/git.zup
docserver-builder -i ./embassy-executor -o webroot/crates/embassy-executor/git.zup
docserver-builder -i ./embassy-futures -o webroot/crates/embassy-futures/git.zup
docserver-builder -i ./embassy-lora -o webroot/crates/embassy-lora/git.zup
docserver-builder -i ./embassy-net -o webroot/crates/embassy-net/git.zup
docserver-builder -i ./embassy-net-driver -o webroot/crates/embassy-net-driver/git.zup
docserver-builder -i ./embassy-net-driver-channel -o webroot/crates/embassy-net-driver-channel/git.zup
docserver-builder -i ./embassy-nrf -o webroot/crates/embassy-nrf/git.zup
docserver-builder -i ./embassy-rp -o webroot/crates/embassy-rp/git.zup
docserver-builder -i ./embassy-sync -o webroot/crates/embassy-sync/git.zup
docserver-builder -i ./embassy-time -o webroot/crates/embassy-time/git.zup
docserver-builder -i ./embassy-usb -o webroot/crates/embassy-usb/git.zup
docserver-builder -i ./embassy-usb-driver -o webroot/crates/embassy-usb-driver/git.zup
docserver-builder -i ./embassy-usb-logger -o webroot/crates/embassy-usb-logger/git.zup
docserver-builder -i ./cyw43 -o webroot/crates/cyw43/git.zup
docserver-builder -i ./cyw43-pio -o webroot/crates/cyw43-pio/git.zup
docserver-builder -i ./embassy-net-wiznet -o webroot/crates/embassy-net-wiznet/git.zup
docserver-builder -i ./embassy-net-enc28j60 -o webroot/crates/embassy-net-enc28j60/git.zup
docserver-builder -i ./embassy-net-esp-hosted -o webroot/crates/embassy-net-esp-hosted/git.zup
docserver-builder -i ./embassy-stm32-wpan -o webroot/crates/embassy-stm32-wpan/git.zup --output-static webroot/static
docserver-builder -i ./embassy-net-adin1110 -o webroot/crates/embassy-net-adin1110/git.zup

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl -n embassy get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp webroot/crates $POD:/data
kubectl cp webroot/static $POD:/data

# build and upload stm32 last
# so that it doesn't prevent other crates from getting docs updates when it breaks.
rm -rf webroot
docserver-builder -i ./embassy-stm32 -o webroot/crates/embassy-stm32/git.zup

POD=$(kubectl -n embassy get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp webroot/crates $POD:/data
