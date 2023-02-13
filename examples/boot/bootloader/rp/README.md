# Bootloader for RP2040

The bootloader uses `embassy-boot` to interact with the flash.

# Usage

Flashing the bootloader

```
cargo flash --release --chip RP2040
```

To debug, use `cargo run` and enable the debug feature flag

``` rust
cargo run --release --features debug
```
