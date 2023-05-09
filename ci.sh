#!/bin/bash

set -euxo pipefail

export DEFMT_LOG=trace

# build examples
#==================

(cd examples; cargo build --bin multisocket --release)
(cd examples; cargo build --bin tcp-client --release)
(cd examples; cargo build --bin tcp-server --release)
(cd examples; cargo build --bin tcp-udp --release)

# build lib
#============

cargo build --target thumbv6m-none-eabi
