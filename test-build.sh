#!/bin/bash

set -euxo pipefail

# examples
(cd examples; cargo build --target thumbv7em-none-eabi --bins)

# embassy std
(cd embassy; cargo build --features log,std)

# embassy embedded
(cd embassy; cargo build --target thumbv7em-none-eabi)
(cd embassy; cargo build --target thumbv7em-none-eabi --features log)
(cd embassy; cargo build --target thumbv7em-none-eabi --features defmt)

# embassy-nrf
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52810)
#(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52811)  # nrf52811-hal doesn't exist yet
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52832)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52833)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840)

(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840,log)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52840,defmt)

