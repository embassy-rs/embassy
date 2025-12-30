#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export PATH=$CARGO_HOME/bin:$PATH

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 786e8e05ef2de5fc1421a4fcd50e79c5dd2d3e4d

cargo embassy-devtool check-crlf
cargo embassy-devtool check-manifest
