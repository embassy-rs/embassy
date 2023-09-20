#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
if [ -f /ci/secrets/teleprobe-token.txt ]; then 
    echo Got teleprobe token!
    export TELEPROBE_HOST=https://teleprobe.embassy.dev
    export TELEPROBE_TOKEN=$(cat /ci/secrets/teleprobe-token.txt)
fi

# needed for "dumb HTTP" transport support
# used when pointing stm32-metapac to a CI-built one.
export CARGO_NET_GIT_FETCH_WITH_CLI=true

hashtime restore /ci/cache/filetime.json || true
hashtime save /ci/cache/filetime.json

./ci.sh
