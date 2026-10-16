# FRDM-MCXA287

Compile-tested only. Not tested on hardware.

Pins follow NXP's [board configuration](https://github.com/nxp-mcuxpresso/mcuxsdk-examples/blob/main/_boards/frdmmcxa287/project_template/pin_mux.c).

| Function | Pin |
| --- | --- |
| Red LED, active-low | P2_14 |
| Green LED, active-low | P2_22 |
| Blue LED, active-low | P2_23 |
| SW2 | P3_17 |
| LPUART1 TX | P1_9 |
| LPUART1 RX | P1_8 |

UART examples use the MCU-Link console at 115200 baud, 8N1, without flow control.
`lpuart_dma` receives exactly 16 bytes before echoing.

From this directory:

```sh
cargo build --bins
cargo build --release --bins
cargo run --release --bin blinky
```

Running requires `flip-link` and probe-rs with MCXA287 support.
The linker uses 2 MiB flash and 512 KiB SRAM. SRAMX is unused.
