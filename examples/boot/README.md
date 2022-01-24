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
# Build 'b'
cargo build --release --features embassy-nrf/nrf52832 --bin b
# Generate binary for 'b'
cargo objcopy --release --features embassy-nrf/nrf52832 --bin b -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --features embassy-nrf/nrf52832 --bin a --chip nRF52832_xxAA
```
