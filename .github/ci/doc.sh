#!/bin/bash
## on push branch=main
## priority -100
## dedup dequeue
## cooldown 15m

set -euxo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
export CARGO_TARGET_DIR=/ci/cache/target
export PATH=$CARGO_HOME/bin:$PATH
mv rust-toolchain-nightly.toml rust-toolchain.toml

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev d015fd5e972a3e550ebef0da6748099b88a93ba6

cargo embassy-devtool doc -o webroot

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl -n embassy get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp webroot/crates $POD:/data
kubectl cp webroot/static $POD:/data
