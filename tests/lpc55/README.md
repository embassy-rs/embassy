# LPC55 USB HIL tests

These hardware-in-the-loop tests cover the `embassy-nxp` LPC55 USB device driver on an
LPCXpresso55S69 EVK (LPC55S69JBD100).

Each firmware prints `Test OK` and stops at a breakpoint on success. On failure, it panics through
`defmt`. A host-driven firmware first prints a fixed ready banner, then a script from
[`host/`](host) runs its USB peer.

## Cabling

- Connect a debug probe from the host to the SWD header. `probe-rs` flashes and runs through it.
  The onboard LPC-LINK2 (CMSIS-DAP) and external probes such as the SEGGER J-Link both work.
  On Windows, `probe-rs` opens the probe through WinUSB. The onboard LPC-LINK2 brings its own,
  while a J-Link normally arrives bound to SEGGER's driver, which probe-rs refuses with
  `incompatible driver is installed`. [Zadig](https://zadig.akeo.ie/) rebinds it to WinUSB,
  after which SEGGER's own tools no longer see that probe.
- Connect **P10** for USB0 full-speed tests and **P9** for USB1 high-speed tests.
- The `alloc`, `alloc_small`, and `bus_raw` tests need no host cable. Their firmware never starts
  either USB bus.

A charge-only cable can look like a driver fault. It powers the board, the firmware arms the bus,
and the host sees no device. Read `DEVCMDSTAT` through the probe to identify this fault.
`VBUS_DEBOUNCED=1`, `DCON=1`, and `DEV_ADDR=0` mean that the device is attached and waiting.
In that state, the cable or host port caused the fault.

## Running

```sh
python3 tests/lpc55/run.py              # the whole suite, ~2 minutes
python3 tests/lpc55/run.py --list       # what it will run
python3 tests/lpc55/run.py --only hs_conformance
python3 tests/lpc55/run.py --keep-going # do not stop at the first failure
```

The orchestrator itself needs only the standard library. Every host peer declares its
dependencies inline (PEP 723) and runs through `uv run --script`, so `uv` has to be on `PATH`
and nothing else has to be installed.

Run the suite as an unprivileged user. This keeps `cargo` and `probe-rs` on the user's `~/.cargo`
and target directories.

- **Linux**: the two conformance entries need raw USB access to a vendor interface, so those two
  and only those run as `sudo -n <uv> run --script host/conformance.py …`, which means
  passwordless `sudo` is required. The CDC tests need access to `/dev/ttyACM*`, normally owned by
  `root:dialout`.
- **Windows**: nothing is elevated. The conformance firmware advertises WinUSB, which Windows
  binds by itself and an ordinary user can open.

Individual firmwares still run standalone:

```sh
cd tests/lpc55 && cargo run --release --bin usb_alloc
```

| Test | Firmware | Host peer | What it proves |
|---|---|---|---|
| `alloc` | `usb_alloc` | None | Max-packet-size caps per type, direction, and instance, including the 1023-byte high-speed iso-IN cap from erratum USB.6. It also covers multi-packet to single-packet slot fallback, endpoint-index exhaustion, and control-endpoint rejection. Both controllers, no cable. |
| `alloc_small` | `usb_alloc_small` | None | EP0 stays reserved when exactly 512 bytes can hold only one double-buffered 64-byte data endpoint. USB0, no cable. |
| `bus_raw` | `usb_bus_raw` | None | Covers `Bus` methods that `embassy-usb` does not call: stall/unstall round-trip, iso endpoints never stalling, `disable` clock and PHY gating, the second-`enable` `reinit` path, and `force_reset`. Both controllers, no cable. |
| `fs_enumerate` | `usb_fs_enumerate` | None | USB0 reaches `Configured` with a non-zero device address within 10 s under `Config::default()`, which proves the full-speed driver brings up its own 48 MHz clock. |
| `hs_enumerate` | `usb_hs_enumerate` | None | The same for USB1 under `MainClock::FroHf96`, plus `DEVCMDSTAT.SPEED == 10b`, so a silent fallback to full speed on P9 fails instead of passing. |
| `dual_pll0` | `usb_dual_pll0` | `host/dual_echo.py` | Both controllers live at once under `MainClock::Pll0_150M`, high speed asserted on USB1, and a concurrent CDC-ACM echo on both ports driven from two host threads. |
| `fs_conformance` | `usb_fs_conformance` | `host/conformance.py` | See below. Full speed. |
| `hs_conformance` | `usb_hs_conformance` | `host/conformance.py` | See below. High speed. |
| `fs_throughput` | `examples/lpc55s69` `usb_fs_throughput` | `scripts/usb_throughput.py` | CDC-ACM bulk throughput with a pass/fail floor, both directions. |
| `hs_throughput` | `examples/lpc55s69` `usb_hs_throughput` | `scripts/usb_throughput.py` | The same at high speed. |

## Conformance test

`usb_{hs,fs}_conformance` expose one vendor interface with two alternate settings. Alt 0 has bulk
OUT, bulk IN, and interrupt IN. Alt 1 has isochronous OUT and IN. The host uses a vendor control
protocol to set modes, trigger transfers, and read device counters.

Both controller binaries use the generic implementation in
[`src/conformance.rs`](src/conformance.rs), so their results are directly comparable.

The firmware also carries MS OS 2.0 descriptors advertising WinUSB compatibility. Windows binds
no driver at all to a vendor interface on its own, so without them `libusb` cannot even open the
device; Linux ignores them. One phase still cannot run there: WinUSB rejects `SET_CONFIGURATION`
from user mode, because the hub driver owns the device configuration. `configuration_cycle`
therefore prints `skipped: …` on Windows and runs normally on Linux. The device-side invariant it
contributes to (`disabled_errors >= 2`) is satisfied by the halt and alt-setting phases anyway.

`host/conformance.py` owns every host-observable expectation and runs these phases in order:

1. **descriptors**: negotiated link speed from `libusb_get_device_speed` (480 or 12 Mbps, the
   one speed source Linux and Windows share), endpoint packet sizes, and two alternate settings
   on interface 0.
2. **control_data**: vendor control echo at 0, 1, 63, 64, 65 and 200 bytes, covering the
   multi-packet `ControlPipe::data_out` loop, the exact-EP0-packet boundary, and a request with
   no data stage.
3. **control_reject**: an unsupported vendor request must STALL. The next request must succeed
   to prove that `ControlPipe::setup` cleared the EP0 stall.
4. **bulk_echo**: round-trip at 0, 1, 7, 8, 63, 64, 65, mps−1, mps, mps+1 and 3·mps bytes.
   The odd lengths cover erratum USB.5 on high speed. The mps multiples cover zero-length packet
   termination, and 0 must round-trip a ZLP.
5. **bulk_sink**: a sustained multi-packet OUT stream checked against a stream-offset ramp.
6. **multi_packet_in**: a device-driven IN stream. At high speed, this uses one 3584-byte
   hardware-packetized slot write followed by a 512-byte write.
7. **halt**: `SET_FEATURE(ENDPOINT_HALT)` on bulk IN with a reply already armed (the EPSKIP
   reclaim path), and on bulk OUT with a read pending. The phase checks `GET_STATUS` in both
   states and requires a transfer to STALL on the wire. It then clears the halt and requires
   an exact echo to prove that the data toggle reset.
8. **configuration_cycle**: `SET_CONFIGURATION(0)` must surface as `Disabled` to the endpoint
   tasks, and `SET_CONFIGURATION(1)` must bring the endpoints back. Linux only; see above.
9. **alt_and_iso**: switching to alt setting 1 must disable the bulk endpoints, and
   `GET_INTERFACE` must report 1. The host then streams 128 isochronous OUT packets and reads 128
   isochronous IN packets, one transfer per direction, with an 80 % delivery floor and a
   per-packet payload check. Submitting a transfer per packet instead measures the host's
   isochronous scheduler rather than the driver: WinUSB restarts the pipe at every transfer
   boundary, which both drops frames and replays buffered ones, so the device sees anywhere from
   54 % to 180 % of them. One transfer is byte-exact on both controllers for every size from 1 to
   128 packets, and 128 is also the most isochronous packets Linux accepts in a single URB. The
   phase ends by returning to alt 0 and checking an exact bulk echo.
10. **interrupt_in**: four triggered interrupt IN packets of 1, 8, 15 and 16 bytes.

The device then checks its private state: no payload mismatches, no buffer overflows, at least
two `Disabled` reports, traffic in both bulk directions, and exactly four interrupt packets.
It prints `Test OK` if all checks pass.

Both host scripts and the firmware use the same ramp as
`examples/lpc55s69/scripts/usb_throughput.py`: byte `k` of a payload is `(k % 512) as u8`.

Full speed uses **512-byte** isochronous endpoints instead of the 1023-byte maximum. Two
1023-byte isochronous endpoints exceed the 1 ms frame's 90 % periodic budget.

A high-speed hub's transaction translator limits 1023-byte full-speed isochronous IN transfers
to one packet per URB on a typical host. Multi-packet URBs return truncated data. `usb_alloc`
covers the 1023-byte allocation boundary on both controllers.

## Throughput reference and gate

Measured on an LPC55S69JBD100 with `examples/lpc55s69/scripts/usb_throughput.py`, CDC-ACM bulk,
**release builds**:

| Controller | Host transport | IN (device to host) | OUT (host to device) | Gate (IN / OUT) |
|------------|----------------|---------------------|----------------------|-----------------|
| USB0 (FS)  | Linux `cdc_acm` | 0.90 MB/s | 0.82 MB/s | 0.67 / 0.61 |
| USB0 (FS)  | Windows libusb  | 0.89 MB/s | 0.83 MB/s | 0.67 / 0.61 |
| USBHSD (HS)| Linux `cdc_acm` | 44.5 MB/s | 17.5 MB/s | 33.0 / 13.0 |
| USBHSD (HS)| Windows libusb  | 44.9 MB/s | 19.0 MB/s | 33.0 / 13.0 |

The gate is roughly 75 % of the measured figure, low enough to absorb host jitter and high
enough to catch a real regression. `run.py` enforces it with the script's `--min-rate` option,
and both hosts clear the same numbers.

FS is ~74 % of the 1.216 MB/s full-speed bulk ceiling (19 packets x 64 B per 1 ms frame).
HS IN writes a whole 3584-byte bulk slot per call, which hardware packetizes. HS OUT reads one
packet per call because [`EndpointOut::read`] owes its caller a single packet. This adds a
per-packet turnaround.

A `dev`-profile build reaches roughly 0.3 MB/s in both directions on either controller. The
unoptimized driver and class layer limit this rate, not the bus. This result is two orders of
magnitude below the hardware rate, so always measure with `--release`.

### Why the throughput firmwares are WinUSB devices

`usb_fs_throughput` and `usb_hs_throughput` carry MS OS 2.0 descriptors too, so Windows binds
WinUSB and the script drives their endpoints through `libusb` rather than a COM port. Linux
ignores the descriptors and still binds `cdc_acm`, so the port stays an ordinary CDC node there.

Windows' CDC driver cannot carry the high-speed stream. `usbser.sys` discards data that arrives
while no read request is outstanding, which costs whole 512-byte packets: 512 to 4096 bytes per
4 MB transfer, in roughly half of all attempts, through pyserial, through blocking reads on a
dedicated reader thread, and at every read and receive-buffer size tried. Keeping several
overlapped requests queued does stop the loss, but the same transfer then runs at 0.78 MB/s
instead of 38. Reading the endpoints directly is both loss-free and faster.

The driver is not at fault, and the same firmware shows it twice: read through libusb it
delivered 25 consecutive 4 MB transfers without losing a byte, and in every failing CDC run its
write loop had already handed over all 4,000,000 bytes and was serving the next command.

These two binaries therefore carry their own product ids, `c0de:cb07` (FS) and `c0de:cb08` (HS),
instead of sharing `c0de:cafe` with the serial examples: Windows caches its driver choice per id,
so an id that has ever enumerated as a CDC port keeps `usbser.sys`.

## VBUS attach and detach (checked on Linux)

Manual checks cover `Event::PowerDetected`, `PowerRemoved`, and the attach-armed soft-connect.
No firmware can remove its own VBUS. A hub with per-port power switching makes these checks
repeatable without touching a cable:

```sh
lsusb -t                                                  # locate the board, e.g. bus 1, hub 10, port 3
lsusb -v -d <hub-vid:pid> | grep -A2 wHubCharacteristic   # must say "Per-port power switching"
port=/sys/bus/usb/devices/1-10:1.0/1-10-port3/disable
sudo sh -c "echo 1 > $port"    # VBUS off
sudo sh -c "echo 0 > $port"    # VBUS on
```

A hub without per-port power switching only disables the port and leaves VBUS up, which does
not exercise this path at all.

We checked both controllers with `usb_fs_serial` (P10), `usb_hs_serial` (P9), and literal cable
pulls on `usb_dual`:

- echo, VBUS drop, VBUS return, and echo again (no reflash in between).
- firmware starts with the cable absent and attaches when the cable arrives after `usb.run()`
  starts polling.
- a partial protocol header sent to `usb_{fs,hs}_throughput` does not survive the drop.
  `usb_throughput.py --protocol-check` passes afterwards because the bench loop builds a fresh
  `Parser` every time `wait_connection` returns.

Pull one cable at a time. VBUS powers the board, so removing both cables cuts its supply.
The firmware then restarts instead of riding the disconnect out.

## Not covered

- **Suspend and resume**, including the `enautoclr_phy_pwd` resume workaround. This needs the host
  to suspend the bus on demand, which no unprivileged sysfs control exposes reliably.
- **Remote wakeup** (`Bus::remote_wakeup`). It requires the same host-driven suspend.
- **USBHSD rejecting a full-speed link.** The `USBHSD negotiated unsupported speed` panic needs
  a full-speed-only USB 1.1 hub between P9 and the host. The driver retains the source branch and
  the normal high-speed P9 path, but this panic remains untested on hardware.
- **`Memory::usb1_sram` double-take and the `Memory::buffer` size assert.** These paths panic,
  and no non-destructive assertion is available. A test would have to assert the firmware panic.
- **`SET_CONFIGURATION` from the host on Windows.** WinUSB refuses the request, so
  `configuration_cycle` runs on Linux only.

[`EndpointOut::read`]: https://docs.rs/embassy-usb-driver/latest/embassy_usb_driver/trait.EndpointOut.html#tymethod.read
