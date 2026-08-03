#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pyserial>=3.5"]
# ///
"""Drive both CDC-ACM ports from `usb_dual_pll0`.

The firmware runs both LPC55 USB controllers from PLL0 at 150 MHz. Two host threads transfer data
at the same time, so clock and endpoint-memory faults under concurrent traffic fail the test.

The script uses only CDC nodes and needs no elevation. On Linux the test user must have access to
`/dev/ttyACM*`, normally through the `dialout` group. On Windows, `usbser.sys` binds the ports
automatically.

Exit 0 on success. On failure, print `FAIL: <port label>: <reason>` to stderr and exit 1.
"""

from __future__ import annotations

import argparse
import sys
import threading
import time

import serial
import serial.tools.list_ports

VID = 0xC0DE

# `idProduct` per controller, from `tests/lpc55/src/bin/usb_dual_pll0.rs`. The USB IDs are the
# only port identity both hosts expose: Linux `by-id` symlinks and Windows COM names share
# nothing else.
PORTS = (("HS", 0xCB03), ("FS", 0xCB04))

RAMP_PERIOD = 512
# Byte k of the stream is `(k % 512) as u8` - the same ramp as
# `examples/lpc55s69/scripts/usb_throughput.py` and `host/conformance.py`.
_RAMP = bytes((k % RAMP_PERIOD) & 0xFF for k in range(RAMP_PERIOD))

PAYLOAD_SIZE = 512
ITERATIONS = 8
READ_TIMEOUT = 5.0
BARRIER_TIMEOUT = 5.0
THREAD_TIMEOUT = ITERATIONS * READ_TIMEOUT + 10.0
PAYLOAD_SEED = {"HS": 0, "FS": 128}


def ramp(n: int, offset: int = 0) -> bytes:
    """`n` payload bytes starting at the specified stream offset."""
    repeats = (offset + n) // RAMP_PERIOD + 1
    return (_RAMP * repeats)[offset : offset + n]


def find_ports(wait: float) -> dict[str, str]:
    """Poll the serial ports until both controllers are present or `wait` elapses."""
    deadline = time.monotonic() + wait
    found: dict[str, str] = {}
    while True:
        present = serial.tools.list_ports.comports()
        for label, pid in PORTS:
            if label in found:
                continue
            for info in present:
                if info.vid == VID and info.pid == pid:
                    found[label] = info.device
                    break
        if len(found) == len(PORTS):
            return found
        if time.monotonic() >= deadline:
            missing = ", ".join(
                f"{label} ({VID:04x}:{pid:04x})" for label, pid in PORTS if label not in found
            )
            raise TimeoutError(f"no CDC port for {missing} after {wait:g} s")
        time.sleep(0.2)


def read_exact(port: serial.Serial, count: int) -> bytes:
    """Read `count` bytes, tolerating a CDC echo split across packets."""
    deadline = time.monotonic() + READ_TIMEOUT
    buf = bytearray()
    while len(buf) < count:
        chunk = port.read(count - len(buf))
        if chunk:
            buf += chunk
        elif time.monotonic() >= deadline:
            raise TimeoutError(f"got {len(buf)} of {count} echoed bytes in {READ_TIMEOUT:g} s")
    return bytes(buf)


def echo_port(
    label: str,
    path: str,
    barrier: threading.Barrier,
    results: dict[str, str | None],
) -> None:
    """Run the echo loop on one port, recording its own failure message."""
    synchronized = False
    try:
        with serial.Serial(path, timeout=0.2, write_timeout=READ_TIMEOUT) as port:
            try:
                barrier.wait(timeout=BARRIER_TIMEOUT)
            except threading.BrokenBarrierError as exc:
                raise TimeoutError(f"peer did not reach the barrier in {BARRIER_TIMEOUT:g} s") from exc
            synchronized = True
            for i in range(ITERATIONS):
                payload = ramp(PAYLOAD_SIZE, PAYLOAD_SEED[label] + i)
                written = port.write(payload)
                if written != len(payload):
                    raise TimeoutError(f"iteration {i}: wrote {written} of {len(payload)} bytes")
                echoed = read_exact(port, PAYLOAD_SIZE)
                if echoed != payload:
                    bad = next(k for k in range(PAYLOAD_SIZE) if echoed[k] != payload[k])
                    raise ValueError(
                        f"iteration {i}: byte {bad} is {echoed[bad]:#04x}, expected {payload[bad]:#04x}"
                    )
    except Exception as exc:  # noqa: BLE001 - reported verbatim as the verdict
        if not synchronized:
            barrier.abort()
        results[label] = f"{type(exc).__name__}: {exc}"
        return
    results[label] = None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--wait", type=float, default=15.0, help="seconds to wait for both CDC ports to appear (default 15)"
    )
    args = parser.parse_args()
    try:
        ports = find_ports(args.wait)
    except TimeoutError as exc:
        print(f"FAIL: ports: {exc}", file=sys.stderr)
        return 1
    barrier = threading.Barrier(2)
    results: dict[str, str | None] = {}
    threads = [
        threading.Thread(
            target=echo_port,
            args=(label, ports[label], barrier, results),
            name=label,
            daemon=True,
        )
        for label, _ in PORTS
    ]
    for thread in threads:
        thread.start()
    deadline = time.monotonic() + THREAD_TIMEOUT
    for thread in threads:
        thread.join(max(0.0, deadline - time.monotonic()))
    hung = {thread.name for thread in threads if thread.is_alive()}
    if hung:
        barrier.abort()

    failed = False
    for label, _ in PORTS:
        reason = (
            f"thread did not finish in {THREAD_TIMEOUT:g} s"
            if label in hung
            else results.get(label, "thread produced no result")
        )
        if reason is not None:
            print(f"FAIL: {label}: {reason}", file=sys.stderr)
            failed = True
    if failed:
        return 1

    for label, _ in PORTS:
        print(
            f"{label}: echoed {ITERATIONS} x {PAYLOAD_SIZE} bytes = "
            f"{ITERATIONS * PAYLOAD_SIZE} bytes, exact match"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
