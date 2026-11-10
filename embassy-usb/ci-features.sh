#!/usr/bin/env bash
# Build-check the embassy-usb class/role feature matrix.
#
# Run from anywhere; the script changes to its own directory. Intended for CI
# and for local verification after touching class modules or Cargo features.
#
# This repository has no root workspace, so this is deliberately a standalone
# script rather than a cargo alias.

set -euo pipefail

cd "$(dirname "$0")"

CARGO="${CARGO:-cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} -Dwarnings"

check() {
    printf '==> cargo check --no-default-features %s\n' "$*"
    "$CARGO" check -q --no-default-features "$@"
}

# Baseline role combinations.
check
check --features device
check --features host
check --features device,host
check --features class-all

# Every device-capable class on its own.
for class in \
    bth ccid cdc-acm cdc-ecm cdc-ncm cmsis-dap-v2 dfu gud hid midi msc uac usbtmc uvc web-usb
do
    check --features "device,class-$class"
done

# Every host-capable class on its own.
for class in ccid cdc-acm cdc-ecm gip hid hub midi msc uac vcp; do
    check --features "host,class-$class"
done

# Dual-role classes with both roles enabled.
for class in ccid cdc-acm cdc-ecm hid midi msc uac; do
    check --features "device,host,class-$class"
done

# Shared host error mapping without any class enabled.
check --features host,embedded-io

# Public test harness (implies host).
check --features test-utils

# Logging backends and the full class set.
check --features device,host,class-all,defmt,block-device-driver
check --features device,host,class-all,log,block-device-driver

# Cross-check the two smallest relevant targets when they are installed.
if rustup target list --installed 2>/dev/null | grep -qx thumbv6m-none-eabi; then
    printf '==> cargo check --target thumbv6m-none-eabi (device,class-cdc-acm)\n'
    "$CARGO" check -q --no-default-features --features device,class-cdc-acm --target thumbv6m-none-eabi
fi

if rustup target list --installed 2>/dev/null | grep -qx thumbv7em-none-eabi; then
    printf '==> cargo check --target thumbv7em-none-eabi (host,class-cdc-acm)\n'
    "$CARGO" check -q --no-default-features --features host,class-cdc-acm --target thumbv7em-none-eabi
fi

printf 'embassy-usb feature matrix: OK\n'
