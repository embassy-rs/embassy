# Embassy NXP MCX-A MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-A family of
microcontrollers using the Embassy async framework.

The `embassy-mcxa` HAL currently provides safe, idiomatic Rust
interfaces for clocks, GPIO, ADC, CLKOUT, CDOG, CRC, CTIMER, DMA,
FlexSPI, I2C, I3C, LPUART, OSTIMER, RTC, SPI, TRNG, and WWDT
peripherals.

FlexSPI support currently targets MCXA5xx devices and the `FLEXSPI0`
peripheral, with blocking, interrupt-driven async, and DMA-backed async
APIs for NOR flash style transfers.
