#!/bin/bash

set -euxo pipefail

cd $(dirname $0)

# embassy-nrf

(cd embassy-nrf-examples; cargo build --target thumbv7em-none-eabi --bins)