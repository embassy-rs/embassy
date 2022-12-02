# Examples using bootloader

Example for RP2040 demonstrating the bootloader. The example consists of application binaries, 'a'
which allows you to press a button to start the DFU process, and 'b' which is the updated
application.


## Prerequisites

* `cargo-binutils`
* `cargo-flash`
* `embassy-boot-rp`

## Usage

```
# Flash bootloader
cargo flash --manifest-path ../../bootloader/rp/Cargo.toml --target thumbv6m-none-eabi --release --chip RP2040
# Build 'b'
cargo build --release --bin b --target thumbv6m-none-eabi
# Generate binary for 'b'
cargo objcopy --release --bin b --target thumbv6m-none-eabi -- -O binary b.bin
```

# Flash `a` (which includes b.bin)

```
cargo flash --release --bin a --chip RP2040 --target thumbv6m-none-eabi
```
