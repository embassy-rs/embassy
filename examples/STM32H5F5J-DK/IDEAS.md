# STM32H5F5J-DK — Projektideen

Ausgangspunkt: Kopie von `examples/stm32h5` (Nucleo-H563ZI). Zielplattform ist das
**STM32H5F5J-DK** Discovery Kit mit externem **gen4-FT813** Display über SPI — nicht das
eingebaute Board-Display.

## Board

| | |
|---|---|
| MCU | STM32H5F5LJH7Q (Cortex-M33, TrustZone, 250 MHz) |
| Flash / RAM | 4 MB / 1,5 MB |
| Onboard Display | 4,3″, 480×272, **LTDC parallel RGB** (wird für FT813-Projekt **nicht** genutzt) |
| Onboard Touch | ST1633I / FT5336 über **I2C** (nur fürs Board-Display relevant) |
| CAN | 3× FDCAN |
| SPI | 6× |
| Debug | ST-Link V3, USB-CDC (USART2 → VCP) |
| Extern | Arduino Uno V3, Pmod, STMod+ |

Referenzen ST: [STM32H5F5J-DK](https://www.st.com/en/evaluation-tools/stm32h5f5j-dk.html),
STM32CubeH5 BSP unter `Projects/STM32H5F5J-DK`.

## Warum gen4-FT813 statt Board-LTDC?

Das gen4-Modul hat einen **FT813Q (EVE2)** im Display — Host spricht **SPI**, Touch ist
integriert. Kein LTDC, kein externer Framebuffer auf dem MCU.

OxivGL/LVGL-Pfad: **`LV_USE_DRAW_EVE`** + SPI `op_cb` (noch nicht implementiert in diesem
Crate).

Wiederverwendbar aus anderen Embassy-Examples:

- `examples/rvt50hqsnwc00-b` — OxivGL UI-Loop, Widgets, Touch-over-CAN-Muster
- `examples/stm32h5/src/bin/can.rs` — FDCAN-Grundgerüst (Pins ggf. anpassen)

Nicht wiederverwendbar: LTDC/`display.rs` vom rvt50, alles RP2350/PIO vom gen4-RP2350.

## Hardware-Anschluss gen4-FT813

Empfohlen: **gen4-PA** Breakout am **Arduino-Header** (SPI1, laut Zephyr/ST Devicetree).

| FT813 (FFC) | Arduino | MCU-Pin | Hinweis |
|-------------|---------|---------|---------|
| SPI_CS | D10 | PA4 | Chip Select |
| SPI_MOSI | D11 | PB5 | |
| SPI_MISO | D12 | PG9 | auch LTDC_B5 — LTDC nicht initialisieren |
| SPI_SCK | D13 | PG11 | |
| SPI_PD | D7 o.ä. | freier GPIO | Power-Down, aktiv low |
| SPI_INT | D2 o.ä. | freier GPIO | optional, EXTI |
| +5 V | 5V | | Display braucht **5 V**, nicht nur 3,3 V |
| GND | GND | | |

Alternativ: **Pmod** oder freies SPI2/SPI3 ohne LTDC-Pin-Konflikte.

Display-Varianten:

- **gen4-FT813-43CT** — 480×272 (gleiche Auflösung wie Board-LCD)
- **gen4-FT813-70CT** — 800×480

## CAN

Board hat 3× FDCAN. Zephyr-Default für FDCAN1:

- RX: PA11
- TX: PA12

## JSON-Knoten (`json_node`) — CANbossTouch-PoC

`src/bin/json_node.rs` + `json_node.json`: Ein JSON beschreibt einen CANopen-artigen
Knoten (Objektverzeichnis + Rhai-Skripte + CAN-Konfiguration); die Firmware ist
generisch. Gleicher Stand wie `examples/STM32H573I-DK` (dort ist die README mit
API-Tabelle und CAN-Framing dokumentiert). LEDs/Button sind hier noch
**Nucleo-H563ZI-Platzhalter** (PB0/PF4/PG4, PC13) bis H5F5-Chipsupport da ist;
CAN `"mode": "loopback"` (Default) läuft davon unabhängig.

```bash
cargo run --release --bin json_node --features scripting,json
```

Transceiver + Terminierung auf Custom-Board / Adapter. CAN-Logik und OxivGL-Touch-CAN
von `rvt50hqsnwc00-b` als Vorlage (`can_raw.rs`, `oxivgl_touch_can.rs`).

## Geplante Crate-Struktur (später)

```
STM32H5F5J-DK/
├── src/bin/
│   ├── blinky.rs          # Bring-up
│   ├── can.rs             # FDCAN (Pins H5F5J-DK)
│   ├── spi_ft813.rs       # EVE Hello-World / Register-Probe
│   └── oxivgl_eve.rs      # OxivGL + LV_USE_DRAW_EVE + UI
├── conf/                  # lv_conf.h (EVE an)
└── IDEAS.md               # diese Datei
```

## Embassy-Status / offene Punkte

1. **`stm32h5f5` fehlt noch in `embassy-stm32`** — aktuell Platzhalter-Feature `stm32h563zi`
   aus der H5-Nucleo-Kopie. Build/Peripherie-Pins passen nicht 1:1 zum Discovery; für echtes
   H5F5-Targeting brauchen wir Metapac/Chipsupport in `stm32-data-generated`.
2. **probe-rs**: `.cargo/config.toml` → `--chip STM32H5F5LJH7Q` (Board-MCU).
3. **OxivGL + EVE**: neuer Display-Treiber (SPI-Glue), kein LTDC-Flush.
4. **Custom Board** (langfristig): gleicher MCU oder STM32U5A9; FT813 bleibt SPI; CAN bleibt
   FDCAN.

## Bring-up-Reihenfolge

1. Blinky + defmt über ST-Link VCP
2. SPI-Loopback / FT813-Chip-ID lesen
3. EVE: Farbe / Rechteck auf gen4
4. FDCAN1 Loopback oder Bus mit Transceiver
5. OxivGL EVE-Backend + Widget-UI von rvt50 portieren
6. Optional: Touch-Events per CAN (wie rvt50)

## Build (vorläufig)

```bash
cd examples/STM32H5F5J-DK
cargo run --bin blinky
```

Solange `stm32h563zi` gesetzt ist, ist das eher ein **Software-Gerüst** — erst nach
H5F5-Chipsupport oder manuellem Pin-Mapping auf dem echten Board sinnvoll testen.

---

## Recherche (Stand Juni 2025)

Zusammenfassung der Recherche aus dem Embassy-Repo, ST-Dokumentation, Zephyr-Devicetree
und 4D-Systems-Datasheets — als Kontext für dieses Projekt und ein späteres Custom-Board.

### Kontext: drei Display-Welten im Repo

| Plattform | Display-Anbindung | Touch | OxivGL im Repo? |
|-----------|-------------------|-------|-----------------|
| **gen4-RP2350-70CT** (`examples/gen4-rp2350-70ct`) | RP2350 + **PIO parallel RGB** 800×480, Blit in PSRAM | FT5446, **I2C** | ja (partial mode → `pio_rgb`) |
| **gen4-FT813** (4D-Modul, **noch kein Example**) | **FT813Q EVE2 im Modul**, Host **SPI** ~30 MHz mode 0 | im FT813 integriert | nein — braucht `LV_USE_DRAW_EVE` |
| **rvt50** (`examples/rvt50hqsnwc00-b`) | STM32U5A9 + **LTDC** RGB565 800×480 | Kapazitiv, **I2C** | ja (LTDC double-buffer) |

**gen4-FT813 ≠ gen4-RP2350-70CT** — gleiche Herstellerlinie, völlig andere Architektur.
FT813-Code existiert im Embassy-Repo **nicht**; RP2350/LTDC-Treiber sind **nicht** portierbar.

Breakout-Optionen 4D Systems:

- **gen4-PA** — FFC → 2,54-mm-Pads, SPI + PD + INT
- **gen4-IB** — minimal, eher für Programmierung

Referenz: [gen4-FT81x Series Datasheet](https://resources.4dsystems.com.au/datasheets/ft8xx/gen4-FT81x-Series/)

### Eingebautes H5F5J-DK-Display (Details)

Das Board-LCD ist ein **paralleles RGB-Panel ohne externen Controller auf dem MCU** — der
**LTDC** des STM32 streamt Pixel in Echtzeit (MIPI-DPI-ähnlich, kein SPI).

Aus `STM32CubeH5` BSP (`Projects/STM32H5F5J-DK/Examples/BSP/BSP.ioc`):

- Auflösung laut ST1633I-BSP: **480×272**
- LTDC-Signale (Auszug): `PA8` CLK, `PA9` R7, `PA12` R4, `PB3` HSYNC, `PB4` R2, `PB7` G7,
  `PB8` G6, `PB9` G5, `PC11` VSYNC, `PC7` R1, `PD1` DE, `PD3` B4, `PD4` B7, `PD7` R3,
  `PE3` G1, `PE4` G3, `PG10` B6, `PG14` B0, `PG15` G2, `PG9` B5, `PI0` B1, `PI12` R5,
  `PI5` R0, `PJ14` R6, `PK12` B3, …
- Framebuffer laut `stm32h5f5j_discovery_conf.h`: Layer0 @ `0x20060000`, Layer1 @ `0x200C0000`
  (internes SRAM)
- **DMA2D** optional zum Füllen (`USE_DMA2D_TO_FILL_RGB_RECT`)

**Touch** (nur Board-LCD):

- Controller: **ST1633I** (I2C, Multi-Touch, Gesten) — BSP nennt zusätzlich FT5336-Flag
- I2C1: `PK9` SCL, `PK10` SDA
- I2C4: `PB6` SCL, `PG6` SDA (u. a. Audio-Codec CS42L51)

Für FT813-Projekt: LTDC + Board-Touch **nicht initialisieren**.

### gen4-FT813 — Host-Schnittstelle (Recherche)

30-poliger FFC am Modul (Logik **3,3 V**, nicht 5V-tolerant):

| Pin | Signal | Richtung | Beschreibung |
|-----|--------|----------|--------------|
| 2 | SPI_PD | Input | Power-Down (Host) |
| 3 | SPI_INT | Output | Interrupt (optional) |
| 4 | SPI_CS | Input | Chip Select |
| 6 | SPI_MOSI | Input | |
| 7 | SPI_MISO | Output | |
| 8 | SPI_SCK | Input | |
| 26–27 | +5 V | Power | Modulversorgung |

LVGL 9.x: EVE-Backend über `lv_draw_eve_display_create()` + `lv_draw_eve_touch_create()`
mit SPI `op_cb` — **kein** `flush_cb` wie bei LTDC/PIO.

Im rvt50/gen4-rp2350 ist `LV_USE_DRAW_EVE 0`. Neues Example braucht eigenen SPI-Glue.

Modellwahl:

| Modell | Auflösung | Anmerkung |
|--------|-----------|-----------|
| gen4-FT813-43CT-CLB | 480×272 | passt zur Board-LCD-Auflösung |
| gen4-FT813-70CT-CLB | 800×480 | gleiche Auflösung wie rvt50 / gen4-RP2350 |

### H5F5J-DK — Peripherie-Recherche (Zephyr + Cube)

MCU **STM32H5F5LJH7Q**, TFBGA225, bis **250 MHz** (BSP/Cube).

Weitere Onboard-Hardware:

- **512 Mbit Octo-SPI NOR** (MX25LM51245G) — XSPI1
- **64 Mbit Octo-SPI PSRAM** (APS6408L) — XSPI2 @ `0x70000000`
- Ethernet 10/100, microSD, USB-C PD (Sink), JPEG-HW, ST-Link V3E

**FDCAN** (laut Zephyr `stm32h5f5j_dk-common.dtsi`):

| Bus | TX | RX | Default |
|-----|----|----|---------|
| FDCAN1 | PA12 | PA11 | enabled (`zephyr,canbus`) |
| FDCAN2 | PB13 | PB12 | enabled |
| FDCAN3 | PE12 | PE13 | disabled (Konflikt ADC3) |

**Arduino SPI1** (für externen FT813 vorgesehen):

| Signal | Pin |
|--------|-----|
| NSS/CS | PA4 (D10) |
| SCK | PG11 (D13) |
| MISO | PG9 (D12) — **Pin-Konflikt** mit LTDC_B5 |
| MOSI | PB5 (D11) |

Weitere Arduino-Pins (Zephyr): D0/D1 = UART7 PE7/PE8, D2 = PA8 (auch LTDC_CLK!), …
→ PD/INT besser auf Pins **ohne** LTDC wählen (z. B. D4/D6).

**USART2** (defmt/VCP): PA2 TX, PA3 RX.

### MCU-Vergleich (Custom-Board-Recherche)

Recherche für späteres eigenes Board (FT813 + CAN + OxivGL):

| MCU | Flash | RAM | FDCAN | OxivGL im Repo | Anmerkung |
|-----|-------|-----|-------|----------------|-----------|
| **STM32H5F5** (dieses DK) | 4 MB | 1,5 MB | 3× | nein | 250 MHz, JPEG, sehr neu |
| **STM32U5A9** (rvt50) | 4 MB | 2,5 MB | 1× | **ja** | max. Code-Reuse OxivGL+CAN |
| **STM32H563** (Nucleo) | 2 MB | 640 KB | 2× | nein | günstiges Prototyping, `examples/stm32h5` |

Empfehlung langfristig Custom-Board: **STM32U5A9** wenn maximaler OxivGL-Reuse;
**STM32H5F5** wenn H5-Leistung/3× CAN/JPEG gewünscht.

### Embassy-Repo — was existiert, was fehlt

**Vorhanden:**

- `examples/stm32h5/` — H563: blinky, **can.rs** (FDCAN1 PA11/PA12), eth, usb, i2c, …
- `examples/rvt50hqsnwc00-b/` — OxivGL, LTDC, Touch, **can_raw.rs**, **oxivgl_touch_can.rs**
- `examples/gen4-rp2350-70ct/` — OxivGL + PIO-RGB (Touch noch problematisch)

**Fehlt:**

- Kein `stm32h5f5*` Feature in `embassy-stm32/Cargo.toml` (Stand Recherche: nur H503/H523/H562/H563/H573)
- Kein gen4-FT813 / EVE-Example
- Kein FT813-Code im gesamten Repo

**Wiederverwendung für dieses Projekt:**

| Von | Was |
|-----|-----|
| rvt50 | `platform.rs`, `widget_view.rs`, UI-Loop, CAN-Touch-Muster |
| stm32h5/can.rs | FDCAN-Setup (Pins an H5F5J-DK anpassen) |
| — | Display: **neu** (EVE/SPI), nicht LTDC |

### OxivGL-Integrationspfad (Recherche)

```
Host (H5F5)                    gen4-FT813 Modul
    │                                │
    ├── SPI (MOSI/MISO/SCK/CS) ─────► FT813Q Coprocessor
    ├── GPIO PD, INT (opt.)  ─────►
    │                                ├── internes Rendering
    │                                └── integrierter Touch
    │
    ├── FDCAN + Transceiver ───────► Bus (Touch-Events, Telemetrie)
    └── SWD / USB-CDC ─────────────► defmt Debug
```

Schritte:

1. `LV_USE_DRAW_EVE 1` in `lv_conf.h`
2. Embassy SPI async + CS/PD GPIO
3. LVGL EVE `op_cb` implementieren
4. UI von rvt50 `widget_view.rs` übernehmen
5. Optional: Touch → CAN wie `oxivgl_touch_can.rs`

### ST-Referenzen

- [STM32H5F5J-DK Produktseite](https://www.st.com/en/evaluation-tools/stm32h5f5j-dk.html) — Schematics (MB2114, MB2274-LCD43)
- [STM32CubeH5](https://github.com/STMicroelectronics/STM32CubeH5) — `Projects/STM32H5F5J-DK/Examples/BSP`
- [Zephyr board doc](https://docs.zephyrproject.org/latest/boards/st/stm32h5f5j_dk/doc/index.html)
- [AN4861 LTDC on STM32](https://www.st.com/resource/en/application_note/an4861-lcdtft-display-controller-ltdc-on-stm32-mcus-stmicroelectronics.pdf) — erklärt LTDC vs SPI-Display (DBI type C = SPI)

### Bekannte Risiken / offene Fragen

1. **Embassy H5F5-Support** — Chip sehr neu; bis Metapac-Update ggf. CubeH5/HAL zum Pin-Abgleich
2. **SPI vs LTDC Pin-Konflikte** — Arduino SPI1 nutzt LTDC-Pins; LTDC muss disabled bleiben
3. **5V-Versorgung Display** — Strombedarf des gen4-Moduls am DK-5V prüfen
4. **Touch auf RP2350-Example** — separates Problem; FT813-Touch läuft über EVE, nicht FT5446
5. **probe-rs** — Chip-String `STM32H5F5LJH7Q`; OpenOCD-Support ggf. noch eingeschränkt (Zephyr-Hinweis)
