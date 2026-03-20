#!/bin/sh
set -eu

cp "$1" "$1.elf"
ELF_FILE="$1.elf"
GDBSERVER="/Applications/SEGGER/JLink/JLinkGDBServerCLExe"
GDBSERVER_LOG="$(mktemp /tmp/nrf54lm20_jlink_gdbserver.XXXXXX).log"
GDBSERVER_PORT=$((20000 + ($$ % 10000) * 2))
RTT_PORT=$((GDBSERVER_PORT + 1))

cleanup() {
    if [ -n "${GDBSERVER_PID:-}" ]; then
        kill "$GDBSERVER_PID" 2>/dev/null || true
        wait "$GDBSERVER_PID" 2>/dev/null || true
    fi
    rm -f "$ELF_FILE"
    rm -f "$GDBSERVER_LOG"
}

trap cleanup EXIT INT TERM

JLinkExe <<EOF
Device nrf54lm20a_m33
SelectInterface SWD
Speed 4000
LoadFile ${ELF_FILE}
r
g
q
EOF

for attempt in $(seq 1 8); do
    : >"$GDBSERVER_LOG"
    "$GDBSERVER" \
        -device nrf54lm20a_m33 \
        -if SWD \
        -speed 4000 \
        -nohalt \
        -singlerun \
        -port "$GDBSERVER_PORT" \
        -rtttelnetport "$RTT_PORT" \
        >"$GDBSERVER_LOG" 2>&1 &
    GDBSERVER_PID=$!

    for _ in $(seq 1 100); do
        if ! kill -0 "$GDBSERVER_PID" 2>/dev/null; then
            break
        fi
        if rg -q "Listening on TCP/IP port ${GDBSERVER_PORT}" "$GDBSERVER_LOG"; then
            break
        fi
        sleep 0.1
    done

    if kill -0 "$GDBSERVER_PID" 2>/dev/null && rg -q "Connected to target" "$GDBSERVER_LOG"; then
        break
    fi

    kill "$GDBSERVER_PID" 2>/dev/null || true
    wait "$GDBSERVER_PID" 2>/dev/null || true
    GDBSERVER_PID=
    sleep 0.25
done

if [ -z "${GDBSERVER_PID:-}" ] || ! kill -0 "$GDBSERVER_PID" 2>/dev/null; then
    cat "$GDBSERVER_LOG" >&2
    exit 1
fi

defmt-print -e "$ELF_FILE" tcp --port "$RTT_PORT"
