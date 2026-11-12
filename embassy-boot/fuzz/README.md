# Fuzzing the embassy-boot power-fail test

A [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) target for the simulator in
[`tests/power_fail.rs`](../tests/power_fail.rs). The input format is the same as `POWER_FAIL_REPLAY` takes.

Known failure classes are counted, not crashes. Any other failure panics with the full report.

## Run

Requires a nightly toolchain and `cargo install cargo-fuzz`. Run in `embassy-boot/fuzz`:

```sh
# Optional: write a seed corpus from the sweep scenarios.
cargo +nightly run --bin seeds

cargo +nightly fuzz run -s none power_fail -- -max_total_time=300 -max_len=128 -dict=power_fail.dict
```

- `--features flash-erase-zero` fuzzes the build for flash that erases to 0x00.
- `POWER_FAIL_KNOWN_CRASH=all`, or a list of `class` or `class:Model`, makes known classes crash.

`../fuzz.sh` runs a fixed, reproducible fuzz session.

## Crashes

Replay a crash with its full trace, in `embassy-boot`:

```sh
POWER_FAIL_REPLAY=fuzz/artifacts/power_fail/crash-<hash> cargo test --test power_fail replay -- --nocapture
```

To keep it as a regression test, add a test to `tests/power_fail.rs` that calls `replay_hex("<hex>")` with the
input hex from the report.
