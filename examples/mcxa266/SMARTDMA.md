# SmartDMA waveform bring-up

`smartdma-waveform` assembles an application-authored program, loads it into
caller-owned static SRAM, initializes its stack/parameters and drives **P0_4 /
SmartDMA PIO0** independently of the CPU. This is not a vendor firmware blob or
a WS2812 driver. The CPU logs the engine PC/SP/mailbox and samples the pad input.

## Connect the analyzer

Disconnect camera modules and shields that could drive the signal. Connect only
a **high-impedance, 3.3 V-compatible analyzer input** and common ground. Do not
connect an analyzer output, 5 V signal, or a powered LED strip to the pin.

| Board | Signal | Ground | Wiring evidence |
|---|---|---|---|
| FRDM-MCXA266 | Camera **J9 pin4**, D0 / P0_4 | **J9 pin18** | SPF-95931, header sheet12 |
| FRDM-MCXA287 | **P0_4 / camera D0** | Board GND | MCU routing verified; exact board connector numbering not independently established |
| FRDM-MCXA577 | Camera **J9 pin4**, D0 / P0_4 | **J9 pin18** | SPF-95896_A1, header sheet13; build-only in this setup |

Do not assume the577 schematic is the287 schematic. Identify the287 board's
P0_4/camera-D0 net from its own pinout/silkscreen before connecting. The266
connector/pin numbers above are established by its actual schematic.

Use a rising-edge trigger and at least 10 MS/s. Capture several milliseconds
first, then a longer trace to check continuity. Expected steady state:

- Nominal **5 kHz** square wave, approximately **200 us period**.
- Approximately **100 us high and 100 us low**.
- The raw heartbeat reload is `system_clock_hz / 10000`, logged at startup.
  Reload versus reload+1 counting remains an external-measurement question: the
  difference is one system-clock period, not an assumed exact timing formula.
- Check the first pulse after reset as well as steady-state edges. No glitch-free
  mux/start transition is promised. No bus-contention timing guarantee is claimed.

CPU mailbox progress and high/low input samples show execution/pad activity;
they do **not** substitute for an external waveform measurement or prove a stop.

## Build and run

From the chosen example directory:

```powershell
cargo build --release --bin smartdma-waveform
```

**Before every replacement flash after running this example**, hold ISP, tap
RESET, then release ISP. This performs a device reset and leaves the boot ROM
idle rather than immediately restarting the old SmartDMA program. A CPU-only
debug reset or a plain power cycle followed by normal application boot is not a
memory-reclamation protocol. Attach-under-reset failed on the current266 probe;
do not compensate with erase/security options.

Rediscover probes with `probe-rs list`; the selectors below identify the attached
boards, not enumeration indices. From `examples/mcxa266`:

```powershell
probe-rs run --chip MCXA266 --probe 1fc9:0143-0:ID4RKCHHUVIX2 --protocol swd --speed 12000 --preverify --verify target/thumbv8m.main-none-eabihf/release/smartdma-waveform
```

From `examples/mcxa287`:

```powershell
probe-rs run --chip MCXA287 --probe 1fc9:0143-0:3WV4BKH0CADZA --protocol swd --speed 12000 --preverify --verify target/thumbv8m.main-none-eabihf/release/smartdma-waveform
```

The firmware intentionally runs indefinitely. Stopping the host runner is not a
SmartDMA stop operation. Preserve the running image/resources until device reset.
Do not use mass erase, provisioning, lifecycle or permission changes.

## Program and safety model

The `.ezh` source has an explicit leading `nop`, then reads ARM2EZH at peripheral
offset `0xe040`, clears both low control bits, and loads internal SP from parameter
word0. The supplied stack grows upward. Parameter words1/2 hold the pin mask and
raw heartbeat reload. The loop toggles only the owned PIO bit and updates the
EZH2ARM mailbox. The assembler inserts no hidden prologue or delay instructions.

The launch ABI requires the explicit NOP, following NXP's AN14650 bootstrap
example (§4.2.13). A first bring-up without it produced incorrect SP/parameters;
do not treat that run as valid timing evidence. The NOP form is a conservative
software ABI requirement, not a claim that the full hardware startup pipeline
has been characterized.

Generic launch is **unsafe**. Code, stack, parameters, pins and the engine clock
vote are retained permanently, including when a running handle is dropped or
forgotten. There is no stop, restart, unload, async cancellation, safe-low fault
recovery or reclamation API yet. A notification/IRQ, stable PC, HOLD or peripheral
reset is not asserted to drain outstanding AHB accesses. This conservative first
implementation avoids relying on Drop for memory safety.

The loader checks metadata and ordinary-SRAM placement before copying/relocating.
The `0x20000000..0x20080000` allowed aperture does not imply every chip has512KiB;
valid Rust storage and the selected application's linker script determine actual
capacity. Existing memory permissions/cache attributes must already be suitable.
No SRAM_XEN, AHBSC, GLIKEY or other security configuration is changed.
