#!/usr/bin/env bash
# USB monitor — see src/usb_monitor.rs
#
#   ./usb-monitor.sh text
#   ./usb-monitor.sh defmt [binary]     default: oxivgl_touch_can
#   ./usb-monitor.sh can_raw            shorthand for defmt + can_raw ELF

set -euo pipefail
cd "$(dirname "$0")"

PROFILE="${PROFILE:-debug}"
FEATURES="${FEATURES:-oxivgl}"

list_ports() {
    echo "USB serial ports for this board:" >&2
    ls -1 /dev/serial/by-id/*rp2350-lcd7* 2>/dev/null || ls -1 /dev/ttyACM* 2>/dev/null || true
}

by_id_port() {
    local if_suffix="$1"
    local match
    match="$(ls /dev/serial/by-id/*rp2350-lcd7*${if_suffix}* 2>/dev/null | head -1 || true)"
    if [[ -n "${match}" && -e "${match}" ]]; then
        echo "${match}"
    fi
}

# Text = if00; defmt = if01 or if02 (Linux numbering varies).
defmt_port() {
    local p
    for suffix in if01 if02 if03; do
        p="$(by_id_port "${suffix}")"
        if [[ -n "${p}" ]]; then
            echo "${p}"
            return
        fi
    done
    for p in /dev/serial/by-id/*rp2350-lcd7*; do
        [[ -e "${p}" ]] || continue
        [[ "${p}" == *if00* ]] && continue
        echo "${p}"
        return
    done
}

# Parse args
MODE="defmt"
BIN="oxivgl_touch_can"
if [[ $# -ge 1 ]]; then
    case "$1" in
        text|defmt)
            MODE="$1"
            BIN="${2:-oxivgl_touch_can}"
            ;;
        *.rs|can_raw|gt911_touch|oxivgl_*)
            BIN="${1%.rs}"
            ;;
        *)
            echo "Unknown: $1" >&2
            list_ports
            echo "Usage: $0 [text|defmt] [binary]" >&2
            exit 1
    esac
fi

if [[ "${MODE}" == text ]]; then
    PORT="${SERIAL_PORT:-$(by_id_port 'if00')}"
    if [[ -z "${PORT}" ]]; then
        echo "Text port (if00) not found." >&2
        list_ports
        exit 1
    fi
    echo "Text monitor on ${PORT}" >&2
    exec cat "${PORT}"
fi

if [[ "${BIN}" == oxivgl_* ]]; then
    cargo build --bin "${BIN}" --features "${FEATURES}" -q
else
    cargo build --bin "${BIN}" -q
fi
ELF="target/thumbv8m.main-none-eabihf/${PROFILE}/${BIN}"

PORT="${SERIAL_PORT:-$(defmt_port)}"
if [[ -z "${PORT}" ]]; then
    echo "defmt port not found (expected if01/if02, not if00)." >&2
    list_ports
    exit 1
fi

if ! command -v defmt-print >/dev/null; then
    echo "defmt-print not found — cargo install defmt-print --locked" >&2
    echo "Or use plain text: ./usb-monitor.sh text" >&2
    exit 1
fi

echo "defmt: ${ELF}" >&2
echo "port:  ${PORT}" >&2
echo "note: open this before flash, or wait ~5s for periodic CAN TX logs" >&2
list_ports >&2
exec defmt-print -e "${ELF}" serial --path "${PORT}" --dtr -v
