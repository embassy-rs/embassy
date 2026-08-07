# HIL setup

## Implemented:
- [x] GPIO + interrupts
- [x] I2C controller + target
- [x] Uart
- [x] ADC
- [x] I3C
- [x] CRC
- [x] Watchdog
- [ ] Flash
- [x] CTimer PWM
- [x] CTimer capture
- [ ] SPI

## Wiring:
- `I2C controller` + `target`: `LPI2C1` on arduino connector `J4` to `LPI2C2` on camera connector `J9`
  - `SDA`: `P1_0` -> `P1_8`
  - `SCL`: `P1_1` -> `P1_9`
- `Uart`: `LPUART3` on mikroBUS connector `J5` to `LPUART2` on arduino connector `J1`
  - `P4_5` (TX) -> `P2_3` (RX)
  - `P4_2` (RX) -> `P2_2` (TX)
- `ADC` + `GPIO` + `CTimer capture` + `CTimer PWM`: `ADC0_A1` on arduino connector `J2` to `CT_INP8` + `CT0_MAT2` on arduino connector `J2`
  - `P2_4` -> `P1_8` (overlaps with I2C SDA but on different connector)
- `I3C`: Only one peripheral, so no loopback tests. But it's connected to the `P3T1755DP` temperature sensor
- `SPI`: `LPSPI0` on mikroBUS connector `J6` to `LPSPI1` on arduino connector `J2`
  - `CS`: `P1_3` -> `P3_11`
  - `SCK`: `P1_1` -> `P3_10` (overlaps with I2C SCL but on different connector)
  - `MISO`/`SDI`: `P1_2` -> `P3_9`
  - `MOSI`/`SDO`: `P1_0` -> `P3_8`

That's 9 wire connections in total.
