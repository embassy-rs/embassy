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
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,smoltcp=info
if [[ -z "${CARGO_TARGET_DIR}" ]]; then
    export CARGO_TARGET_DIR=target_ci
fi

cargo embassy-devtool build

# temporarily disabled, these boards are dead.
rm -rf out/tests/stm32f103c8
rm -rf out/tests/nrf52840-dk
rm -rf out/tests/nrf52833-dk
rm -rf out/tests/nrf5340-dk

# disabled because these boards are not on the shelf
rm -rf out/tests/mspm0g3507

# rm out/tests/stm32wb55rg/wpan_mac
rm out/tests/stm32wb55rg/wpan_ble

# unstable, I think it's running out of RAM?
rm out/tests/stm32f207zg/eth

# temporarily disabled, flaky.
rm out/tests/stm32f207zg/usart_rx_ringbuffered
rm out/tests/stm32l152re/usart_rx_ringbuffered

# doesn't work, gives "noise error", no idea why. usart_dma does pass.
rm out/tests/stm32u5a5zj/usart

# probe-rs error: "multi-core ram flash start not implemented yet"
# As of 2025-02-17 these tests work when run from flash
rm out/tests/pimoroni-pico-plus-2/multicore
rm out/tests/pimoroni-pico-plus-2/gpio_multicore
rm out/tests/pimoroni-pico-plus-2/spinlock_mutex_multicore
# Doesn't work when run from ram on the 2350
rm out/tests/pimoroni-pico-plus-2/flash
# This test passes locally but fails on the HIL, no idea why
rm out/tests/pimoroni-pico-plus-2/i2c
# The pico2 plus doesn't have the adcs hooked up like the picoW does.
rm out/tests/pimoroni-pico-plus-2/adc
# temporarily disabled
rm out/tests/pimoroni-pico-plus-2/pwm

# flaky
rm out/tests/rpi-pico/pwm
rm out/tests/rpi-pico/cyw43-perf
rm out/tests/rpi-pico/uart_buffered

rm out/tests/stm32h563zi/usart_dma

# tests are implemented but the HIL test farm doesn't actually have these boards, yet
rm -rf out/tests/stm32c071rb
rm -rf out/tests/stm32f100rd
rm -rf out/tests/stm32f107vc

if [[ -z "${TELEPROBE_TOKEN-}" ]]; then
    echo No teleprobe token found, skipping running HIL tests
    exit
fi

teleprobe client run -r out/tests
