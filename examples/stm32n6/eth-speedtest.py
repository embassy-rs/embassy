#!/usr/bin/env python3
"""TCP throughput tester for the stm32n6 `eth_speedtest.rs` example.

The board runs a TCP server (default 192.168.137.2:8000). This utility connects,
sends a one-byte mode selector, then streams/receives data and reports the rate.
The receiving side's number is the authoritative throughput; for `rx` the board
prints its measured rate over the debug probe (RTT/defmt).

Modes:
  tx     board transmits, PC receives    -> this tool reports board->PC rate
  rx     board receives, PC transmits    -> board reports rate (RTT); this tool
                                            prints the PC->board offered rate
  bidir  both directions at once         -> reports both
  all    runs tx, rx, bidir in succession (separate connection each)

Direct PC<->board cable: set the PC adapter to 192.168.137.1 / 255.255.255.0.

Examples:
  python eth-speedtest.py tx
  python eth-speedtest.py rx --seconds 10
  python eth-speedtest.py all --host 192.168.137.2 --port 8000
"""

import argparse
import socket
import threading
import time

CHUNK = 1 << 18  # 256 KiB per syscall
PAYLOAD = b"\xA5" * CHUNK
MODE_BYTE = {"tx": b"T", "rx": b"R", "bidir": b"B"}


def mbps(nbytes: int, seconds: float) -> float:
    return (nbytes * 8 / seconds / 1e6) if seconds > 0 else 0.0


def report(label: str, nbytes: int, seconds: float) -> None:
    """Print one throughput line, matching the board's defmt format:
    `<direction>: <bytes> bytes in <ms> ms -> <X.XX> Mbit/s`."""
    print(f"{label}: {nbytes} bytes in {int(seconds * 1000)} ms -> {mbps(nbytes, seconds):.2f} Mbit/s")


def recv_until_eof(sock: socket.socket):
    """Read until the peer closes. Returns (bytes, seconds-from-first-byte)."""
    total = 0
    start = None
    buf = bytearray(CHUNK)
    while True:
        n = sock.recv_into(buf)
        if n == 0:
            break
        if start is None:
            start = time.perf_counter()
        total += n
    elapsed = (time.perf_counter() - start) if start is not None else 0.0
    return total, elapsed


def blast_for(sock: socket.socket, seconds: float):
    """Send as fast as possible for `seconds`, then half-close. Returns
    (bytes, seconds)."""
    total = 0
    start = time.perf_counter()
    deadline = start + seconds
    while time.perf_counter() < deadline:
        total += sock.send(PAYLOAD)
    elapsed = time.perf_counter() - start
    try:
        sock.shutdown(socket.SHUT_WR)  # FIN so the board sees EOF
    except OSError:
        pass
    return total, elapsed


def run_test(mode: str, host: str, port: int, seconds: float) -> int:
    """Open a fresh connection, run one test, report. Returns 0 on success."""
    print(f"\n=== {mode} : connecting to {host}:{port} ===")
    try:
        sock = socket.create_connection((host, port), timeout=10)
    except OSError as e:
        print(f"connect failed: {e}")
        return 1

    sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_SNDBUF, 1 << 20)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 1 << 20)
    except OSError:
        pass
    sock.settimeout(seconds + 30)

    sock.sendall(MODE_BYTE[mode])

    try:
        if mode == "tx":
            # Board transmits; we receive until it closes.
            nbytes, secs = recv_until_eof(sock)
            report("board->PC", nbytes, secs)

        elif mode == "rx":
            # We transmit; the board measures and reports over RTT.
            nbytes, secs = blast_for(sock, seconds)
            report("PC->board (offered)", nbytes, secs)
            print("  (board prints the authoritative PC->board rate over the debug probe)")
            # Wait for the board's FIN before exiting.
            recv_until_eof(sock)

        else:  # bidir
            tx_result = {}

            def sender():
                tx_result["v"] = blast_for(sock, seconds)

            t = threading.Thread(target=sender, daemon=True)
            t.start()
            rx_bytes, rx_secs = recv_until_eof(sock)
            t.join()
            tx_bytes, tx_secs = tx_result.get("v", (0, 0.0))
            report("PC->board (offered)", tx_bytes, tx_secs)
            report("board->PC", rx_bytes, rx_secs)
    except OSError as e:
        print(f"socket error during test: {e}")
        return 1
    finally:
        sock.close()

    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mode", choices=list(MODE_BYTE.keys()) + ["all"])
    ap.add_argument("--host", default="192.168.137.2", help="board IP (default: %(default)s)")
    ap.add_argument("--port", type=int, default=8000, help="board port (default: %(default)s)")
    ap.add_argument(
        "--seconds",
        type=float,
        default=10.0,
        help="PC send window for rx/bidir; should match the board's DURATION (default: %(default)s)",
    )
    args = ap.parse_args()

    modes = ["tx", "rx", "bidir"] if args.mode == "all" else [args.mode]

    rc = 0
    for i, mode in enumerate(modes):
        if i > 0:
            time.sleep(1.0)  # let the board abort, flush, and re-listen
        rc |= run_test(mode, args.host, args.port, args.seconds)

    return rc


if __name__ == "__main__":
    raise SystemExit(main())
