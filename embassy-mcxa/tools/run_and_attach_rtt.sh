#!/usr/bin/env bash
set -euo pipefail

ELF="${1:-target/thumbv8m.main-none-eabihf/debug/examples/hello}"
PROBE_ID="${2:-1fc9:0143:H3AYDQVQMTROB}"
CHIP="${3:-MCXA276}"
SPEED="${4:-1000}"

# 1) Flash & run using the existing run.sh (probe is in use only during this step)
./run.sh "$ELF"

# 2) Give target a short moment to boot and set up RTT CB in RAM
sleep 0.5

# 3) Attach RTT/defmt using probe-rs (no flashing)
exec probe-rs attach \
  --chip "$CHIP" \
  --probe "$PROBE_ID" \
  --protocol swd \
  --speed "$SPEED" \
  "$ELF" \
  --rtt-scan-memory \
  --log-format oneline

