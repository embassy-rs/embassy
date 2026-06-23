#!/usr/bin/env bash
# USB text monitor — see src/usb_monitor.rs
#
#   ./usb-monitor.sh text
#
# defmt logs: use probe-rs RTT (same terminal as cargo run):
#   cargo run --bin oxivgl_widget_demo --features oxivgl

set -euo pipefail
cd "$(dirname "$0")"

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

if [[ $# -ge 1 && "$1" == "defmt" ]]; then
    echo "defmt now uses probe-rs RTT — run: cargo run --bin <binary> [--features oxivgl]" >&2
    exit 0
fi

PORT="${SERIAL_PORT:-$(by_id_port 'if00')}"
if [[ -z "${PORT}" ]]; then
    echo "Text port not found." >&2
    list_ports
    exit 1
fi
echo "Text monitor on ${PORT}" >&2
exec cat "${PORT}"
