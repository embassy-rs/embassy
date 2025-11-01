#!/usr/bin/env bash
set -euo pipefail

ELF="${1:-}"
if [[ -z "${ELF}" ]]; then
  echo "Usage: $0 <elf_file>"
  exit 1
fi
if [[ ! -f "${ELF}" ]]; then
  echo "ELF not found: ${ELF}"
  exit 1
fi

# Configurable via env
CHIP="${CHIP:-MCXA276}"
SPEED="${PROBE_SPEED:-1000}"   # kHz
# Default to J-Link if PROBE not provided
PROBE_OPT=(--probe "${PROBE:-1366:0101:000600110607}")
PORT="${PROBE_RS_GDB_PORT:-1337}"

cleanup() {
  if [[ -n "${GDB_SERVER_PID:-}" ]]; then kill "${GDB_SERVER_PID}" 2>/dev/null || true; fi
  [[ -n "${GDB_SCRIPT:-}" ]] && rm -f "${GDB_SCRIPT}" || true
  [[ -n "${SERVER_LOG:-}" ]] && rm -f "${SERVER_LOG}" || true
}
trap cleanup EXIT

if ! command -v probe-rs >/dev/null 2>&1; then
  echo "probe-rs not found (cargo install probe-rs --features cli)"
  exit 1
fi
if ! command -v gdb-multiarch >/dev/null 2>&1; then
  echo "gdb-multiarch not found; install it (e.g., sudo apt install gdb-multiarch)."
  exit 1
fi

# Start probe-rs GDB server and capture its output to a log (do not hide errors)
SERVER_LOG=$(mktemp)
set +e
probe-rs gdb --chip "${CHIP}" --protocol swd --speed "${SPEED}" --non-interactive "${ELF}" "${PROBE_OPT[@]}" \
  >"${SERVER_LOG}" 2>&1 &
GDB_SERVER_PID=$!
set -e

# Wait for server readiness without touching the TCP port to avoid corrupting the GDB protocol
ready=""
for _ in {1..50}; do
  if grep -q "Firing up GDB stub" "${SERVER_LOG}"; then ready=1; break; fi
  if grep -q "Connecting to the chip was unsuccessful" "${SERVER_LOG}"; then
    echo "probe-rs gdb server failed to connect to target. Log:" >&2
    echo "----- probe-rs gdb log -----" >&2
    sed -e 's/^/  /' "${SERVER_LOG}" >&2 || true
    exit 1
  fi
  sleep 0.1
done
if [[ -z "${ready}" ]]; then
  echo "probe-rs gdb server did not report readiness. Log:" >&2
  echo "----- probe-rs gdb log -----" >&2
  sed -e 's/^/  /' "${SERVER_LOG}" >&2 || true
  exit 1
fi

# GDB script: load to RAM and run, no reset
GDB_SCRIPT=$(mktemp)
cat >"${GDB_SCRIPT}" <<EOF
set pagination off
set confirm off
set mem inaccessible-by-default off

# Connect and load without reset
target remote :${PORT}
monitor halt
load
# Set VTOR to point to our RAM vector table at 0x20000000
# This ensures the CPU uses the correct initial SP and Reset vector
set *0xE000ED08 = 0x20000000
# Now read SP and PC from our vector table and set them
set \$sp = *(unsigned int*)0x20000000
set \$pc = *(unsigned int*)0x20000004
# Run target (blocks here until you Ctrl+C, like before)
continue
EOF

# Run gdb against the server
if ! gdb-multiarch -q -batch -x "${GDB_SCRIPT}" "${ELF}"; then
  echo "GDB failed to load/run. probe-rs gdb server log:" >&2
  echo "----- probe-rs gdb log -----" >&2
  sed -e 's/^/  /' "${SERVER_LOG}" >&2 || true
  exit 1
fi

echo "Program loaded and started (no reset)"
