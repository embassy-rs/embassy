#!/usr/bin/env bash
# Build the hello example for both RV64 toolchain targets and run each binary
# under qemu-system-riscv64 against every compatible CPU profile QEMU exposes,
# verifying that stdout exactly matches the expected semihosting output and
# that QEMU exits cleanly via the program's `process::exit(0)`.
#
# Run from anywhere; the script cd's into its own directory.

set -euo pipefail

cd -- "$(dirname -- "$0")"

if ! command -v qemu-system-riscv64 >/dev/null 2>&1; then
    echo "qemu-system-riscv64 not found on PATH; install it (e.g. apt install qemu-system-misc) and re-run." >&2
    exit 2
fi

EXPECTED=$'Hello from embassy-executor on RISC-V 64!\nHello from a spawned task!\nGoodbye.'

# Targets and the CPUs each one is compatible with.
#
# `riscv64imac` is RV64IMAC and runs on every CPU QEMU emulates, including
# embedded cores like sifive-e51 that have no F/D extensions.
#
# `riscv64gc` adds F+D, so it cannot run on cores without floating point;
# sifive-e51 is RV64IMAC and is excluded for that reason.
IMAC_CPUS=(rv64 sifive-u54 sifive-e51 thead-c906 veyron-v1 max)
GC_CPUS=(rv64 sifive-u54              thead-c906 veyron-v1 max)

build_target() {
    local target="$1"
    echo "== building hello for $target =="
    cargo build --release --target "$target" --bin hello >/dev/null
}

run_one() {
    local target="$1" cpu="$2" bin="$3"
    local out code
    set +e
    # -display/-serial/-monitor none keeps semihosting independent of stdio;
    # plain -nographic muxes serial+monitor onto stdio and hangs under a TTY.
    out=$(timeout 10 qemu-system-riscv64 -machine virt -cpu "$cpu" \
            -semihosting -display none -serial none -monitor none \
            -bios none -kernel "$bin" 2>/dev/null)
    code=$?
    set -e
    if [ "$code" = "0" ] && [ "$out" = "$EXPECTED" ]; then
        printf '  %-12s  cpu=%-12s  PASS\n' "$target" "$cpu"
        return 0
    else
        printf '  %-12s  cpu=%-12s  FAIL  (exit %s)\n' "$target" "$cpu" "$code"
        printf '    --- got ---\n%s\n    -----------\n' "$out"
        return 1
    fi
}

build_target riscv64imac-unknown-none-elf
build_target riscv64gc-unknown-none-elf

IMAC_BIN=target/riscv64imac-unknown-none-elf/release/hello
GC_BIN=target/riscv64gc-unknown-none-elf/release/hello

echo
echo "== running matrix =="

pass=0; fail=0
for cpu in "${IMAC_CPUS[@]}"; do
    if run_one riscv64imac "$cpu" "$IMAC_BIN"; then pass=$((pass+1)); else fail=$((fail+1)); fi
done
for cpu in "${GC_CPUS[@]}"; do
    if run_one riscv64gc   "$cpu" "$GC_BIN";   then pass=$((pass+1)); else fail=$((fail+1)); fi
done

echo
echo "Total: $pass pass / $fail fail"
[ "$fail" = "0" ]
