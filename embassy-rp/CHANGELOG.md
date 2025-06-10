# Changelog for embassy-rp

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.4.0 - 2025-03-09

- Add PIO functions. ([#3857](https://github.com/embassy-rs/embassy/pull/3857))  
  The functions added in this change are `get_addr` `get_tx_threshold`, `set_tx_threshold`, `get_rx_threshold`, `set_rx_threshold`, `set_thresholds`.
- Expose the watchdog reset reason. ([#3877](https://github.com/embassy-rs/embassy/pull/3877))
- Update pio-rs, reexport, move instr methods to SM. ([#3865](https://github.com/embassy-rs/embassy/pull/3865))
- rp235x: add ImageDef features. ([#3890](https://github.com/embassy-rs/embassy/pull/3890))
- doc: Fix "the the" ([#3903](https://github.com/embassy-rs/embassy/pull/3903))
- pio: Add access to DMA engine byte swapping ([#3935](https://github.com/embassy-rs/embassy/pull/3935))
- Modify BufferedUart initialization to take pins before interrupts ([#3983](https://github.com/embassy-rs/embassy/pull/3983))

## 0.3.1 - 2025-02-06

Small release fixing a few gnarly bugs, upgrading is strongly recommended.

- Fix a race condition in the time driver that could cause missed interrupts. ([#3758](https://github.com/embassy-rs/embassy/issues/3758), [#3763](https://github.com/embassy-rs/embassy/pull/3763))
- rp235x: Make atomics work across cores. ([#3851](https://github.com/embassy-rs/embassy/pull/3851))
- rp235x: add workaround "SIO spinlock stuck after reset" bug, same as RP2040 ([#3851](https://github.com/embassy-rs/embassy/pull/3851))
- rp235x: Ensure core1 is reset if core0 resets. ([#3851](https://github.com/embassy-rs/embassy/pull/3851))
- rp235xb: correct ADC channel numbers. ([#3823](https://github.com/embassy-rs/embassy/pull/3823))
- rp235x: enable watchdog tick generator. ([#3777](https://github.com/embassy-rs/embassy/pull/3777))
- Relax I2C address validity check to allow using 7-bit addresses that would be reserved for 10-bit addresses. ([#3809](https://github.com/embassy-rs/embassy/issues/3809), [#3810](https://github.com/embassy-rs/embassy/pull/3810))

## 0.3.0 - 2025-01-05

- Updated `embassy-time` to v0.4
- Initial rp235x support
- Setup timer0 tick when initializing clocks
- Allow separate control of duty cycle for each channel in a pwm slice by splitting the Pwm driver.
- Implement `embedded_io::Write` for Uart<'d, T: Instance, Blocking> and UartTx<'d, T: Instance, Blocking>
- Add `set_pullup()` to OutputOpenDrain.

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
