# HIL setup

## Implemented:
- [x] GPIO + interrupts
- [x] I2C controller + target
- [x] Uart
- [x] LPUART BBQueue continuous DMA RX
- [x] ADC
- [x] I3C
- [x] CRC
- [x] TRNG blocking/async reads, mixed-mode calls, and stale-IRQ rejection
- [x] Watchdog
- [x] RTC datetime readback, free-running rollover, resetting a running counter, and repeated async alarms (no wiring)
- [ ] Flash
- [x] CTimer PWM
- [x] CTimer capture
- [x] LPTMR polling, async waits, cancellation, prescalers and FRO16K
- [x] LPTMR Embassy time backend (additional `time-driver-lptmr` build)
- [x] FREQME blocking/async one-shot and continuous internal clocks including SLOW_CLK, cancellation, IRQs and reconstruction (no wiring)
- [x] FREQME runtime scale/source/power reconfiguration and threshold-error recovery
- [x] ERM/EIM single-bit SRAM ECC diagnostics, counter saturation, async IRQs, cancellation and reconstruction (no wiring)
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

## RTC

The tests in this crate target MCXA266 only. Build the RTC regression with
`cargo build --release --bin rtc_alarm`, then run it with
`teleprobe local run --chip MCXA266 --probe <probe-serial> --protocol swd --elf target/thumbv8m.main-none-eabihf/release/rtc_alarm`.

MCXA287 uses the MCXA5-family calendar RTC, not this crate's seconds-counter RTC.
Its RTC rollover and repeated-alarm checks live in
`examples/mcxa287/src/bin/rtc-alarm.rs`.

## ERM/EIM

Build from this directory with `cargo build --release --bin erm_eim`. This
RAM-linked binary alone places vectors, code, data, RTT and stack above SRAM A0
(`0x20002000`, 120 KiB). Its linker reserves two scratch words at `0x20000000`
and `0x20000004`, initialized with full-word writes before injection. Other
binaries retain the original layout; the build script selects `ecc-scratch.x`
for this binary and `ram.x` for the others using bin-scoped linker arguments.

ECC must already be enabled in `SYSCON.RAM_CTRL`; the test checks but never
changes this policy. No DMA or other tasks run. Do not use debugger memory
views or polling in `0x20000000..0x20002000`; RTT lives outside that bank.
Only bounded single-bit pulses are armed. Double-bit descriptors are checked
only while disabled; no uncorrectable events or flash faults are generated.
Every assertion, timeout, log and await occurs with injection disabled.

Source, final ELF layout and injection disassembly require independent review
before any hardware run. The test finishes with `Test OK` and a breakpoint.
