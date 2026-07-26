# Examples using bootloader

Example for nRF demonstrating the bootloader. The example consists of application binaries, 'a'
which allows you to press a button to start the DFU process, and 'b' which is the updated
application.


## Prerequisites

* `cargo-binutils`
* `cargo-flash`
* `embassy-boot-nrf`

## Usage



```
# Use bare metal linker script
cp memory-bl.x ../../bootloader/nrf/memory.x

# Flash bootloader
cargo flash --manifest-path ../../bootloader/nrf/Cargo.toml --features embassy-nrf/nrf52840 --target thumbv7em-none-eabi --release --chip nRF52840_xxAA
# Build 'b'
cargo build --release --bin b --features embassy-nrf/nrf52840,time-driver-rtc1
# Generate binary for 'b'
cargo objcopy --release --bin b --features embassy-nrf/nrf52840 --target thumbv7em-none-eabi -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --bin a --features embassy-nrf/nrf52840 --target thumbv7em-none-eabi --chip nRF52840_xxAA
```

You should then see a solid LED. Pressing button 1 will cause the DFU to be loaded by the bootloader. Upon
successfully loading, you'll see the LED flash. After 5 seconds, because there is no petting of the watchdog,
you'll see the LED go solid again. This indicates that the bootloader has reverted the update.
