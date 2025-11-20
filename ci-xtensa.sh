#!/bin/bash

set -eo pipefail

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,smoltcp=info
export RUSTUP_TOOLCHAIN=esp
if [[ -z "${CARGO_TARGET_DIR}" ]]; then
    export CARGO_TARGET_DIR=target_ci
fi

cargo embassy-devtool build --group xtensa
