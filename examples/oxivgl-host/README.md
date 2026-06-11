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

```bash
cd examples/oxivgl-host
cargo run
```

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
