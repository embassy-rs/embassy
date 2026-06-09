# AGENTS.md

## Cursor Cloud specific instructions

Embassy is a **Rust embedded async framework** monorepo (no root `Cargo.toml` workspace). Each crate under `embassy-*`, `examples/*`, and `tests/*` is an independent Cargo project.

### Toolchain

- Rust **1.92** is pinned in `rust-toolchain.toml` (with cross-compile targets and `rust-src`, `rustfmt`, `llvm-tools`).
- Running any `cargo` command in the repo auto-selects that toolchain via rustup.

### Required dev tools (beyond rustup)

Install once on a fresh VM (system package + cargo binaries):

- `libssl-dev` and `pkg-config` — needed to build `cargo-batch` from source.
- `cargo-embassy-devtool` — pinned in `.github/ci/build.sh`:
  `cargo install --git https://github.com/embassy-rs/cargo-embassy-devtool --locked --rev 1cc6a2c6d2ec06607499df33e147310095b1afd5`
- `cargo-batch` — required by `./ci.sh`:
  `cargo install --git https://github.com/embassy-rs/cargo-batch cargo --bin cargo-batch --locked`

### What runs without hardware

| Task | Command | Notes |
|------|---------|-------|
| Host unit tests | Run `cargo test` lines from `.github/ci/test.sh` directly | **Do not** run `test.sh` as-is in Cloud: it sets `RUSTUP_HOME=/ci/cache/rustup` and will fail with permission errors. |
| Std async demo | `cd examples/std && cargo run --bin tick` | Logs `tick` once per second; no TAP or probe-rs needed. |
| Std networking | `examples/std` `net` / `tcp_accept` / etc. | Requires `sudo sh tap.sh` to create `tap99` (see `examples/std/README.md`). |
| Full compile check | `./ci.sh` or `cargo embassy-devtool check` | Checks all crates/examples; can take several minutes. Skips HIL without `TELEPROBE_TOKEN`. |

### Hardware / CI-only paths

- **probe-rs + MCU**: flash/run board examples under `examples/<board>/`.
- **Teleprobe** (`TELEPROBE_TOKEN`): on-device HIL tests via `ci.sh` after `cargo embassy-devtool build`.
- **QEMU**: `examples/qemu-virt-riscv64` needs `qemu-system-riscv64`.
- **rustfmt CI**: `.github/ci/rustfmt.sh` swaps in `rust-toolchain-nightly.toml` and needs nightly rustfmt with edition 2024 unstable features.

### Lint / format

- Format all Rust: `./fmtall.sh` (uses nightly rustfmt).
- CI rustfmt check: `.github/ci/rustfmt.sh` (requires nightly toolchain file swap).
- Local rustfmt check without mutating `rust-toolchain.toml`: `rustfmt +nightly-2025-12-11 --check --skip-children --unstable-features --edition 2024` on `.rs` files (channel from `rust-toolchain-nightly.toml`).

### cargo-embassy-devtool

- `cargo embassy-devtool check [CRATE]` takes a crate name (e.g. `embassy-std-examples`), not `--manifest-path`.
