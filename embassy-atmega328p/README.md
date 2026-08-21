# embassy-atmega328p

An Embassy-compatible, `no_std` hardware abstraction layer for the
Microchip ATmega328P.

This crate is intentionally independent from `embassy-executor`. It can be
used by a blocking application on its own, or together with Embassy's AVR
executor in an asynchronous application.

## Status

The initial implementation provides:

- ownership tokens for all GPIO pins on ports B, C and D;
- input pins with floating or internal pull-up configuration;
- push-pull output pins;
- `embedded-hal` 1.0 digital trait implementations;
- blocking USART0, SPI master and TWI/I2C master drivers;
- blocking 10-bit ADC support for PC0 through PC5;
- two-channel fast PWM using Timer0 or Timer2;
- a bit-banged Dallas/Maxim 1-Wire master with CRC-8;
- an optional Embassy time driver backed by Timer1;
- 16 MHz and 8 MHz CPU-clock configurations;
- a compiling Embassy executor + async timer Blinky example;
- an optional re-export of the low-level PAC.

GPIO external interrupts and asynchronous peripheral transfers are not
implemented yet. USART, SPI, I2C and ADC currently use blocking APIs.

## Using the HAL from another project

While both projects are checked out on the same machine, a path dependency is
the quickest option:

```toml
[dependencies]
embassy-atmega328p = { path = "../embassy/embassy-atmega328p" }
avr-device = { version = "0.8.1", features = ["atmega328p", "rt"] }
```

Enable the time driver that matches the board's CPU clock when using
`embassy-time`. A normal Arduino Uno or classic Nano runs at 16 MHz:

```toml
[dependencies]
embassy-atmega328p = {
    path = "../embassy/embassy-atmega328p",
    features = ["time-driver-16mhz"],
}
embassy-executor = {
    path = "../embassy/embassy-executor",
    features = ["nightly", "platform-avr", "executor-thread"],
}
embassy-time = { path = "../embassy/embassy-time" }
avr-device = { version = "0.8.1", features = ["atmega328p", "rt"] }
panic-halt = "1.0"
```

Use `time-driver-8mhz` instead for an ATmega328P actually clocked at 8 MHz.
The selected feature must match the fuse and clock configuration or all
durations will be scaled incorrectly. Since the default clock is 16 MHz, an
8 MHz application must also set `default-features = false` and explicitly
enable `rt` if it needs the runtime.

The crate can also be used directly from the Embassy Git repository. Cargo
searches the complete repository for the requested package, so the crate does
not have to be at the repository root.

Pin applications to a tested commit with `rev`; otherwise Cargo follows the
latest commit on the repository's default branch:

```toml
[dependencies]
embassy-atmega328p = {
    git = "https://github.com/embassy-rs/embassy.git",
    rev = "<tested-commit-sha>",
}
avr-device = { version = "0.8.1", features = ["atmega328p", "rt"] }
```

For an async 16 MHz application, enable the matching time-driver feature:

```toml
embassy-atmega328p = {
    git = "https://github.com/embassy-rs/embassy.git",
    rev = "<tested-commit-sha>",
    features = ["time-driver-16mhz"],
}
embassy-executor = {
    git = "https://github.com/embassy-rs/embassy.git",
    rev = "<same-tested-commit-sha>",
    features = ["nightly", "platform-avr", "executor-thread"],
}
embassy-time = {
    git = "https://github.com/embassy-rs/embassy.git",
    rev = "<same-tested-commit-sha>",
}
```

Use the same revision for all Embassy crates so their internal APIs stay in
sync. Keep `publish = false` while this HAL is private and hardware validation
is still in progress.

## GPIO example

```rust,ignore
#![no_std]
#![no_main]

use embassy_atmega328p::gpio::{Level, Output};
use embedded_hal::digital::StatefulOutputPin;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut led = Output::new(p.PB5, Level::Low); // Arduino Uno D13

    loop {
        led.toggle().unwrap();
    }
}
```

`PC6` is the reset pin with the factory fuse configuration. Do not use it as
GPIO unless the reset-disable fuse has deliberately been programmed.

The complete async version is in
[`examples/atmega328p/src/bin/async_blinky.rs`](../examples/atmega328p/src/bin/async_blinky.rs).

## Timer1 ownership

The Embassy time driver uses Timer/Counter1 in normal mode with a `/8`
prescaler, including its overflow and compare-A interrupts. Applications must
not reconfigure Timer1 while either `time-driver-*` feature is enabled. Timer0
and Timer2 remain available to the PWM drivers or application-specific code.

## Peripheral pin map

| Peripheral | Pins | Notes |
| --- | --- | --- |
| USART0 | PD0/RX, PD1/TX | Blocking 8N1 |
| TWI/I2C | PC4/SDA, PC5/SCL | External pull-ups required |
| SPI master | PB2/SS, PB3/MOSI, PB4/MISO, PB5/SCK | Use another GPIO for device CS if desired |
| ADC | PC0 through PC5 | 10-bit blocking conversions |
| PWM0 | PD6/OC0A, PD5/OC0B | Timer0, 8-bit fast PWM |
| PWM2 | PB3/OC2A, PD3/OC2B | Timer2, 8-bit fast PWM |
| 1-Wire | Any GPIO | External pull-up, normally 4.7 kOhm |

Pin ownership makes incompatible combinations fail to compile. For example,
PB3 cannot simultaneously be SPI/MOSI and Timer2/OC2A.

## AVR build settings

AVR currently requires a nightly compiler and `build-std`:

```toml
# .cargo/config.toml in the application
[build]
target = "avr-none"

[unstable]
build-std = ["core"]

[target.avr-none]
rustflags = ["-C", "target-cpu=atmega328p"]
```

The final link step also requires an AVR GNU toolchain providing `avr-gcc`.
Application profiles must abort on panic:

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```
