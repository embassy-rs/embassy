# Bootloader for RP235x

The bootloader uses `embassy-boot` to interact with the flash.

# Usage

Flashing the bootloader either with `cargo run -r` (see "runner" `.cargo/config.toml`), or:

```
cargo flash --release --chip RP235x
```

By default, `Cargo.toml` specifies the `rp235xa` variant, you may need to change that.
