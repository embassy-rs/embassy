#!/bin/bash

set -euo pipefail

export CARGO_TARGET_DIR=$PWD/target_ci
export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace

find . -name '*.rs' -not -path '*target*' -not -path '*stm32-metapac-gen/out/*' -not -path '*stm32-metapac/src/*' | xargs rustfmt --check  --skip-children --unstable-features --edition 2018

# Generate stm32-metapac
if [ ! -d "stm32-metapac-backup" ]
then
    cp -r stm32-metapac stm32-metapac-backup
fi

rm -rf stm32-metapac
cp -r stm32-metapac-backup stm32-metapac

# for some reason Cargo stomps the cache if we don't specify --target.
# This happens with vanilla Cargo, not just cargo-batch. Bug?
(cd stm32-metapac-gen; cargo run --release --target x86_64-unknown-linux-gnu)
rm -rf stm32-metapac
mv stm32-metapac-gen/out stm32-metapac

cargo batch  \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi --features nightly \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi --features nightly,log,executor-agnostic \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv7em-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy/Cargo.toml --target thumbv6m-none-eabi --features nightly,defmt \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52805,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52810,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52811,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52820,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52832,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52833,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf9160-s,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf9160-ns,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-app-s,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-app-ns,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,nrf5340-net,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,log,gpiote,time-driver-rtc1 \
    --- build --release --manifest-path embassy-nrf/Cargo.toml --target thumbv7em-none-eabi --features nightly,nrf52840,defmt,gpiote,time-driver-rtc1,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f410tb,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f411ce,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32f429zi,log,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32h755zi-cm7,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32h7b3ai,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32l476vg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32wb15cc,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32l072cz,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32l041f6,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32l151cb-a,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f398ve,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32g0c1ve,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7m-none-eabi --features nightly,stm32f217zg,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv8m.main-none-eabihf --features nightly,stm32l552ze,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv6m-none-eabi --features nightly,stm32wl54jc-cm0p,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-stm32/Cargo.toml --target thumbv7em-none-eabi --features nightly,stm32wle5ub,defmt,exti,time-driver-any,unstable-traits \
    --- build --release --manifest-path embassy-boot/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840 \
    --- build --release --manifest-path embassy-boot/stm32/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wl55jc-cm4 \
    --- build --release --manifest-path docs/modules/ROOT/examples/basic/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-pac/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-hal/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-irq/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path docs/modules/ROOT/examples/layer-by-layer/blinky-async/Cargo.toml --target thumbv7em-none-eabi \
    --- build --release --manifest-path examples/std/Cargo.toml --target x86_64-unknown-linux-gnu --out-dir out/examples/std \
    --- build --release --manifest-path examples/nrf/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/nrf \
    --- build --release --manifest-path examples/rp/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/rp \
    --- build --release --manifest-path examples/stm32f0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32f0 \
    --- build --release --manifest-path examples/stm32f1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f1 \
    --- build --release --manifest-path examples/stm32f2/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f2 \
    --- build --release --manifest-path examples/stm32f3/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f3 \
    --- build --release --manifest-path examples/stm32f4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f4 \
    --- build --release --manifest-path examples/stm32f7/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32f7 \
    --- build --release --manifest-path examples/stm32g0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32g0 \
    --- build --release --manifest-path examples/stm32g4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32g4 \
    --- build --release --manifest-path examples/stm32h7/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h7 \
    --- build --release --manifest-path examples/stm32l0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32l0 \
    --- build --release --manifest-path examples/stm32l1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32l1 \
    --- build --release --manifest-path examples/stm32l4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32l4 \
    --- build --release --manifest-path examples/stm32u5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32u5 \
    --- build --release --manifest-path examples/stm32wb/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wb \
    --- build --release --manifest-path examples/stm32wl/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/stm32wl \
    --- build --release --manifest-path examples/boot/nrf/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/boot/nrf --bin b \
    --- build --release --manifest-path examples/boot/stm32l0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/boot/stm32l0 --bin b \
    --- build --release --manifest-path examples/boot/stm32l1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/boot/stm32l1 --bin b \
    --- build --release --manifest-path examples/boot/stm32l4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/boot/stm32l4 --bin b \
    --- build --release --manifest-path examples/boot/stm32wl/Cargo.toml --target thumbv7em-none-eabihf --out-dir out/examples/boot/stm32wl --bin b \
    --- build --release --manifest-path examples/wasm/Cargo.toml --target wasm32-unknown-unknown --out-dir out/examples/wasm \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7m-none-eabi --features stm32f103c8 --out-dir out/tests/bluepill-stm32f103c8 \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32f429zi --out-dir out/tests/nucleo-stm32f429zi \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32g491re --out-dir out/tests/nucleo-stm32g491re \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv6m-none-eabi --features stm32g071rb --out-dir out/tests/nucleo-stm32g071rb \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32h755zi --out-dir out/tests/nucleo-stm32h755zi \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32wb55rg --out-dir out/tests/nucleo-stm32wb55rg \
    --- build --release --manifest-path tests/stm32/Cargo.toml --target thumbv7em-none-eabi --features stm32u585ai --out-dir out/tests/iot-stm32u585ai \


function run_elf {
    echo Running target=$1 elf=$2
    STATUSCODE=$(
        curl \
            -sS \
            --output /dev/stderr \
            --write-out "%{http_code}" \
            -H "Authorization: Bearer $TELEPROBE_TOKEN" \
            https://teleprobe.embassy.dev/targets/$1/run --data-binary @$2
    )
    echo
    echo HTTP Status code: $STATUSCODE
    test "$STATUSCODE" -eq 200
}

if [[ -z "${TELEPROBE_TOKEN-}" ]]; then
    if [[ -z "${ACTIONS_ID_TOKEN_REQUEST_TOKEN-}" ]]; then
        echo No teleprobe token found, skipping running HIL tests
        exit
    fi

    export TELEPROBE_TOKEN=$(curl -sS -H "Authorization: Bearer $ACTIONS_ID_TOKEN_REQUEST_TOKEN" "$ACTIONS_ID_TOKEN_REQUEST_URL" | jq -r '.value')
fi

for board in $(ls out/tests); do 
    echo Running tests for board: $board
    for elf in $(ls out/tests/$board); do 
        run_elf $board out/tests/$board/$elf
    done
done
