#!/bin/bash

set -euxo pipefail

cd $(dirname $0)

cargo fmt --all -- --check

# embassy std
(cd embassy-std-examples; cargo build)

# embassy embedded
(cd embassy; cargo build --target thumbv7em-none-eabi)
(cd embassy; cargo build --target thumbv7em-none-eabi --features log)
(cd embassy; cargo build --target thumbv7em-none-eabi --features defmt)
(cd embassy; cargo build --target thumbv6m-none-eabi --features defmt)

# embassy-nrf

(cd embassy-nrf-examples; cargo build --target thumbv7em-none-eabi --bins)

(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52810)
#(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52811)  # nrf52811-hal doesn't exist yet
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52832)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52833)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840)

(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840,log)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840,defmt)

# embassy-stm32f4

(cd embassy-stm32f4-examples; cargo build --target thumbv7em-none-eabi --bins)
(cd embassy-stm32f4; cargo build --target thumbv7em-none-eabi --features stm32f405)
(cd embassy-stm32f4; cargo build --target thumbv7em-none-eabi --features stm32f405,defmt)

# embassy-stm32l0

(cd embassy-stm32l0; cargo build --target thumbv6m-none-eabi --features stm32l0x2)
(cd embassy-stm32l0; cargo build --target thumbv6m-none-eabi --features stm32l0x2,defmt)
