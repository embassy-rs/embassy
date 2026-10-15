# Changelog for embassy-efm32

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- First release, with support for the EFM32GG (Giant Gecko) family: clocks, GPIO (with edge and level waits), USART/UART and an RTC-based time driver.
- USART/UART driver (`usart` module) with `Blocking` and `Async` modes, TX/RX halves (`split`, `split_ref`, `UartTx`, `UartRx`), configurable frame format, and `ROUTE` locations derived from the `TxPin`/`RxPin` traits.
- `swd-as-gpio` and `lfxo-as-gpio` features to hand the SWD and LFXO crystal pins out as GPIOs.
- `Peripherals` only contains the pins the selected chip's package bonds out.
- Implement the `embedded-hal` 0.2 and 1.0, `embedded-hal-async`, `embedded-io` and `embedded-io-async` traits.
