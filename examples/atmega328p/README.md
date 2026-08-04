# ATmega328P examples

These examples exercise the `embassy-atmega328p` HAL on an ATmega328P. The
default configuration assumes a 16 MHz CPU clock, as used by a classic Arduino
Uno or Nano.

## Prerequisites

- a Rust nightly toolchain with the `rust-src` component;
- an AVR GNU toolchain providing `avr-gcc` and `avr-objcopy`;
- `avrdude`, or another programmer supported by the board.

The local `.cargo/config.toml` selects `avr-none`, builds `core`, and sets the
CPU to `atmega328p`. Run Cargo commands from this directory so that Cargo loads
that configuration.

## Check all examples

```console
cargo +nightly check --bins
```

## Build one example

```console
cargo +nightly build --release --bin async_blinky
avr-objcopy -O ihex -R .eeprom target/avr-none/release/async_blinky.elf async_blinky.hex
```

Available binaries are `async_blinky`, `adc`, `i2c`, `onewire`, `pwm`, `spi`,
and `usart`.

## Upload through an Arduino-compatible bootloader

Replace `COM5` with the board's serial port. A classic Uno bootloader commonly
uses 115200 baud:

```console
avrdude -p atmega328p -c arduino -P COM5 -b 115200 -D -U flash:w:async_blinky.hex:i
```

Some classic Nano bootloaders use 57600 baud instead. Boards programmed over
ISP require the matching `avrdude` programmer option rather than `-c arduino`.

## Clock selection

The examples depend on the HAL's `time-driver-16mhz` feature. For a board that
actually runs at 8 MHz, change that dependency to disable default features and
enable `time-driver-8mhz`. The selected frequency must match the clock source
and fuse configuration.

## Hardware notes

- `async_blinky` drives PB5, the built-in LED pin on a classic Uno.
- I2C requires external pull-up resistors on PC4/SDA and PC5/SCL.
- 1-Wire normally requires an external 4.7 kOhm pull-up resistor.
- PC6 is RESET with the factory fuse configuration and is not a normal GPIO
  unless the reset-disable fuse is deliberately programmed.
