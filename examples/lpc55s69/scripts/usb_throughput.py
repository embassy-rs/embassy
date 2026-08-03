#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "pyserial>=3.5",
#     "pyusb>=1.3; sys_platform == 'win32'",
#     "libusb-package>=1.0.26; sys_platform == 'win32'",
# ]
# ///
"""Measure LPC55 USB CDC-ACM throughput.

The full-speed and high-speed firmware use the same wire protocol:

    host -> device: one command byte, then a little-endian u32 byte count

    b'I' + u32 N   The device sends N bytes. Byte k is `(k % 512) & 0xff`.
    b'O' + u32 N   The host sends the same ramp. The device returns the received
                   byte count as exactly four little-endian bytes.

Use a release build:

    cargo run --release --bin usb_hs_throughput   # then use --dir in/out here

An unoptimized `dev` build makes the driver and `embassy-usb` class layer CPU-bound below
1 MB/s. This is about two orders of magnitude below the bus rate.

On Windows the endpoints are driven through libusb rather than the CDC driver; see
`LibusbPort` for the measurements behind that.
"""

import argparse
import re
import struct
import sys
import time
from typing import NoReturn

import serial
import serial.tools.list_ports

# A `--port` value of this shape selects the CDC node by USB id instead of naming a device: the
# device names differ per host (`/dev/ttyACM0`, `COM7`) but the ids never do.
USB_ID = re.compile(r"^([0-9a-fA-F]{4}):([0-9a-fA-F]{4})$")

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
    if value > 0xFFFF_FFFF:
        raise argparse.ArgumentTypeError("byte count must fit in u32")
    return value


def ramp_slice(offset: int, length: int) -> bytes:
    """The `length` payload bytes starting at stream offset `offset`."""
    start = offset % RAMP_PERIOD
    return RAMP_TILED[start : start + length]


def fail(message: str) -> NoReturn:
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def resolve_port(spec: str) -> str | None:
    """The device name for `spec`, or None while a `vid:pid` selector has no match."""
    ids = USB_ID.match(spec)
    if ids is None:
        return spec
    vid, pid = int(ids.group(1), 16), int(ids.group(2), 16)
    for info in serial.tools.list_ports.comports():
        if info.vid == vid and info.pid == pid:
            return info.device
    return None


if sys.platform == "win32":
    import libusb_package
    import usb.core
    import usb.util

    # pyusb maps LIBUSB_ERROR_TIMEOUT onto this, whatever the platform.
    _ERRNO_TIMEOUT = 110
    # One bulk request per call; the callers all loop, so a short return is fine.
    _MAX_REQUEST = 1 << 16

    class LibusbPort:
        """The peer's CDC data interface, driven through libusb instead of `usbser.sys`.

        Windows' CDC driver cannot carry this benchmark. It discards stream data that
        arrives while no read request is outstanding, which at the high-speed rate costs
        whole 512-byte packets - 512 to 4096 bytes per 4 MB, in roughly half of all
        transfers - through pyserial, through blocking reads on a dedicated thread, and at
        every read and receive-buffer size tried. Keeping several overlapped requests queued
        does stop the loss, but then the same transfer runs at 0.78 MB/s instead of 38.

        Reading the endpoints directly avoids all of it: 25 consecutive 4 MB transfers with
        no loss, at 45 MB/s rather than 38, which also matches the rate Linux measures. It
        is the device driver under test here, not the host's CDC driver.

        The firmware's MS OS 2.0 descriptors are what let Windows bind WinUSB to the
        function. Linux ignores them and still binds `cdc_acm`, so pyserial drives this test
        there and the port stays an ordinary CDC node.
        """

        def __init__(self, spec: str, timeout: float) -> None:
            ids = USB_ID.match(spec)
            if ids is None:
                raise serial.SerialException(
                    f"--port must be a vid:pid selector on Windows, not {spec!r}: the throughput "
                    "firmware binds WinUSB and so has no COM port"
                )
            vid, pid = int(ids.group(1), 16), int(ids.group(2), 16)
            self.name = f"{vid:04x}:{pid:04x}"
            self._timeout_ms = max(1, int(timeout * 1000))
            device = usb.core.find(
                idVendor=vid, idProduct=pid, backend=libusb_package.get_libusb1_backend()
            )
            if device is None:
                raise serial.SerialException(f"no USB device {self.name}")
            try:
                device.set_configuration()
                # The data interface is the one carrying the two bulk endpoints; the
                # notification interface has a single interrupt IN.
                interface = next(i for i in device.get_active_configuration() if i.bNumEndpoints == 2)
                usb.util.claim_interface(device, interface.bInterfaceNumber)
            except (usb.core.USBError, NotImplementedError, StopIteration) as exc:
                usb.util.dispose_resources(device)
                raise serial.SerialException(f"cannot claim {self.name}: {exc}") from exc
            self._device = device
            self._interface = interface.bInterfaceNumber
            self._in = next(e.bEndpointAddress for e in interface if e.bEndpointAddress & 0x80)
            self._out = next(e.bEndpointAddress for e in interface if not e.bEndpointAddress & 0x80)

        def read(self, size: int) -> bytes:
            if size <= 0:
                return b""
            try:
                return bytes(self._device.read(self._in, min(size, _MAX_REQUEST), self._timeout_ms))
            except usb.core.USBError as exc:
                if exc.errno == _ERRNO_TIMEOUT:
                    return b""  # the same signal a pyserial read timeout gives
                raise serial.SerialException(f"read from {self.name} failed: {exc}") from exc

        def write(self, data: bytes) -> int:
            if not data:
                return 0
            try:
                return int(self._device.write(self._out, data, self._timeout_ms))
            except usb.core.USBError as exc:
                if exc.errno == _ERRNO_TIMEOUT:
                    raise serial.SerialTimeoutException(f"write to {self.name} timed out") from exc
                raise serial.SerialException(f"write to {self.name} failed: {exc}") from exc

        def flush(self) -> None:
            """Nothing to wait for: `write` returns once libusb has handed the data over."""

        def reset_input_buffer(self) -> None:
            """Drops whatever an interrupted earlier run left in flight."""
            while True:
                try:
                    if not len(self._device.read(self._in, _MAX_REQUEST, 50)):
                        return
                except usb.core.USBError:
                    return

        def reset_output_buffer(self) -> None:
            """Nothing to drop: `write` does not return until the data is submitted."""

        def close(self) -> None:
            if self._device is not None:
                try:
                    usb.util.release_interface(self._device, self._interface)
                except usb.core.USBError:
                    pass
                usb.util.dispose_resources(self._device)
                self._device = None

    Port = LibusbPort
else:
    Port = serial.Serial


def open_port(spec: str, timeout: float) -> Port:
    """Opens the benchmark peer. Raises `serial.SerialException` while it is not there."""
    if sys.platform == "win32":
        return LibusbPort(spec, timeout)
    device = resolve_port(spec)
    if device is None:
        raise serial.SerialException(f"no serial port with USB id {spec}")
    return serial.Serial(device, timeout=timeout, write_timeout=timeout)


def read_exact(port: Port, count: int) -> bytes:
    data = bytearray()
    while len(data) < count:
        chunk = port.read(count - len(data))
        if not chunk:
            fail(f"timed out after {len(data)} of {count} bytes")
        data.extend(chunk)
    return bytes(data)


def write_split(port: Port, frame: bytes, split_at: int) -> None:
    port.write(frame[:split_at])
    port.flush()
    time.sleep(0.02)
    port.write(frame[split_at:])
    port.flush()


def run_protocol_check(port: Port) -> None:
    def expect_ack(expected: int, case: str) -> None:
        acknowledged = struct.unpack("<I", read_exact(port, 4))[0]
        if acknowledged != expected:
            fail(f"{case}: device acknowledged {acknowledged} bytes, expected {expected}")

    for split_at in range(1, 5):
        in_frame = b"I" + struct.pack("<I", 257)
        write_split(port, in_frame, split_at)
        received = read_exact(port, 257)
        if received != ramp_slice(0, 257):
            fail(f"IN header split at {split_at}: payload mismatch")

        out_frame = b"O" + struct.pack("<I", 257)
        write_split(port, out_frame, split_at)
        port.write(ramp_slice(0, 257))
        port.flush()
        expect_ack(257, f"OUT header split at {split_at}")

    port.write(b"O" + struct.pack("<I", 0))
    port.flush()
    expect_ack(0, "zero-length OUT")

    port.write(b"X" + struct.pack("<I", 0) + b"I" + struct.pack("<I", 17))
    port.flush()
    if read_exact(port, 17) != ramp_slice(0, 17):
        fail("unknown-command recovery: payload mismatch")

    combined = (
        b"O" + struct.pack("<I", 3) + ramp_slice(0, 3) + b"I" + struct.pack("<I", 17)
    )
    port.write(combined)
    port.flush()
    expect_ack(3, "coalesced OUT")
    if read_exact(port, 17) != ramp_slice(0, 17):
        fail("coalesced IN: payload mismatch")

    print("protocol stream check passed")


def run_in(port: Port, count: int) -> tuple[float, int]:
    """Device -> host. Returns (elapsed seconds, mismatching byte count)."""
    frame = b"I" + struct.pack("<I", count)
    start = time.perf_counter()
    port.write(frame)
    port.flush()

    received = 0
    mismatches = 0
    while received < count:
        chunk = port.read(min(CHUNK, count - received))
        if not chunk:
            fail(f"timed out after {received} of {count} bytes")
        expected = ramp_slice(received, len(chunk))
        # The whole-buffer compare uses C memcmp. Count bytes only after a mismatch.
        if chunk != expected:
            mismatches += sum(
                1
                for actual, expected_byte in zip(chunk, expected)
                if actual != expected_byte
            )
        received += len(chunk)
    elapsed = time.perf_counter() - start
    return elapsed, mismatches


def run_out(port: Port, count: int) -> float:
    """Host -> device. Returns elapsed seconds."""
    frame = b"O" + struct.pack("<I", count)
    start = time.perf_counter()
    port.write(frame)
    port.flush()

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

    acknowledged = struct.unpack("<I", read_exact(port, 4))[0]
    if acknowledged != count:
        fail(f"device acknowledged {acknowledged} bytes, expected {count}")
    elapsed = time.perf_counter() - start
    return elapsed


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Measure USB CDC-ACM throughput against the LPC55 throughput firmware."
    )
    parser.add_argument(
        "--port",
        required=True,
        help="serial device (/dev/ttyACM0, COM7) or a USB id selector such as c0de:cafe",
    )
    parser.add_argument(
        "--bytes",
        type=parse_count,
        default=4_000_000,
        help="payload size in bytes (underscores allowed, default 4_000_000)",
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dir", choices=("in", "out"), help="transfer direction")
    mode.add_argument(
        "--protocol-check",
        action="store_true",
        help="check fragmented and coalesced protocol frames",
    )
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
            port = open_port(args.port, args.timeout)
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
        if args.protocol_check:
            run_protocol_check(port)
        elif args.dir == "in":
            elapsed, mismatches = run_in(port, count)
            print(f"payload mismatches: {mismatches} (0 expected)")
        else:
            elapsed = run_out(port, count)
            mismatches = 0
    except serial.SerialTimeoutException as exc:
        fail(f"serial timeout: {exc}")
    finally:
        port.close()

    if args.protocol_check:
        return

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
