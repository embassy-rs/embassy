# Riverdi RVT50HQSNWC00-B Examples

This folder contains examples for the **Riverdi RVT50HQSNWC00-B** display module.

## Board Information

- **Display**: 5.0" TFT LCD with capacitive touch
- **Resolution**: 800x480 pixels
- **Microcontroller**: STM32U5A9NJH6Q (Cortex-M33)
- **Flash Memory**: 4MB (4096KB)
- **RAM**: 2.5MB (2560KB)
- **Interface**: RGB, I2C for touch controller

## Getting Started

### Prerequisites

1. Install [probe-rs](https://probe.rs/) for flashing and debugging:
   ```bash
   cargo install probe-rs --locked
   ```

2. Install the ARM toolchain:
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

### Building and Running

To build and run the hello world example:

```bash
cd examples/rvt50hqsnwc00-b
cargo run --bin hello_world
```

This will:
1. Compile the example
2. Flash it to the board using probe-rs
3. Start a debug session

## Available Examples

- `hello_world.rs` - Simple hello world that prints messages via RTT
- `gpio.rs` - Poll the user button (`PH3`) and flash the user LED (`PE5`)
- `can_raw.rs` - FDCAN demo on connector P5 (pattern TX + LED state RX)
- `rlvgl_demo.rs` - Minimal [rlvgl](https://github.com/SoftOboros/rlvgl) UI (label + tappable button)
- `widget_demo.rs` - Multi-widget **rlvgl** demo (label, button, slider, bar, switch, checkbox)
- `oxivgl_widget_demo.rs` - Multi-widget **OxivGL** demo (real C LVGL v9.5 via [oxivgl](https://github.com/emobotics-dev/oxivgl))

### UI stacks (`rlvgl` vs `oxivgl`)

| Feature | Library | Toolchain | Description |
|---------|---------|-----------|-------------|
| `rlvgl` | [rlvgl](https://github.com/SoftOboros/rlvgl) | Stable Rust 1.92+ | Pure-Rust LVGL-style UI on Embassy LTDC |
| `oxivgl` | [oxivgl](https://github.com/emobotics-dev/oxivgl) | **Nightly** (see `rust-toolchain.toml`) | C LVGL v9.5 — same generation as [Riverdi's Cube LVGL port](https://github.com/riverdi/riverdi-50-stm32u5-lvgl) |

Enable **one** UI feature per binary, e.g. `--features rlvgl` or `--features oxivgl`.

OxivGL builds also need:

- `arm-none-eabi-gcc` and `libnewlib-arm-none-eabi` (LVGL is compiled from source by `oxivgl-sys`)
- **Nightly Rust** (`rust-toolchain.toml` in this crate)

### rlvgl demo

The `rlvgl_demo` binary uses **rlvgl 0.2.1** with an Embassy LTDC RGB565 backend. It draws a title and a counter button on the 800×480 panel.

```bash
cargo run --bin rlvgl_demo --features rlvgl
cargo run --bin rlvgl_demo --features rlvgl,touch   # capacitive touch input
```

### rlvgl widget demo

```bash
cargo run --bin widget_demo --features rlvgl
cargo run --bin widget_demo --features rlvgl,touch
```

### OxivGL widget demo

Real LVGL v9.5 via OxivGL, with `conf/lv_conf.h` and an STM32U5 LTDC flush driver in `src/oxivgl/`.

```bash
cargo run --bin oxivgl_widget_demo --features oxivgl
cargo run --bin oxivgl_widget_demo --features oxivgl,touch
```

Touch uses two Embassy tasks: `touch_feed::run_touch_int_task` sleeps on the
`CTP_INT` EXTI line (PE6, active-low) and only polls I2C while a contact is
active, queueing press/release samples into a bounded channel; the UI task
drains the queue, publishes each sample (safe, no `unsafe` — critical-section
mutex) and calls `lv_indev_read()` after each `timer_handler()` (EVENT-mode
indev with paused read timer — required on STM32; TIMER mode left `pt=(0,0)`
in logs). Between taps the touch task is fully idle — zero I2C traffic.

With `touch`, RTT logs include:

- `oxivgl touch task: interrupt-driven via CTP_INT` — task start (boot)
- `oxivgl touch down/up` — raw I2C coordinates
- `oxivgl touch int wake / spurious` (`DEFMT_LOG=debug`) — EXTI wake-ups
- `oxivgl indev pressed` — LVGL pointer state vs layout hit-test index
- `oxivgl widget event` — bubbled `PRESSED` / `CLICKED` on scene buttons
- `oxivgl touch dbg` (every 2 s) — `i2c_ok`, `active_obj`, `layout_hit`,
  `lvgl_events`, `int_wakeups`

### OxivGL host demo (SDL, no hardware)

Same protronic lighting-scene UI on a PC — useful to confirm widget events work before debugging board touch:

```bash
cd examples/oxivgl-host
cargo run
```

See `examples/oxivgl-host/README.md` (requires SDL2 dev libraries).

Optional: point `LVGL_SRC_DIR` at Riverdi's vendored tree when building:

```bash
export LVGL_SRC_DIR=/path/to/riverdi-50-stm32u5-lvgl/Middlewares/Third_Party/LVGL
```

## Configuration

The examples are configured for:
- **Chip**: STM32U5A9NJH6Q
- **Target**: `thumbv8m.main-none-eabihf`
- **Runner**: `probe-rs run --chip STM32U5A9NJH6Q`

The `memory-x` feature in `embassy-stm32` automatically provides the correct memory layout for the STM32U5A9NJH6Q chip (4MB Flash, 2.5MB RAM).

## Display Configuration

The LTDC (LCD-TFT Display Controller) is configured with:
- **Resolution**: 800x480 pixels
- **Color Format**: RGB565 (16 bits per pixel)
- **Pixel Clock**: ~25 MHz (from PLL3)
- **Timing Parameters**:
  - Horizontal Sync: 4 pulses
  - Horizontal Back Porch: 8 pulses
  - Horizontal Front Porch: 8 pulses
  - Vertical Sync: 4 pulses
  - Vertical Back Porch: 8 pulses
  - Vertical Front Porch: 8 pulses

Board support helpers for LTDC init live in `src/rvt50_board.rs` (`init_display`, `ltdc_configuration`).

### Pin Configuration

The following pins are used for LTDC:

| Signal | Pin | Function |
|--------|-----|----------|
| CLK    | PD3 | Pixel clock |
| HSYNC  | PE0 | Horizontal sync |
| VSYNC  | PD13 | Vertical sync |
| DE     | PF11 | Data enable |
| R3     | PD8 | Red bit 3 |
| R4     | PD9 | Red bit 4 |
| R5     | PD10 | Red bit 5 |
| R6     | PD11 | Red bit 6 |
| R7     | PD12 | Red bit 7 |
| G2     | PE9 | Green bit 2 |
| G3     | PE10 | Green bit 3 |
| G4     | PE11 | Green bit 4 |
| G5     | PE12 | Green bit 5 |
| G6     | PE13 | Green bit 6 |
| G7     | PE14 | Green bit 7 |
| B3     | PD15 | Blue bit 3 |
| B4     | PD0 | Blue bit 4 |
| B5     | PD1 | Blue bit 5 |
| B6     | PE7 | Blue bit 6 |
| B7     | PE8 | Blue bit 7 |

### Touch Controller

The touch controller typically uses I2C:
- **SCL**: PG13
- **SDA**: PG14
- **Reset**: PE3
- **Address**: 0x41 (touch-panel variants)

## Memory Usage

The STM32U5A9NJH6Q has 2.5MB of RAM. A full-screen double-buffered RGB565 display uses:
- **Frame Buffer 1**: 800 * 480 * 2 = 768,000 bytes (~750KB)
- **Frame Buffer 2**: 800 * 480 * 2 = 768,000 bytes (~750KB)
- **Total for double buffering**: ~1.5MB

This leaves approximately 1MB of RAM for other tasks and the stack.

## Adding New Examples

To add a new example:

1. Create a new file in `src/bin/` (e.g., `my_example.rs`)
2. Use the `#![no_std]` and `#![no_main]` attributes
3. Import necessary crates from embassy
4. Use `#[embassy_executor::main]` for the main function

Example template:

```rust
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let _p = embassy_stm32::init(config);
    
    info!("My example running!");
    
    loop {
        // Your code here
    }
}
```

## User button (PH3 / BOOT0)

The **USR BTN** (S1) is wired to `PH3`, which is also the STM32U5 **BOOT0** boot-strap
pin. Out of the factory, the MCU reads the boot mode from the PH3 pin level at reset.

To use PH3 as a normal GPIO or EXTI input (user button, as in `gpio` / `can_raw`), the
PH3 pad must be released from the boot-loader path by programming the user option bytes
in flash **once per device**:

| Step | Action |
|------|--------|
| 1 | Set **`nSWBOOT0 = 0`** in `FLASH->OPTR` (user option byte) |
| 2 | BOOT0 is then taken from the internal **`nBOOT0`** option bit instead of PH3 |
| 3 | Set **`nBOOT0`** as needed (typically `0` to boot from internal flash) |
| 4 | Launch the new option bytes (power cycle or OB launch) |

After `nSWBOOT0 = 0` is stored in `FLASH_OPTR`, the PH3 pad is no longer used as a
hardware boot strap and can be declared as standard GPIO (e.g. in STM32CubeMX) or
configured in firmware via `Input::new` / `ExtiInput::new`.

**Programming with STM32CubeProgrammer**

1. Connect SWD and open *STM32CubeProgrammer*.
2. Go to **OB** (option bytes) for the connected STM32U5.
3. Set **nSWBOOT0** to **0** (BOOT0 from option bit).
4. Set **nBOOT0** to **0** if you want to keep booting from internal flash.
5. Apply / launch option bytes and reset the board.

Embassy examples do **not** modify option bytes at runtime; this is a one-time
hardware configuration step. Until it is done, verify button behaviour with the
`gpio` polling example before relying on EXTI in `can_raw`.

## Hardware Connections

The RVT50HQSNWC00-B module typically connects to a host MCU via:
- **RGB Interface**: For display data (24 bits: 8 red, 8 green, 8 blue)
- **I2C**: For touch controller communication
- **Power**: 3.3V

Refer to the [RVT50HQSNWC00-B datasheet](https://download.riverdi.com/RVT50HQSNWC00-B/DS_RVT50HQSNWC00-B_Rev.1.1.pdf) for detailed pinout and connection information.

## Troubleshooting

### Chip Not Recognized

If `probe-rs` doesn't recognize the chip, try:
```bash
probe-rs chip list
```

And update the runner in `.cargo/config.toml` with the exact chip name.

### Build Errors

Ensure you have the latest dependencies:
```bash
cargo update
```

### Flashing Issues

Make sure your probe is properly connected and the board is powered.

### Display Not Working

1. Check that the LTDC clock is configured correctly (typically 25 MHz)
2. Verify the display timing parameters match your panel specifications
3. Ensure all RGB data lines are properly connected
4. Check that the display enable (DE) and control signals are active

### User button or EXTI not working (PH3)

1. Run `cargo run --bin gpio` and check whether `initial=` / `count=` change on RTT when
   pressing S1.
2. If GPIO polling works but `can_raw` EXTI does not, check EXTI wiring (`EXTI3`) in code.
3. If GPIO polling also fails, program **`nSWBOOT0 = 0`** in option bytes (see
   [User button (PH3 / BOOT0)](#user-button-ph3--boot0)) so PH3 is no longer a boot strap.
4. Do not hold the user button during reset — PH3 high at reset enters the system bootloader.

## Resources

- [Riverdi RVT50HQSNWC00-B Datasheet](https://download.riverdi.com/RVT50HQSNWC00-B/DS_RVT50HQSNWC00-B_Rev.1.1.pdf)
- [STM32U5 Series Reference Manual](https://www.st.com/resource/en/reference_manual/dm00314054-stm32u5-series-advanced-arm-based-mcus-stmicroelectronics.pdf)
- [Embassy Documentation](https://docs.embassy.dev/)
- [probe-rs Documentation](https://probe.rs/docs/getting-started/installation/)

## License

All examples are licensed under either MIT or Apache-2.0 at your option.
