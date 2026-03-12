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

cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 8f4cfa11324c582467c2aab161ef963ff7a2b884
cargo install --git https://github.com/embassy-rs/docserver --locked --rev 09805a79beef037d283192146e2b546cb1c0e931

cargo embassy-devtool doc -o webroot

export KUBECONFIG=/ci/secrets/kubeconfig.yml
POD=$(kubectl get po -l app=docserver -o jsonpath={.items[0].metadata.name})
kubectl cp webroot/crates $POD:/data
kubectl cp webroot/static $POD:/data
