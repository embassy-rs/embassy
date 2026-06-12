# Demo touch project

JSON configuration for the OxivGL hall lighting UI with CAN press/hold/repeat on the
**RVT50** (default `TOUCH_PROJECT`).

- `hall_config.json` — UI strings and field metadata for the **5-column shell layout**
  (800×480). Percentage `left`/`top`/`width`/`height` keys are **not used** by the
  OxivGL hall demo; layout pixels are fixed in `hall_view.rs`.
- `can_config.json` — FDCAN / SocketCAN settings, `minp` feedback map, baud rate (`can0`)

Override at build time:

```bash
TOUCH_PROJECT=Demo cargo build ...
```
