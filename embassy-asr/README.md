# embassy-asr

[Embassy](https://embassy.dev/) support for ASR microcontrollers.

## Current support

- ASR6601, verified against the official `tremo.svd` v1.6.2 and the ASR6601
  Reference Manual v1.5.0
- Peripheral ownership and type-level interrupt infrastructure
- `embassy-time` driver using LPTIM0 at 32,768 Hz from XO32K
  (`time-driver-lptim0`, enabled by default)
- Cortex-M thread executor support through `embassy-executor`
- Drivers for GPIO, UART, LPUART, SPI, I2C, timers, DMA, ADC, DAC, CRC, RNG,
  flash, power, AFEC, and RCC

## Documentation

- [API documentation](https://docs.embassy.dev/embassy-asr/)
- [Source](https://github.com/embassy-rs/embassy/tree/main/embassy-asr)

## Time driver

The default `time-driver-lptim0` feature uses LPTIM0 as Embassy's monotonic
clock. It requires a working 32.768 kHz crystal on XO32K, takes exclusive
ownership of LPTIM0, and enables the `LPTIM0` interrupt at NVIC priority 2.
Application code must not access LPTIM0 through the raw PAC.

Register programming follows the vendor `tremo_lptimer` / `tremo_rcc` drivers
and the SDK `lptimer_wakeup_stop` example. The PAC is `asr6601-pac` 0.1.2.

## Critical sections

The application must provide a `critical-section` implementation. For a
single-core Cortex-M application, enable `cortex-m`'s
`critical-section-single-core` feature.

## Hardware validation

The following paths were exercised on an ASR6601SE:

- GPIO, RCC clock/reset, LPTIM time driver, GPTIM/BSTIM, IWDG/WWDG, and CRC
- UART0/1 and LPUART, including UART1 RX DMA with live GNSS NMEA data
- SPI loopback using the PL022 internal loopback bit
- I2C0/I2C2 master and I2C0 master ↔ I2C1 slave loopback
- ADC and DAC analog paths, including the internal DAC-to-ADC channel
- DMA memory-to-memory transfers across 8/16/32-bit widths, burst sizes and
  increment/fixed/decrement address modes, up to the 2047-item limit
- DMA status/mask/clear registers and TFR/BLOCK/SRC_TRAN/DST_TRAN interrupts
- DMA reload, abort/suspend, and linked lists on channel 0 of both controllers
- SSP2 TX/RX DMA request routing through the SYSCFG mux
- ADC DMA from the internal DAC channel

Known DMA caveats:

- `BLOCK_TS` is 11 bits, so a transfer is limited to 2047 items.
- Linked lists are implemented only on DMA channel 0; channels 1–3 reject
  linked-list transfers.
- SSP RX DMA requires four-item bursts. One-item bursts leave up to three
  bytes unread in the RX FIFO.
- DMA completion status can be missed if it is polled only after the channel
  auto-disables. Use the interrupt path or the channel-enable bit as a
  fallback.
- `STATUS_ERR`/`RAW_ERR`/`CLEAR_ERR` were not induced by the safe error tests.
- DAC DMA requires a timer/TRGO trigger; it has not been validated.
- UART TX DMA has not been data-verified because there is no loopback path.

## Limitations

- The time driver requires an external 32.768 kHz crystal. Boards without one
  cannot use `time-driver-lptim0`.
- The LPTIM0 compare channel on this silicon can miss very short deadlines;
  sub-millisecond alarms may be deferred by up to one 16-bit counter period.
  See `src/time_driver.rs` for details.
