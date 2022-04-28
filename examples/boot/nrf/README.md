# Examples using bootloader

Example for nRF52 demonstrating the bootloader. The example consists of application binaries, 'a'
which allows you to press a button to start the DFU process, and 'b' which is the updated
application.


## Prerequisites

* `cargo-binutils`
* `cargo-flash`
* `embassy-boot-nrf`

## Usage



```
# Use bare metal linker script
cp memory-bl.x ../../../embassy-boot/nrf/memory.x

# Flash bootloader
cargo flash --manifest-path ../../../embassy-boot/nrf/Cargo.toml --features embassy-nrf/nrf52840 --release --chip nRF52840_xxAA
# Build 'b'
cargo build --release --bin b
# Generate binary for 'b'
cargo objcopy --release --bin b -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --bin a --chip nRF52840_xxAA
```
