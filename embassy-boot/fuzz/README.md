# Fuzzing the embassy-boot power-fail test

A coverage-guided fuzzer (libFuzzer, through `cargo-fuzz`) on the simulator of [`tests/power_fail.rs`](../tests/power_fail.rs): the real `BootLoader` and `BlockingFirmwareUpdater` on a NOR flash model of one MCU family, power cut at any flash operation, invariants checked after every boot.

This crate has its own `[workspace]`, is never built by embassy CI and gives `embassy-boot` no dependency. It includes `tests/power_fail/{flash,scenario,zoo}.rs` by `#[path]`, so an input is the same byte string as `POWER_FAIL_REPLAY` takes (byte 0 picks the model out of the 19, the next four the layout, the rest are steps).

Known classes (the list at the top of `tests/power_fail.rs`, and `zoo::expect`) are counted and printed with an example, not crashes. Anything else panics with the full report, which libFuzzer saves as a crash.

## Install

A nightly toolchain (`-Zsanitizer`) and `cargo-fuzz`. The repo's `rust-toolchain.toml` pins stable, so `fuzz/rust-toolchain.toml` picks nightly for commands run in `embassy-boot/fuzz`.

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Run

All commands run in `embassy-boot/fuzz`.

Write the seed corpus (the sweep scenarios of `tests/power_fail.rs` on every model, plain and with a power cut at a few operations, into `corpus/power_fail/`; about 2800 small files):

```sh
cargo run --bin seeds
```

Fuzz for five minutes (`-s none`: no AddressSanitizer, the code has no `unsafe` and it runs about 1.5 times faster):

```sh
cargo fuzz run -s none power_fail -- -max_total_time=300 -max_len=128 -dict=power_fail.dict
```

A run on main prints only lines like `known class ecc-overwrite on H7: 64 runs (of 1902); example 0d01...` (the first example of each class and then every power of two). It stops on the first input that breaks an invariant, no known class explains, or that is of a class asked for below.

- `-jobs=4 -workers=4` runs four processes on the same corpus; `-max_len=128` (in the command above) keeps inputs short for speed, the useful ones are under 40 bytes; it is a libFuzzer option, nothing is filtered in `check`.
- `power_fail.dict` has the step bytes (download, mark_updated, boot, mark_booted, mark_dfu, cut) and a few sequences.
- Known classes as crashes: `POWER_FAIL_KNOWN_CRASH=all` (every class), or a list of `class` or `class:Model`, e.g. `POWER_FAIL_KNOWN_CRASH=nwrite-exceeded:Nrf52,torn-magic-reads-swap:Qspi`. Use it to find a class again, or to check how fast the fuzzer reaches it.
- Flash that erases to 0x00: `cargo fuzz run -s none --features flash-erase-zero power_fail` (runs only the L0, L0c3 and L1 models, like the test's feature of the same name).

## A crash

libFuzzer prints the report (model, layout, steps, trace of every flash operation, the broken invariant) and writes the input to `artifacts/power_fail/crash-<hash>`.

Make it small:

```sh
cargo fuzz tmin -s none power_fail artifacts/power_fail/crash-<hash>
```

Turn a file into the hex string the test takes (in `embassy-boot/fuzz`, where `artifacts/` is):

```sh
od -An -tx1 -v artifacts/power_fail/minimized-from-<hash> | tr -d ' \n'
```

Replay it with its full trace, or keep it as a regression test (both in `embassy-boot`):

```sh
POWER_FAIL_REPLAY=<hex> cargo test --test power_fail replay -- --nocapture
```

```rust
#[test]
fn found_by_the_fuzzer() {
    replay_hex("<hex>");
}
```

`replay_hex` is in `tests/power_fail.rs`: the test fails with the report while the bug is there. A known class that is not listed yet belongs in `Failure::known` in `tests/power_fail/scenario.rs` (and the list at the top of `tests/power_fail.rs`), with a test like `nrf52_two_cuts_exceed_n_write`.

## Why not AFL++

`cargo-afl` was tried: the same `check` function is a one-line target. It does not build with LLVM 23 (tried on Arch Linux): AFL++ 4.40c's LLVM mode stops at "llvm_mode only supports llvm ... your version is too new". It is left out until it builds and has been run.
