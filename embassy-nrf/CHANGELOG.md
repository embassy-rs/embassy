# Changelog for embassy-nrf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Drop `sealed` mod
- nrf52840: Add dcdc voltage parameter to configure REG0 regulator
- radio: Add support for IEEE 802.15.4 and BLE via radio peripheral
- spim: Reduce trace-level messages ("Copying SPIM tx buffer..")
- uart: Add support for rx- or tx-only BufferedUart
- uart: Implement splitting Rx/Tx
- spi: Allow specifying OutputDrive for SPI spins
- pdm: Fix gain register value derivation
- spim: Implement chunked DMA transfers
- spi: Add bounds checks for EasyDMA buffer size
- uarte: Add support for handling RX errors
- nrf51: Implement support for nrf51 chip
- pwm: Expose `duty` method
- pwm: Fix infinite loop
- spi: Add support for configuring bit order for bus
- pwm: Expose `pwm::PWM_CLK_HZ` and add `is_enabled` method
- gpio: Drop GPIO Pin generics (API break)

## 0.1.0 - 2024-01-12

- First release with support for following NRF chips:
  - nrf52805
  - nrf52810
  - nrf52811
  - nrf52820
  - nrf52832
  - nrf52833
  - nrf52840
  - nrf5340
  - nrf9160

