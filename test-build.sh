#!/bin/bash

set -euxo pipefail

# examples
(cd examples; cargo build --target thumbv7em-none-eabihf --bins)

# embassy
(cd embassy; cargo build --target thumbv7em-none-eabihf)
(cd embassy; cargo build --target thumbv7em-none-eabihf --features log)
(cd embassy; cargo build --target thumbv7em-none-eabihf --features defmt)

# embassy-nrf
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52810)
#(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52811)  # nrf52811-hal doesn't exist yet
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52832)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52833)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52840)

(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52840,log)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52840,defmt)

