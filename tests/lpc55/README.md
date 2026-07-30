# LPC55 USB HIL tests

Hardware-in-the-loop tests for the `embassy-nxp` LPC55 USB device driver, run on an
LPCXpresso55S69 EVK (LPC55S69JBD100).

## Cabling

- The onboard SEGGER J-Link (debug header) must be connected to the host — `probe-rs` flashes
  and runs over it.
- **P10** (USB0, full-speed): host cable required for `usb_fs_enumerate`.
- **P9** (USB1, high-speed): host cable required for `usb_hs_enumerate`.
- `usb_alloc` needs no host cable at all; it only exercises endpoint allocation.

Each test prints `Test OK` and halts on a breakpoint on success, and panics via `defmt` on
failure.

## Running

```sh
cargo run --bin usb_alloc          # no host cable
cargo run --bin usb_fs_enumerate   # host cable on P10
cargo run --bin usb_hs_enumerate   # host cable on P9
```

## Throughput reference

Not part of this crate — `examples/lpc55s69` carries the benches — but recorded here because
the numbers are easy to misread. Measured on an LPC55S69JBD100 with
`scripts/usb_throughput.py`, CDC-ACM bulk, **release builds**:

| Controller | IN (device to host) | OUT (host to device) |
|------------|---------------------|----------------------|
| USB0 (FS)  | 0.90 MB/s           | 0.82 MB/s            |
| USBHSD (HS)| 44.5 MB/s           | 17.5 MB/s            |

FS is ~74 % of the 1.216 MB/s full-speed bulk ceiling (19 packets x 64 B per 1 ms frame).
HS IN writes a whole 3584-byte bulk slot per call, which hardware packetizes; HS OUT reads one
packet per call because [`EndpointOut::read`] owes its caller a single packet, so it pays a
per-packet turnaround.

A `dev`-profile build measures roughly 0.3 MB/s in both directions on either controller. That
is the unoptimized driver and class layer being CPU-bound, not the bus, and it is two orders of
magnitude below the hardware — always measure with `--release`.

[`EndpointOut::read`]: https://docs.rs/embassy-usb-driver/latest/embassy_usb_driver/trait.EndpointOut.html#tymethod.read
