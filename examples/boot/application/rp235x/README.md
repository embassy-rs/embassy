# Examples using bootloader

Example for Raspberry Pi Pico 2 demonstrating the bootloader. The example consists of application binaries, 'a'
which waits for 5 seconds before flashing the 'b' binary, which blinks the LED.

The 'b' binary marks the new binary (itself) as active, so the device will run the blinking program on every subsequent reset.
When 'b' boots the first time the blinking is fast, and every reset after that it wil be slower.

## Prerequisites

* `cargo-binutils`
* `cargo-flash`

## Usage

```
# Flash bootloader
cargo flash --manifest-path ../../bootloader/rp235x/Cargo.toml --release --chip RP235x

# Generate binary for 'b'
cargo objcopy --release --bin b -- -O binary b.bin

# Flash `a` (which includes b.bin)
cargo flash --release --bin a --chip RP235x
```

Depending on how you flash you might have to reset the device after, then wait as the updating process does take a couple of seconds.

By default, `Cargo.toml` specifies the `rp235xa` variant, you may need to change that.
