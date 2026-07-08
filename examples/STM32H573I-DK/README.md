# STM32H573I-DK examples

Embassy examples for the [STM32H573I-DK](https://www.st.com/en/evaluation-tools/stm32h573i-dk.html) Discovery kit (STM32H573IIK3Q, Cortex-M33, 2 MB flash, 640 KB SRAM, FDCAN).

Board facts (Zephyr devicetree `stm32h573i_dk`):

- **User LEDs** (active-low): LD1 green **PI9**, LD2 orange **PI8**, LD3 red **PF1**, LD4 blue **PF4**
- **User button**: **PC13** (active-low, internal pull-up)
- **FDCAN1**: **PA11** (RX) / **PA12** (TX)
- **HSE**: 25 MHz crystal (used as FDCAN kernel clock)
- **Debug**: ST-LINK V3E; defmt/RTT via probe-rs

```bash
cargo run --release --bin blinky
```

## JSON node (`json_node`) — PoC for CANbossTouch, now with CAN

Successor of the WBA65 `json_node` PoC — moved to the H5 because **the WBA6 has no CAN**. A JSON file ([`src/bin/json_node.json`](./src/bin/json_node.json)) describes **one CANopen-style node**: an **object dictionary** (index/subindex/type/access/initial value), **Rhai scripts** (`once` at boot, `cyclic` with a period) and a **CAN section**. The firmware (`src/bin/json_node.rs`) is fully generic — it parses the JSON, builds the OD in RAM, registers the OD/board API in Rhai, brings up FDCAN1 and schedules the scripts. Change the JSON, rebuild, get a different node; no Rust changes.

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
| `led(n, on)`, `leds(mask)` | User LEDs 0..3 (LD1..LD4), `leds` takes a bitmask |
| `button()` | User button: 1 = pressed |

### CAN ("PDO" transport for the OD)

Configured in the JSON:

```json
"can": { "mode": "loopback", "bitrate": 250000, "tpdo": "0x180", "rpdo": "0x200" }
```

- Every OD value change is **sent** as a classic frame on COB-ID `tpdo + node_id` (default `0x190` for node 16). Payload: index `u16` LE, sub `u8`, dtype `u8` (0 = int, 1 = f32, 2 = bool), value 4 bytes LE. Strings are not transmitted.
- **Received** frames on COB-ID `rpdo + node_id` (default `0x210`) write into the OD with the same layout. `access` is enforced as the bus view: only `rw`/`wo` entries are writable from the bus, `ro`/`const` are rejected.
- `"mode": "loopback"` works standalone (TX frames echo back internally — no transceiver needed); `"mode": "normal"` talks to the real bus. With a USB-CAN adapter: `candump can0` shows the TPDOs; `cansend can0 210#0121000200000000` sets `blink_on` (0x2101.00) to false and stops the light chaser.

Scripts are device-internal and may also write `ro` entries — that is how the device updates its own inputs; only `const` is rejected for scripts. Every change is logged over defmt (`od 0x6200.01 dout_leds = 2 (was 0)`), integer writes are clamped to the declared type's range, a runaway script is aborted after 500k Rhai operations, and script errors are logged without stopping the scheduler.

The demo node (id 16, "IO-Modul") mirrors the user button into `0x6000.01 din_button`, runs a light chaser over all four LEDs on `0x6200.01 dout_leds` (button overrides: all on), counts cycles in `0x2100.00` and prints a heartbeat line every 2 s.

Next steps: replace the raw "PDO" framing with real CANopen (SDO server / PDO mapping via CANopenNode), load the JSON at runtime (UART/flash) instead of `include_str!`, drive script periods from OD entries (e.g. `0x1017`).
