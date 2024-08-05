# Changelog for embassy-rp

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.2.0 - 2024-08-05

- Add read_to_break_with_count
- add option to provide your own boot2
- Add multichannel ADC
- Add collapse_debuginfo to fmt.rs macros.
- Use raw slices .len() method instead of unsafe hacks.
- Add missing word "pin" in rp pwm documentation
- Add Clone and Copy to Error types
- fix spinlocks staying locked after reset.
- wait until read matches for PSM accesses.
- Remove generics 
- fix drop implementation of BufferedUartRx and BufferedUartTx
- implement `embedded_storage_async::nor_flash::MultiwriteNorFlash`
- rp usb: wake ep-wakers after stalling
- rp usb: add stall implementation
- Add parameter for enabling pull-up and pull-down in RP PWM input mode
- rp: remove mod sealed.
- rename pins data type and the macro
- rename pwm channels to pwm slices, including in documentation
- rename the Channel trait to Slice and the PwmPin to PwmChannel
- i2c: Fix race condition that appears on fast repeated transfers.
- Add a basic "read to break" function
