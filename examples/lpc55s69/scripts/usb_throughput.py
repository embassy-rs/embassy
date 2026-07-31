#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pyserial>=3.5"]
# ///
"""Host-side throughput bench for the LPC55 USB CDC-ACM throughput firmware.

Wire protocol (identical on the full-speed and high-speed firmware):

  host -> device: one command byte, then the byte count as little-endian u32.

    b'I' + u32 N   device streams N bytes to the host. Byte k of the stream is
                   (k % 512), i.e. a repeating 0..511 ramp.
    b'O' + u32 N   host sends N payload bytes (same ramp); the device consumes
                   them and replies with exactly 4 bytes, the received count as
                   little-endian u32.

Build the firmware with `--release`. An unoptimized `dev` build of the driver
and the `embassy-usb` class layer is CPU-bound at well under 1 MB/s, roughly
two orders of magnitude below the bus, so a `dev`-profile measurement says
nothing about the hardware:

    cargo run --release --bin usb_hs_throughput   # then --dir in/out here
"""

import argparse
import struct
import sys
import time
from typing import NoReturn

import serial

RAMP_PERIOD = 512
# Byte k of the stream is `(k % 512) as u8`, so the 512-long ramp is 0..255 twice.
RAMP = bytes((k % RAMP_PERIOD) & 0xFF for k in range(RAMP_PERIOD))
CHUNK = 1 << 20
# Pre-tiled so `ramp_slice` is a slice of a constant rather than a fresh
# `RAMP * reps` allocation per chunk: at tens of MB/s that allocation shows up
# in the measurement.
RAMP_TILED = RAMP * (CHUNK // RAMP_PERIOD + 1)


def parse_count(text: str) -> int:
    value = int(text.replace("_", ""), 0)
    if value <= 0:
        raise argparse.ArgumentTypeError("byte count must be positive")
    return value


def ramp_slice(offset: int, length: int) -> bytes:
    """The `length` payload bytes starting at stream offset `offset`."""
    start = offset % RAMP_PERIOD
    return RAMP_TILED[start : start + length]


def fail(message: str) -> NoReturn:
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def run_in(port: serial.Serial, count: int) -> tuple[float, int]:
    """Device -> host. Returns (elapsed seconds, mismatching byte count)."""
    port.write(b"I" + struct.pack("<I", count))
    port.flush()

    start = time.perf_counter()
    received = 0
    mismatches = 0
    while received < count:
        chunk = port.read(min(CHUNK, count - received))
        if not chunk:
            fail(f"timed out after {received} of {count} bytes")
        expected = ramp_slice(received, len(chunk))
        # The whole-buffer compare is a C memcmp; only a failing chunk pays for
        # the per-byte count. Counting unconditionally caps this script at
        # roughly 30 MB/s and silently understates the device.
        if chunk != expected:
            mismatches += sum(1 for a, b in zip(chunk, expected) if a != b)
        received += len(chunk)
    elapsed = time.perf_counter() - start
    return elapsed, mismatches


def run_out(port: serial.Serial, count: int) -> float:
    """Host -> device. Returns elapsed seconds."""
    port.write(b"O" + struct.pack("<I", count))
    port.flush()

    start = time.perf_counter()
    sent = 0
    while sent < count:
        chunk = ramp_slice(sent, min(CHUNK, count - sent))
        written = port.write(chunk)
        if written is None:
            written = len(chunk)
        if written == 0:
            fail(f"write stalled after {sent} of {count} bytes")
        sent += written
    port.flush()

    ack = port.read(4)
    elapsed = time.perf_counter() - start
    if len(ack) != 4:
        fail(f"short acknowledgement: got {len(ack)} of 4 bytes")
    acked = struct.unpack("<I", ack)[0]
    if acked != count:
        fail(f"device acknowledged {acked} bytes, expected {count}")
    return elapsed


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Measure USB CDC-ACM throughput against the LPC55 throughput firmware."
    )
    parser.add_argument("--port", required=True, help="serial device, e.g. /dev/ttyACM0")
    parser.add_argument(
        "--bytes",
        type=parse_count,
        default=4_000_000,
        help="payload size in bytes (underscores allowed, default 4_000_000)",
    )
    parser.add_argument("--dir", required=True, choices=("in", "out"), help="transfer direction")
    parser.add_argument(
        "--timeout", type=float, default=5.0, help="serial read timeout in seconds (default 5)"
    )
    parser.add_argument(
        "--wait-port",
        type=float,
        default=0.0,
        help="seconds to wait for the port to appear (default 0, one attempt)",
    )
    parser.add_argument(
        "--min-rate",
        type=float,
        default=None,
        help="fail if measured throughput is below this many MB/s (default: no minimum)",
    )
    args = parser.parse_args()

    count = args.bytes
    deadline = time.monotonic() + args.wait_port
    while True:
        try:
            port = serial.Serial(args.port, timeout=args.timeout, write_timeout=args.timeout)
            break
        except serial.SerialException as exc:
            # With the default --wait-port of 0 the deadline has already passed, so this is
            # a single attempt followed by the original hard failure.
            if time.monotonic() >= deadline:
                fail(f"cannot open {args.port}: {exc}")
            time.sleep(0.25)
    try:
        port.reset_input_buffer()
        port.reset_output_buffer()
        if args.dir == "in":
            elapsed, mismatches = run_in(port, count)
            print(f"payload mismatches: {mismatches} (0 expected)")
        else:
            elapsed = run_out(port, count)
            mismatches = 0
    except serial.SerialTimeoutException as exc:
        fail(f"serial timeout: {exc}")
    finally:
        port.close()

    mbps_bytes = count / 1e6 / elapsed
    mbits = count * 8 / 1e6 / elapsed
    print(f"direction: {args.dir}  bytes: {count}  elapsed: {elapsed:.3f} s")
    print(f"throughput: {mbps_bytes:.3f} MB/s (MB = 1e6 bytes) = {mbits:.3f} Mbps")

    if args.min_rate is not None and mbps_bytes < args.min_rate:
        print(
            f"error: throughput {mbps_bytes:.3f} MB/s below minimum {args.min_rate} MB/s",
            file=sys.stderr,
        )
        sys.exit(1)
    if mismatches:
        sys.exit(1)


if __name__ == "__main__":
    main()
