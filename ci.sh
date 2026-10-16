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

# always run check to prime cache
cargo embassy-devtool check --force-incremental

if [[ -z "${TELEPROBE_TOKEN-}" ]]; then
    echo No teleprobe token found, skipping running HIL tests
    exit
fi

cargo embassy-devtool build

# temporarily disabled, these boards are dead.
rm -rf out/tests/stm32f103c8
rm -rf out/tests/nrf52840-dk
rm -rf out/tests/nrf52833-dk
rm -rf out/tests/nrf5340-dk
rm -rf out/tests/nrf51422-dk

# disabled because these boards are not on the shelf
rm -rf out/tests/mspm0g3507

# rm -f out/tests/stm32wb55rg/wpan_mac
# rm -f out/tests/stm32wb55rg/wpan_ble

# temporarily disabled
rm -rf out/tests/stm32wba65ri
rm -rf out/tests/stm32l152re
rm -rf out/tests/stm32f207zg
rm -rf out/tests/nrf9160-dk

# disabled because of network error

rm -f out/tests/stm32f767zi/adc
rm -f out/tests/stm32l552ze/gpio
rm -f out/tests/stm32h7a3zi/rng
rm -f out/tests/stm32l552ze/crypto_hash
rm -f out/tests/stm32l496zg/usart_dma
rm -f out/tests/stm32wba52cg/crypto_ecdh
rm -f out/tests/stm32wl55jc/stop
rm -f out/tests/frdm-mcx-a266/gpio
rm -f out/tests/stm32f429zi/timer
rm -f out/tests/stm32f303ze/rtc
rm -f out/tests/stm32u585ai/crypto_aes
rm -f out/tests/stm32u083rc/spi
rm -f out/tests/stm32l496zg/rng
rm -f out/tests/stm32l4a6zg/usart_dma
rm -f out/tests/frdm-mcx-a266/i3c
rm -f out/tests/stm32u5a5zj/rng
rm -f out/tests/stm32f446re/stop
rm -f out/tests/stm32h7s3l8/cordic
rm -f out/tests/stm32g491re/timer
rm -f out/tests/stm32wb55rg/crypto_ecdh
rm -f out/tests/stm32f091rc/rtc
rm -f out/tests/stm32c031c6/gpio
rm -f out/tests/stm32h753zi/crypto_aes
rm -f out/tests/stm32l4r5zi/gpio
rm -f out/tests/stm32f429zi/usart
rm -f out/tests/stm32f303ze/adc
rm -f out/tests/stm32g071rb/spi
rm -f out/tests/stm32u585ai/adc
rm -f out/tests/stm32u083rc/rng
rm -f out/tests/stm32h755zi/timer
rm -f out/tests/stm32h563zi/stop
rm -f out/tests/stm32l4a6zg/rtc
rm -f out/tests/stm32l073rz/timer
rm -f out/tests/stm32wl55jc/spi
rm -f out/tests/stm32h7a3zi/usart_dma
rm -f out/tests/stm32h503rb/gpio
rm -f out/tests/stm32u5a5zj/timer
rm -f out/tests/stm32h7s3l8/crypto_ecdsa
rm -f out/tests/stm32g491re/rtc
rm -f out/tests/stm32wb55rg/gpio
rm -f out/tests/stm32f091rc/usart
rm -f out/tests/stm32c031c6/timer
rm -f out/tests/stm32h753zi/usart
rm -f out/tests/stm32l4r5zi/rng
rm -f out/tests/stm32g071rb/spi_dma
rm -f out/tests/stm32h755zi/crypto_aes
rm -f out/tests/stm32h563zi/gpio
rm -f out/tests/stm32l073rz/eeprom
rm -f out/tests/stm32h503rb/usart_dma
rm -f out/tests/stm32h755zi/usart_dma
rm -f out/tests/stm32f303ze/usart
rm -f out/tests/stm32u585ai/crypto_hash
rm -f out/tests/stm32u083rc/usart_dma
rm -f out/tests/stm32f429zi/usart_dma
rm -f out/tests/stm32g071rb/timer
rm -f out/tests/stm32l4a6zg/spi
rm -f out/tests/stm32c031c6/usart
rm -f out/tests/stm32g491re/stop
rm -f out/tests/stm32wb55rg/timer
rm -f out/tests/stm32h7a3zi/fdcan
rm -f out/tests/stm32h563zi/cordic
rm -f out/tests/stm32h753zi/adc
rm -f out/tests/stm32wl55jc/adc
rm -f out/tests/stm32f091rc/adc
rm -f out/tests/stm32l4r5zi/timer
rm -f out/tests/stm32h7s3l8/crypto_ecdh
rm -f out/tests/stm32l073rz/usart
rm -f out/tests/stm32h503rb/timer
rm -f out/tests/stm32u585ai/usart_dma
rm -f out/tests/stm32u083rc/usart
rm -f out/tests/stm32g071rb/gpio
rm -f out/tests/stm32l4a6zg/gpio
rm -f out/tests/stm32c031c6/spi
rm -f out/tests/stm32g491re/rng
rm -f out/tests/stm32wb55rg/rtc
rm -f out/tests/stm32h563zi/timer
rm -f out/tests/stm32f091rc/gpio
rm -f out/tests/stm32l4r5zi/rtc
rm -f out/tests/stm32h7s3l8/cryp
rm -f out/tests/stm32h503rb/usart
rm -f out/tests/stm32g491re/usart
rm -f out/tests/stm32c031c6/usart_dma
rm -f out/tests/stm32g491re/usart_dma
rm -f out/tests/stm32h563zi/spi_dma
rm -f out/tests/stm32wb55rg/crypto_ecdsa
rm -f out/tests/stm32wb55rg/crypto_ec
rm -f out/tests/stm32h563zi/fdcan
rm -f out/tests/stm32l4r5zi/adc
rm -f out/tests/stm32f091rc/timer
rm -f out/tests/stm32l4r5zi/spi
rm -f out/tests/stm32h7s3l8/usart
rm -f out/tests/stm32h7s3l8/crypto_hash
rm -f out/tests/stm32h503rb/spi
rm -f out/tests/stm32h503rb/adc

# disabled until wired
rm -f out/tests/stm32f446re/pwm_ringbuffered

# unstable, I think it's running out of RAM?
rm -f out/tests/stm32f207zg/eth

# temporarily disabled, flaky.
rm -f out/tests/stm32f207zg/usart_rx_ringbuffered
rm -f out/tests/stm32l152re/usart_rx_ringbuffered

# doesn't work, gives "noise error", no idea why. usart_dma does pass.
rm -f out/tests/stm32u5a5zj/usart

# probe-rs error: "multi-core ram flash start not implemented yet"
# As of 2025-02-17 these tests work when run from flash
rm -f out/tests/pimoroni-pico-plus-2/multicore
rm -f out/tests/pimoroni-pico-plus-2/gpio_multicore
rm -f out/tests/pimoroni-pico-plus-2/spinlock_mutex_multicore
# Doesn't work when run from ram on the 2350
rm -f out/tests/pimoroni-pico-plus-2/flash
# This test passes locally but fails on the HIL, no idea why
rm -f out/tests/pimoroni-pico-plus-2/i2c
# The pico2 plus doesn't have the adcs hooked up like the picoW does.
rm -f out/tests/pimoroni-pico-plus-2/adc
# temporarily disabled
rm -f out/tests/pimoroni-pico-plus-2/pwm
rm -f out/tests/frdm-mcx-a266/trng

# flaky
rm -f out/tests/rpi-pico/pwm
# rm -f out/tests/rpi-pico/cyw43-perf
rm -f out/tests/rpi-pico/uart_buffered
rm -f out/tests/rpi-pico/spi_async

rm -f out/tests/stm32h563zi/usart_dma

# tests are implemented but the HIL test farm doesn't actually have these boards, yet
rm -rf out/tests/stm32c071rb
rm -rf out/tests/stm32f100rd
rm -rf out/tests/stm32f107vc

teleprobe client run -r out/tests
