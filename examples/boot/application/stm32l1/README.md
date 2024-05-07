# Examples using bootloader

Example for STM32L1 demonstrating the bootloader. The example consists of application binaries, 'a'
which allows you to press a button to start the DFU process, and 'b' which is the updated
application.

## Prerequisites

- `cargo-binutils`
- `cargo-flash`
- `embassy-boot-stm32`

## Usage

```
# Flash bootloader
cargo flash --manifest-path ../../bootloader/stm32/Cargo.toml --release --features embassy-stm32/stm32l151cb-a --chip STM32L151CBxxA
# Build 'b'
cargo build --release --bin b
# Generate binary for 'b'
cargo objcopy --release --bin b -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --bin a --chip STM32L151CBxxA
```
