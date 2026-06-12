# OxivGL host demo (protronic widget scene)

SDL2 port of the Riverdi RVT50 **protronic** lighting-scene demo from
`examples/rvt50hqsnwc00-b`. Use it on a development PC to verify OxivGL widget
layout, event bubbling, and mouse click handling without hardware.

## Prerequisites

- **Nightly Rust** (see `rust-toolchain.toml`)
- **SDL2** development libraries, e.g. on Debian/Ubuntu:
  ```bash
  sudo apt-get install libsdl2-dev pkg-config
  ```

## Run

### Widget scene demo (no CAN)

```bash
cd examples/oxivgl-host
cargo run
```

### Hall lighting UI with SocketCAN

Same **5-column shell layout** as `oxivgl_widget_demo`, with UI strings from JSON.
Default touch project: [`DemoHost`](../touch-projects/DemoHost/) (`hall_name`: **Sporthalle Demo**,
SocketCAN on **`vcan0`**).

```bash
cd examples/oxivgl-host
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set vcan0 up
cargo run --bin oxivgl_touch_can
```

Hold scene buttons in the SDL window to send the one-hot CAN command bitmask; release
to send the all-zero frame. Button highlight state follows `minp` feedback on CAN ID `0x285`.

To use a real CAN interface instead, override the project at build time:

```bash
TOUCH_PROJECT=Demo cargo run --bin oxivgl_touch_can
# then bring up can0 at 500 kbit/s before running
```

Latin-1 Montserrat fonts (ä ö ü ß, …) are compiled from
`examples/rvt50hqsnwc00-b/fonts/` via `LVGL_FONTS_DIR` in `.cargo/config.toml`.
Regenerate with `examples/rvt50hqsnwc00-b/fonts/generate.sh` if glyph coverage changes.

Click scene buttons in the SDL window. Terminal output shows:

- Widget layout bounds at startup (compare touch coordinates on the board)
- LVGL pointer events (`PRESSED`, `CLICKED`, …) with target handle and button index

## Relation to the board demo

| Board (`oxivgl_widget_demo`) | Host (this crate) |
|------------------------------|-------------------|
| LTDC RGB565 flush | SDL window |
| I2C touch via `TouchInput` (TIMER mode) | SDL mouse indev (TIMER mode) |
| `defmt` RTT logging | `log` / stderr |
| `touch_dbg` heartbeat task | Event lines printed directly |

Both targets use the same render cadence: `view.update()`, then four
`timer_handler()` passes per frame. On the board, `TouchInput::publish()` feeds
the I2C sample immediately before each tick — the same slot where SDL injects
mouse coordinates on the host.

If clicks work here but not on the RVT50, the widget tree and event wiring are
fine — focus on I2C sampling and coordinate mapping. If clicks fail here too,
inspect `WidgetView` flags (`CLICKABLE`, `EVENT_BUBBLE`) and event registration.
