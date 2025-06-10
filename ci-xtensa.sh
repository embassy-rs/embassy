#!/bin/bash

set -eo pipefail

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,smoltcp=info
export RUSTUP_TOOLCHAIN=esp
if [[ -z "${CARGO_TARGET_DIR}" ]]; then
    export CARGO_TARGET_DIR=target_ci
fi

cargo batch \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features log \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32s2-none-elf --features defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features defmt,arch-spin,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32s2-none-elf --features defmt,arch-spin,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32s3-none-elf --features defmt,arch-spin,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features arch-spin \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features arch-spin,rtos-trace \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target xtensa-esp32-none-elf --features arch-spin,executor-thread \
    --- build --release --manifest-path embassy-sync/Cargo.toml --target xtensa-esp32s2-none-elf --features defmt \
    --- build --release --manifest-path embassy-time/Cargo.toml --target xtensa-esp32s2-none-elf --features defmt,defmt-timestamp-uptime,mock-driver \
    --- build --release --manifest-path embassy-time-queue-utils/Cargo.toml --target xtensa-esp32s2-none-elf \
    --- build --release --manifest-path embassy-time-queue-utils/Cargo.toml --target xtensa-esp32s2-none-elf --features generic-queue-8 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,medium-ethernet,packet-trace \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,multicast,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,dhcpv4,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,dhcpv4,medium-ethernet,dhcpv4-hostname \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv6,medium-ieee802154 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet,medium-ieee802154 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target xtensa-esp32-none-elf --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip,medium-ethernet,medium-ieee802154 \
