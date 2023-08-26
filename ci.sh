#!/bin/bash

set -euo pipefail

export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace,embassy_hal_internal=debug,embassy_net_esp_hosted=debug,cyw43=info,cyw43_pio=info,smoltcp=info

TARGET=$(rustc -vV | sed -n 's|host: ||p')

BUILD_EXTRA=""
if [ $TARGET = "x86_64-unknown-linux-gnu" ]; then
    BUILD_EXTRA="--- build --release --manifest-path examples/std/Cargo.toml --target $TARGET --out-dir out/examples/std"
fi

find . -name '*.rs' -not -path '*target*' | xargs rustfmt --check  --skip-children --unstable-features --edition 2021

cargo batch  \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,log \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt,arch-cortex-m,executor-thread,executor-interrupt,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-interrupt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-interrupt,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread,executor-interrupt \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target thumbv7em-none-eabi --features nightly,arch-cortex-m,executor-thread,executor-interrupt,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32 \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32,integrated-timers \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32,executor-thread \
    --- build --release --manifest-path embassy-executor/Cargo.toml --target riscv32imac-unknown-none-elf --features nightly,arch-riscv32,executor-thread,integrated-timers \
    --- build --release --manifest-path embassy-sync/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-time/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt,defmt-timestamp-uptime,tick-hz-32_768,generic-queue-8 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,dhcpv4,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,dhcpv4,medium-ethernet,nightly \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv6,medium-ieee802154 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet,medium-ieee802154 \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv6,medium-ethernet,nightly \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ethernet \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ethernet,nightly \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip,nightly \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip,medium-ethernet,nightly \
    --- build --release --manifest-path embassy-net/Cargo.toml --target thumbv7em-none-eabi --features defmt,tcp,udp,dns,proto-ipv4,proto-ipv6,medium-ip,medium-ethernet,medium-ieee802154,nightly \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52805,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52810,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52811,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52820,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52832,gpiote,time-driver-rtc1,reset-pin-as-gpio \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52833,gpiote,time-driver-rtc1,unstable-traits,nfc-pins-as-gpio \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf9160-s,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf9160-ns,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-app-s,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-app-ns,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-net,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,log,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,defmt,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly,unstable-traits,defmt \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly,unstable-traits,log \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly,unstable-traits \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly,intrinsics \
    --- build --release --manifest-path embassy-rp/Cargo.toml --target thumbv6m-none-eabi --features nightly,qspi-as-gpio \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,exti \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt,exti,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,exti,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,time-driver-any \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,exti \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt,exti,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features stm32l552ze,nightly,defmt \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f410tb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f411ce,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f413vh,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f429zi,log,exti,time-driver-any,unstable-traits,embedded-sdmmc \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f730i8,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32h755zi-cm7,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32h7b3ai,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32l476vg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32l422cb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32wb15cc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32l072cz,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32l041f6,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32l151cb-a,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f398ve,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f378cc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32g0c1ve,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f217zg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,stm32l552ze,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32wl54jc-cm0p,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32wle5jb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32g474pe,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f107vc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f103re,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f100c4,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32h503rb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32h562ag,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path cyw43/Cargo.toml --target thumbv6m-none-eabi --features ''\
    --- build --release --manifest-path cyw43/Cargo.toml --target thumbv6m-none-eabi --features 'log' \
    --- build --release --manifest-path cyw43/Cargo.toml --target thumbv6m-none-eabi --features 'defmt' \
    --- build --release --manifest-path cyw43/Cargo.toml --target thumbv6m-none-eabi --features 'log,firmware-logs' \
    --- build --release --manifest-path cyw43/Cargo.toml --target thumbv6m-none-eabi --features 'defmt,firmware-logs' \
    --- build --release --manifest-path cyw43-pio/Cargo.toml --target thumbv6m-none-eabi --features '' \
    --- build --release --manifest-path cyw43-pio/Cargo.toml --target thumbv6m-none-eabi --features 'overclock' \
    --- build --release --manifest-path embassy-boot/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840,nightly \
    --- build --release --manifest-path embassy-boot/nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns,nightly \
    --- build --release --manifest-path embassy-boot/rp/Cargo.toml --target thumbv6m-none-eabi --features nightly \
    --- build --release --manifest-path embassy-boot/stm32/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wl55jc-cm4,nightly \
    --- build --release --manifest-path docs/modules/ROOT/examples/basic/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-pac/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-hal/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-irq/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-async/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path examples/nrf52840/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/nrf52840 \
    --- build --release --manifest-path examples/nrf52840-rtic/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/nrf52840-rtic \
    --- build --release --manifest-path examples/nrf5340/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/nrf5340 \
    --- build --release --manifest-path examples/rp/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/rp \
    --- build --release --manifest-path examples/stm32f0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32f0 \
    --- build --release --manifest-path examples/stm32f1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f1 \
    --- build --release --manifest-path examples/stm32f2/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f2 \
    --- build --release --manifest-path examples/stm32f3/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f3 \
    --- build --release --manifest-path examples/stm32f334/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f334 \
    --- build --release --manifest-path examples/stm32f4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f4 \
    --- build --release --manifest-path examples/stm32f7/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f7 \
    --- build --release --manifest-path examples/stm32c0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32c0 \
    --- build --release --manifest-path examples/stm32g0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32g0 \
    --- build --release --manifest-path examples/stm32g4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32g4 \
    --- build --release --manifest-path examples/stm32h5/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h5 \
    --- build --release --manifest-path examples/stm32h7/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h7 \
    --- build --release --manifest-path examples/stm32l0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32l0 \
    --- build --release --manifest-path examples/stm32l1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32l1 \
    --- build --release --manifest-path examples/stm32l4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32l4 \
    --- build --release --manifest-path examples/stm32l5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32l5 \
    --- build --release --manifest-path examples/stm32u5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32u5 \
    --- build --release --manifest-path examples/stm32wb/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wb \
    --- build --release --manifest-path examples/stm32wl/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wl \
    --- build --release --manifest-path examples/boot/application/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840,skip-include --out-dir out/examples/boot/nrf  \
    --- build --release --manifest-path examples/boot/application/nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns,skip-include --out-dir out/examples/boot/nrf \
    --- build --release --manifest-path examples/boot/application/rp/Cargo.toml --target thumbv6m-none-eabi --features skip-include --out-dir out/examples/boot/rp \
    --- build --release --manifest-path examples/boot/application/stm32f3/Cargo.toml --target thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32f3 \
    --- build --release --manifest-path examples/boot/application/stm32f7/Cargo.toml --target thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32f7 \
    --- build --release --manifest-path examples/boot/application/stm32h7/Cargo.toml --target thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32h7 \
    --- build --release --manifest-path examples/boot/application/stm32l0/Cargo.toml --target thumbv6m-none-eabi --features skip-include --out-dir out/examples/boot/stm32l0 \
    --- build --release --manifest-path examples/boot/application/stm32l1/Cargo.toml --target thumbv7m-none-eabi --features skip-include --out-dir out/examples/boot/stm32l1 \
    --- build --release --manifest-path examples/boot/application/stm32l4/Cargo.toml --target thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32l4 \
    --- build --release --manifest-path examples/boot/application/stm32wl/Cargo.toml --target thumbv7em-none-eabihf --features skip-include --out-dir out/examples/boot/stm32wl \
    --- build --release --manifest-path examples/boot/bootloader/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840 \
    --- build --release --manifest-path examples/boot/bootloader/nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns \
    --- build --release --manifest-path examples/boot/bootloader/rp/Cargo.toml --target thumbv6m-none-eabi \
    --- build --release --manifest-path examples/boot/bootloader/stm32/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wl55jc-cm4 \
    --- build --release --manifest-path examples/wasm/Cargo.toml --target wasm32-unknown-unknown --out-dir out/examples/wasm \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32f103c8 --out-dir out/tests/stm32f103c8 \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi --out-dir out/tests/stm32f429zi \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32g491re --out-dir out/tests/stm32g491re \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32g071rb --out-dir out/tests/stm32g071rb \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32c031c6 --out-dir out/tests/stm32c031c6 \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi --out-dir out/tests/stm32h755zi \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32wb55rg --out-dir out/tests/stm32wb55rg \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h563zi --out-dir out/tests/stm32h563zi \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32u585ai --out-dir out/tests/stm32u585ai \
    --- build --release --manifest-path tests/rp/Cargo.toml --target thumbv6m-none-eabi --out-dir out/tests/rpi-pico \
    --- build --release --manifest-path tests/nrf/Cargo.toml --target thumbv7em-none-eabi --out-dir out/tests/nrf52840-dk \
    --- build --release --manifest-path tests/riscv32/Cargo.toml --target riscv32imac-unknown-none-elf \
    $BUILD_EXTRA


if [[ -z "${TELEPROBE_TOKEN-}" ]]; then
    echo No teleprobe token found, skipping running HIL tests
    exit
fi

teleprobe client run -r out/tests
