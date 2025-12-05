#!/usr/bin/env bash
set -euo pipefail

ELF="${1:-}"
PROBE_ID="${2:-1366:0101:000600110607}" # default to your J-Link
CHIP="${3:-MCXA276}"
SPEED="${4:-1000}"
PORT="${PROBE_RS_GDB_PORT:-1337}"

if [[ -z "${ELF}" || ! -f "${ELF}" ]]; then
  echo "Usage: $0 <elf> [probe-id] [chip] [speed-khz]" >&2
  exit 1
fi

if ! command -v probe-rs >/dev/null 2>&1; then
  echo "probe-rs not found (cargo install probe-rs --features cli)" >&2
  exit 1
fi
if ! command -v gdb-multiarch >/dev/null 2>&1; then
  echo "gdb-multiarch not found; install it (e.g., sudo apt install gdb-multiarch)." >&2
  exit 1
fi

# Start probe-rs GDB server
SERVER_LOG=$(mktemp)
probe-rs gdb --chip "${CHIP}" --protocol swd --speed "${SPEED}" --non-interactive "${ELF}" --probe "${PROBE_ID}" \
  >"${SERVER_LOG}" 2>&1 &
GDB_SERVER_PID=$!

# Wait for readiness
for _ in {1..50}; do
  if grep -q "Firing up GDB stub" "${SERVER_LOG}"; then break; fi
  if grep -q "Connecting to the chip was unsuccessful" "${SERVER_LOG}"; then
    echo "probe-rs gdb server failed. Log:" >&2
    sed -e 's/^/  /' "${SERVER_LOG}" >&2 || true
    kill "${GDB_SERVER_PID}" 2>/dev/null || true
    exit 1
  fi
  sleep 0.1
done

# GDB script: load, resume, detach
GDB_SCRIPT=$(mktemp)
cat >"${GDB_SCRIPT}" <<EOF
set pagination off
set confirm off
set mem inaccessible-by-default off

target remote :${PORT}
monitor halt
load
set language c
# Start clean from Reset vector in RAM
set {int}0xE000ED08 = 0x20000000
set \$xpsr = 0x01000000
set \$sp = 0x20020000
set \$pc = Reset
monitor resume
detach
quit
EOF

# Run GDB to program and resume target, then exit (probe released)
if ! gdb-multiarch -q -x "${GDB_SCRIPT}" "${ELF}"; then
  echo "GDB failed; server log:" >&2
  sed -e 's/^/  /' "${SERVER_LOG}" >&2 || true
  kill "${GDB_SERVER_PID}" 2>/dev/null || true
  exit 1
fi

# Stop server now that we've detached
kill "${GDB_SERVER_PID}" 2>/dev/null || true
rm -f "${GDB_SCRIPT}" "${SERVER_LOG}" || true

echo "Flashed, resumed, and detached (probe free)."

