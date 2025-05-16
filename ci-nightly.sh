#!/bin/bash

set -eo pipefail

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,smoltcp=info
if [[ -z "${CARGO_TARGET_DIR}" ]]; then
    export CARGO_TARGET_DIR=target_ci
fi

cargo batch  \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,log \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt,arch-cortex-m,executor-thread,executor-interrupt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-interrupt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread,executor-interrupt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32 \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32,executor-thread \
    --- build --release --manifest-path examples/nrf52840-rtic/Cargo.toml --target thumbv7em-none-eabi --artifact-dir out/examples/nrf52840-rtic \

RUSTFLAGS="$RUSTFLAGS -C target-cpu=atmega328p" cargo build --release --manifest-path embassy-executor/Cargo.toml --target avr-none -Z build-std=core,alloc --features nightly,arch-avr,avr-device/atmega328p
