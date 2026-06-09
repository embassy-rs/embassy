# Sporthalle Ludwigsfelde — touch UI project

Example configuration for the RVT50 `lvgl_touch_can` binary.

## Files

| File | Purpose |
|------|---------|
| `hall_config.json` | UI layout (percentage coordinates), labels, field metadata |
| `can_config.json` | FDCAN TX/RX IDs, repeat interval, `minp` feedback bit map |

Configs are compiled into the firmware at build time. Edit the JSON files and rebuild to change layout or protocol mapping.

## Button token order

For each field in `fields[]` (in order): `field:<id>:500`, `field:<id>:300`, `field:<id>:0`.

Then group commands:

- `group:play-fields:500`
- `group:play-fields:300`
- `group:all-fields:0`

Each token maps to one bit in the 6-byte TX frame on ID `512` (`0x200`).

## On-device test checklist

1. Flash `lvgl_touch_can` with probe-rs
2. Press a field lux button — CAN analyzer shows one-hot bit on `0x200`
3. Hold button — frame repeats every `command_repeat_ms` (25 ms)
4. Release — all-zero frame on `0x200`
5. Simulate RX on `0x285` with mapped bit set — matching button highlights
6. Group buttons use the last three token indices
