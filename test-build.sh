#!/bin/bash

set -euxo pipefail

# examples
(cd examples; cargo build --target thumbv7em-none-eabihf --bins)

# embassy
(cd embassy; cargo build --target thumbv7em-none-eabihf)

# embassy-nrf
(cd embassy-nrf; cargo build --target thumbv7em-none-eabi --features 52810)
#(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52811)  # nrf52811-hal doesn't exist yet
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52832)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52833)
(cd embassy-nrf; cargo build --target thumbv7em-none-eabihf --features 52840)

