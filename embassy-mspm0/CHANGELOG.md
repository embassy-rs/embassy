# Changelog for embassy-mspm0

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- feat: Add I2C Controller (blocking & async) + examples for mspm0l1306, mspm0g3507 (tested MCUs) (#4435)
- fix gpio interrupt not being set for mspm0l110x
- feat: Add window watchdog implementation based on WWDT0, WWDT1 peripherals (#4574)
- feat: Add MSPM0C1105/C1106 support
- feat: Add adc implementation (#4646)
- fix: gpio OutputOpenDrain config (#4735)
- fix: add MSPM0C1106 to build test matrix
- feat: add MSPM0H3216 support
- feat: Add i2c target implementation (#4605)
- fix: group irq handlers must check for NO_INTR (#4785)
- feat: Add read_reset_cause function
- feat: Add module Mathacl & example for mspm0g3507 (#4897)
- feat: Add MSPM0G5187 support
- feat: add CPU accelerated division function (#4966)
- feat: Add trng implementation (#5172)
