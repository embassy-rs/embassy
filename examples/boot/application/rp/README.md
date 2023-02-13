# Examples using bootloader

Example for Raspberry Pi Pico demonstrating the bootloader. The example consists of application binaries, 'a'
which waits for 5 seconds before flashing the 'b' binary, which blinks the LED.

NOTE: The 'b' binary does not mark the new binary as active, so if you reset the device, it will roll back to the 'a' binary before automatically updating it again.

## Prerequisites

* `cargo-binutils`
* `cargo-flash`
* `embassy-boot-rp`

## Usage

```
# Flash bootloader
cargo flash --manifest-path ../../bootloader/rp/Cargo.toml --release --chip RP2040

# Build 'b'
cargo build --release --bin b

# Generate binary for 'b'
cargo objcopy --release --bin b -- -O binary b.bin

# Flash `a` (which includes b.bin)
cargo flash --release --bin a --chip RP2040
```
