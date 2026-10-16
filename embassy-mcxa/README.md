# Embassy NXP MCX-A MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-A family of
microcontrollers using the Embassy async framework. This HAL provides
safe, idiomatic Rust interfaces for GPIO, UART, and OSTIMER
peripherals.


## Initial MCXA156 support

Select `mcxa156` for the exact device. This implementation keeps the 12 MHz
reset system clock and provides GPIO on all five banks, OSTIMER, and blocking
LPUART0. Full clock configuration, other peripheral drivers, the LPTMR time
backend and the HAL low-power executor are not implemented for this part.
The examples are under `examples/mcxa156`; probe VCOM is UART, not native USB.

Dedicated pins are excluded from safe HAL pin tokens unless their feature is
enabled. Opted-in pins are initialized in the disabled GPIO state, consistent
with the other MCXA implementations:

| Feature | MCXA156 pins released |
| --- | --- |
| `swd-as-gpio` | P0_0 (SWDIO/TMS), P0_1 (SWCLK/TCLK); prevents normal SWD use |
| `swd-swo-as-gpio` | P0_2 (SWO/TDO) |
| `jtag-extras-as-gpio` | P0_3 (TDI), P0_6 and P3_29 (package-dependent ISPMODE_N) |
| `dangerous-reset-as-gpio` | P1_29 (RESET_B); recovery is not validated |
| `sosc-as-gpio` | P1_30 (XTAL48M), P1_31 (EXTAL48M) |

A156 has no ROSC 32 kHz pin pair in this mapping. Pin tokens describe the
PAC's package union, not the bond-out of every package. Both ISP locations
are conservatively reserved; only one is the ISP input on a given package.
Consult the exact package pinout before connecting peripherals.
