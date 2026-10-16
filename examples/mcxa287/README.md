# FRDM-MCXA287

FRDM-MCXA287 examples using `embassy-mcxa`.

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

## FlexSPI

The board has an 8 MiB W25Q64 (JEDEC EF 40 17).
Chip select is P3_0, clock P3_7, DQS P3_6, and data P3_8..P3_11.

These examples erase/program external flash. Back up data first.
Ranges below are flash offsets, with exclusive ends.

| Examples | Modified range |
| --- | --- |
| `flexspi-quad-{polling,interrupt,dma3}-transfer` | `0x003e8000..0x003ec000` |
| `flexspi-quad-memory-mapped` | `0x00400000..0x00401000` |
| `flexspi-rom` | `0x003e0000..0x003e2000` |
| `flexspi-stress` | `0x007d0000..0x007d6000` |
| `flexspi-soak`, `flexspi-cancel-soak` | Across the device |

Transfer, memory-mapped, ROM, and stress examples passed hardware tests.
The two soak examples are compile-tested only.
