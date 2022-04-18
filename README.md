# Embassy

Embassy is the next-generation framework for embedded applications. Write safe, correct and energy-efficient embedded code faster, using the Rust programming language, its async facilities, and the Embassy libraries.

## [Documentation](https://embassy.dev/embassy/dev/index.html) - [API reference](https://docs.embassy.dev/) - [Website](https://embassy.dev/) - [Chat](https://matrix.to/#/#embassy-rs:matrix.org)
## Rust + async ❤️ embedded

The Rust programming language is blazingly fast and memory-efficient, with no runtime, garbage collector or OS. It catches a wide variety of bugs at compile time, thanks to its full memory- and thread-safety, and expressive type system. 

Rust's <a href="https://rust-lang.github.io/async-book/">async/await</a> allows for unprecedently easy and efficient multitasking in embedded systems. Tasks get transformed at compile time into state machines that get run cooperatively. It requires no dynamic memory allocation, and runs on a single stack,  so no per-task stack size tuning is required. It obsoletes the need for a traditional RTOS with kernel context switching, and is <a href="https://tweedegolf.nl/en/blog/65/async-rust-vs-rtos-showdown">faster and smaller than one!</a>

## Batteries included

- **Hardware Abstraction Layers** - HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed. The Embassy project maintains HALs for select hardware, but you can still use HALs from other projects with Embassy.
  - <a href="https://docs.embassy.dev/embassy-stm32/">embassy-stm32</a>, for all STM32 microcontroller families.
  - <a href="https://docs.embassy.dev/embassy-nrf/">embassy-nrf</a>, for the Nordic Semiconductor nRF52, nRF53, nRF91 series.

- **Time that Just Works** - 
No more messing with hardware timers. <a href="https://docs.embassy.dev/embassy/git/thumbv7em-none-eabihf/time/index.html">embassy::time</a> provides Instant, Duration and Timer types that are globally available and never overflow.

- **Real-time ready** - 
Tasks on the same async executor run cooperatively, but you can create multiple executors with different priorities, so that higher priority tasks preempt lower priority ones. See the <a href="https://github.com/embassy-rs/embassy/blob/master/examples/nrf/src/bin/multiprio.rs">example</a>.

- **Low-power ready** - 
Easily build devices with years of battery life. The async executor automatically puts the core to sleep when there's no work to do. Tasks are woken by interrupts, there is no busy-loop polling while waiting.
 
- **Networking** - 
The <a href="https://docs.embassy.dev/embassy-net/">embassy-net</a> network stack implements extensive networking functionality, including Ethernet, IP, TCP, UDP, ICMP and DHCP. Async drastically simplifies managing timeouts and serving multiple connections concurrently.

- **Bluetooth** - 
The <a href="https://github.com/embassy-rs/nrf-softdevice">nrf-softdevice</a> crate provides Bluetooth Low Energy 4.x and 5.x support for nRF52 microcontrollers.

- **LoRa** - 
<a href="https://docs.embassy.dev/embassy-lora/">embassy-lora</a> supports LoRa networking on STM32WL wireless microcontrollers and Semtech SX127x transceivers.

- **USB** - 
<a href="https://docs.embassy.dev/embassy-usb/">embassy-usb</a> implements a device-side USB stack. Implementations for common classes such as USB serial (CDC ACM) and USB HID are available, and a rich builder API allows building your own.

- **Bootloader and DFU** - 
<a href="https://github.com/embassy-rs/embassy/tree/master/embassy-boot">embassy-boot</a> is a lightweight bootloader supporting firmware application upgrades in a power-fail-safe way, with trial boots and rollbacks.


## Sneak peek

```rust
use defmt::info;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::Peripherals;

// Declare async tasks
#[embassy::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);

    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.set_high();
        Timer::after(Duration::from_millis(150)).await;
        led.set_low();
        Timer::after(Duration::from_millis(150)).await;
    }
}

// Main is itself an async task as well.
#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    // Spawned tasks run in the background, concurrently.
    spawner.spawn(blink(p.P0_13.degrade())).unwrap();

    let mut button = Input::new(p.P0_11, Pull::Up);
    loop {
        // Asynchronously wait for GPIO events, allowing other tasks
        // to run, or the core to sleep.
        button.wait_for_low().await;
        info!("Button pressed!");
        button.wait_for_high().await;
        info!("Button released!");
    }
}
```

## Examples

Examples are found in the `examples/` folder seperated by the chip manufacturer they are designed to run on. For example:

*   `examples/nrf` run on the `nrf52840-dk` board (PCA10056) but should be easily adaptable to other nRF52 chips and boards.
*   `examples/stm32xx` for the various STM32 families.
*   `examples/rp` are for the RP2040 chip.
*   `examples/std` are designed to run locally on your PC.

### Running examples

- Setup git submodules (needed for STM32 examples)
```
git submodule init
git submodule update
```

- Install `probe-run` with defmt support.

```
cargo install probe-run
```

- Change directory to the sample's base directory. For example:

```
cd examples/nrf
```

- Run the example

For example:

```
cargo run --bin blinky
```

## Developing Embassy with Rust Analyzer based editors

The [Rust Analyzer](https://rust-analyzer.github.io/) is used by [Visual Studio Code](https://code.visualstudio.com/)
and others. Given the multiple targets that Embassy serves, there is no Cargo workspace file. Instead, the Rust Analyzer 
must be told of the target project to work with. In the case of Visual Studio Code, 
please refer to the `.vscode/settings.json` file's `rust-analyzer.linkedProjects`setting.

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

Several features require nightly:

- The `#[embassy::main]` and `#[embassy::task]` attribute macros.
- Async traits

These are enabled by activating the `nightly` Cargo feature. If you do so, Embassy is guaranteed to compile on the exact nightly version specified in `rust-toolchain.toml`. It might compile with older or newer nightly versions, but that may change in any new patch release.

## Why the name?

EMBedded ASYnc! :)

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

