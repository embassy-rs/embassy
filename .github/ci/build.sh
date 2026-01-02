#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export PATH=$CARGO_HOME/bin:$PATH
if [ -f /ci/secrets/teleprobe-token.txt ]; then
    echo Got teleprobe token!
    export TELEPROBE_HOST=https://teleprobe.embassy.dev
    export TELEPROBE_TOKEN=$(cat /ci/secrets/teleprobe-token.txt)
    export TELEPROBE_CACHE=/ci/cache/teleprobe_cache.json
fi

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

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 3ca80f7065acbe0b69b7da463fab60e744f9de79

./ci.sh

# Save lockfiles
echo Saving lockfiles...
find . -type f -name Cargo.lock -exec tar -cf /ci/cache/lockfiles.tar '{}' \+
