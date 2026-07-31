# LPC55 USB HIL tests

Hardware-in-the-loop tests for the `embassy-nxp` LPC55 USB device driver, run on an
LPCXpresso55S69 EVK (LPC55S69JBD100).

Each firmware prints `Test OK` and halts on a breakpoint on success, and panics via `defmt` on
failure. Tests that need a host peer print a fixed ready banner first and are driven by a
script in [`host/`](host).

## Cabling

- The onboard SEGGER J-Link (debug header) must be connected to the host — `probe-rs` flashes
  and runs over it.
- **P10** (USB0, full speed): host cable required by every test except `alloc` and `bus_raw`.
- **P9** (USB1, high speed): same.
- `usb_alloc` and `usb_bus_raw` need no host cable at all: neither ever brings the bus up.

## Running

```sh
python3 tests/lpc55/run.py              # the whole suite, ~2 minutes
python3 tests/lpc55/run.py --list       # what it will run
python3 tests/lpc55/run.py --only hs_conformance
python3 tests/lpc55/run.py --keep-going # do not stop at the first failure
```

Run it **unprivileged**, so `cargo` and `probe-rs` keep the user's `~/.cargo` and target
directory. The orchestrator elevates only the two conformance host scripts, which need raw USB
access to a vendor interface, as `sudo -n <this interpreter> host/conformance.py …`. Passwordless
`sudo` is therefore required for the conformance tests; everything else runs as the user
(the CDC-based tests only need `/dev/ttyACM*`, which is `root:dialout`).

Individual firmwares still run standalone:

```sh
cd tests/lpc55 && cargo run --release --bin usb_alloc
```

| Test | Firmware | Host peer | What it proves |
|---|---|---|---|
| `alloc` | `usb_alloc` | — | Max-packet-size caps per type, direction and instance, including the 1023-byte high-speed iso-IN cap from erratum USB.6; the multi-packet to single-packet slot fallback; endpoint-index exhaustion; control-endpoint rejection. Both controllers, no cable. |
| `bus_raw` | `usb_bus_raw` | — | The `Bus` methods `embassy-usb` never calls: stall/unstall round-trip, iso endpoints never stalling, `disable` gating the clocks and powering down the PHY, a second `enable` taking the `reinit` path, and `force_reset`. Both controllers, no cable. |
| `fs_enumerate` | `usb_fs_enumerate` | — | USB0 reaches `Configured` with a non-zero device address within 10 s under `Config::default()`, which proves the full-speed driver brings up its own 48 MHz clock. |
| `hs_enumerate` | `usb_hs_enumerate` | — | The same for USB1 under `MainClock::FroHf96`, plus `DEVCMDSTAT.SPEED == 10b`, so a silent fallback to full speed on P9 fails instead of passing. |
| `dual_pll0` | `usb_dual_pll0` | `host/dual_echo.py` | Both controllers live at once under `MainClock::Pll0_150M`, high speed asserted on USB1, and a concurrent CDC-ACM echo on both ports driven from two host threads. |
| `fs_conformance` | `usb_fs_conformance` | `host/conformance.py` | See below. Full speed. |
| `hs_conformance` | `usb_hs_conformance` | `host/conformance.py` | See below. High speed. |
| `fs_throughput` | `examples/lpc55s69` `usb_fs_throughput` | `scripts/usb_throughput.py` | CDC-ACM bulk throughput with a pass/fail floor, both directions. |
| `hs_throughput` | `examples/lpc55s69` `usb_hs_throughput` | `scripts/usb_throughput.py` | The same at high speed. |

## Conformance test

`usb_{hs,fs}_conformance` expose one vendor interface with two alternate settings — alt 0 has
bulk OUT, bulk IN and interrupt IN, alt 1 has isochronous OUT and IN — plus a vendor control
protocol the host uses to set a mode, trigger single transfers and read back device-side
counters. The firmware is one generic implementation in [`src/conformance.rs`](src/conformance.rs)
parameterised per controller, so the two runs are directly comparable.

`host/conformance.py` owns every host-observable expectation and runs these phases in order:

1. **descriptors** — negotiated link speed from sysfs (480 or 12 Mbps), endpoint packet sizes,
   two alternate settings on interface 0.
2. **control_data** — vendor control echo at 0, 1, 63, 64, 65 and 200 bytes, covering the
   multi-packet `ControlPipe::data_out` loop, the exact-EP0-packet boundary and a request with
   no data stage.
3. **control_reject** — an unsupported vendor request must STALL, and the next request must
   succeed, proving `ControlPipe::setup` cleared the EP0 stall.
4. **bulk_echo** — round-trip at 0, 1, 7, 8, 63, 64, 65, mps−1, mps, mps+1 and 3·mps bytes.
   The odd lengths cover erratum USB.5 on high speed, the multiples of mps cover zero-length
   packet termination, and 0 must round-trip a ZLP.
5. **bulk_sink** — a sustained multi-packet OUT stream checked against a stream-offset ramp.
6. **multi_packet_in** — a device-driven IN stream, which on high speed is one 3584-byte
   hardware-packetized slot write plus a 512-byte one.
7. **halt** — `SET_FEATURE(ENDPOINT_HALT)` on bulk IN with a reply already armed (the EPSKIP
   reclaim path) and on bulk OUT with a read pending; `GET_STATUS` in both states; a transfer
   that must STALL on the wire; `clear_halt`; and an exact echo afterwards, which is the
   data-toggle-reset proof.
8. **configuration_cycle** — `SET_CONFIGURATION(0)` must surface as `Disabled` to the endpoint
   tasks, and `SET_CONFIGURATION(1)` must bring the endpoints back.
9. **alt_and_iso** — switching to alt setting 1 must disable the bulk endpoints, `GET_INTERFACE`
   must report 1, then 128 isochronous OUT packets and 128 isochronous IN packets each way with
   an 80 % delivery floor and a per-packet payload check, then a return to alt 0 and an exact
   bulk echo.
10. **interrupt_in** — four triggered interrupt IN packets of 1, 8, 15 and 16 bytes.

The device then checks what only it can see — no payload mismatches, no buffer overflows, at
least two `Disabled` reports, traffic in both bulk directions, exactly four interrupt packets —
and prints `Test OK`.

Both host scripts and the firmware use the same ramp as
`examples/lpc55s69/scripts/usb_throughput.py`: byte `k` of a payload is `(k % 512) as u8`.

Full speed runs its isochronous endpoints at **512** bytes rather than the 1023-byte full-speed
maximum. Two 1023-byte isochronous endpoints do not fit the 1 ms frame's 90 % periodic budget,
and a 1023-byte full-speed isochronous IN through a high-speed hub's transaction translator only
survives one packet per URB on a typical host — multi-packet URBs come back truncated. The
1023-byte allocation boundary itself is covered by `usb_alloc` on both controllers.

## Throughput reference and gate

Measured on an LPC55S69JBD100 with `examples/lpc55s69/scripts/usb_throughput.py`, CDC-ACM bulk,
**release builds**:

| Controller | IN (device to host) | OUT (host to device) | Gate (IN / OUT) |
|------------|---------------------|----------------------|-----------------|
| USB0 (FS)  | 0.90 MB/s           | 0.82 MB/s            | 0.67 / 0.61     |
| USBHSD (HS)| 44.5 MB/s           | 17.5 MB/s            | 33.0 / 13.0     |

The gate is roughly 75 % of the measured figure, low enough to absorb host jitter and high
enough to catch a real regression. `run.py` enforces it with the script's `--min-rate` option.

FS is ~74 % of the 1.216 MB/s full-speed bulk ceiling (19 packets x 64 B per 1 ms frame).
HS IN writes a whole 3584-byte bulk slot per call, which hardware packetizes; HS OUT reads one
packet per call because [`EndpointOut::read`] owes its caller a single packet, so it pays a
per-packet turnaround.

A `dev`-profile build measures roughly 0.3 MB/s in both directions on either controller. That
is the unoptimized driver and class layer being CPU-bound, not the bus, and it is two orders of
magnitude below the hardware — always measure with `--release`.

## Not covered

- **Suspend and resume**, including the `enautoclr_phy_pwd` resume workaround. Needs the host to
  suspend the bus on demand, which no unprivileged sysfs knob exposes reliably.
- **Remote wakeup** (`Bus::remote_wakeup`). Only observable out of a host-driven suspend, so it
  inherits the same blocker.
- **`Event::PowerDetected` / `PowerRemoved`**. Needs VBUS to physically drop, i.e. someone
  unplugging P9 or P10 mid-run.
- **`Memory::usb1_sram` double-take and the `Memory::buffer` size and alignment asserts.** These
  are panicking paths with no non-destructive assertion available; a test for them would have to
  assert that the firmware panicked.

[`EndpointOut::read`]: https://docs.rs/embassy-usb-driver/latest/embassy_usb_driver/trait.EndpointOut.html#tymethod.read
