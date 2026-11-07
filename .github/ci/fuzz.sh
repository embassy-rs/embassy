#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

export RUSTUP_HOME=/ci/cache/rustup
export CARGO_HOME=/ci/cache/cargo
# Kept apart from the other jobs' target dir: this one builds with nightly.
export CARGO_TARGET_DIR=/ci/cache/target-fuzz
export PATH=$CARGO_HOME/bin:$PATH

cd embassy-boot/fuzz
cp ../../rust-toolchain-nightly.toml rust-toolchain.toml

cargo install cargo-fuzz --locked --version 0.13.2

# Seed corpus: the sweep scenarios of tests/power_fail.rs on every model, in a list that fixes their order.
rm -rf corpus/power_fail corpus/power_fail-seeds
cargo run --bin seeds corpus/power_fail-seeds

# The same run every time: a fixed number of runs (150000 took about 45 s on a Ryzen AI 9 HX 370) and a fixed
# seed; -reload=0 and -use_cmp=0 take out the reload timer and compared values that differ between processes (addresses). Known
# classes are counted; any other failure fails the job, and its report has the input hex for POWER_FAIL_REPLAY.
cargo fuzz run -s none power_fail -- -runs=150000 -seed=1 -seed_inputs=@corpus/power_fail-seeds.list -reload=0 -use_cmp=0 -max_len=128 -dict=power_fail.dict -verbosity=0 -print_final_stats=1
