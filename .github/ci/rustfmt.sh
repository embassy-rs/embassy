#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
mv rust-toolchain-nightly.toml rust-toolchain.toml

find . -name '*.rs' -not -path '*target*' | xargs rustfmt --check  --skip-children --unstable-features --edition 2021
