# STM32N6 Examples

Simple standalone examples for the STM32N6570-DK and NUCLEO-N657X0-Q, primarily intended for dev mode — loaded directly to RAM via probe-rs with no flash boot required.

For a full two-stage boot system with firmware updates from external flash, see [stm32n6-flashboot](../stm32n6-flashboot/).

## NeoChrom (NemaGFX)

The `neochrom` and `neochrom_lcd` examples use [`embassy-stm32-neochrom`](../../embassy-stm32-neochrom),
which pulls its NemaGFX bindings from `stm32-bindings` on crates.io — no local generation step needed.
See also ST's [`x-cube-image-processing`](https://github.com/STMicroelectronics/x-cube-image-processing)
reference for the STM32N6570-DK.

CI builds use the default `stub-gpu2d` feature (link-only HAL stub, no hardware needed). Both examples
are hardware-validated on the STM32N6570-DK; for real GPU2D:

```bash
cargo run --release --bin neochrom --no-default-features
cargo run --release --bin neochrom_lcd --no-default-features
cargo run --release --bin neochrom_graphics --no-default-features
```

- `neochrom` — exercises fill, line, circle, and triangle APIs on a 64×64 RGBA8888 buffer.
- `neochrom_lcd` — GPU-renders into double-buffered 800×480 RGB565 LTDC framebuffers in AXISRAM, driving the panel end-to-end.
- `neochrom_graphics` — full 60 FPS dashboard: a `embedded-3dgfx` scene rasterized through the GPU2D, `embedded-gui` widgets, and a GPU-blitted text atlas. Runs at ~152 µs of GPU time per frame.

## Neural-ART NPU

- `npu_mobilenet` — **real** NPU bring-up. Runs an ST Edge AI compiled MobileNetV1-0.25
  network (single hardware epoch) through `embassy_stm32::npu`, on a fixed public-domain
  Mexican marigold input (`npu-model/mexican_marigold_96x96.rgb`). The trailing *software*
  epoch blocks (Softmax, Dequantize) that ST's compiler emits are reproduced with
  `embedded-nn` kernels, exposed as `embassy_stm32::npu::epoch` behind the `npu-nn` feature.
- Full end-to-end inference (weights streamed from external flash, hardware NPU epoch, CPU
  softmax/dequantize/argmax) takes ~320 ms, ~150 ms of which is the NPU epoch itself. Two clock
  configs make that possible: PLL1 at 800 MHz CPU / 200 MHz system bus (same config as
  `neochrom_lcd`), and — the dominant factor by far — Octal DTR (8-line, double-transfer-rate)
  reads from the external NOR instead of a single-lane `FASTREAD`, matching ST's own
  `stm32n6570_discovery_xspi.c` bring-up. Without the Octal DTR switch this runs in ~9.6 s
  regardless of CPU/NPU clock, since streaming this model's ~7.9 MB of weights over a slow,
  single-lane, non-DTR link dominates everything else.
- The compiled model classifies flowers (101 classes, likely Oxford 102 Flowers minus one
  unused class — see `STAI_NETWORK_OUT_1_SIZE` in `Model/STM32N6570-DK/stai_network.h`); the
  input is a flower so the logits actually carry signal. There's no golden-reference check
  against a host interpreter: the `.tflite` published alongside this compiled blob in ST's
  repo (`Model/mobilenet_v1_0.25_96_tfs_int8.tflite`) is a *different*, 5-class model (confirmed
  via `embedded-nn-tflite`: input `[1,96,96,3]` uint8, output `[1,5]` float32) — the repo ships
  two unversioned, mismatched models under the same demo, and no 101-class `.tflite` is
  published to validate against instead. See `GOLDEN_LOGITS` in `npu_mobilenet.rs`.

ST publishes the compiled model under SLA0044, so it is not vendored here. `build.rs` imports
it from a checkout of ST's public GettingStarted repository and the example is gated behind the
`npu-model` feature. Hardware-validated on the STM32N6570-DK; the Nucleo path is untested but
follows the same steps with `board-nucleo` swapped in throughout.

### 1. Get the model

```bash
git clone --depth 1 https://github.com/STMicroelectronics/STM32N6-GettingStarted-ImageClassification
# Match the board dir to the feature you'll build with below (DK is the default feature).
export STM32N6_GETTINGSTARTED_MODEL_DIR=$PWD/STM32N6-GettingStarted-ImageClassification/Model/STM32N6570-DK
```

For the Nucleo, use `Model/NUCLEO-N657X0-Q` instead.

### 2. Program the model weights into external flash (one-time)

The example reads network weights from the board's external xSPI2 NOR at runtime — `cargo run`
only loads the firmware itself into RAM (dev mode), it does not program external flash. Do this
once per board, via [STM32CubeProgrammer](https://www.st.com/en/development-tools/stm32cubeprog.html)
(not the same tool as `probe-rs`/`cargo run` below — install it separately):

```bash
# Adjust to your STM32CubeProgrammer install location.
CLI="<STM32CubeProgrammer_N6 Install Folder>/bin/STM32_Programmer_CLI"
DKEL="<STM32CubeProgrammer_N6 Install Folder>/bin/ExternalLoader/MX66UW1G45G_STM32N6570-DK.stldr"

"$CLI" -c port=SWD mode=HOTPLUG -el "$DKEL" -hardRst \
    -w "$STM32N6_GETTINGSTARTED_MODEL_DIR/network_data.hex"
```

For the Nucleo, use `MX25UM51245G_STM32N6570-NUCLEO.stldr` instead. This only needs redoing if
you erase/reflash that flash region or swap to a different model. If the erase/write fails with
`Init function fail` (the external loader itself never gets going), fully power-cycle the board
first — this chip's PMIC is sequenced by the boot ROM at power-on, and a `-hardRst` (NRST pulse)
alone doesn't redo that negotiation.

### 3. Build and run

```bash
# Board defaults to the STM32N6570-DK; use --no-default-features --features npu-model,board-nucleo for the Nucleo.
cargo run --features npu-model --bin npu_mobilenet
```

The example verifies the weights were actually programmed (reading back a known value at
`0x7038_0000`) and prints a clear error instead of silently misbehaving if step 2 was skipped.
Expect it to end with something like:

```text
NPU epoch complete in ~120_000_000 cycles (~150 ms at 800 MHz CPU clock)
NPU logits (i8): [...]
probabilities : [...]
=> class 55 (argmax i8 score 47), p(class) = 0.671875
```

(Exact logits/probabilities vary slightly run-to-run — quantization noise, not a bug — but the
winning class and its confidence stay consistent.)

The differences between the two boards (power supply topology, NOR size) are selected by the
`board-dk` / `board-nucleo` features.

### Troubleshooting

- **Board must be in development mode** (boot switches `BOOT0`/`BOOT1` — see "Boot Modes" in
  ST's repo README, or `Doc/Boot-Overview.md`) — same requirement for `cargo run`'s RAM load
  and for step 2's flash programming.
- **`probe-rs` reports `SwdApFault`, or a hard fault at reset with `PC=0` before any log
  output**: power-cycle the board (unplug/replug, don't just hit reset). This chip's debug port
  can get wedged after certain reset sequences and a real power-on-reset clears it.
- **`probe-rs` reports `Target voltage (VAPP) is 0.00 V`**: the board itself isn't powered —
  check the power switch/cable, not the debug connection.
- **The firmware hangs during the `octal DTR switch: ...` log lines** (no further output,
  `probe-rs` doesn't time out on its own): power-cycle the board. The flash's protocol-select
  bit (`CR2` register 1) is volatile, so a power-cycle also resets it back to default SPI mode;
  a plain reset does not reliably do the same. This has been observed once, transiently, and
  ran cleanly on every attempt afterward — if it recurs consistently rather than clearing on
  power-cycle, that would point at a real bug in the mode-switch sequence
  (`switch_flash_to_octal_dtr` in `npu_mobilenet.rs`), not just a one-off.

