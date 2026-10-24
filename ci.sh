#!/bin/bash

set -eo pipefail

if ! command -v cargo-batch &> /dev/null; then
    echo "cargo-batch could not be found. Install it with the following command:"
    echo ""
    echo "    cargo install --git https://github.com/embassy-rs/cargo-batch cargo --bin cargo-batch --locked"
    echo ""
    exit 1
fi

if ! command -v cargo-embassy-devtool &> /dev/null; then
    echo "cargo-embassy-devtool could not be found. Install it with the following command:"
    echo ""
    echo "    cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked"
    echo ""
    exit 1
fi

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,xarxa=info
if [[ -z "${CARGO_TARGET_DIR}" ]]; then
    export CARGO_TARGET_DIR=target_ci
fi

cargo embassy-devtool build

if [[ -z "${TELEPROBE_TOKEN-}" ]]; then
    echo No teleprobe token found, skipping running HIL tests
    exit
fi

# temporarily disabled, these boards are dead.
rm -rf out/tests/stm32f103c8
rm -rf out/tests/stm32l152re
rm -rf out/tests/stm32f207zg
rm -rf out/tests/nrf52840-dk
rm -rf out/tests/nrf52833-dk
rm -rf out/tests/nrf5340-dk
rm -rf out/tests/nrf51422-dk

# temporarily disabled

rm -f out/tests/stm32u585ai/saes

# temporarily disabled, flaky.
rm -f out/tests/stm32wba65ri/adc

# tests are implemented but the HIL test farm doesn't actually have these boards, yet
rm -rf out/tests/stm32c071rb
rm -rf out/tests/stm32f100rd
rm -rf out/tests/stm32f107vc
rm -rf out/tests/mspm0g3507

teleprobe client run -r out/tests
