#!/usr/bin/env python3
"""Host driver for the LPC55 USB conformance firmware.

Pairs with `tests/lpc55/src/conformance.rs` (bins `usb_hs_conformance` and
`usb_fs_conformance`). Needs raw USB access, so it runs as root; the orchestrator
(`tests/lpc55/run.py`) elevates only this script.

Every host-observable expectation lives here. The device only asserts the
invariants it alone can see, once this script sends FINISH.

Exit 0 on pass. On failure, print `FAIL: <phase>: <detail>` to stderr, exit 1.
"""

from __future__ import annotations

import argparse
from array import array
import glob
import os
import sys
import time
from typing import NamedTuple, NoReturn

import usb.core
import usb.util

VID = 0xC0DE

# Vendor requests. Recipient is always interface, wIndex always 0.
SET_MODE = 0x01
GET_REPORT = 0x02
ECHO_WRITE = 0x03
ECHO_READ = 0x04
TRIGGER_INT = 0x05
FINISH = 0x06
UNSUPPORTED = 0x07
RESET = 0x08

VENDOR_OUT = 0x41  # bmRequestType: host-to-device, vendor, interface
VENDOR_IN = 0xC1  # bmRequestType: device-to-host, vendor, interface

# Modes, matching `conformance::Mode`.
MODE_IDLE = 0
MODE_ECHO = 1
MODE_SINK = 2
MODE_SOURCE = 3
MODE_ISO_SINK = 4
MODE_ISO_SOURCE = 5

# Standard requests, issued raw where the point is to bypass the kernel's own
# endpoint bookkeeping.
STD_OUT_ENDPOINT = 0x02
STD_IN_ENDPOINT = 0x82
STD_OUT_DEVICE = 0x00
STD_IN_INTERFACE = 0x81
REQ_GET_STATUS = 0x00
REQ_SET_FEATURE = 0x03
REQ_GET_INTERFACE = 0x0A
REQ_SET_CONFIGURATION = 0x09
FEATURE_ENDPOINT_HALT = 0x00

EP_BULK_OUT = 0x01
EP_BULK_IN = 0x81
EP_INT_IN = 0x82

ERR_PIPE = 32  # EPIPE: the endpoint answered with a STALL handshake
ERR_TIMEOUT = 110  # ETIMEDOUT: the endpoint NAKed until the deadline
ISO_PACKET_TIMEOUT_MS = 100

RAMP_PERIOD = 512
# Byte k of any payload is `(k % 512) as u8` - the same ramp as
# `examples/lpc55s69/scripts/usb_throughput.py` and `conformance::ramp_byte`.
_RAMP = bytes((k % RAMP_PERIOD) & 0xFF for k in range(RAMP_PERIOD))


REPORT_LEN = 30


class Report(NamedTuple):
    """The GET_REPORT payload, in wire order (see `conformance::REPORT_LEN`)."""

    bulk_out_bytes: int
    bulk_in_bytes: int
    iso_out_bytes: int
    disabled_errors: int
    overflow_errors: int
    iso_out_packets: int
    iso_in_packets: int
    int_in_packets: int
    ramp_mismatches: int
    zlp_out_count: int


class Failure(Exception):
    """A phase failed. The message is already human-readable."""


def ramp(n: int, offset: int = 0) -> bytes:
    """`n` payload bytes starting at stream offset `offset`."""
    if n == 0:
        return b""
    start = offset % RAMP_PERIOD
    return (_RAMP * ((start + n) // RAMP_PERIOD + 1))[start : start + n]


def fail(phase: str, detail: str) -> NoReturn:
    print(f"FAIL: {phase}: {detail}", file=sys.stderr)
    sys.exit(1)


def require(cond: bool, detail: str) -> None:
    if not cond:
        raise Failure(detail)


class Device:
    """The conformance device plus its vendor-protocol helpers."""

    def __init__(self, dev: usb.core.Device) -> None:
        self.dev = dev

    def ctrl_out(self, request: int, w_value: int = 0, data: bytes = b"") -> None:
        self.dev.ctrl_transfer(VENDOR_OUT, request, w_value, 0, data, 5000)

    def ctrl_in(self, request: int, length: int, w_value: int = 0) -> bytes:
        return bytes(self.dev.ctrl_transfer(VENDOR_IN, request, w_value, 0, length, 5000))

    def report(self) -> Report:
        raw = self.ctrl_in(GET_REPORT, REPORT_LEN)
        require(len(raw) == REPORT_LEN, f"GET_REPORT returned {len(raw)} bytes")

        def u32(o: int) -> int:
            return int.from_bytes(raw[o : o + 4], "little")

        def u16(o: int) -> int:
            return int.from_bytes(raw[o : o + 2], "little")

        return Report(
            bulk_out_bytes=u32(0),
            bulk_in_bytes=u32(4),
            iso_out_bytes=u32(8),
            disabled_errors=u32(12),
            overflow_errors=u32(16),
            iso_out_packets=u16(20),
            iso_in_packets=u16(22),
            int_in_packets=u16(24),
            ramp_mismatches=u16(26),
            zlp_out_count=u16(28),
        )

    def set_mode(self, mode: int, param: int = 0) -> None:
        self.ctrl_out(SET_MODE, 0, bytes([mode, 0, param & 0xFF, (param >> 8) & 0xFF]))

    def write_bulk(self, data: bytes, timeout: int = 2000) -> int:
        return self.dev.write(EP_BULK_OUT, data, timeout)

    def raw_write_bulk(self, data: bytes, timeout: int) -> int:
        """Submits a bulk OUT transfer without pyusb's endpoint lookup.

        `Device.write` resolves the address against the *active* alt setting and
        raises `ValueError` when it is not there, which would hide what the
        device does with a transfer to a disabled endpoint. libusb neither knows
        nor cares about alt settings here, and the backend ignores the interface
        argument.
        """
        ctx = self.dev._ctx
        return ctx.backend.bulk_write(ctx.handle, EP_BULK_OUT, 0, array("B", data), timeout)

    def read_bulk(self, n: int, timeout: int = 2000) -> bytes:
        """Reads exactly `n` bytes from bulk IN, or one packet when `n == 0`.

        The device terminates every echoed packet with a short packet, so libusb
        completes each read early and a multi-packet echo needs several reads.
        """
        if n == 0:
            return bytes(self.dev.read(EP_BULK_IN, 64, timeout))
        out = bytearray()
        deadline = time.monotonic() + timeout / 1000 + 1.0
        while len(out) < n:
            out += bytes(self.dev.read(EP_BULK_IN, n - len(out) + 64, timeout))
            if time.monotonic() > deadline:
                break
        return bytes(out)

    def echo(self, n: int, timeout: int = 2000) -> None:
        """One bulk round trip that must come back byte-for-byte."""
        payload = ramp(n)
        self.write_bulk(payload, timeout)
        got = self.read_bulk(n, timeout)
        require(len(got) == n, f"echoed {len(got)} bytes, expected {n}")
        require(got == payload, f"echo of {n} bytes came back corrupt")

    def ep_status(self, ep_addr: int) -> int:
        raw = self.dev.ctrl_transfer(STD_IN_ENDPOINT, REQ_GET_STATUS, 0, ep_addr, 2, 2000)
        return int.from_bytes(bytes(raw), "little")


def find_device(pid: int, budget: float) -> Device:
    """Waits for a device that is present *and* answers GET_REPORT.

    A previous test's firmware halted at a breakpoint stays enumerated but
    unresponsive, so presence alone is not enough.
    """
    deadline = time.monotonic() + budget
    last = f"no device {VID:04x}:{pid:04x}"
    while time.monotonic() < deadline:
        dev = usb.core.find(idVendor=VID, idProduct=pid)
        if dev is None:
            last = f"no device {VID:04x}:{pid:04x}"
        else:
            d = Device(dev)
            try:
                d.report()
                return d
            except (usb.core.USBError, Failure) as exc:
                last = f"device present but GET_REPORT failed: {exc}"
                usb.util.dispose_resources(dev)
        time.sleep(0.25)
    raise Failure(f"{last} (waited {budget:.0f} s)")


def sysfs_speed(pid: int) -> str:
    """The negotiated link speed in Mbps, as the kernel reports it."""
    for vid_path in glob.glob("/sys/bus/usb/devices/*/idVendor"):
        base = os.path.dirname(vid_path)
        try:
            with open(vid_path) as f:
                if int(f.read().strip(), 16) != VID:
                    continue
            with open(os.path.join(base, "idProduct")) as f:
                if int(f.read().strip(), 16) != pid:
                    continue
            with open(os.path.join(base, "speed")) as f:
                return f.read().strip()
        except (OSError, ValueError):
            continue
    raise Failure(f"no sysfs entry for {VID:04x}:{pid:04x}")


# ------------------------------------------------------------------ phases


def phase_descriptors(d: Device, cfg: Config) -> tuple[int, int]:
    """Returns the isochronous OUT and IN endpoint addresses of alt setting 1."""
    want = "480" if cfg.speed == "hs" else "12"
    got = sysfs_speed(cfg.pid)
    require(got == want, f"negotiated {got} Mbps, expected {want} Mbps")

    conf = d.dev[0]
    alts = [i for i in conf if i.bInterfaceNumber == 0]
    require(len(alts) == 2, f"interface 0 has {len(alts)} alt settings, expected 2")

    alt0 = next(i for i in alts if i.bAlternateSetting == 0)
    eps0 = {e.bEndpointAddress: e for e in alt0}
    require(
        set(eps0) == {EP_BULK_OUT, EP_BULK_IN, EP_INT_IN},
        f"alt 0 endpoints are {sorted(hex(a) for a in eps0)}",
    )
    for addr in (EP_BULK_OUT, EP_BULK_IN):
        mps = eps0[addr].wMaxPacketSize
        require(mps == cfg.bulk_mps, f"endpoint {addr:#04x} mps is {mps}, expected {cfg.bulk_mps}")
    int_mps = eps0[EP_INT_IN].wMaxPacketSize
    require(int_mps == 16, f"interrupt IN mps is {int_mps}, expected 16")

    alt1 = next(i for i in alts if i.bAlternateSetting == 1)
    iso = list(alt1)
    require(len(iso) == 2, f"alt 1 has {len(iso)} endpoints, expected 2")
    iso_out = [e for e in iso if e.bEndpointAddress & 0x80 == 0]
    iso_in = [e for e in iso if e.bEndpointAddress & 0x80 != 0]
    require(len(iso_out) == 1 and len(iso_in) == 1, "alt 1 must expose one iso OUT and one iso IN")
    # On an isochronous endpoint, bits 12:11 of wMaxPacketSize carry the
    # transactions-per-microframe count; the size is the low 11 bits.
    out_mps = iso_out[0].wMaxPacketSize & 0x7FF
    in_mps = iso_in[0].wMaxPacketSize & 0x7FF
    require(out_mps == cfg.iso_out_mps, f"iso OUT mps is {out_mps}, expected {cfg.iso_out_mps}")
    require(in_mps == cfg.iso_in_mps, f"iso IN mps is {in_mps}, expected {cfg.iso_in_mps}")
    return iso_out[0].bEndpointAddress, iso_in[0].bEndpointAddress


def phase_control_data(d: Device) -> None:
    # 65 and 200 force the multi-packet `ControlPipe::data_out` loop on a
    # 64-byte EP0; 64 is the exact-mps boundary; 0 has no data stage at all.
    for n in (0, 1, 63, 64, 65, 200):
        payload = ramp(n)
        d.ctrl_out(ECHO_WRITE, 0, payload)
        # A zero-length control IN is not requested: ask for one byte and
        # require a zero-length response, which is the short-response path.
        got = d.ctrl_in(ECHO_READ, max(n, 1))
        require(got == payload, f"n={n}: echo returned {len(got)} bytes {got[:8].hex()}...")


def phase_control_reject(d: Device) -> None:
    try:
        d.ctrl_in(UNSUPPORTED, 8)
    except usb.core.USBError as exc:
        require(exc.errno == ERR_PIPE, f"UNSUPPORTED raised errno {exc.errno}, expected {ERR_PIPE}")
    else:
        raise Failure("UNSUPPORTED was accepted, expected a STALL")
    # Succeeding here proves `ControlPipe::setup` cleared the EP0 stall.
    d.report()


def phase_bulk_echo(d: Device, cfg: Config) -> None:
    mps = cfg.bulk_mps
    d.set_mode(MODE_ECHO)
    # The odd lengths cover erratum USB.5 on high speed; mps and 3 * mps cover
    # zero-length-packet termination; 0 must round-trip a ZLP.
    for n in sorted({0, 1, 7, 8, 63, 64, 65, mps - 1, mps, mps + 1, 3 * mps}):
        try:
            d.echo(n)
        except usb.core.USBError as exc:
            raise Failure(f"n={n}: {exc}") from exc
        except Failure as exc:
            raise Failure(f"n={n}: {exc}") from exc
    # Exactly one zero-length OUT packet in this phase: only n == 0 sends one.
    zlp = d.report().zlp_out_count
    require(zlp == 1, f"device counted {zlp} zero-length OUT packets, expected 1")


def phase_bulk_sink(d: Device, cfg: Config) -> None:
    """Sustained OUT in one multi-packet write, ramp-checked against a stream offset."""
    total = 8 * cfg.bulk_mps
    before = d.report()
    d.set_mode(MODE_SINK)
    try:
        written = d.write_bulk(ramp(total), 5000)
    except usb.core.USBError as exc:
        raise Failure(f"bulk OUT write failed: {exc}") from exc
    require(written == total, f"wrote {written} of {total} bytes")

    deadline = time.monotonic() + 5.0
    got, rep = 0, before
    while time.monotonic() < deadline:
        rep = d.report()
        got = rep.bulk_out_bytes - before.bulk_out_bytes
        if got >= total:
            break
        time.sleep(0.05)
    require(got == total, f"device received {got} of {total} bytes")
    require(rep.ramp_mismatches == 0, f"{rep.ramp_mismatches} ramp mismatches")
    d.set_mode(MODE_IDLE)


def phase_multi_packet_in(d: Device, cfg: Config) -> None:
    total = cfg.source_total
    d.set_mode(MODE_SOURCE, total)
    out = bytearray()
    deadline = time.monotonic() + 5.0
    while len(out) < total:
        try:
            out += bytes(d.dev.read(EP_BULK_IN, total - len(out), 2000))
        except usb.core.USBError as exc:
            raise Failure(f"bulk IN read failed after {len(out)} bytes: {exc}") from exc
        if time.monotonic() > deadline:
            break
    require(len(out) == total, f"read {len(out)} of {total} bytes")
    require(bytes(out) == ramp(total), "source payload does not match the ramp")


def phase_halt(d: Device, cfg: Config) -> None:
    mps = cfg.bulk_mps
    d.set_mode(MODE_ECHO)
    for ep_addr in (EP_BULK_IN, EP_BULK_OUT):
        if ep_addr == EP_BULK_IN:
            # Arm an IN reply *before* halting and never read it: that is what
            # makes `endpoint_set_stalled` take the EPSKIP reclaim path with a
            # slot still active.
            try:
                d.write_bulk(ramp(mps))
            except usb.core.USBError as exc:
                raise Failure(f"{ep_addr:#04x}: priming OUT write failed: {exc}") from exc
            time.sleep(0.05)

        d.dev.ctrl_transfer(
            STD_OUT_ENDPOINT, REQ_SET_FEATURE, FEATURE_ENDPOINT_HALT, ep_addr, None, 2000
        )
        status = d.ep_status(ep_addr)
        require(status & 1 == 1, f"{ep_addr:#04x}: GET_STATUS is {status:#06x} after SET_FEATURE")

        # Prove the halt is live on the wire, not just in the device's bookkeeping.
        if ep_addr == EP_BULK_IN:
            try:
                got = d.read_bulk(mps, 500)
            except usb.core.USBError as exc:
                require(
                    exc.errno == ERR_PIPE,
                    f"{ep_addr:#04x}: bulk IN raised errno {exc.errno}, expected {ERR_PIPE}",
                )
            else:
                raise Failure(
                    f"{ep_addr:#04x}: halted bulk IN returned {len(got)} bytes instead of stalling"
                )
        else:
            try:
                d.write_bulk(ramp(mps), 500)
            except usb.core.USBError as exc:
                require(
                    exc.errno == ERR_PIPE,
                    f"{ep_addr:#04x}: bulk OUT raised errno {exc.errno}, expected {ERR_PIPE}",
                )
            else:
                raise Failure(f"{ep_addr:#04x}: halted bulk OUT was accepted instead of stalling")

        # `clear_halt` and not a raw CLEAR_FEATURE: libusb_clear_halt goes
        # through the kernel, which sends CLEAR_FEATURE(HALT) *and* resets the
        # host-side data toggle. A raw control transfer would leave the host on
        # DATA1 while the driver's deferred TR_PENDING puts the device on DATA0,
        # and every later transfer would silently hang.
        d.dev.clear_halt(ep_addr)
        status = d.ep_status(ep_addr)
        require(status & 1 == 0, f"{ep_addr:#04x}: GET_STATUS is {status:#06x} after CLEAR_FEATURE")

        # The round trip is the toggle-reset proof.
        try:
            d.echo(mps)
        except usb.core.USBError as exc:
            raise Failure(f"{ep_addr:#04x}: echo after unhalt failed: {exc}") from exc
        except Failure as exc:
            raise Failure(f"{ep_addr:#04x}: after unhalt: {exc}") from exc


def phase_configuration_cycle(d: Device, cfg: Config) -> None:
    before = d.report()
    d.dev.ctrl_transfer(STD_OUT_DEVICE, REQ_SET_CONFIGURATION, 0x0000, 0, None, 2000)
    time.sleep(0.05)
    # SET_CONFIGURATION(0) disables every endpoint, which the bulk task must see
    # as `Disabled` on its pending read.
    rep = d.report()
    require(
        rep.disabled_errors > before.disabled_errors,
        f"SET_CONFIGURATION(0) did not disable the endpoints "
        f"(disabled_errors stayed at {rep.disabled_errors})",
    )
    d.dev.ctrl_transfer(STD_OUT_DEVICE, REQ_SET_CONFIGURATION, 0x0001, 0, None, 2000)
    # Both transfers bypassed the kernel, so resynchronise the host toggles with
    # the device's post-enable TR_PENDING state.
    d.dev.clear_halt(EP_BULK_OUT)
    d.dev.clear_halt(EP_BULK_IN)
    d.set_mode(MODE_ECHO)
    try:
        d.echo(cfg.bulk_mps)
    except usb.core.USBError as exc:
        raise Failure(f"echo after the configuration cycle failed: {exc}") from exc


def phase_alt_and_iso(d: Device, cfg: Config, iso_out_addr: int, iso_in_addr: int) -> None:
    before_alt = d.report()
    # Goes through the kernel, which resets the host toggles of the affected
    # endpoints, so no clear_halt is needed afterwards.
    d.dev.set_interface_altsetting(interface=0, alternate_setting=1)
    time.sleep(0.05)

    # That the bulk endpoints are now disabled cannot be shown from the host:
    # both pyusb and the kernel refuse to submit a transfer to an endpoint that
    # is not in the active alt setting, so nothing reaches the wire. The proof is
    # device-side instead - the bulk task's pending `read` must have come back
    # as `Disabled`, which is exactly what `endpoint_set_enabled(false)` does.
    rep = d.report()
    require(
        rep.disabled_errors > before_alt.disabled_errors,
        "selecting alt setting 1 did not disable the bulk endpoints "
        f"(disabled_errors stayed at {rep.disabled_errors})",
    )

    raw = d.dev.ctrl_transfer(STD_IN_INTERFACE, REQ_GET_INTERFACE, 0, 0, 1, 2000)
    require(raw[0] == 1, f"GET_INTERFACE returned {raw[0]}, expected 1")

    packet_count = 128
    min_packets = (packet_count * 4 + 4) // 5

    # --- isochronous OUT ---
    before = d.report()
    d.set_mode(MODE_ISO_SINK)
    time.sleep(0.05)
    packet = ramp(cfg.iso_out_mps)
    for i in range(packet_count):
        try:
            written = d.dev.write(iso_out_addr, packet, 5000)
        except usb.core.USBError as exc:
            raise Failure(f"isochronous OUT packet {i} failed: {exc}") from exc
        require(written == len(packet), f"isochronous OUT packet {i} wrote {written} of {len(packet)} bytes")
    time.sleep(0.2)
    rep = d.report()
    received_packets = rep.iso_out_packets - before.iso_out_packets
    intended_bytes = packet_count * cfg.iso_out_mps
    received_bytes = rep.iso_out_bytes - before.iso_out_bytes
    min_bytes = (intended_bytes * 4 + 4) // 5
    require(
        received_packets >= min_packets,
        f"device saw {received_packets} of {packet_count} isochronous OUT packets (under 80 %)",
    )
    require(
        received_bytes >= min_bytes,
        f"device saw {received_bytes} of {intended_bytes} isochronous OUT bytes (under 80 %)",
    )
    mismatches = rep.ramp_mismatches - before.ramp_mismatches
    require(mismatches == 0, f"{mismatches} isochronous OUT ramp mismatches")

    # --- isochronous IN ---
    mps = cfg.iso_in_mps
    before = d.report()
    d.set_mode(MODE_ISO_SOURCE)
    time.sleep(0.05)
    received_packets = 0
    received_bytes = 0
    for i in range(packet_count):
        try:
            data = bytes(d.dev.read(iso_in_addr, mps, ISO_PACKET_TIMEOUT_MS))
        except usb.core.USBError as exc:
            if exc.errno != ERR_TIMEOUT:
                raise Failure(f"isochronous IN packet {i} failed: {exc}") from exc
            continue
        require(len(data) == mps, f"isochronous IN packet {i} has {len(data)} bytes, expected {mps}")
        require(data == ramp(mps), f"isochronous IN packet {i} does not match the ramp")
        received_packets += 1
        received_bytes += len(data)
    rep = d.report()
    intended_bytes = packet_count * mps
    min_bytes = (intended_bytes * 4 + 4) // 5
    device_packets = rep.iso_in_packets - before.iso_in_packets
    require(
        received_packets >= min_packets,
        f"host saw {received_packets} of {packet_count} isochronous IN packets (under 80 %)",
    )
    require(
        received_bytes >= min_bytes,
        f"host saw {received_bytes} of {intended_bytes} isochronous IN bytes (under 80 %)",
    )
    require(
        device_packets >= min_packets,
        f"device sent {device_packets} of {packet_count} isochronous IN packets (under 80 %)",
    )

    d.set_mode(MODE_IDLE)
    d.dev.set_interface_altsetting(interface=0, alternate_setting=0)
    d.set_mode(MODE_ECHO)
    try:
        # Proves the bulk endpoints came back enabled with a reset toggle.
        d.echo(cfg.bulk_mps)
    except usb.core.USBError as exc:
        raise Failure(f"echo after returning to alt 0 failed: {exc}") from exc


def phase_interrupt_in(d: Device) -> None:
    d.set_mode(MODE_IDLE)
    for n in (1, 8, 15, 16):
        d.ctrl_out(TRIGGER_INT, n)
        try:
            got = bytes(d.dev.read(EP_INT_IN, 16, 2000))
        except usb.core.USBError as exc:
            raise Failure(f"n={n}: interrupt IN read failed: {exc}") from exc
        require(got == ramp(n), f"n={n}: interrupt IN returned {got.hex()}")
    packets = d.report().int_in_packets
    require(packets == 4, f"device counted {packets} interrupt IN packets, expected 4")


# ------------------------------------------------------------------ driver


class Config(NamedTuple):
    pid: int
    speed: str
    bulk_mps: int
    iso_out_mps: int
    iso_in_mps: int
    source_total: int


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--pid", required=True, type=lambda s: int(s, 0), help="idProduct, e.g. 0xcb01")
    parser.add_argument("--speed", required=True, choices=("hs", "fs"))
    parser.add_argument("--wait", type=float, default=15.0, help="seconds to wait for the device")
    parser.add_argument(
        "--iso-out-mps",
        type=int,
        default=None,
        help="expected isochronous OUT wMaxPacketSize (default 1024 on hs, 512 on fs)",
    )
    parser.add_argument(
        "--iso-in-mps",
        type=int,
        default=None,
        help="expected isochronous IN wMaxPacketSize (default 1023 on hs, 512 on fs)",
    )
    args = parser.parse_args()

    hs = args.speed == "hs"

    def pick(given: int | None, default: int) -> int:
        return default if given is None else given

    cfg = Config(
        pid=args.pid,
        speed=args.speed,
        bulk_mps=512 if hs else 64,
        # Full speed runs its isochronous endpoints at 512 and not the 1023-byte
        # maximum: two 1023-byte endpoints overrun the 1 ms frame's periodic
        # budget, and a 1023-byte full-speed isochronous IN through a high-speed
        # hub's transaction translator only survives one packet per URB.
        iso_out_mps=pick(args.iso_out_mps, 1024 if hs else 512),
        iso_in_mps=pick(args.iso_in_mps, 1023 if hs else 512),
        source_total=4096 if hs else 1024,
    )

    phase = "acquire"
    d = None
    try:
        d = find_device(cfg.pid, args.wait)
        # Re-issues SET_CONFIGURATION(1), exercising the enable path.
        d.dev.set_configuration()
        usb.util.claim_interface(d.dev, 0)
        # Every absolute expectation below counts from here, so the script is
        # re-runnable against a firmware that is already up.
        d.ctrl_out(RESET)
        print(f"[{phase}] {VID:04x}:{cfg.pid:04x} responding")

        phase = "descriptors"
        iso_out_addr, iso_in_addr = phase_descriptors(d, cfg)
        print(f"[{phase}] ok")

        for phase, fn in (
            ("control_data", lambda: phase_control_data(d)),
            ("control_reject", lambda: phase_control_reject(d)),
            ("bulk_echo", lambda: phase_bulk_echo(d, cfg)),
            ("bulk_sink", lambda: phase_bulk_sink(d, cfg)),
            ("multi_packet_in", lambda: phase_multi_packet_in(d, cfg)),
            ("halt", lambda: phase_halt(d, cfg)),
            ("configuration_cycle", lambda: phase_configuration_cycle(d, cfg)),
            ("alt_and_iso", lambda: phase_alt_and_iso(d, cfg, iso_out_addr, iso_in_addr)),
            ("interrupt_in", lambda: phase_interrupt_in(d)),
        ):
            fn()
            print(f"[{phase}] ok")

        phase = "finish"
        print(f"[report] {d.report()}")
        d.ctrl_out(FINISH)
        print(f"[{phase}] ok")
    except Failure as exc:
        fail(phase, str(exc))
    except usb.core.USBError as exc:
        fail(phase, f"unexpected USB error: {exc}")
    finally:
        if d is not None:
            try:
                usb.util.release_interface(d.dev, 0)
            except usb.core.USBError:
                pass
            usb.util.dispose_resources(d.dev)


if __name__ == "__main__":
    main()
