#!/bin/bash

set -euxo pipefail

cargo build --target thumbv7em-none-eabihf -p embassy-examples --bins
cargo build --target thumbv7em-none-eabihf -p embassy

# Build with all feature combinations
cd embassy-nrf
cargo build --target thumbv7em-none-eabihf -p embassy-nrf --features 52810
#cargo build --target thumbv7em-none-eabihf -p embassy-nrf --features 52811  # nrf52811-hal doesn't exist yet
cargo build --target thumbv7em-none-eabihf -p embassy-nrf --features 52832
cargo build --target thumbv7em-none-eabihf -p embassy-nrf --features 52833
cargo build --target thumbv7em-none-eabihf -p embassy-nrf --features 52840
