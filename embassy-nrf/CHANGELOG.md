# Changelog for embassy-nrf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- added: Add basic RTC support for nRF54L
- changed: apply trimming values from FICR.TRIMCNF on nrf53/54l
- changed: do not panic on BufferedUarte overrun
- added: allow direct access to the input pin of `gpiote::InputChannel`
- bugfix: use DETECTMODE_SEC in GPIOTE in secure mode
- added: allow configuring the idle state of GPIO pins connected to PWM channels
- changed: allow configuring the PWM peripheral in the constructor of `SimplePwm`
- changed: support setting duty cycles with inverted polarity in `SimplePwm`
- added: support setting the duty cycles of all channels at once in `SimplePwm`
- changed: updated to nrf-pac with nrf52/nrf53/nrf91 register layout more similar to nrf54
- added: support for nrf54l peripherals: uart, gpiote, twim, twis, spim, spis, dppi, pwm, saadc, cracen
- added: support for changing nrf54l clock speed
- bugfix: Do not write to UICR from non-secure code on nrf53
- bugfix: Add delay to uart init anomaly fix
- changed: `BufferedUarte::read_ready` now uses the same definition for 'empty' so following read calls will not block when true is returned

## 0.8.0 - 2025-09-30

- changed: Remove `T: Instance` generic params in all drivers.
- changed: nrf54l: Disable glitch detection and enable DC/DC in init.
- changed: Add embassy-net-driver-channel implementation for IEEE 802.15.4
- changed: add persist() method for gpio and ppi
- added: basic RTC driver
- changed: add persist() method for gpio, gpiote, timer and ppi
- changed: impl Drop for Timer
- added: expose `regs` for timer driver
- added: timer driver CC `clear_events` method
- changed: Saadc reset in Drop impl, anomaly 241 - high power usage

## 0.7.0 - 2025-08-26

- bugfix: use correct analog input SAADC pins on nrf5340

## 0.6.0 - 2025-08-04

- changed: update to latest embassy-time-queue-utils

## 0.5.0 - 2025-07-16

- changed: update to latest embassy-usb-driver

## 0.4.1 - 2025-07-14

- changed: nrf52833: configure internal LDO
- changed: nrf5340: add more options to clock config
- bugfix: clean the SAADC's register while dropping
- changed: Remove Peripheral trait, rename PeripheralRef->Peri.
- changed: take pins before interrupts in buffered uart init
- changed: nrf5340: add wdt support
- changed: remove nrf radio BLE
- changed: add Blocking/Async Mode param.
- bugfix: fix PWM loop count
- bugfix: fixing the nrf54l drive configuration bug
- changed: add temp driver for nrf5340
- changed: add support for rand 0.9

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

