# embassy-asr

[Embassy](https://embassy.dev/) support for ASR microcontrollers.

## Current support

- ASR6601
- Peripheral ownership and type-level interrupt infrastructure
- `embassy-time` driver using the always-on RTC at 32,768 Hz
- Cortex-M thread executor support through `embassy-executor`
- Early HAL drivers for GPIO, UART, SPI, I2C, timers, DMA, ADC, DAC,
  LPUART, LPTIMER, CRC, RNG, flash, power, AFEC, and RCC

## RTC time driver

Enable the `time-driver-rtc` feature to use the RTC calendar as Embassy's
monotonic clock and its cyclic counter for wakeups.

The driver:

- requires a working 32.768 kHz crystal on XO32K;
- resets RTC and starts its calendar at `2000-01-01 00:00:00` during
  `embassy_asr::init(Config::default())`;
- owns RTC exclusively, so application code must not access RTC through the
  raw PAC;
- rounds hardware wakeups shorter than 164 ticks up to about 5 ms;
- supports one global Embassy time driver; and
- enables the RTC interrupt at NVIC priority 2.

The application must provide a `critical-section` implementation. For a
single-core Cortex-M application, enable `cortex-m`'s
`critical-section-single-core` feature.

The current initialization intentionally does not preserve a bootloader or
previous low-power session's RTC calendar. Applications that need retained
wall-clock state must save it before initialization and restore it separately.
