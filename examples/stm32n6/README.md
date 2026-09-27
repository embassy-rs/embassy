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
- The model classifies `daisy, dandelion, rose, sunflower, tulip`; the input is a flower so
  the logits actually carry signal. The NPU logits are checked against `embedded-nn`'s host
  interpreter running the model's public `.tflite` source (tolerance 2 LSB), and that
  reference is itself validated bit-for-bit against Google's official TFLite runtime
  (`ai-edge-litert`).

ST publishes the compiled model under SLA0044, so it is not vendored here. `build.rs` imports
it from a checkout of ST's public GettingStarted repository and the example is gated behind the
`npu-model` feature:

```bash
git clone --depth 1 https://github.com/STMicroelectronics/STM32N6-GettingStarted-ImageClassification
export STM32N6_GETTINGSTARTED_MODEL_DIR=$PWD/STM32N6-GettingStarted-ImageClassification/Model/NUCLEO-N657X0-Q

# Board defaults to the STM32N6570-DK; use --no-default-features --features npu-model,board-nucleo for the Nucleo.
cargo build --features npu-model --bin npu_mobilenet
```

The example expects `network_data.xSPI2.bin` (from the same repository) to have been programmed
at absolute xSPI2 address `0x7038_0000`, as ST's own README describes; it verifies this at run
time and prints a clear error otherwise. The remaining differences between the two boards
(power supply topology, NOR size) are selected by the `board-dk` / `board-nucleo` features.

