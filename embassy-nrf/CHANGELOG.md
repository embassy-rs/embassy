# Changelog for embassy-nrf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.2.0 - 2024-08-05

- Support for NRF chips:
  - nrf51
  - nrf9151
- Support for new peripherals:
  - EGU
  - radio - low-level support for IEEE 802.15.4 and BLE via radio peripheral
- Peripheral changes:
  - gpio: Drop GPIO Pin generics (API break)
  - pdm: Fix gain register value derivation
  - pwm:
    - Expose `duty` method
    - Expose `pwm::PWM_CLK_HZ` and add `is_enabled` method
    - Allow specifying OutputDrive for PWM channels
    - Fix infinite loop
  - spim:
    - Reduce trace-level messages ("Copying SPIM tx buffer..")
    - Support configuring bit order for bus
    - Allow specifying OutputDrive for SPI pins
    - Add bounds checks for EasyDMA buffer size
    - Implement chunked DMA transfers
  - uart:
    - Add support for rx- or tx-only BufferedUart
    - Implement splitting Rx/Tx
    - Add support for handling RX errors
- Miscellaneous changes:
  - Add `collapse_debuginfo` to fmt.rs macros.
  - Drop `sealed` mod
  - nrf52840: Add dcdc voltage parameter to configure REG0 regulator

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

