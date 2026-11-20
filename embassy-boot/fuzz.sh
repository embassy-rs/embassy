#!/bin/bash
# Runs the power-fail fuzzer for a fixed number of runs. Requires nightly and cargo-fuzz.
set -euo pipefail

cd "$(dirname "$0")/fuzz"

rm -rf corpus/power_fail corpus/power_fail-seeds
cargo +nightly run --bin seeds corpus/power_fail-seeds

# Fixed run count, seed and seed order, so the run is reproducible.
cargo +nightly fuzz run -s none power_fail -- -runs=150000 -seed=1 -seed_inputs=@corpus/power_fail-seeds.list -reload=0 -use_cmp=0 -max_len=128 -dict=power_fail.dict -verbosity=0 -print_final_stats=1
