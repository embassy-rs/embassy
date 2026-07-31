#!/usr/bin/env python3
"""Runs the whole LPC55 USB hardware-in-the-loop suite.

Each entry flashes one firmware over the J-Link with `cargo run --release`, waits for its
ready banner on the RTT log, drives the host side of the test, and requires the device to
print `Test OK`.

Run it unprivileged from anywhere in the repo: `cargo` and `probe-rs` must keep the user's
`~/.cargo` and target directories. Only the pyusb-based conformance script is elevated, with
this interpreter, because raw USB access to the vendor interface needs root and a plain
`sudo python3` would not resolve pyusb.

    python3 tests/lpc55/run.py [--only NAME]... [--list] [--keep-going]

Hardware: an LPCXpresso55S69 EVK with the onboard J-Link connected, plus host cables on both
**P9** (USB1, high speed) and **P10** (USB0, full speed).
"""

from __future__ import annotations

import argparse
import os
import re
import signal
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TESTS = os.path.join("tests", "lpc55")
EXAMPLES = os.path.join("examples", "lpc55s69")

READY_TIMEOUT = 90.0
OK_TIMEOUT = 90.0
# `probe-rs run` keeps the core halted at the breakpoint, so the child never exits on its own.
KILL_GRACE = 5.0
# Lets the previous probe-rs session detach and the host see the device go away.
SETTLE = 1.5

FS_PORT = "/dev/serial/by-id/usb-Embassy_USB-FS_throughput_test_12345678-if00"
HS_PORT = "/dev/serial/by-id/usb-Embassy_USB-HS_throughput_test_12345678-if00"

# Roughly 75 % of the release-profile numbers recorded in README.md, so the gate catches a
# real regression without tripping on host jitter.
FS_IN_MIN, FS_OUT_MIN = "0.67", "0.61"
HS_IN_MIN, HS_OUT_MIN = "33.0", "13.0"


def conformance(pid: str, speed: str) -> list[str]:
    return [sys.executable, os.path.join("host", "conformance.py"), "--pid", pid, "--speed", speed]


def throughput(port: str, direction: str, count: str, min_rate: str) -> list[str]:
    return [
        sys.executable,
        os.path.join("scripts", "usb_throughput.py"),
        "--port",
        port,
        "--dir",
        direction,
        "--bytes",
        count,
        "--wait-port",
        "15",
        "--min-rate",
        min_rate,
    ]


class Test:
    def __init__(
        self,
        name: str,
        cwd: str,
        binary: str,
        ready: str | None = None,
        host: list[list[str]] | None = None,
        sudo: bool = False,
        expect_ok: bool = True,
        note: str = "",
    ) -> None:
        self.name = name
        self.cwd = cwd
        self.binary = binary
        self.ready = ready
        self.host = host or []
        self.sudo = sudo
        self.expect_ok = expect_ok
        self.note = note


# Fastest and least cable-dependent first, so a broken setup fails early and cheaply.
TESTS_LIST = [
    Test("alloc", TESTS, "usb_alloc", note="endpoint allocation limits, no host cable"),
    Test("bus_raw", TESTS, "usb_bus_raw", note="raw Bus disable/enable/reinit/force_reset"),
    Test("fs_enumerate", TESTS, "usb_fs_enumerate", note="USB0 reaches Configured (P10)"),
    Test("hs_enumerate", TESTS, "usb_hs_enumerate", note="USBHSD at 480 Mbps (P9)"),
    Test(
        "dual_pll0",
        TESTS,
        "usb_dual_pll0",
        ready="dual pll0 ready",
        host=[[sys.executable, os.path.join("host", "dual_echo.py")]],
        note="both controllers concurrently on PLL0 at 150 MHz",
    ),
    Test(
        "fs_conformance",
        TESTS,
        "usb_fs_conformance",
        ready="conformance ready",
        host=[conformance("0xcb02", "fs")],
        sudo=True,
        note="full-speed control/bulk/iso/interrupt conformance",
    ),
    Test(
        "hs_conformance",
        TESTS,
        "usb_hs_conformance",
        ready="conformance ready",
        host=[conformance("0xcb01", "hs")],
        sudo=True,
        note="high-speed control/bulk/iso/interrupt conformance",
    ),
    Test(
        "fs_throughput",
        EXAMPLES,
        "usb_fs_throughput",
        ready="Initialization complete",
        host=[
            throughput(FS_PORT, "in", "1_000_000", FS_IN_MIN),
            throughput(FS_PORT, "out", "1_000_000", FS_OUT_MIN),
        ],
        expect_ok=False,
        note=f"CDC bulk throughput gate, >= {FS_IN_MIN}/{FS_OUT_MIN} MB/s",
    ),
    Test(
        "hs_throughput",
        EXAMPLES,
        "usb_hs_throughput",
        ready="Initialization complete",
        host=[
            throughput(HS_PORT, "in", "4_000_000", HS_IN_MIN),
            throughput(HS_PORT, "out", "4_000_000", HS_OUT_MIN),
        ],
        expect_ok=False,
        note=f"CDC bulk throughput gate, >= {HS_IN_MIN}/{HS_OUT_MIN} MB/s",
    ),
]

# probe-rs itself is noisy about SWD retries during flashing, and those lines are not device
# output; only match what the firmware could have printed.
PANIC_MARKERS = ("panicked", "PANIC", "[ERROR]")

# A `probe-rs` failure is a top-level `Error:` followed by an indented, numbered `Caused by:`
# chain. The top level says which stage failed, the deepest cause says why.
PROBE_ERROR = re.compile(r"^\s*Error: (.*\S)")
PROBE_CAUSE = re.compile(r"^\s*\d+: (.*\S)")


class Firmware:
    """`cargo run --release --bin <bin>`, with its output captured line by line."""

    def __init__(self, test: Test) -> None:
        self.test = test
        self.lines: list[str] = []
        self._lock = threading.Lock()
        self.proc = subprocess.Popen(
            ["cargo", "run", "--release", "--bin", test.binary],
            cwd=os.path.join(REPO, test.cwd),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        self._pump = threading.Thread(target=self._read, daemon=True)
        self._pump.start()

    def _read(self) -> None:
        assert self.proc.stdout is not None
        for line in self.proc.stdout:
            line = line.rstrip("\n")
            with self._lock:
                self.lines.append(line)
            print(f"  [{self.test.name}] {line}", flush=True)

    def wait_for(self, needle: str, timeout: float) -> bool:
        """Waits for `needle`. Returns early on a device panic - it will not come."""
        deadline = time.monotonic() + timeout
        while True:
            with self._lock:
                if any(needle in line for line in self.lines):
                    return True
                panicked = any(m in ln for ln in self.lines for m in PANIC_MARKERS)
            if panicked:
                return False
            if self.proc.poll() is not None:
                # One last look: the child may have exited right after printing.
                time.sleep(0.2)
                with self._lock:
                    return any(needle in line for line in self.lines)
            if time.monotonic() >= deadline:
                return False
            time.sleep(0.1)

    def panics(self) -> list[str]:
        with self._lock:
            return [ln for ln in self.lines if any(m in ln for m in PANIC_MARKERS)]

    def probe_failure(self) -> str | None:
        """Describes an abnormal `probe-rs` exit. It otherwise sits at the breakpoint forever."""
        code = self.proc.poll()
        if code is None:
            return None
        with self._lock:
            lines = list(self.lines)
        parts = [f"probe-rs exited {code}"]
        for pattern in (PROBE_ERROR, PROBE_CAUSE):
            hits = [m.group(1) for ln in lines if (m := pattern.match(ln))]
            if hits:
                # probe-rs punctuates some messages and not others; `: ` is the separator here.
                parts.append(hits[-1].rstrip("."))
        return ": ".join(parts)

    def stop(self) -> None:
        if self.proc.poll() is None:
            self.proc.send_signal(signal.SIGINT)
            try:
                self.proc.wait(KILL_GRACE)
            except subprocess.TimeoutExpired:
                self.proc.kill()
        self.proc.wait()
        self._pump.join(timeout=2.0)


def run_host(test: Test, argv: list[str]) -> str | None:
    """Runs one host command. Returns None on success or a failure description."""
    cmd = ["sudo", "-n", *argv] if test.sudo else list(argv)
    printable = " ".join(os.path.basename(c) if c == sys.executable else c for c in cmd)
    print(f"  [{test.name}] host: {printable}", flush=True)
    proc = subprocess.run(
        cmd,
        cwd=os.path.join(REPO, test.cwd),
        capture_output=True,
        text=True,
    )
    for stream in (proc.stdout, proc.stderr):
        for line in stream.splitlines():
            print(f"  [{test.name}] | {line}", flush=True)
    if proc.returncode != 0:
        return f"host command exited {proc.returncode}: {printable}"
    return None


def run_test(test: Test) -> str | None:
    """Runs one entry. Returns None on pass or a one-line reason on failure."""
    print(f"\n=== {test.name}: {test.note} ===", flush=True)
    fw = Firmware(test)

    def verdict(timeout_reason: str) -> str:
        # A device panic or a dead probe-rs explains a missing banner far better than a timeout:
        # a flash that failed means the firmware never ran, so nothing was ever going to appear.
        panics = fw.panics()
        if panics:
            return f"device panicked: {panics[0].strip()}"
        return fw.probe_failure() or timeout_reason

    try:
        if test.ready is not None and not fw.wait_for(test.ready, READY_TIMEOUT):
            return verdict(f"ready banner {test.ready!r} not seen within {READY_TIMEOUT:.0f} s")

        for argv in test.host:
            failure = run_host(test, argv)
            if failure is not None:
                return verdict(failure)

        if test.expect_ok and not fw.wait_for("Test OK", OK_TIMEOUT):
            return verdict(f"device did not print 'Test OK' within {OK_TIMEOUT:.0f} s")

        panics = fw.panics()
        if panics:
            return f"device panicked: {panics[0].strip()}"
        return fw.probe_failure()
    finally:
        fw.stop()
        time.sleep(SETTLE)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--only",
        action="append",
        metavar="NAME",
        help="run only this test; repeatable",
    )
    parser.add_argument("--list", action="store_true", help="list the tests and exit")
    parser.add_argument(
        "--keep-going", action="store_true", help="run the remaining tests after a failure"
    )
    args = parser.parse_args()

    if args.list:
        for test in TESTS_LIST:
            print(f"{test.name:16} {test.cwd}/{test.binary:22} {test.note}")
        return

    selected = TESTS_LIST
    if args.only:
        known = {t.name for t in TESTS_LIST}
        unknown = [n for n in args.only if n not in known]
        if unknown:
            parser.error(f"unknown test(s): {', '.join(unknown)}; try --list")
        selected = [t for t in TESTS_LIST if t.name in set(args.only)]

    results: list[tuple[str, str | None]] = []
    for test in selected:
        failure = run_test(test)
        results.append((test.name, failure))
        if failure is not None:
            print(f"  [{test.name}] FAIL: {failure}", flush=True)
            if not args.keep_going:
                break

    print("\n=== summary ===")
    width = max(len(name) for name, _ in results)
    for name, failure in results:
        print(f"{name:{width}}  {'PASS' if failure is None else 'FAIL  ' + failure}")
    skipped = len(selected) - len(results)
    if skipped:
        print(f"({skipped} test(s) not run; pass --keep-going to continue past a failure)")
    sys.exit(1 if any(f is not None for _, f in results) or skipped else 0)


if __name__ == "__main__":
    main()
