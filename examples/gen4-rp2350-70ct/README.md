# gen4-RP2350-70CT — OxivGL widget demo

A clean [OxivGL](https://crates.io/crates/oxivgl) (Rust LVGL v9.5) demo for the
[4D Systems gen4-RP2350-70CT](https://4dsystems.com.au/) — an RP2350B driving a
7" 800×480 RGB panel with an FT5446 capacitive touch controller and external
APS6404 PSRAM.

It is a focused port of the `rp2350-touch-lcd-7` example, stripped of the CAN /
hall-sensor projects, retargeted to the gen4 board pin-out and touch controller.

## Hardware

Pin assignments follow the vendor board header `gen4_rp2350_70ct.h`:

| Function            | GPIO            |
|---------------------|-----------------|
| Backlight           | 17              |
| DE / VSYNC / HSYNC / PCLK | 18 / 19 / 20 / 21 |
| RGB565 DATA0..DATA15 | 22..=37 (DATA0 = blue LSB) |
| Touch INT / SCL / SDA / RST | 38 / 39 / 46 / 47 (I2C1, FT5446 @ 0x38) |
| PSRAM CS            | 0 (QMI CS1)     |

The pixel clock is 25 MHz (`LCD_CLK_FREQ`). Two full-screen framebuffers and the
DMA staging buffers live in PSRAM; the panel is scanned out continuously by PIO1
(HSYNC/VSYNC) + PIO2 (DE + 16-bit RGB) with DMA, and LVGL renders full frames
into the back buffer which is swapped on scan-out completion.

## Build & run

Requires the pinned nightly toolchain (`rust-toolchain.toml`) and the vendored
`oxivgl` / `oxivgl-sys` crates (under `../rvt50hqsnwc00-b/vendor`).

```sh
cargo build --release --bin oxivgl_widget_demo
# flash + defmt-rtt logs over a probe:
cargo run --release --bin oxivgl_widget_demo
```
