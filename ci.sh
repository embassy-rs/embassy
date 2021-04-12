#!/bin/bash

set -euxo pipefail

# build for std
(cd embassy-net; cargo build --no-default-features --features log,medium-ethernet,tcp)
(cd embassy-net; cargo build --no-default-features --features log,medium-ethernet,tcp,dhcpv4)
(cd embassy-net; cargo build --no-default-features --features log,medium-ip,tcp)
(cd embassy-net; cargo build --no-default-features --features log,medium-ethernet,medium-ip,tcp,dhcpv4)

# build for embedded
(cd embassy-net; cargo build --target thumbv7em-none-eabi --no-default-features --features log,medium-ethernet,medium-ip,tcp,dhcpv4)
(cd embassy-net; cargo build --target thumbv7em-none-eabi --no-default-features --features defmt,smoltcp/defmt,medium-ethernet,medium-ip,tcp,dhcpv4)

# build examples
(cd embassy-net-examples; cargo build)
