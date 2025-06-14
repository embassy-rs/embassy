# Changelog for embassy-nrf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.3.1 - 2025-01-09

- bugfix: nrf twim return errors in async\_wait instead of waiting indefinitely
- bugfix: fix missing setting input as disconnected.
- changed: Modify Uarte and BufferedUarte initialization to take pins before interrupts ([#3983](https://github.com/embassy-rs/embassy/pull/3983))


## 0.3.0 - 2025-01-06

Firstly, this release switches embassy-nrf to chiptool-based `nrf-pac`
implementations and lots of improvements, but also changes to API like
peripheral and interrupt naming.

Second big change is a refactoring of time driver contract with
embassy-time-driver. From now on, the timer queue is handled by the
time-driver implementation and `generic-queue` feature is provided by
the `embassy-time-queue-utils` crate. Newly required dependencies are
following:
  - embassy-time-0.4
  - embassy-time-driver-0.2
  - embassy-time-queue-utils-0.1

Add support for following NRF chips:
  - nRF54L15 (only gpio and timer support)

Support for chip-specific features:
  - RESET operations for nrf5340
  - POWER operations (system-off and wake-on-field) for nrf52840 and nrf9160

- nfc:
  - Adds support for NFC Tag emulator driver
- pwm:
  - Fix incorrect pin assignments
  - Properly disconnect inputs when pins are set as output
- uart:
  - `try_write` support for `BufferedUarte`
  - Support for `embedded_io_async` trait
- spim:
  - Support SPIM4 peripheral on nrf5340-app
- time:
  - Generic refactor of embassy-time-driver API
  - Fix for missed executor alarms in certain occasions (issue #3672, PR #3705).
- twim:
  - Implement support for transactions
  - Remove support for consecutive Read operations due to hardware limitations

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

