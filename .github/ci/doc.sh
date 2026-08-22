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

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev f8a8cce4092ef2566fbae04f088daa70c9e1fe93
cargo install --git https://github.com/embassy-rs/docserver --locked --rev f024dc8ca0dceb7f1211d2afa957c5cc80909682

mv rust-toolchain-nightly.toml rust-toolchain.toml

cargo embassy-devtool doc -o webroot --monitor

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp webroot/crates $POD:/data
kubectl cp webroot/static $POD:/data
