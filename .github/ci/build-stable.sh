#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target

# needed for "dumb HTTP" transport support
# used when pointing stm32-metapac to a CI-built one.
export CARGO_NET_GIT_FETCH_WITH_CLI=true

hashtime restore /ci/cache/filetime.json || true
hashtime save /ci/cache/filetime.json

sed -i 's/channel.*/channel = "stable"/g' rust-toolchain.toml

./ci_stable.sh
