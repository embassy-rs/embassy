# Changelog for embassy-rp

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- Fix several minor typos in documentation
- Add PIO SPI
- Add PIO I2S input
- Add PIO onewire parasite power strong pullup
- add `wait_for_alarm` and `alarm_scheduled` methods to rtc module ([#4216](https://github.com/embassy-rs/embassy/pull/4216))
- rp235x: use msplim for stack guard instead of MPU
- Add reset_to_usb_boot for rp235x ([#4705](https://github.com/embassy-rs/embassy/pull/4705))
- Add fix #4822 in PIO onewire. Change to disable the state machine before setting y register ([#4824](https://github.com/embassy-rs/embassy/pull/4824))
- Add PIO::Ws2812 color order support

## 0.8.0 - 2025-08-26

## 0.7.1 - 2025-08-26
- add `i2c` internal pullup options ([#4564](https://github.com/embassy-rs/embassy/pull/4564))

## 0.7.0 - 2025-08-04

- changed: update to latest embassy-time-queue-utils

## 0.6.0 - 2025-07-16

- update to latest embassy-usb-driver

## 0.5.0 - 2025-07-15

- Fix wrong `funcsel` on RP2350 gpout/gpin ([#3975](https://github.com/embassy-rs/embassy/pull/3975))
- Fix potential race condition in `ADC::wait_for_ready` ([#4012](https://github.com/embassy-rs/embassy/pull/4012))
- `flash`: rename `BOOTROM_BASE` to `BOOTRAM_BASE` ([#4014](https://github.com/embassy-rs/embassy/pull/4014))
- Remove `Peripheral` trait & rename `PeripheralRef` to `Peri` ([#3999](https://github.com/embassy-rs/embassy/pull/3999))
- Fix watchdog count on RP235x ([#4021](https://github.com/embassy-rs/embassy/pull/4021))
- I2C: ensure that wakers are registered before checking status of `wait_on` helpers ([#4043](https://github.com/embassy-rs/embassy/pull/4043))
- Modify `Uarte` and `BufferedUarte` initialization to take pins before interrupts ([#3983](https://github.com/embassy-rs/embassy/pull/3983))
- `uart`: increase RX FIFO watermark from 1/8 to 7/8 ([#4055](https://github.com/embassy-rs/embassy/pull/4055))
- Add `spinlock_mutex` ([#4017](https://github.com/embassy-rs/embassy/pull/4017))
- Enable input mode for PWM pins on RP235x and disable it on drop ([#4093](https://github.com/embassy-rs/embassy/pull/4093))
- Add `impl rand_core::CryptoRng for Trng` ([#4096](https://github.com/embassy-rs/embassy/pull/4096))
- `pwm`: enable pull-down resistors for pins in `Drop` implementation ([#4115](https://github.com/embassy-rs/embassy/pull/4115))
- Rewrite PIO onewire implementation ([#4128](https://github.com/embassy-rs/embassy/pull/4128))
- Implement RP2040 overclocking ([#4150](https://github.com/embassy-rs/embassy/pull/4150))
- Implement RP235x overclocking ([#4187](https://github.com/embassy-rs/embassy/pull/4187))
- `trng`: improve error handling ([#4139](https://github.com/embassy-rs/embassy/pull/4139))
- Remove `<T: Instance>` from `Uart` and `BufferedUart` ([#4155](https://github.com/embassy-rs/embassy/pull/4155))
- Make bit-depth of I2S PIO program configurable ([#4193](https://github.com/embassy-rs/embassy/pull/4193))
- Add the possibility to document `bind_interrupts` `struct`s ([#4206](https://github.com/embassy-rs/embassy/pull/4206))
- Add missing `Debug` and `defmt::Format` `derive`s for ADC & `AnyPin` ([#4205](https://github.com/embassy-rs/embassy/pull/4205))
- Add `rand-core` v0.9 support ([#4217](https://github.com/embassy-rs/embassy/pull/4217))
- Update `embassy-sync` to v0.7.0 ([#4234](https://github.com/embassy-rs/embassy/pull/4234))
- Add compatibility with ws2812 leds that have 4 addressable lights ([#4236](https://github.com/embassy-rs/embassy/pull/4236))
- Implement input/output inversion ([#4237](https://github.com/embassy-rs/embassy/pull/4237))
- Add `multicore::current_core` API ([#4362](https://github.com/embassy-rs/embassy/pull/4362))

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
