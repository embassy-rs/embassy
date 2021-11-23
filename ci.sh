#!/bin/bash

set -euo pipefail

export CARGO_TARGET_DIR=$PWD/target_ci
export RUSTFLAGS=-Dwarnings

find -name '*.rs' -not -path '*target*' -not -path '*stm32-metapac-gen/out/*'  | xargs rustfmt --check  --skip-children --unstable-features --edition 2018

cargo batch  \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi --features log,executor-agnostic \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi --features defmt \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv6m-none-eabi --features defmt \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52805 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52810 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52811 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52820 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52832 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52833 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf9160-s \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf9160-ns \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-app-s \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-app-ns \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-net \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840,log \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f411ce,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi,log \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi-cm7,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l476vg,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32l072cz,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32l151cb-a,defmt \
    --- build --release --manifest-path examples/std/Cargo.toml --target x86_64-unknown-linux-gnu --out-dir out/examples/std \
    --- build --release --manifest-path examples/nrf/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/nrf \
    --- build --release --manifest-path examples/rp/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/rp \
    --- build --release --manifest-path examples/stm32f0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32f0 \
    --- build --release --manifest-path examples/stm32f1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f1 \
    --- build --release --manifest-path examples/stm32f4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f4 \
    --- build --release --manifest-path examples/stm32f7/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f7 \
    --- build --release --manifest-path examples/stm32g0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32g0 \
    --- build --release --manifest-path examples/stm32h7/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h7 \
    --- build --release --manifest-path examples/stm32l0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32l0 \
    --- build --release --manifest-path examples/stm32l1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32l1 \
    --- build --release --manifest-path examples/stm32l4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32l4 \
    --- build --release --manifest-path examples/stm32u5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32u5 \
    --- build --release --manifest-path examples/stm32wb55/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wb55 \
    --- build --release --manifest-path examples/stm32wl55/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wl55 \
    --- build --release --manifest-path examples/wasm/Cargo.toml --target wasm32-unknown-unknown --out-dir out/examples/wasm \
