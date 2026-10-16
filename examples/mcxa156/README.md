# FRDM-MCXA156 draft examples

This draft board package selects the exact MCXA156 through the branch's
`mcxa156` feature with the explicit OSTIMER time driver. Both HAL PAC dependencies use upstream `embassy-rs/nxp-pac`
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

The merged-PAC software build is validated independently from those hardware
checks. Current upstream Embassy integration and final contribution readiness
remain tracked by the coordination plan; these examples are still a draft.
