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

## Power

`power-wfe-gated` uses the default executor.
The four `power-deepsleep*` examples require the HAL executor:

```sh
cargo run --release --no-default-features --features executor-platform --bin power-deepsleep
```

All five configurations passed functional timer-wake checks on hardware.
The GPIO example also passed an SW2 interrupt check; SW2 toggles the blue LED.
Current draw, CLKOUT waveforms, and wake latency were not measured.

Red LED pulses last 100 ms, every second (five seconds in the GPIO example).
Gating examples expose CLKOUT on P4_2 and executor activity on P2_3.
Clock gating disconnects the debugger. A startup delay allows recovery;
if the target becomes stuck, reset or enter ISP mode before reflashing.

## Peripheral checks

PWM, FREQME, CRC, TRNG, memory DMA, and ADC temperature passed hardware checks.
LPTMR passed both standalone waits and the `time-driver-lptmr` backend:

```sh
cargo run --release --features time-driver-lptmr --bin lptmr
```

Temperature channel 26 and constants match NXP's MCXA287 SDK.
Temperature accuracy and TRNG entropy quality were not measured.

## RTC

The calendar RTC example needs the board's 32.768 kHz crystal:

```sh
cargo run --release --features unstable-osc32k --bin rtc-alarm
```

It enables OSC32K through the HAL clock configuration before constructing the
RTC, while leaving the default FRO16K enabled. RTC clock selection survives a
system reset; if the previously selected source is stopped, even the clock-select
register is inaccessible. Keeping both sources available allows initialization
from either retained selection. The example also allows the RTC register
interface to synchronize after enabling clocks before accessing the block.

The example selects OSC32K, sets the calendar, reads through a separate time
provider while waiting for an alarm, and checks a minute rollover. It prints
`RTC example complete - Test PASSED!` and stops at a breakpoint if a debugger is
attached. Without a debugger, the task returns normally. This is a functional
check, not a crystal-accuracy or low-power measurement.
