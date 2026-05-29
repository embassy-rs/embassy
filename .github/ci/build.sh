#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

# === OOMinator (TEMPORARY) ===========================================
# Deliberately exhausts memory to verify the kernel/systemd-oomd OOM killer
# now picks this job cgroup. Expect the job to be OOM-killed. REMOVE ME.
echo "OOMinator: allocating memory until OOM-killed..."
oominator_blob=""
oominator_chunk=$(head -c 10000000 /dev/zero | tr '\0' 'x')  # ~10MB
while true; do
    oominator_blob="${oominator_blob}${oominator_chunk}"
    echo "OOMinator: ~$(( ${#oominator_blob} / 1000000 )) MB allocated"
done
# === end OOMinator ===================================================

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

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 7d6d61819cb5a54bd6fe7da88359c7949142932a

./ci.sh

# Save lockfiles
echo Saving lockfiles...
find . -type f -name Cargo.lock -exec tar -cf /ci/cache/lockfiles.tar '{}' \+
