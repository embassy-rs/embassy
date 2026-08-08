#!/usr/bin/env python3
"""Run the LPC55 USB hardware-in-the-loop suite.

For each entry, the runner flashes release firmware through `probe-rs`, waits for its RTT ready
banner, starts the host peer, and requires `Test OK`.

Runs on Linux and Windows. This file is stdlib-only; every host peer declares its own
dependencies inline and runs through `uv run --script`.

Run the suite as an unprivileged user so `cargo` and `probe-rs` retain the user's `~/.cargo` and
target directories. On Linux the two conformance entries need root for raw USB access and are
the only ones run with `sudo -n`. On Windows the conformance firmware's MS OS 2.0 descriptors
make Windows bind WinUSB by itself, which an unprivileged process can open, so the suite
elevates nothing.

    python3 tests/lpc55/run.py [--only NAME]... [--list] [--keep-going]

Use an LPCXpresso55S69 EVK with a debug probe on the SWD header. Connect the host to P9
(USB1, high speed) and P10 (USB0, full speed).
"""

from __future__ import annotations

import argparse
from collections.abc import Sequence
from dataclasses import dataclass
import os
import re
import shutil
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

WINDOWS = sys.platform == "win32"

# `uv` runs the host peers. Absolute, so `sudo -n` finds it whatever `secure_path` says.
UV = shutil.which("uv")

# SIGHUP is POSIX-only. SIGBREAK is what a Windows console raises in its place.
HANDLED_SIGNALS = tuple(
    getattr(signal, name) for name in ("SIGINT", "SIGTERM", "SIGHUP", "SIGBREAK") if hasattr(signal, name)
)
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


if WINDOWS:
    import ctypes
    from ctypes import wintypes

    _KERNEL32 = ctypes.WinDLL("kernel32", use_last_error=True)
    _KERNEL32.CreateJobObjectW.argtypes = (wintypes.LPVOID, wintypes.LPCWSTR)
    _KERNEL32.CreateJobObjectW.restype = wintypes.HANDLE
    _KERNEL32.AssignProcessToJobObject.argtypes = (wintypes.HANDLE, wintypes.HANDLE)
    _KERNEL32.AssignProcessToJobObject.restype = wintypes.BOOL
    _KERNEL32.TerminateJobObject.argtypes = (wintypes.HANDLE, wintypes.UINT)
    _KERNEL32.TerminateJobObject.restype = wintypes.BOOL
    _KERNEL32.CloseHandle.argtypes = (wintypes.HANDLE,)
    _KERNEL32.CloseHandle.restype = wintypes.BOOL


class Child:
    """A child process and everything it spawns, stoppable as one unit.

    `cargo run` execs `probe-rs`, and a `probe-rs` that outlives its test keeps the debug
    probe and fails every entry after it, so stopping a test has to reach the whole tree.
    POSIX gets a process group. Windows gets a job object, because `taskkill /T` walks
    parent links that are already gone once `cargo` itself has exited, while
    `TerminateJobObject` does not care about parentage.
    """

    def __init__(self, cmd: Sequence[str], cwd: str, merge_stderr: bool) -> None:
        self._job = None
        if WINDOWS:
            self._job = _KERNEL32.CreateJobObjectW(None, None)
            if not self._job:
                raise ctypes.WinError(ctypes.get_last_error())
            group: dict[str, object] = {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
        else:
            group = {"start_new_session": True}
        try:
            self.proc = subprocess.Popen(
                list(cmd),
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT if merge_stderr else subprocess.PIPE,
                text=True,
                bufsize=1,
                **group,
            )
        except BaseException:
            self._close_job()
            raise
        if self._job is not None:
            # `cargo` still has to reach the filesystem before it can fork `probe-rs`, so
            # it is childless here and nothing escapes the job.
            handle = wintypes.HANDLE(int(self.proc._handle))
            if not _KERNEL32.AssignProcessToJobObject(self._job, handle):
                error = ctypes.get_last_error()
                self.stop()
                raise ctypes.WinError(error)

    def stop(self) -> None:
        """Interrupts the tree, then makes sure nothing in it survived."""
        if WINDOWS:
            self._stop_windows()
        else:
            terminate_group(self.proc, signal.SIGINT)

    def _stop_windows(self) -> None:
        if self._job is None:
            return
        if self.proc.poll() is None:
            try:
                # SIGINT's counterpart here. `probe-rs` installs a console handler, so it
                # gets to release the debug probe: a hard kill leaves the probe wedged and
                # the next session fails to open it. `cargo` shares the group and forwards
                # nothing, so the event has to go to the group rather than the child.
                self.proc.send_signal(signal.CTRL_C_EVENT)
            except OSError:
                pass
            try:
                self.proc.wait(timeout=KILL_GRACE)
            except subprocess.TimeoutExpired:
                pass
        _KERNEL32.TerminateJobObject(self._job, 1)
        try:
            self.proc.wait(timeout=KILL_GRACE)
        except subprocess.TimeoutExpired:
            pass
        self._close_job()

    def _close_job(self) -> None:
        if self._job is not None:
            _KERNEL32.CloseHandle(self._job)
            self._job = None

# A USB id is the only peer identity Linux and Windows share: device names
# (`/dev/serial/by-id/...`, `COM7`) have nothing in common, and on Windows these two
# firmwares are not serial ports at all. One id per firmware, distinct from the serial
# examples' `c0de:cafe`, so a stale binding or a leftover node can never be mistaken for
# the peer under test.
FS_USB_ID = "c0de:cb07"
HS_USB_ID = "c0de:cb08"

# Roughly 75 % of the release-profile numbers recorded in README.md, so the gate catches a
# real regression without tripping on host jitter.
FS_IN_MIN, FS_OUT_MIN = "0.67", "0.61"
HS_IN_MIN, HS_OUT_MIN = "33.0", "13.0"
THROUGHPUT = os.path.join("scripts", "usb_throughput.py")


def uv_command(argv: Sequence[str]) -> list[str]:
    """`uv run --script` resolves each peer's inline PEP 723 dependencies, so neither pyusb
    nor pyserial has to be installed for the suite."""
    assert UV is not None  # `main` refuses to run the suite without it
    return [UV, "run", "--script", *argv]


def conformance_command(pid: str, speed: str) -> tuple[str, ...]:
    return (os.path.join("host", "conformance.py"), "--pid", pid, "--speed", speed)


def throughput_command(usb_id: str, direction: str, count: str, min_rate: str) -> tuple[str, ...]:
    return (
        THROUGHPUT,
        "--port",
        usb_id,
        "--dir",
        direction,
        "--bytes",
        count,
        "--wait-port",
        "15",
        "--min-rate",
        min_rate,
    )


def protocol_check(usb_id: str) -> tuple[str, ...]:
    return (THROUGHPUT, "--port", usb_id, "--protocol-check", "--wait-port", "15")


@dataclass(frozen=True)
class Test:
    name: str
    cwd: str
    binary: str
    ready: str | None = None
    host: tuple[tuple[str, ...], ...] = ()
    # Raw USB access to the vendor interface. Only Linux needs a privilege for it.
    raw_usb: bool = False
    expect_ok: bool = True
    note: str = ""


def conformance_test(name: str, pid: str, speed: str, note: str) -> Test:
    return Test(
        name,
        TESTS,
        f"usb_{name}",
        ready="conformance ready",
        host=(conformance_command(pid, speed),),
        raw_usb=True,
        note=note,
    )


def throughput_test(speed: str, usb_id: str, count: str, in_min: str, out_min: str) -> Test:
    name = f"{speed}_throughput"
    return Test(
        name,
        EXAMPLES,
        f"usb_{name}",
        ready="Initialization complete",
        host=(
            protocol_check(usb_id),
            throughput_command(usb_id, "in", count, in_min),
            throughput_command(usb_id, "out", count, out_min),
        ),
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
        host=((os.path.join("host", "dual_echo.py"),),),
        note="both controllers concurrently on PLL0 at 150 MHz",
    ),
    conformance_test("fs_conformance", "0xcb02", "fs", "full-speed control/bulk/iso/interrupt conformance"),
    conformance_test("hs_conformance", "0xcb01", "hs", "high-speed control/bulk/iso/interrupt conformance"),
    throughput_test("fs", FS_USB_ID, "1_000_000", FS_IN_MIN, FS_OUT_MIN),
    throughput_test("hs", HS_USB_ID, "4_000_000", HS_IN_MIN, HS_OUT_MIN),
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
        self.child = Child(
            ["cargo", "run", "--release", "--bin", test.binary],
            cwd=os.path.join(REPO, test.cwd),
            merge_stderr=True,
        )
        self._pump = threading.Thread(target=self._read, daemon=True)
        try:
            self._pump.start()
        except BaseException:
            self.child.stop()
            raise

    def _read(self) -> None:
        assert self.child.proc.stdout is not None
        for line in self.child.proc.stdout:
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
            if self.child.proc.poll() is not None:
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
        code = self.child.proc.poll()
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
        self.child.stop()
        self._pump.join(timeout=KILL_GRACE)
        if self._pump.is_alive():
            raise RuntimeError(f"firmware output thread for {self.test.name} did not stop")


def run_host(test: Test, argv: tuple[str, ...]) -> str | None:
    """Runs one host command. Returns None on success or a failure description."""
    cmd = uv_command(argv)
    if test.raw_usb and not WINDOWS:
        cmd = ["sudo", "-n", *cmd]
    printable = " ".join(os.path.basename(c) if c == UV else c for c in cmd)
    print(f"  [{test.name}] host: {printable}", flush=True)
    defer_handled_signals()
    try:
        child = Child(cmd, cwd=os.path.join(REPO, test.cwd), merge_stderr=False)
    except BaseException:
        resume_handled_signals()
        raise

    timed_out = False
    try:
        resume_handled_signals()
        try:
            stdout, stderr = child.proc.communicate(timeout=HOST_TIMEOUT)
        except subprocess.TimeoutExpired:
            timed_out = True
            child.stop()
            stdout, stderr = child.proc.communicate(timeout=KILL_GRACE)
    finally:
        defer_handled_signals()
        try:
            child.stop()
        finally:
            resume_handled_signals()

    for stream in (stdout, stderr):
        for line in stream.splitlines():
            print(f"  [{test.name}] | {line}", flush=True)
    if timed_out:
        return f"host command timed out after {HOST_TIMEOUT:g} s: {printable}"
    if child.proc.returncode != 0:
        return f"host command exited {child.proc.returncode}: {printable}"
    return None


def attempt_test(test: Test) -> str | None:
    """Runs one entry once. Returns None on pass or a one-line reason on failure."""
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


# The debug probe sometimes refuses the next session right after the previous one released
# it. probe-rs reports that as a failure to open, before the firmware has run at all, and it
# clears by itself, so it is worth another attempt rather than a verdict on the test.
PROBE_NOT_READY = (
    "Failed to open the debug probe",
    "bulk write timed out",
    "endpoint stalled",
    "Access to the probe was denied",
)
PROBE_RETRIES = 2
PROBE_RETRY_SETTLE = 4.0


def run_test(test: Test) -> str | None:
    """Runs one entry, retrying while only the probe is at fault."""
    for attempt in range(PROBE_RETRIES + 1):
        failure = attempt_test(test)
        if failure is None or not any(marker in failure for marker in PROBE_NOT_READY):
            return failure
        if attempt < PROBE_RETRIES:
            print(f"  [{test.name}] probe not ready: {failure}", flush=True)
            print(f"  [{test.name}] retrying in {PROBE_RETRY_SETTLE:g} s", flush=True)
            time.sleep(PROBE_RETRY_SETTLE)
    return failure


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

    if UV is None:
        sys.exit("error: `uv` is not on PATH; the host peers run through `uv run --script`")

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
