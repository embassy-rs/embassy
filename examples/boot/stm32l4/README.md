# Examples using bootloader

Example for STM32L4 demonstrating the bootloader. The example consists of application binaries, 'a'
which allows you to press a button to start the DFU process, and 'b' which is the updated
application.


## Prerequisites

* `cargo-binutils`
* `cargo-flash`
* `embassy-boot-stm32`

## Usage

```
# Flash bootloader
cargo flash --manifest-path ../../../embassy-boot/stm32/Cargo.toml --release --features embassy-stm32/stm32l475vg --chip STM32L475VG
# Build 'b'
cargo build --release --bin b
# Generate binary for 'b'
cargo objcopy --release --bin b -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --bin a --chip STM32L475VG
```
