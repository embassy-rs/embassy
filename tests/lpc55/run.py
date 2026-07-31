#!/usr/bin/env python3
"""Run the LPC55 USB hardware-in-the-loop suite.

For each entry, the runner flashes release firmware through `probe-rs`, waits for its RTT ready
banner, starts the host peer, and requires `Test OK`.

Run the suite as an unprivileged user so `cargo` and `probe-rs` retain the user's `~/.cargo` and
target directories. Only the raw-USB conformance entries use `sudo -n` with this interpreter.
A plain `sudo python3` might not resolve pyusb.

    python3 tests/lpc55/run.py [--only NAME]... [--list] [--keep-going]

Use an LPCXpresso55S69 EVK with a debug probe on the SWD header. Connect the host to P9
(USB1, high speed) and P10 (USB0, full speed).
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
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
HOST_TIMEOUT = 90.0
# `probe-rs run` keeps the core halted at the breakpoint, so the child never exits on its own.
KILL_GRACE = 5.0
# Lets the previous probe-rs session detach and the host see the device go away.
SETTLE = 1.5

HANDLED_SIGNALS = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
_SIGNAL_NUMBER: int | None = None
_SIGNAL_DEFERRED = False


def handle_signal(signum: int, _frame: object) -> None:
    global _SIGNAL_NUMBER
    if _SIGNAL_NUMBER is None:
        _SIGNAL_NUMBER = signum
    if not _SIGNAL_DEFERRED:
        raise KeyboardInterrupt


def defer_handled_signals() -> None:
    global _SIGNAL_DEFERRED
    _SIGNAL_DEFERRED = True


def resume_handled_signals() -> None:
    global _SIGNAL_DEFERRED
    _SIGNAL_DEFERRED = False
    if _SIGNAL_NUMBER is not None:
        raise KeyboardInterrupt


def process_group_exists(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    return True


def wait_process_group(proc: subprocess.Popen[str], timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while process_group_exists(proc.pid):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return False
        if proc.poll() is None:
            try:
                proc.wait(timeout=min(0.1, remaining))
            except subprocess.TimeoutExpired:
                pass
        else:
            time.sleep(min(0.05, remaining))
    return True


def terminate_group(proc: subprocess.Popen[str], first_signal: int) -> None:
    for signum in (first_signal, signal.SIGTERM, signal.SIGKILL):
        if not process_group_exists(proc.pid):
            break
        try:
            os.killpg(proc.pid, signum)
        except ProcessLookupError:
            break
        if wait_process_group(proc, KILL_GRACE):
            break

    if proc.poll() is None:
        try:
            proc.wait(timeout=KILL_GRACE)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            proc.wait(timeout=KILL_GRACE)
    else:
        proc.wait()

FS_PORT = "/dev/serial/by-id/usb-Embassy_USB-FS_throughput_test_12345678-if00"
HS_PORT = "/dev/serial/by-id/usb-Embassy_USB-HS_throughput_test_12345678-if00"

# Roughly 75 % of the release-profile numbers recorded in README.md, so the gate catches a
# real regression without tripping on host jitter.
FS_IN_MIN, FS_OUT_MIN = "0.67", "0.61"
HS_IN_MIN, HS_OUT_MIN = "33.0", "13.0"
THROUGHPUT = (sys.executable, os.path.join("scripts", "usb_throughput.py"))



def conformance_command(pid: str, speed: str) -> tuple[str, ...]:
    return (sys.executable, os.path.join("host", "conformance.py"), "--pid", pid, "--speed", speed)


def throughput_command(port: str, direction: str, count: str, min_rate: str) -> tuple[str, ...]:
    return (*THROUGHPUT, "--port", port, "--dir", direction, "--bytes", count, "--wait-port", "15", "--min-rate", min_rate)


def protocol_check(port: str) -> tuple[str, ...]:
    return (*THROUGHPUT, "--port", port, "--protocol-check", "--wait-port", "15")


@dataclass(frozen=True)
class Test:
    name: str
    cwd: str
    binary: str
    ready: str | None = None
    host: tuple[tuple[str, ...], ...] = ()
    sudo: bool = False
    expect_ok: bool = True
    note: str = ""


def conformance_test(name: str, pid: str, speed: str, note: str) -> Test:
    return Test(
        name, TESTS, f"usb_{name}", ready="conformance ready", host=(conformance_command(pid, speed),), sudo=True, note=note
    )


def throughput_test(speed: str, port: str, count: str, in_min: str, out_min: str) -> Test:
    name = f"{speed}_throughput"
    return Test(
        name,
        EXAMPLES,
        f"usb_{name}",
        ready="Initialization complete",
        host=(protocol_check(port), throughput_command(port, "in", count, in_min), throughput_command(port, "out", count, out_min)),
        expect_ok=False,
        note=f"CDC bulk throughput gate, >= {in_min}/{out_min} MB/s",
    )


# Fastest and least cable-dependent first, so a broken setup fails early and cheaply.
TESTS_LIST = (
    Test("alloc", TESTS, "usb_alloc", note="endpoint allocation limits, no host cable"),
    Test("alloc_small", TESTS, "usb_alloc_small", note="exact 512-byte endpoint memory"),
    Test("bus_raw", TESTS, "usb_bus_raw", note="raw Bus disable/enable/reinit/force_reset"),
    Test("fs_enumerate", TESTS, "usb_fs_enumerate", note="USB0 reaches Configured (P10)"),
    Test("hs_enumerate", TESTS, "usb_hs_enumerate", note="USBHSD at 480 Mbps (P9)"),
    Test(
        "dual_pll0",
        TESTS,
        "usb_dual_pll0",
        ready="dual pll0 ready",
        host=((sys.executable, os.path.join("host", "dual_echo.py")),),
        note="both controllers concurrently on PLL0 at 150 MHz",
    ),
    conformance_test("fs_conformance", "0xcb02", "fs", "full-speed control/bulk/iso/interrupt conformance"),
    conformance_test("hs_conformance", "0xcb01", "hs", "high-speed control/bulk/iso/interrupt conformance"),
    throughput_test("fs", FS_PORT, "1_000_000", FS_IN_MIN, FS_OUT_MIN),
    throughput_test("hs", HS_PORT, "4_000_000", HS_IN_MIN, HS_OUT_MIN),
)

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
            start_new_session=True,
        )
        self._pump = threading.Thread(target=self._read, daemon=True)
        try:
            self._pump.start()
        except BaseException:
            terminate_group(self.proc, signal.SIGINT)
            raise

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
        terminate_group(self.proc, signal.SIGINT)
        self._pump.join(timeout=KILL_GRACE)
        if self._pump.is_alive():
            raise RuntimeError(f"firmware output thread for {self.test.name} did not stop")


def run_host(test: Test, argv: tuple[str, ...]) -> str | None:
    """Runs one host command. Returns None on success or a failure description."""
    cmd = ["sudo", "-n", *argv] if test.sudo else list(argv)
    printable = " ".join(os.path.basename(c) if c == sys.executable else c for c in cmd)
    print(f"  [{test.name}] host: {printable}", flush=True)
    defer_handled_signals()
    try:
        proc = subprocess.Popen(
            cmd,
            cwd=os.path.join(REPO, test.cwd),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
    except BaseException:
        resume_handled_signals()
        raise

    timed_out = False
    try:
        resume_handled_signals()
        try:
            stdout, stderr = proc.communicate(timeout=HOST_TIMEOUT)
        except subprocess.TimeoutExpired:
            timed_out = True
            terminate_group(proc, signal.SIGINT)
            stdout, stderr = proc.communicate(timeout=KILL_GRACE)
    finally:
        defer_handled_signals()
        try:
            terminate_group(proc, signal.SIGINT)
        finally:
            resume_handled_signals()

    for stream in (stdout, stderr):
        for line in stream.splitlines():
            print(f"  [{test.name}] | {line}", flush=True)
    if timed_out:
        return f"host command timed out after {HOST_TIMEOUT:g} s: {printable}"
    if proc.returncode != 0:
        return f"host command exited {proc.returncode}: {printable}"
    return None


def run_test(test: Test) -> str | None:
    """Runs one entry. Returns None on pass or a one-line reason on failure."""
    print(f"\n=== {test.name}: {test.note} ===", flush=True)
    defer_handled_signals()
    try:
        fw = Firmware(test)
    except BaseException:
        resume_handled_signals()
        raise

    try:
        resume_handled_signals()

        def verdict(timeout_reason: str) -> str:
            # A device panic or a dead probe-rs explains a missing banner far better than a timeout:
            # a flash that failed means the firmware never ran, so nothing was ever going to appear.
            panics = fw.panics()
            if panics:
                return f"device panicked: {panics[0].strip()}"
            return fw.probe_failure() or timeout_reason

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
        defer_handled_signals()
        try:
            fw.stop()
            time.sleep(SETTLE)
        finally:
            resume_handled_signals()


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
    for caught_signal in HANDLED_SIGNALS:
        signal.signal(caught_signal, handle_signal)
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(128 + (_SIGNAL_NUMBER or signal.SIGINT))
