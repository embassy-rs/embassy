# STM32WBA65I-DK1 examples

Embassy examples for the [STM32WBA65I-DK1](https://www.st.com/en/evaluation-tools/stm32wba65i-dk1.html) Discovery kit (STM32WBA65RIV7, 512 KB SRAM, BLE 6.0, OLED, RGB LEDs, joystick).

## Rhai playground (`ble_rhai`)

Interactive BLE scripting demo — connect with a phone or nRF Connect, write to the NUS RX characteristic, and get results on TX notify.

```bash
cargo run --release --bin ble_rhai --features scripting
```

Advertises as **RhaiPlay**. Script dispatch happens after 500 ms idle or on disconnect (same as the WBA5MM module demo). Input is buffered up to **1024 bytes** per script (multiple BLE writes).

### Rhai API

| Function | Description |
|----------|-------------|
| `led(on)` | Green LED on/off (same as `led(0, on)`); `on` is bool or 0/1 |
| `led(n, on)` | User LED `n`: 0=green, 1=red, 2=blue; `on` is bool or 0/1 |
| `led_toggle(n)` | Toggle LED `n`; returns 1 if on, 0 if off |
| `rgb(r, g, b)` | Set all three LEDs at once; each channel bool or 0/1 |
| `joy()` | Joystick: 0=none, 1=select, 2=left, 3=down, 4=up, 5=right |
| `sleep(ms)` | Async delay (use instead of a spin `wait()` loop) |
| `oled_line(n, text)` | Write a line (0–7) on the 128×64 OLED |
| `oled_clear()` | Clear the OLED line buffer |
| `print("…")` | Send text over BLE **and** mirror to OLED line 7 |
| `ts()` | Uptime ticks (32768 Hz) |
| `heap_free()` | Free heap bytes |
| `help()` | Print full API listing over BLE |

Strings use Rhai's `MoreStringPackage` (method syntax on string values):

| Method / op | Description |
|-------------|-------------|
| `s.len`, `s.is_empty` | Length in characters |
| `s.contains(x)`, `s.starts_with(x)`, `s.ends_with(x)` | Search |
| `s.index_of(x)` | Find substring/char, `-1` if missing |
| `s.sub_string(i, n)`, `s.trim()`, `s.replace(a, b)` | Edit |
| `s.to_upper()`, `s.to_lower()`, `s.split(",")` | Transform / split |

Example session over BLE:

```text
help()
let s = "hello,world";
print(s.split(","));
print(s.to_upper());
rgb(1, 0, 0)
rgb(true, false, false)
print("hello")
oled_line(6, "hello")
joy()
let x = 0; for i in 0..100 { x += i; } x
```

The joystick also drives live RGB feedback while idle. The OLED shows a splash screen at boot and script-driven lines afterward.

See [RHAI_PLAYGROUND.md](./RHAI_PLAYGROUND.md) for future API ideas, demo scripts, and roadmap.

### Hardware notes

- **LEDs**: active-low on PD8 (green), PD9 (red), PB10 (blue). LD3 needs R42 populated on some boards.
- **Joystick**: analog ladder on PA3 / ADC4 ch6.
- **OLED**: SSD1306 on SPI3 (PA0 SCK, PB8 MOSI, PE1 CS, PE0 D/C, PE3 RST).
- **Debug**: ST-LINK VCP on USART1 (PB12 TX, PA8 RX).

## JSON node (`json_node`) — PoC for CANbossTouch

A JSON file ([`src/bin/json_node.json`](./src/bin/json_node.json)) describes **one CANopen-style node** the same way CANbossTouch's `eds/network.json` describes the network: an **object dictionary** (index/subindex/type/access/initial value) plus **Rhai scripts** (`once` at boot, `cyclic` with a period). The firmware (`src/bin/json_node.rs`) is fully generic — it parses the JSON, builds the OD in RAM, registers the OD/board API in Rhai and schedules the scripts. Change the JSON, rebuild, get a different node; no Rust changes. No LVGL, no BLE — output goes to defmt/RTT, board I/O (LEDs, joystick) plays the role of process data.

```bash
cargo run --release --bin json_node --features scripting,json
```

### Rhai API

| Function | Description |
|----------|-------------|
| `od_read(index, sub)` | Read an OD entry by index/subindex, e.g. `od_read(0x6200, 1)` |
| `od_write(index, sub, v)` | Write an OD entry (i32/bool/f32/string; coerced + clamped to the declared type) |
| `get("name")` / `set("name", v)` | Same, by datapoint name |
| `od_dump()` | Log the whole object dictionary over defmt |
| `node_id()`, `node_name()` | Identity from the JSON |
| `uptime_ms()`, `sleep(ms)` | Time |
| `led(n, on)`, `rgb(r, g, b)`, `joy()` | Board I/O (same semantics as `ble_rhai`) |

Every OD value change is logged (`od 0x6200.01 dout_leds = 2 (was 0)`) — think of it as a simulated PDO. `access` describes the (future) bus view via SDO/PDO: scripts are device-internal and may write `ro` entries — that is how a device updates its own inputs — only `const` is rejected. Integer writes are clamped to the declared type's range (`u8`, `i16`, …), a runaway script is aborted after 500k Rhai operations, and script errors are logged without stopping the scheduler.

The demo node (id 16, "IO-Modul") mirrors the joystick into `0x6000.01 din_joystick`, runs a light chaser on `0x6200.01 dout_leds` (joystick overrides it), counts cycles in `0x2100.00` and prints a heartbeat line every 2 s.

Next steps: hook the OD up to real CANopen (SDO server / PDO mapping via CANopenNode), load the JSON at runtime (UART/BLE/flash) instead of `include_str!`, drive script periods from OD entries (e.g. `0x1017`).

## Other examples

Copied from `stm32wba5mm` and retargeted to `stm32wba65ri`. Pin assignments may differ from the Discovery kit — check schematics before running peripheral demos other than `blinky` and `ble_rhai`.
