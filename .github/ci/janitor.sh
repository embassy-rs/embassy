#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export PATH=$CARGO_HOME/bin:$PATH

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 7d6d61819cb5a54bd6fe7da88359c7949142932a

cargo embassy-devtool check-crlf
cargo embassy-devtool check-manifest
