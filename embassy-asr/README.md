# embassy-asr

[Embassy](https://embassy.dev/) support for ASR microcontrollers.

## Current support

- ASR6601 (verified against official `tremo.svd` v1.6.2 and the ASR6601
  Reference Manual v1.5.0)
- Peripheral ownership and type-level interrupt infrastructure
- `embassy-time` driver using LPTIM0 at 32,768 Hz from XO32K (`time-driver-lptim0`)
- Cortex-M thread executor support through `embassy-executor`
- Early HAL drivers for GPIO, UART, SPI, I2C, timers, DMA, ADC, DAC,
  LPUART, LPTIMER, CRC, RNG, flash, power, AFEC, and RCC

## LPTIM0 time driver

Enable the `time-driver-lptim0` feature (default) to use LPTIM0 as Embassy's
monotonic clock. `time-driver-rtc` is deprecated but kept for compatibility.

The driver:

- requires a working 32.768 kHz crystal on XO32K (same always-on domain as RTC);
- resets LPTIM0, selects XO32K in `RCC.CR1` (`lptimer0_clk_sel`), sets
  `ARR=0xFFFF`, enables `ARRM` for 16-to-64 bit extension and `CMPM` for alarms
  during `embassy_asr::init(Config::default())`;
- owns LPTIM0 exclusively, so application code must not access LPTIM0 through the
  raw PAC;
- free-runs at 32,768 Hz (2 s per overflow), defers far-future compares beyond
  0xC000 ticks to `next_period()` like `embassy-stm32/src/time_driver/lptim.rs`;
- supports one global Embassy time driver; and
- enables the `LPTIMER0` interrupt at NVIC priority 2.

Register programming follows vendor `tremo_lptimer` / `tremo_rcc` and the SDK
`lptimer_wakeup_stop` example. PAC is `Rimpampa/ASR6601-PAC@svd` (official
`tremo.svd` v1.6.2).

## RTC time driver (deprecated)

Enable `time-driver-rtc` to use the RTC calendar (kept for backward compat).

The application must provide a `critical-section` implementation. For a
single-core Cortex-M application, enable `cortex-m`'s
`critical-section-single-core` feature.

The current initialization intentionally does not preserve a bootloader or
previous low-power session's RTC calendar. Applications that need retained
wall-clock state must save it before initialization and restore it separately.
