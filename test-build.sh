#!/bin/bash

set -euxo pipefail

# build for std
(cd embassy-net; cargo build --features log)

# build for embedded
(cd embassy-net; cargo build --target thumbv7em-none-eabi --features log)
(cd embassy-net; cargo build --target thumbv7em-none-eabi --features defmt)

# build examples
(cd embassy-net-examples; cargo build)
