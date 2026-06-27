# Rhai Playground — ideas & roadmap

Ideas for the `ble_rhai` demo on **STM32WBA65I-DK1** (`examples/stm32wba65i-dk1`).

Current baseline: BLE NUS shell, 256 KB heap, RGB LEDs, joystick ADC, SSD1306 OLED, interrupt-priority BLE + thread eval split (ported from `stm32wba5mm`).

---

## Board hardware not yet exposed to Rhai

| Peripheral | Pins / bus | Rhai API sketch | Notes |
|------------|------------|-----------------|-------|
| User button (B1) | PA3 EXTI (center press can also be ADC) | `button()` or `button_pressed()` | Separate from joystick ladder; EXTI + debounce task |
| Digital microphone | PDM / I2S on mezzanine | `mic_level()` → i32 | Needs PDM or MEMS driver; fun “VU meter” on OLED |
| Audio jacks | SAI / codec path | `tone(hz, ms)` | Heavy — needs codec bring-up (Cube BSP reference) |
| Grove connector | I2C / UART / GPIO | `grove_i2c_read(addr, reg)` | Depends on Grove module; good “shield” story |
| Arduino header | SPI1, I2C1, GPIO | `spi_write(...)`, `digital_read(pin)` | Pin map table in `board.rs`; validate vs RF keep-out |
| Die temperature | internal sensor | `temp_c()` → i32 (°C × 10) | ADC special channel, cheap win |
| HW RNG | onboard | `rand()` → i32 | Already init’d for BLE stack — expose read-only |
| SAES / AES / PKA | crypto blocks | `aes_encrypt_block(...)` | Only if wrapped safely; demo “crypto playground” |
| USB VCP | USART1 PB12/PA8 | — | `usart_rhai` already exists; could unify REPL backend |

---

## Rhai API extensions (easy → hard)

### Quick wins

- **`sleep(ms)`** — `embassy_time::Timer::after_millis` inside a registered async fn is tricky; easier: blocking spin with `Instant` deadline in eval task context, or dispatch to a timer task via channel.
- **`board()`** → string constant `"STM32WBA65I-DK1"` for scripts that adapt across boards.
- **`joy_name()`** — map `joy()` int to `"up"` / `"left"` / … (return string; works with `only_i32` if we use numeric codes only, or enable string returns).
- **`uptime_s()`** — seconds as i32, friendlier than raw 32768 Hz ticks.
- **`ops()`** — expose Rhai `on_progress` operation counter for benchmarking scripts.
- **`led_toggle(n)`** — read-modify-write LED state without a bool arg.
- **`oled_status(line)`** — dedicated status row (line 7) that firmware updates with BLE/heap info, scripts can't overwrite.

### Medium effort

- **`CorePackage`** — WBA65 has 512 KB SRAM vs 128 KB on WBA55; try re-enabling Rhai `CorePackage` and measure stack during init (failed on WBA5MM due to stack, not heap).
- **Immediate vs batch eval** — toggle over BLE: `\n` = expression eval, `\x04` (EOT) = flush buffer (current 500 ms idle behaviour).
- **Expression mode** — `engine.eval_expression` for single-line REPL (like `usart_rhai`) alongside full `eval` for blocks.
- **Script presets** — store 4–8 small scripts in last flash page; `run_preset(n)` loads and evals.
- **OLED drawing** — `oled_bar(row, pct)`, `oled_plot(x, y)` for tiny charts (extend framebuffer in display task, not full embedded-graphics from Rhai).
- **Joystick events** — push `"joy:up"` over BLE notify on edge, so phone-side scripts don't need polling loops.

### Ambitious

- **BLE GATT playground** — second service with Rhai-defined characteristics (handle table exported as `gatt_notify(handle, bytes)`).
- **Matter / Thread hooks** — out of scope for REPL, but “advertise custom UUID” from script could be a teaching demo.
- **TrustZone / SAES** — secure counter in protected memory, Rhai reads public half only.
- **SD-card / USB-MSD** — load `.rhai` files from ST-Link mass-storage (if exposed on DK1).

---

## UX / transport

- **Companion app** — minimal web BLE client (Web Bluetooth) with line editor, syntax highlight, preset buttons, OLED mirror panel.
- **Welcome script** — on first CCCD subscribe, auto-run a demo:
  ```text
  rgb(true,false,false); sleep(200); rgb(false,true,false); sleep(200); rgb(false,false,true)
  oled_line(6, "ready");
  ```
- **Prompt echo** — show `heap_free()` in the `> ` prompt after each eval.
- **Error OLED mirror** — on `err: ...`, also `oled_line(7, "ERR")` with truncated message.
- **Disconnect policy** — option to cancel in-flight eval on disconnect (today: buffer flush + eval continues).

---

## Architecture / robustness

| Topic | Idea |
|-------|------|
| LED contention | `input_task` drives joy→RGB feedback only when `!EVAL_RUNNING`; scripts own LEDs during eval |
| OLED bandwidth | Coalesce `oled_line` updates (max 10 Hz) to reduce I2C traffic during tight loops |
| Heap telemetry | Log peak heap usage per script in defmt; expose `heap_peak()` after eval |
| Stack | With 512 KB SRAM, consider larger thread stack in `memory.x` for deeper Rhai recursion |
| Float profile | Cargo feature `scripting-float` enabling full Rhai floats (disable `only_i32`) for DK1 only |
| Dual transport | Single eval task, two feeders: BLE NUS + USART VCP (merge `usart_rhai` architecture) |
| Watchdog | Pet IWDG in BLE task; Rhai `sleep` must not starve it — document max script runtime |

---

## Demo scripts (for README / factory test)

```text
// Knight Rider
for cycle in 0..3 {
    led(0, true); sleep(100); led(0, false);
    led(1, true); sleep(100); led(1, false);
    led(2, true); sleep(100); led(2, false);
    led(1, true); sleep(100); led(1, false);
}

// Joystick paint
let j = joy();
if j == 4 { rgb(false,true,false) }
if j == 3 { rgb(true,true,false) }
if j == 2 { rgb(true,false,false) }
if j == 5 { rgb(false,false,true) }
if j == 1 { rgb(true,true,true) }

// Joystick poll loop (OLED + RGB, switch)
fn wait(ms){
    let t=ms*32768 / 1000;
    let s=ts();
    while ts()-s<t{}
}
fn oled(s){
    oled_line(5, s);
}
for i in 0..1000{
    let j = joy();
    switch j {
    0 => { rgb(0, 0, 0); oled("none"); }
    1 => { rgb(1, 1, 1); oled("select"); }
    2 => { rgb(1, 0, 0); oled("left"); }
    3 => { rgb(1, 1, 0); oled("down"); }
    4 => { rgb(0, 1, 0); oled("up"); }
    5 => { rgb(0, 0, 1); oled("right"); }
    _ => { rgb(0, 0, 0); oled("?"); }
    }
    wait(300);
}

// Heap stress (should fail cleanly)
let a = []; for i in 0..5000 { a += i; } len(a)
```

---

## Bring-up order (new features)

1. `temp_c()` + `rand()` — no new tasks
2. `sleep(ms)` + LED contention fix
3. `CorePackage` trial + stack measurement
4. Joystick BLE edge events
5. Grove / Arduino GPIO table
6. PDM microphone level meter
7. Web BLE companion

---

## References

- Board: [STM32WBA65I-DK1](https://www.st.com/en/evaluation-tools/stm32wba65i-dk1.html), schematics MB2130 + MB2143
- Zephyr pin map: `boards/st/stm32wba65i_dk1/stm32wba65i_dk1.dts`
- Origin demo: `examples/stm32wba5mm/src/bin/ble_rhai.rs`
- Serial fallback: `examples/stm32wba65i-dk1/src/bin/usart_rhai.rs`
