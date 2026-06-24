# gen4-RP2350-70CT rlvgl demo

Embassy + **[rlvgl](https://github.com/protronic/rlvgl)** (pure Rust LVGL reimplementation) for the **4D Systems gen4-RP2350-70CT** 7" 800×480 capacitive touch display.

Hardware template: [`~/work/pio/gen4_rp2350_lvgl`](../../pio/gen4_rp2350_lvgl) (Pico SDK + Graphics4D PIO RGB).  
UI: [`rlvgl`](https://crates.io/crates/rlvgl) **0.2.4** from crates.io (`cargo add rlvgl`).

## Hardware

| Feature | Detail |
|---------|--------|
| MCU | RP2350B, 16 MiB flash, 8 MiB PSRAM |
| Display | 800×480 RGB565, PIO parallel RGB (`rgb70.pio`) |
| PSRAM | APS6404L on QMI CS1 (GPIO 0) — scan-out framebuffer |
| Touch | FT5446 on I2C1 (SDA=46, SCL=39, INT=38, RST=47) |
| Backlight | GPIO 17 (PWM, `Contrast(15)` in Graphics4D) |

Pin map from `boards/gen4_rp2350_70ct.h` in the gen4 PIO LVGL repo.

## Architecture

```text
rlvgl WidgetNode + BlitterRenderer (RGB565)
    └─ draw into PSRAM framebuffer
PIO1: HSYNC / VSYNC / DE  +  PIO2: RGB565 pixel stream @ ~36 MHz
    └─ DMA bounce buffers (60 lines) → continuous panel scan-out
FT5446 INT task → touch channel → PressDown / PressRelease events
```

Constants match the gen4 C port (`main.cpp` / `Graphics4D.cpp`):

| Parameter | gen4 C | This example |
|-----------|--------|--------------|
| System clock | 230 MHz | 230 MHz |
| PIO bounce buffer | 60 lines | 60 lines |
| PCLK target | 36 MHz | 36 MHz |
| Touch | FT5446 @ 0x38, swap XY | same |

## Build

```bash
cd examples/gen4-rp2350-70ct
cargo build --release --bin rlvgl_widget_demo
cargo run --release --bin rlvgl_widget_demo
```

Flash with probe-rs (configured in `.cargo/config.toml`).

## Cargo features

Two independent UI demos render into the same PIO RGB scan-out framebuffer, each
behind its own Cargo feature, so the board support (PIO RGB scan-out, FT5446
touch, DMA self-test) can be built without pulling in any UI dependency:

- `rlvgl-demo` (**default**) — pure-Rust [`rlvgl`](https://crates.io/crates/rlvgl)
  widget tree (`rlvgl_widget_demo`).
- `oxivgl-demo` — real C LVGL v9.5 via [`oxivgl`](https://crates.io/crates/oxivgl)
  0.5.0 / `oxivgl-sys` 0.2.2 (`oxivgl_widget_demo`).

```bash
# default: includes the rlvgl widget demo
cargo build --release --bin rlvgl_widget_demo

# OxivGL (C LVGL) widget demo
cargo run --release --no-default-features --features oxivgl-demo --bin oxivgl_widget_demo

# board support only, no UI dependency
cargo build --release --no-default-features --bin dma_selftest
```

Each demo binary declares its own `required-features`, so it is automatically
skipped when the matching feature is disabled.

### OxivGL notes

The OxivGL demo compiles the LVGL C sources, so it has extra requirements:

- **Nightly toolchain** — pinned in this crate's `rust-toolchain.toml`
  (`oxivgl` needs `feature(type_alias_impl_trait)`); the rlvgl demo builds on it
  too.
- **`conf/lv_conf.h`** and **`fonts/`** (Montserrat 14/16 with Latin-1 / German
  umlaut coverage) are wired in via `.cargo/config.toml`
  (`DEP_LV_CONFIG_PATH`, `LVGL_FONTS_DIR`).
- **Vendored crates** under `vendor/` (patched in via `[patch.crates-io]`):
  - `vendor/oxivgl-sys` — `0.2.2` patched to cross-compile for bare-metal thumb
    (link SDL2 only on the host, point bindgen at the `arm-none-eabi` newlib
    headers). Needs `arm-none-eabi-gcc` + newlib headers on the host.
  - `vendor/oxivgl` — `0.5.0` with the example PNG-asset conversion removed
    (it relied on a `pypng` version that broke LVGL's `LVGLImage.py`).
- Rendering uses LVGL `PARTIAL` mode: the flush callback blits each dirty region
  straight into the single persistent PSRAM framebuffer (same anti-flicker
  strategy as the rlvgl demo).

## Known limitations

- **Full-frame refresh is still somewhat slow.** The display uses a single
  persistent PSRAM framebuffer with partial re-rendering: only the few widgets
  that actually change each frame are redrawn, which keeps animations smooth and
  flicker-free. A *complete* repaint of the whole 800×480 frame (e.g. after a
  touch that changes static widgets) writes the full ~768 KiB over the shared
  QMI/PSRAM bus and is therefore noticeably slower. Full repaints are rare, so
  this is acceptable for the demo, but a fully dynamic UI that redraws large
  areas every frame would need additional optimization (e.g. dirty-region
  tracking, more bounce-buffer slack, or rendering into an off-PSRAM backbuffer).

## Related

- `examples/rp2350-touch-lcd-7` — Waveshare 7" (OxivGL / C LVGL path)
- `~/work/pio/gen4_rp2350_lvgl` — original Pico SDK C++ reference
- [rlvgl on crates.io](https://crates.io/crates/rlvgl) — pure Rust LVGL reimplementation
