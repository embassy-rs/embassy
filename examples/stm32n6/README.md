# STM32N6 Examples

Simple standalone examples for the STM32N6570-DK and NUCLEO-N657X0-Q, primarily intended for dev mode — loaded directly to RAM via probe-rs with no flash boot required.

For a full two-stage boot system with firmware updates from external flash, see [stm32n6-flashboot](../stm32n6-flashboot/).

## NeoChrom (NemaGFX)

The `neochrom` and `neochrom_lcd` examples use [`embassy-stm32-neochrom`](../../embassy-stm32-neochrom).
See also ST's [`x-cube-image-processing`](https://github.com/STMicroelectronics/x-cube-image-processing)
reference for the STM32N6570-DK. Generate `stm32-bindings` first:

```bash
cd ../../../stm32-bindings
cargo run --release --bin stm32-bindings-gen -- --module nema_gfx
cd ../embassy/examples/stm32n6
```

CI builds use the default `stub-gpu2d` feature (link-only HAL stub). For real GPU2D on the STM32N6570-DK:

```bash
cargo run --release --bin neochrom --no-default-features
cargo run --release --bin neochrom_lcd --no-default-features
```

- `neochrom` — exercises fill, line, circle, and triangle APIs on a 64×64 RGBA8888 buffer.
- `neochrom_lcd` — GPU-renders into double-buffered 800×480 RGB565 LTDC framebuffers in AXISRAM.
