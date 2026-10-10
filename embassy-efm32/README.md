# Embassy EFM32 HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The Embassy EFM32 HAL targets the Silicon Labs EFM32 (Gecko) family of hardware. The HAL implements both blocking and
async APIs for its peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals
to complete operations in low power mode and handling interrupts, so that applications can focus on more important matters.

NOTE: The Embassy HALs can be used both for non-async and async operations. For async, you can choose which runtime you want to use.

For a complete list of available peripherals and features, see the [embassy-efm32 documentation](https://docs.embassy.dev/embassy-efm32).

## Hardware support

The `embassy-efm32` HAL is built on the peripheral access crates (PACs) from [efm32-rs](https://github.com/efm32-rs), and
currently supports the following families:

* EFM32GG (Giant Gecko) ([examples](https://github.com/embassy-rs/embassy/tree/main/examples/efm32gg), for the Bosch XDK110)

Select your chip with a Cargo feature, e.g. `efm32gg990`. Features are named after the silicon die, so one feature covers
all flash sizes of a chip (e.g. `efm32gg990` for both the EFM32GG990F512 and EFM32GG990F1024).

The crate is structured so that other Gecko families (EFM32PG, EFM32HG, EFM32LG, ...) can be added alongside, each in
its own `src/chips/*.rs` module.

Only a basic set of peripherals is supported so far: clocks, GPIO (including waiting for edges through the external
interrupts) and USART/UART (asynchronous mode). There's no DMA, ADC, I2C, SPI, TIMER or low-power support yet.

## Time driver

If the `time-driver-rtc` feature is enabled, the HAL uses the RTC peripheral as a global time driver for [embassy-time](https://crates.io/crates/embassy-time), with a tick rate of 32768 Hz.

The RTC is clocked from the internal LFRCO by default, which is only accurate to a few percent. On boards with a
32.768 kHz crystal, select it with `cmu::Config::lfa_source` for accurate timekeeping.

## Reserved pins

Some pins are not handed out in `Peripherals` by default, because they serve a dedicated function:

* PB7/PB8 are the 32.768 kHz crystal's pins. Enable the `lfxo-as-gpio` feature to use them as GPIOs instead (the
  crystal is then unavailable).
* PF0/PF1 are the SWD debug pins. Enable the `swd-as-gpio` feature to use them as GPIOs instead (a debugger can then no
  longer attach to the running firmware).

With either feature, `init` puts the pins in the disconnected state, like the other pins at reset.

Pins that the selected chip's package doesn't bond out aren't in `Peripherals` either.

## Embedded-hal

The `embassy-efm32` HAL implements the traits from [embedded-hal](https://crates.io/crates/embedded-hal) (v0.2 and 1.0) and [embedded-hal-async](https://crates.io/crates/embedded-hal-async), as well as [embedded-io](https://crates.io/crates/embedded-io) and [embedded-io-async](https://crates.io/crates/embedded-io-async).

## Interoperability

This crate can run on any executor.

The HAL doesn't provide a [critical-section](https://crates.io/crates/critical-section) implementation. On single-core
EFM32s, enable the `critical-section-single-core` feature of the `cortex-m` crate in your application, as the examples do.
