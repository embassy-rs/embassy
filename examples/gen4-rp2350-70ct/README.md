# gen4-RP2350-70CT OxivGL demo

Embassy port of the [gen4 PIO LVGL reference firmware](https://github.com/protronic/gen4_rp2350_lvgl) for the **4D Systems gen4-RP2350-70CT** 7" 800×480 touch display.

## Hardware

| Feature | Detail |
|---------|--------|
| MCU | RP2350B, 16 MiB flash |
| Display | 800×480 RGB565, PIO parallel RGB (`rgb70.pio`) |
| PSRAM | APS6404L on QMI CS1 (GPIO 0) — scan-out framebuffer |
| Touch | FT5446 on I2C1 (SDA=46, SCL=39, INT=38, RST=47) |
| Backlight | GPIO 17 (PWM) |

Pin map matches `boards/gen4_rp2350_70ct.h` in the gen4 PIO LVGL repo.

## Architecture

```text
OxivGL (partial render, SRAM stripes)
    └─ flush: memcpy → PSRAM framebuffer
PIO1: HSYNC / VSYNC / DE  +  PIO2: RGB565 pixel stream
    └─ DMA bounce buffers (60 lines) → continuous panel scan-out
FT5446 INT task → touch channel → LVGL pointer indev
```

This follows the gen4 C port (`lv_demo_widgets` in `main.cpp`): partial LVGL rendering with a single PSRAM framebuffer, not the Waveshare double-buffer swap used in `rp2350-touch-lcd-7`.

## Build

Requires **nightly Rust** (see `rust-toolchain.toml`) and `arm-none-eabi-gcc` for LVGL C sources.

```bash
cd examples/gen4-rp2350-70ct
cargo build --release --bin oxivgl_widget_demo --features oxivgl
cargo run --release --bin oxivgl_widget_demo --features oxivgl
```

Flash with probe-rs (configured in `.cargo/config.toml`).

## Related examples

- `examples/rp2350-touch-lcd-7` — Waveshare 7" PIO RGB (GT911, double-buffer full refresh)
- `gen4_rp2350_lvgl` — original Pico SDK C++ PIO LVGL port this example is based on
