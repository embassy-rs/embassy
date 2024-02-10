# Embassy

Embassy is the next-generation framework for embedded applications. Write safe, correct and energy-efficient embedded code faster, using the Rust programming language, its async facilities, and the Embassy libraries.

## <a href="https://embassy.dev/book/dev/index.html">Documentation</a> - <a href="https://docs.embassy.dev/">API reference</a> - <a href="https://embassy.dev/">Website</a> - <a href="https://matrix.to/#/#embassy-rs:matrix.org">Chat</a>
## Rust + async ❤️ embedded

The Rust programming language is blazingly fast and memory-efficient, with no runtime, garbage collector or OS. It catches a wide variety of bugs at compile time, thanks to its full memory- and thread-safety, and expressive type system. 

Rust's <a href="https://rust-lang.github.io/async-book/">async/await</a> allows for unprecedently easy and efficient multitasking in embedded systems. Tasks get transformed at compile time into state machines that get run cooperatively. It requires no dynamic memory allocation, and runs on a single stack,  so no per-task stack size tuning is required. It obsoletes the need for a traditional RTOS with kernel context switching, and is <a href="https://tweedegolf.nl/en/blog/65/async-rust-vs-rtos-showdown">faster and smaller than one!</a>

## Batteries included

- **Hardware Abstraction Layers** - HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed. The Embassy project maintains HALs for select hardware, but you can still use HALs from other projects with Embassy.
  - <a href="https://docs.embassy.dev/embassy-stm32/">embassy-stm32</a>, for all STM32 microcontroller families.
  - <a href="https://docs.embassy.dev/embassy-nrf/">embassy-nrf</a>, for the Nordic Semiconductor nRF52, nRF53, nRF91 series.
  - <a href="https://docs.embassy.dev/embassy-rp/">embassy-rp</a>, for the Raspberry Pi RP2040 microcontroller.
  - <a href="https://github.com/esp-rs">esp-rs</a>, for the Espressif Systems ESP32 series of chips.
    - Embassy HAL support for Espressif chips is being developed in the [esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) repository.
    - Async WiFi, Bluetooth and ESP-NOW is being developed in the [esp-rs/esp-wifi](https://github.com/esp-rs/esp-wifi) repository.

- **Time that Just Works** - 
No more messing with hardware timers. <a href="https://docs.embassy.dev/embassy-time">embassy_time</a> provides Instant, Duration and Timer types that are globally available and never overflow.

- **Real-time ready** - 
Tasks on the same async executor run cooperatively, but you can create multiple executors with different priorities, so that higher priority tasks preempt lower priority ones. See the <a href="https://github.com/embassy-rs/embassy/blob/master/examples/nrf52840/src/bin/multiprio.rs">example</a>.

- **Low-power ready** - 
Easily build devices with years of battery life. The async executor automatically puts the core to sleep when there's no work to do. Tasks are woken by interrupts, there is no busy-loop polling while waiting.
 
- **Networking** - 
The <a href="https://docs.embassy.dev/embassy-net/">embassy-net</a> network stack implements extensive networking functionality, including Ethernet, IP, TCP, UDP, ICMP and DHCP. Async drastically simplifies managing timeouts and serving multiple connections concurrently.

- **Bluetooth** - 
The <a href="https://github.com/embassy-rs/nrf-softdevice">nrf-softdevice</a> crate provides Bluetooth Low Energy 4.x and 5.x support for nRF52 microcontrollers.
The <a href="https://github.com/embassy-rs/embassy/tree/main/embassy-stm32-wpan">embassy-stm32-wpan</a> crate provides Bluetooth Low Energy 5.x support for stm32wb microcontrollers.

- **LoRa** - The <a href="https://github.com/lora-rs/lora-rs">lora-rs</a> project provides an async LoRa and LoRaWAN stack that works well on Embassy.

- **USB** - 
<a href="https://docs.embassy.dev/embassy-usb/">embassy-usb</a> implements a device-side USB stack. Implementations for common classes such as USB serial (CDC ACM) and USB HID are available, and a rich builder API allows building your own.

- **Bootloader and DFU** - 
<a href="https://github.com/embassy-rs/embassy/tree/master/embassy-boot">embassy-boot</a> is a lightweight bootloader supporting firmware application upgrades in a power-fail-safe way, with trial boots and rollbacks.


## Sneak peek

```rust,ignore
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::Peripherals;

// Declare async tasks
#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);

    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.set_high();
        Timer::after_millis(150).await;
        led.set_low();
        Timer::after_millis(150).await;
    }
}

// Main is itself an async task as well.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

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

*   `examples/nrf52840` run on the `nrf52840-dk` board (PCA10056) but should be easily adaptable to other nRF52 chips and boards.
*   `examples/nrf5340` run on the `nrf5340-dk` board (PCA10095).
*   `examples/stm32xx` for the various STM32 families.
*   `examples/rp` are for the RP2040 chip.
*   `examples/std` are designed to run locally on your PC.

### Running examples

- Install `probe-rs`.

```bash
cargo install probe-rs --features cli
```

- Change directory to the sample's base directory. For example:

```bash
cd examples/nrf52840
```

- Ensure `Cargo.toml` sets the right feature for the name of the chip you are programming.
  If this name is incorrect, the example may fail to run or immediately crash
  after being programmed.

- Ensure `.cargo/config.toml` contains the name of the chip you are programming.

- Run the example

For example:

```bash
cargo run --release --bin blinky
```

For more help getting started, see [Getting Started][1] and [Running the Examples][2].

## Developing Embassy with Rust Analyzer based editors

The [Rust Analyzer](https://rust-analyzer.github.io/) is used by [Visual Studio Code](https://code.visualstudio.com/)
and others. Given the multiple targets that Embassy serves, there is no Cargo workspace file. Instead, the Rust Analyzer 
must be told of the target project to work with. In the case of Visual Studio Code, 
please refer to the `.vscode/settings.json` file's `rust-analyzer.linkedProjects`setting.

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on stable Rust 1.75 and up. It *might*
compile with older versions but that may change in any new patch release.

## Why the name?

EMBedded ASYnc! :)

## License

Embassy is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[1]: https://github.com/embassy-rs/embassy/wiki/Getting-Started
[2]: https://github.com/embassy-rs/embassy/wiki/Running-the-Examples
