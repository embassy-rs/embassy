# Examples for RISC-V 64 (in QEMU `virt`)

These examples target a generic RV64IMAC core running on QEMU's `virt`
machine and exercise the `platform-riscv64` thread executor.

Run individual examples with:

```sh
cargo run --bin <crate-name>
```

for example:

```sh
cargo run --bin hello
```

Expected output:

```
Hello from embassy-executor on RISC-V 64!
Hello from a spawned task!
Goodbye.
```

QEMU then exits with status 0 via the semihosting `SYS_EXIT` call.

## Checklist before running examples

Before running these examples, you will need:

* `qemu-system-riscv64` on your `PATH` (typically packaged as `qemu-system-misc`
  on Debian/Ubuntu, `qemu-system-riscv` on Fedora/Arch, or `qemu` from Homebrew).

The runner is configured in `.cargo/config.toml`:

```
qemu-system-riscv64 -machine virt -cpu rv64 -semihosting -nographic -bios none -kernel <elf>
```

`-bios none` is important: it skips OpenSBI so the kernel ELF is loaded
directly at `0x80000000` (the start of RAM on the `virt` machine), which
matches `memory.x`.

## Running manually

If `cargo run` does not pick up the runner for some reason, you can invoke
QEMU directly:

```sh
cargo build --bin hello --release
qemu-system-riscv64 \
    -machine virt -cpu rv64 \
    -semihosting -nographic -bios none \
    -kernel target/riscv64imac-unknown-none-elf/release/hello
```

Press `Ctrl-A X` to terminate QEMU if a program hangs.

## Verified QEMU CPU profiles

The example is verified end-to-end against the following `qemu-system-riscv64
-cpu` profiles. Run `./test-cpus.sh` to reproduce — it builds the binary for
both Rust toolchain targets and asserts that stdout matches exactly and QEMU
exits with status 0 for every supported combination.

| Toolchain target               | CPU profiles                                                      |
|--------------------------------|-------------------------------------------------------------------|
| `riscv64imac-unknown-none-elf` | `rv64`, `sifive-u54`, `sifive-e51`, `thead-c906`, `veyron-v1`, `max` |
| `riscv64gc-unknown-none-elf`   | `rv64`, `sifive-u54`, `thead-c906`, `veyron-v1`, `max`            |

`riscv64gc` requires F+D extensions, so it is not run against `sifive-e51`
(which is RV64IMAC only). That binary will trap during `riscv-rt`'s
floating-point initialisation on a core without F/D — it's an architectural
mismatch, not an executor bug.

The `thead-c906` profile prints a benign QEMU-side warning about disabling
the Zfa extension; the program runs to completion regardless.
