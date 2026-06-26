# STM32WBA65I-DK1 examples

Embassy examples for the [STM32WBA65I-DK1](https://www.st.com/en/evaluation-tools/stm32wba65i-dk1.html) Discovery kit (STM32WBA65RIV7, 512 KB SRAM, BLE 6.0, OLED, RGB LEDs, joystick).

## Rhai playground (`ble_rhai`)

Interactive BLE scripting demo — connect with a phone or nRF Connect, write to the NUS RX characteristic, and get results on TX notify.

```bash
cargo run --release --bin ble_rhai --features scripting
```

Advertises as **RhaiPlay**. Script dispatch happens after 500 ms idle or on disconnect (same as the WBA5MM module demo).

### Rhai API

| Function | Description |
|----------|-------------|
| `led(n, on)` | User LED `n`: 0=green, 1=red, 2=blue |
| `rgb(r, g, b)` | Set all three LEDs at once |
| `joy()` | Joystick: 0=none, 1=select, 2=left, 3=down, 4=up, 5=right |
| `oled_line(n, text)` | Write a line (0–7) on the 128×64 OLED |
| `oled_clear()` | Clear the OLED line buffer |
| `ts()` | Uptime ticks (32768 Hz) |
| `heap_free()` | Free heap bytes |

Example session over BLE:

```text
rgb(true, false, false)
oled_line(6, "hello")
joy()
let x = 0; for i in 0..100 { x += i; } x
```

The joystick also drives live RGB feedback while idle. The OLED shows a splash screen at boot and script-driven lines afterward.

See [RHAI_PLAYGROUND.md](./RHAI_PLAYGROUND.md) for future API ideas, demo scripts, and roadmap.

### Hardware notes

- **LEDs**: active-low on PD8 (green), PD9 (red), PB10 (blue). LD3 needs R42 populated on some boards.
- **Joystick**: analog ladder on PA3 / ADC4 ch6.
- **OLED**: SSD1306 on I2C1 (PB2/PB1), address 0x3C.
- **Debug**: ST-LINK VCP on USART1 (PB12 TX, PA8 RX).

## Other examples

Copied from `stm32wba5mm` and retargeted to `stm32wba65ri`. Pin assignments may differ from the Discovery kit — check schematics before running peripheral demos other than `blinky` and `ble_rhai`.
