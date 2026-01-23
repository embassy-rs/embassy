#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export PATH=$CARGO_HOME/bin:$PATH
mv rust-toolchain-nightly.toml rust-toolchain.toml

# needed for "dumb HTTP" transport support
# used when pointing stm32-metapac to a CI-built one.
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Restore lockfiles
if [ -f /ci/cache/lockfiles.tar ]; then
    echo Restoring lockfiles...
    tar xf /ci/cache/lockfiles.tar
fi

hashtime restore /ci/cache/filetime.json || true
hashtime save /ci/cache/filetime.json

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 786e8e05ef2de5fc1421a4fcd50e79c5dd2d3e4d

./ci-nightly.sh

# Save lockfiles
echo Saving lockfiles...
find . -type f -name Cargo.lock -exec tar -cf /ci/cache/lockfiles.tar '{}' \+
