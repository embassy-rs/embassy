# Waveshare RP2350-Touch-LCD-7 Examples

Embassy port of the **OxivGL** (LVGL v9.5) hall lighting UI, **GT911** capacitive touch, and **XL2515** CAN stack from `examples/rvt50hqsnwc00-b`, adapted for the [Waveshare RP2350-Touch-LCD-7](https://www.waveshare.com/wiki/RP2350-Touch-LCD-7).

## Board hardware

| Function | Chip | Interface | Notes |
|----------|------|-----------|-------|
| Display 800×480 RGB565 | ST7262 | PIO RGB (DE/HSYNC/VSYNC/PCLK + 16 data) | Pins from Waveshare BSP |
| Touch (5-point) | GT911 | I2C1 @ **0x5D** | INT=GPIO18, RST=GPIO19 |
| CAN controller | XL2515 (MCP2515-compatible) | SPI0 | CS=5, INT=1; transceiver **SIT65HVD230** |
| PSRAM 2 MiB | APS6404L | QMI CS1 | CS=GPIO0 |
| RTC (onboard) | PCF85063 | I2C | Same bus as GT911 |

Pin map source: Waveshare demo BSP (`libraries/bsp/*.h`) in [RP2350-Touch-LCD-7-Demo.zip](https://files.waveshare.com/wiki/RP2350-Touch-LCD-7/RP2350-Touch-LCD-7-Demo.zip).

### Datasheets

| Part | Link |
|------|------|
| GT911 | https://files.waveshare.com/wiki/common/GT911_EN_Datasheet.pdf |
| PCF85063 | https://files.waveshare.com/wiki/common/PCF85063A.pdf |
| XL2515 | MCP2515-compatible; see [Microchip MCP2515](https://ww1.microchip.com/downloads/en/devicedoc/mcp2515-can-controller-with-spi-interface-20001801j.pdf) |
| SIT65HVD230 | TI [SN65HVD230](https://www.ti.com/product/SN65HVD230) class transceiver |
| ST7262 | Panel driver IC on RGB FPC; timing via Waveshare `pio_rgb` / `bsp_st7262` |

## Examples

| Binary | Feature | Description |
|--------|---------|-------------|
| `gt911_touch` | — | Poll GT911 and log touch coordinates (RTT) |
| `can_raw` | — | XL2515 TX/RX loop (id `0x123`) |
| `oxivgl_widget_demo` | `oxivgl` | Multi-widget OxivGL demo (800×480) |
| `oxivgl_touch_can` | `oxivgl` | Hall lighting UI + CAN (JSON from `touch-projects/Demo/`) |

### Build (non-OxivGL)

```bash
cd examples/rp2350-touch-lcd-7
cargo build --bin gt911_touch
cargo build --bin can_raw
```

### Build OxivGL (requires nightly + `arm-none-eabi-gcc`)

```bash
cd examples/rp2350-touch-lcd-7
cargo build --bin oxivgl_widget_demo --features oxivgl
cargo build --bin oxivgl_touch_can --features oxivgl
```

Flash and stream logs with **probe-rs** (default runner in `.cargo/config.toml`):

```bash
cargo run --bin oxivgl_touch_can --features oxivgl
```

Attach to firmware that is already running:

```bash
probe-rs attach --chip RP235x
```

Default touch project: [`DemoHost`](../touch-projects/DemoHost/) (**50 kbit/s** CAN, tx id `0x285`).

```bash
# 500 kbit/s CAN (touch project Demo):
TOUCH_PROJECT=Demo cargo run --bin oxivgl_touch_can --features oxivgl
```

CAN bitrate comes from `touch-projects/<name>/can_config.json` (`baud` field). XL2515 supports 5k–1M (see `xl2515.rs`).

## Logging (probe-rs + defmt-rtt)

defmt logs over SWD/RTT (same as `examples/rvt50hqsnwc00-b`):

| Command | Effect |
|---------|--------|
| `cargo run --bin gt911_touch` | Flash + logs |
| `cargo run --bin oxivgl_touch_can --features oxivgl` | Flash hall/CAN demo + logs |
| `probe-rs attach --chip RP235x` | Logs only (board already running) |

`DEFMT_LOG` in `.cargo/config.toml` sets the log level (default `debug`).

## Architecture notes

- **Display**: Full-screen LVGL refresh into PSRAM double buffers, then PIO RGB DMA scan-out (ported from Waveshare `pio_rgb.c` / `RP2350-Touch-7-Exp` LVGL C port). GT911 touch and XL2515 CAN unchanged.
- **Touch**: Same INT-driven task + channel queue as RVT50 (`touch_feed.rs`), GT911 register protocol from Waveshare `bsp_gt911.c`.
- **CAN**: On-chip FDCAN is **not** available on RP2350; Waveshare uses **XL2515** over SPI. Application protocol reuses `touch-hall-common` unchanged.

### PIO RGB + GT911 fixes (feature branch)

The fixes live on branch `cursor/rp2350-pio-rgb-oxivgl-f557` ([PR #18](https://github.com/protronic/embassy/pull/18)), **not** on `main` yet.

If the log shows `PIO RGB scan-out stub` or `GT911 not detected on I2C @ 0x5d` without a retry at `0x14`, you are still running `main`.

```bash
git fetch origin
git checkout cursor/rp2350-pio-rgb-oxivgl-f557
cd examples/rp2350-touch-lcd-7
cargo clean
cargo build --bin oxivgl_widget_demo --features oxivgl
cargo run --bin oxivgl_widget_demo --features oxivgl
```

The first log line must include `firmware=cursor/rp2350-pio-rgb-oxivgl-f557@…`. On the correct build you should see:

- `GT911 ready @ 0x5d`
- `PIO RGB scan-out started (800x480 @ 16 MHz pclk)`

**Not** `PIO RGB scan-out stub`.

## Target

- MCU: **RP2350B** (`embassy-rp` feature `rp235xb`)
- Target: `thumbv8m.main-none-eabihf`
- Flash: 16 MiB (`memory.x`)
