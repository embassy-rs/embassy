# embassy-stm32-neochrom

Embassy integration for ST NeoChrom (GPU2D) using the [NemaGFX](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/Middleware/NemaGFX) middleware via [`stm32-bindings`](https://github.com/embassy-rs/stm32-bindings).

## Status

Early bring-up. Provides a safe driver over the generated NemaGFX FFI and the platform HAL from `nema-gfx-hal`.

## Local development

Generate `stm32-bindings` first:

```bash
cd ../stm32-bindings
cargo run --release --bin stm32-bindings-gen -- --module nema_gfx
```

Then build this crate (example chip feature):

```bash
cd ../embassy/embassy-stm32-neochrom
cargo check --target thumbv8m.main-none-eabihf --features stm32n657x0,stub-gpu2d
```

## Features

| Feature | Description |
|---------|-------------|
| `stm32n657x0` | STM32N657 + Cortex-M55 NemaGFX library |
| `stub-gpu2d` | Link GPU2D HAL stub instead of STM32Cube (default) |
| `neochrom-m55` | NemaGFX bindings + M55 prebuilt library only |
| `embedded-graphics` | `NeoChromTarget` DrawTarget wrapper |

Disable `default-features` and `stub-gpu2d` when wiring a real STM32Cube GPU2D HAL on hardware.

## Driver highlights

- **Batched frames**: `begin_frame()` / `*_in_frame()` / `end_frame()` or `end_frame_async().await`
- **Persistent command list**: 8 KiB circular CL (ST Resize_GPU pattern)
- **Error propagation**: `nema_get_error()` and GPU2D `SystemError`
- **Cache coherency**: I-cache invalidate + D-cache clean/invalidate around submissions
- **Stroke APIs**: `draw_stroke_rect`, `draw_stroke_line_aa`, `draw_stroke_triangle_aa`, etc.
- **Textured triangles**: `blit_tri_fit`, `blit_tri_uv`

One-shot helpers such as `clear()`, `fill_rect()`, and `blit()` remain available; they submit immediately when no frame is open.
