#!/bin/bash

set -euxo pipefail
cd $(dirname $0)

export CARGO_TARGET_DIR=$(pwd)/target
export DEFMT_LOG=trace

# build examples
#==================

(cd examples/rpi-pico-w; WIFI_NETWORK=foo WIFI_PASSWORD=bar cargo build --release)


# build with log/defmt combinations
#=====================================

cargo build --target thumbv6m-none-eabi --features ''
cargo build --target thumbv6m-none-eabi --features 'log'
cargo build --target thumbv6m-none-eabi --features 'defmt'
cargo build --target thumbv6m-none-eabi --features 'log,firmware-logs'
cargo build --target thumbv6m-none-eabi --features 'defmt,firmware-logs'

(cd cyw43-pio; cargo build --target thumbv6m-none-eabi --features '')
(cd cyw43-pio; cargo build --target thumbv6m-none-eabi --features 'overclock')
