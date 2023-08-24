#!/bin/bash

set -euo pipefail

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace

cargo batch  \
    --- build --release --manifest-path embassy-boot/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840 \
    --- build --release --manifest-path embassy-boot/nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns \
    --- build --release --manifest-path embassy-boot/rp/Cargo.toml --target thumbv6m-none-eabi \
    --- build --release --manifest-path embassy-boot/stm32/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wl55jc-cm4 \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features log \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv6m-none-eabi --features defmt \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,dhcpv4,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52805,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52810,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52811,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52820,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52832,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52833,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf9160-s,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf9160-ns,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-app-s,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-app-ns,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nrf5340-net,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840,log,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nrf52840,defmt,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features unstable-traits,defmt \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features unstable-traits,log \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features qspi-as-gpio \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32g473cc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32g491re,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32u585zi,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32wb55vy,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32wl55cc-cm4,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l4r9zi,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f303vc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f411ce,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f411ce,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi,log,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi,log,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi-cm7,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi-cm7,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l476vg,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l476vg,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32l072cz,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32l072cz,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32l151cb-a,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32l151cb-a,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f410tb,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f410tb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi,log,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi,log,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi-cm7,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi-cm7,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l476vg,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32l476vg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32l072cz,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32l072cz,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32l151cb-a,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32l151cb-a,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32f217zg,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32f217zg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path examples/nrf52840/Cargo.toml --target thumbv7em-none-eabi --no-default-features --out-dir out/examples/nrf52840 --bin raw_spawn \
    --- build --release --manifest-path examples/stm32l0/Cargo.toml --target thumbv6m-none-eabi --no-default-features --out-dir out/examples/stm32l0 --bin raw_spawn \
