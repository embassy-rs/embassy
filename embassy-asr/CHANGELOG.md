# Changelog for embassy-asr

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

- Initial HAL bring-up for the ASR6601.
- GPIO, UART, LPUART, SPI, I2C, timer, LPTIMER, DMA, ADC, DAC, CRC, RNG,
  flash, power, AFEC, watchdog, and RCC drivers.
- `embassy-time` driver using LPTIM0 at 32,768 Hz.
- Use the `asr6601-pac` 0.1.2 field accessors for LPTIM, GPTIM, BSTIM, RNG,
  SYSCFG, I2C, DMA controller status/mask/clear, IWDG, and the AFEC analog
  window.
- RCC dual-domain and LPTIM clock gates now use PAC fields instead of raw
  bit masks.
- Clock-frequency and LPTIM flag constants are shared between the RCC, UART,
  LPUART, and time-driver modules instead of being duplicated.
- UART and LPUART share their data-bit, stop-bit, and flow-control enums.
- DMA transfers are limited to 2047 items. Hardware `BLOCK_TS` is 11 bits;
  the vendor header's 12-bit definition truncates larger counts.
- Linked-list transfers are rejected on DMA channels 1-3. Only channel 0
  implements the `LLP` register on this hardware.
- SPI DMA uses four-item bursts. With one-item bursts the SSP RX DMA request
  leaves the final three FIFO bytes unread.
- Document the hardware-verified peripheral and DMA paths, including the
  channel-0 linked-list and 2047-item limits.
