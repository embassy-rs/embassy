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

### Basic Examples

- `hello_world.rs` - Simple hello world that prints messages via RTT

### Display Examples

- `lvgl_minimal.rs` - Minimal display example with LTDC and double buffering
  - Demonstrates LTDC initialization
  - Shows basic graphics (gradient, moving rectangle, borders)
  - Uses RGB565 format (16 bits per pixel)
  - Double buffering for smooth animation
  - Run with: `cargo run --bin lvgl_minimal`

- `lvgl_simple.rs` - Simple LVGL foundation example
  - Similar to lvgl_minimal but with simpler code structure
  - Good starting point for adding LVGL widgets
  - Run with: `cargo run --bin lvgl_simple`

### LVGL Examples (Optional)

The following examples require the LVGL feature to be enabled:

- `lvgl_demo.rs` - Full LVGL demo with widgets
  - Requires `lvgl` and `lvgl-sys` crates
  - Enable with: `cargo run --features lvgl --bin lvgl_demo`
  - Demonstrates various LVGL widgets (buttons, labels, sliders, etc.)
  - Includes touch input support (I2C)

To enable LVGL support, uncomment the LVGL dependencies in `Cargo.toml`:

```toml
[dependencies]
# Uncomment these for LVGL support
lvgl = "0.6.2"
lvgl-sys = "0.6.2"
```

Or use the feature flag:
```bash
cargo run --features lvgl --bin lvgl_demo
```

- `lvgl_touch.rs` - LVGL with capacitive touch input
  - Run with: `cargo run --bin lvgl_touch --features lvgl,touch`

- `lvgl_touch_can.rs` - JSON-driven hall lighting UI with CAN press/hold/repeat
  - Project configs in `touch-projects/SporthalleLudwigsfelde/`
  - UI via [lv_binding_rust](https://github.com/lvgl/lv_binding_rust) in `src/lvgl/` (display, touch, theme, hall UI)
  - Board patterns from [riverdi-50-stm32u5-lvgl](https://github.com/riverdi/riverdi-50-stm32u5-lvgl); legacy C port in `lvgl-port/` for older demos only
  - One-hot TX on CAN ID `0x200`, `minp` feedback on `0x285`
  - Requires `arm-none-eabi-gcc` and picolibc headers for bindgen (`lvgl-sys` builds LVGL as C)
  - Recommended build wrapper (sets bindgen + cross CC):
    ```bash
    ./scripts/cargo-lvgl.sh run --bin lvgl_touch_can --features lvgl,touch
    ```
  - Or manually: `source scripts/lvgl-env.sh` then `cargo run --bin lvgl_touch_can --features lvgl,touch`
  - Debian/Ubuntu: `sudo apt install gcc-arm-none-eabi picolibc-arm-none-eabi`

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
- **Pixel Clock**: ~30 MHz (from PLL3)
- **Timing Parameters**:
  - Horizontal Sync: 5 pulses
  - Horizontal Back Porch: 40 pulses
  - Horizontal Front Porch: 20 pulses
  - Vertical Sync: 5 pulses
  - Vertical Back Porch: 10 pulses
  - Vertical Front Porch: 20 pulses

### Pin Configuration

The following pins are used for LTDC:

| Signal | Pin | Function |
|--------|-----|----------|
| CLK    | PD3 | Pixel clock |
| HSYNC  | PE0 | Horizontal sync |
| VSYNC  | PD13 | Vertical sync |
| DE     | PD6 | Data enable |
| R0     | PC6 | Red bit 0 |
| R1     | PC7 | Red bit 1 |
| R2     | PE15 | Red bit 2 |
| R3     | PD8 | Red bit 3 |
| R4     | PD9 | Red bit 4 |
| R5     | PD10 | Red bit 5 |
| R6     | PD11 | Red bit 6 |
| R7     | PD12 | Red bit 7 |
| G0     | PC8 | Green bit 0 |
| G1     | PC9 | Green bit 1 |
| G2     | PE9 | Green bit 2 |
| G3     | PE10 | Green bit 3 |
| G4     | PE11 | Green bit 4 |
| G5     | PE12 | Green bit 5 |
| G6     | PE13 | Green bit 6 |
| G7     | PE14 | Green bit 7 |
| B0     | PB9 | Blue bit 0 |
| B1     | PB2 | Blue bit 1 |
| B2     | PD14 | Blue bit 2 |
| B3     | PD15 | Blue bit 3 |
| B4     | PD0 | Blue bit 4 |
| B5     | PD1 | Blue bit 5 |
| B6     | PE7 | Blue bit 6 |
| B7     | PE8 | Blue bit 7 |

### Touch Controller

The touch controller typically uses I2C:
- **SCL**: PB6
- **SDA**: PB7
- **Address**: 0x38 (common for FT5x06/GT911 controllers)

## Memory Usage

The STM32U5A9NJH6Q has 2.5MB of RAM. The display examples use:
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

## LVGL Integration Notes

To integrate LVGL with the display:

1. **Add LVGL dependencies** to `Cargo.toml`:
   ```toml
   lvgl = "0.6.2"
   lvgl-sys = "0.6.2"
   ```

2. **Create a display driver** that implements `lvgl::Display` trait

3. **Create an input device driver** that implements `lvgl::InputDevice` trait

4. **Initialize LVGL** in your task:
   ```rust
   let mut display = lvgl::Display::new(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16);
   let draw_buf = lvgl::DrawBuffer::new(...);
   display.set_draw_buffer(&draw_buf);
   ```

5. **Register drivers** with LVGL:
   ```rust
   lvgl::Display::register(&mut my_display);
   lvgl::InputDevice::register(&mut my_touch);
   ```

6. **Main loop**:
   ```rust
   loop {
       lvgl::tick_inc(5); // Increment LVGL tick
       lvgl::handler();   // Handle LVGL tasks
       // Your rendering code here
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

1. Check that the LTDC clock is configured correctly (typically 25-30 MHz)
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
- [LVGL Documentation](https://docs.lvgl.io/master/)

## License

All examples are licensed under either MIT or Apache-2.0 at your option.
