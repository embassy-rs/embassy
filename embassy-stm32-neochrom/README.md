# embassy-stm32-neochrom

Embassy integration for ST NeoChrom (GPU2D) using the [NemaGFX](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/Middleware/NemaGFX) middleware via [`stm32-bindings`](https://github.com/embassy-rs/stm32-bindings).

## Status

Hardware-validated on STM32N6 (NUCLEO-N6570/N6570-DK): GPU2D init, the cache-hold
handshake, and batched clear/fill/stroke/blit operations all work end-to-end,
including driving an LTDC panel (see `examples/stm32n6/src/bin/neochrom_lcd.rs`).
The async completion path (`end_frame_async()`, the interrupt-driven waker in
`gpu2d_bridge.rs`) is implemented and builds clean but has not yet been
exercised on real hardware — `end_frame()` (blocking) is the exercised path.
U5 and H7RS chip features are wired up in `Cargo.toml` (same NemaGFX core,
different prebuilt library) but have no example or hardware validation yet.

## Local development

This crate depends on `stm32-bindings` (`>= 0.3.1`, the first release with the
NemaGFX bindings), published on crates.io — no local `[patch]` needed.

Build (example chip feature):

```bash
cd embassy-stm32-neochrom
cargo check --target thumbv8m.main-none-eabihf --features stm32n657x0,stub-gpu2d
```

## Features

NemaGFX ships one prebuilt library per CPU core / GPU variant; `stm32-bindings`
forwards these as `neochrom-*` presets, and each chip feature below pulls in
exactly one automatically — enable exactly one chip feature, since
`stm32-bindings`'s `build.rs` rejects linking multiple NemaGFX core variants
at once.

| Feature | NemaGFX preset | GPU variant | Typical parts | Status |
|---------|----------------|-------------|----------------|--------|
| `stm32n657x0` / `stm32n655x0` / `stm32n647x0` | `neochrom-m55` | NeoChrom | STM32N6xx | hardware-validated |
| `stm32u599nj` | `neochrom-m33-revc` | NeoChrom | STM32U5x7/x9 | untested |
| `stm32u5g9nj` | `neochrom-m33-nemapvg` | NeoChromVG | STM32U5F9/U5G9 | untested |
| `stm32h7s7z8` / `stm32h7r7z8` | `neochrom-m7` | NeoChrom | STM32H7R7/H7S7 | untested |
| `stub-gpu2d` | — | — | — | default; CI / link tests, no hardware needed |
| `embedded-graphics` | — | — | — | `NeoChromTarget` `embedded-graphics` `DrawTarget` wrapper |
| `defmt` / `log` | — | — | — | Logging backend |

Disable `default-features` and `stub-gpu2d` when wiring the real GPU2D peripheral on hardware.

## Driver highlights

- **Batched frames**: `begin_frame()` / `*_in_frame()` / `end_frame()` or `end_frame_async().await`, over a `GpuSurface` render target (`FrameBuffer`, `ExternalFrameBuffer`, or your own type)
- **Persistent command list**: 8 KiB circular CL (ST Resize_GPU pattern)
- **Error propagation**: `nema_get_error()` and GPU2D `SystemError`
- **Cache coherency**: I-cache invalidate + D-cache clean/invalidate around submissions, plus servicing NemaGFX's GPU-initiated cache-hold handshake on the error interrupt
- **Stroke APIs**: `draw_stroke_rect`, `draw_stroke_line_aa`, `draw_stroke_triangle_aa`, etc.
- **Textured triangles**: `blit_tri_fit`, `blit_tri_uv`

One-shot helpers such as `clear()`, `fill_rect()`, and `blit()` remain available; they submit immediately when no frame is open.

## NemaGFX version

Bindings and prebuilt libraries are vendored by `stm32-bindings`, currently pinned to x-cube-image-processing v1.0.0 (NemaGFX v1.4.17). This crate never touches NemaGFX headers or `.a` files directly — everything comes through `stm32-bindings`' generated FFI.

## License

This crate is MIT OR Apache-2.0, but NemaGFX itself is licensed separately under Think Silicon / ST terms with different redistribution conditions — review `stm32-bindings`' vendored license before publishing an application or crate that links the prebuilt NemaGFX library.
