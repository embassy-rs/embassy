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

Flash with probe-rs (default `.cargo/config.toml` runner), or switch the runner to `picotool` / `elf2uf2-rs` (see commented lines in `.cargo/config.toml`). UF2 also works — hold BOOT, copy `.uf2`:

```bash
cargo run --bin oxivgl_touch_can --features oxivgl
```

Default touch project: [`DemoHost`](../touch-projects/DemoHost/) (**50 kbit/s** CAN, tx id `0x285`).

```bash
# 500 kbit/s CAN (touch project Demo):
TOUCH_PROJECT=Demo cargo run --bin oxivgl_touch_can --features oxivgl
```

CAN bitrate comes from `touch-projects/<name>/can_config.json` (`baud` field). XL2515 supports 5k–1M (see `xl2515.rs`).

## USB monitor

Two CDC ports on the Type-C cable:

| Port | sysfs | Content |
|------|-------|---------|
| `if00` | Plain text (`usb_monitor::line`) |
| `if01` or `if02` | `defmt` (Linux may skip `if01`) |

**Do not assume `ttyACM0`** — numbering varies (`ttyACM1`/`ttyACM2` is common). Always use `./usb-monitor.sh` or `/dev/serial/by-id/...`.

```bash
./usb-monitor.sh text              # if00
./usb-monitor.sh can_raw           # defmt decode for can_raw on if01
./usb-monitor.sh defmt oxivgl_touch_can
```

`DEFMT_LOG` in `.cargo/config.toml` controls defmt level (default `info`).

## Architecture notes

- **Display**: LVGL partial flush → PSRAM double buffers (same pattern as RVT50 LTDC). PIO RGB **scan-out** is stubbed in `pio_rgb.rs`; LVGL rendering and buffer management work — full DMA/PIO port follows Waveshare `pio_rgb.pio`.
- **Touch**: Same INT-driven task + channel queue as RVT50 (`touch_feed.rs`), GT911 register protocol from Waveshare `bsp_gt911.c`.
- **CAN**: On-chip FDCAN is **not** available on RP2350; Waveshare uses **XL2515** over SPI. Application protocol reuses `touch-hall-common` unchanged.

## Target

- MCU: **RP2350B** (`embassy-rp` feature `rp235xb`)
- Target: `thumbv8m.main-none-eabihf`
- Flash: 16 MiB (`memory.x`)
