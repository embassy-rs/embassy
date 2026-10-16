# FRDM-MCXA156 examples

These examples select the exact `mcxa156` feature and the explicit OSTIMER
time driver. Both HAL PAC dependencies use upstream `embassy-rs/nxp-pac`
revision `52488ea8ddf6c19d6a84a48d906b2bb48c4193bf`, containing the merged
MCXA156 MetaPAC. No local PAC path or Wavenumber PAC fork is needed.

From this directory, build both examples:

```text
cargo build --release --bins
```

Run here so Cargo reads `.cargo/config.toml`, including the target, linker
scripts, and flash runner. After the first resolution, use `--locked` for
repeat builds. The repository ignores Cargo locks; retain the resolved lock
and binary identity with hardware evidence.

Connect the board's onboard debug probe and confirm the correct probe/target
before flashing. Enumerate with `probe-rs list`, identify the board, and pass
its full USB selector explicitly when several probes are connected:

```text
cargo run --locked --release --bin blinky -- --probe <VID:PID:SERIAL>
cargo run --locked --release --bin vcom_echo -- --probe <VID:PID:SERIAL>
```

- `blinky`: green LED P3_13 is active low, alternating 500 ms on / 500 ms off.
- `vcom_echo`: LPUART0 uses P0_3 TX and P0_2 RX, alternate function 2, at
  115200 8-N-1 without flow control. The probe's serial interface emits
  `FRDM-MCXA156 embassy serial ready\r\n` and echoes received bytes.

This console is UART through the debug probe, not native MCU USB CDC.
Verify exact echo of `MCXA156 VCOM echo 0123456789\r\n`, then repeat both
examples after warm reset and full power removal. Record board/probe/host,
commands, source/lock/binary hashes, and raw observations. Confirm all power
paths are removed for the cold-start check.

The initial HAL keeps the 12 MHz reset system clock and supports GPIO,
OSTIMER, and blocking LPUART0. Other clock operating points, native USB, the
LPTMR time backend, and the HAL low-power executor are not implemented.

The package enables `swd-swo-as-gpio` and `jtag-extras-as-gpio` for the UART
pins P0_2/P0_3. This also releases the package-dependent ISP pins P0_6/P3_29;
applications that need ROM ISP entry on reset should account for that choice.
SWDIO/SWCLK and reset remain reserved, so the examples keep probe access.
See the HAL README for all dedicated-pin opt-ins and package-union limits.

The examples have been exercised on FRDM-MCXA156 PCB A1 / schematic A3.
A build alone does not validate wiring or behavior; repeat the checks above
when changing the board configuration or runtime implementation.
