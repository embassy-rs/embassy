# FRDM-MCXA156 examples

There are two samples for the FRDM-MCXA156:

- `blinky`: This toggles the active-low green LED on P3_13 every 500 ms.
- `vcom_echo`: This uses LPUART0 on P0_3 (TX) and P0_2 (RX) at
  115200 8-N-1 without flow control. This UART is wired through the onboard
  debugger's virtual COM port. At boot it prints
  `FRDM-MCXA156 embassy serial ready\r\n`, then echoes received bytes.

From this directory, build both examples:

```text
cargo build --release --bins
```

Connect the board's onboard debug probe and confirm the correct probe/target
before flashing.

If you have multiple debug probes, use `probe-rs list` to identify the board
and pass its full USB selector:

```text
cargo run --locked --release --bin blinky -- --probe <VID:PID:SERIAL>
cargo run --locked --release --bin vcom_echo -- --probe <VID:PID:SERIAL>
```

These samples were tested on the FRDM-MCXA156 PCB A1 / schematic A3
(stickers on the back of the PCB), with J-Link firmware on the onboard
debug interface.
