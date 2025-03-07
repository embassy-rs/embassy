# Examples for AVR microcontrollers
Run individual examples with
```
cargo run --bin <module-name> --release
```
for example
```
cargo run --bin blinky
```

## Checklist before running examples
You may need to adjust `.cargo/config.toml`, `Cargo.toml` and possibly update pin numbers or peripherals to match the specific MCU or board you are using.

* [ ] Update `.cargo/config.toml` with the correct target and runner command for your hardware. (use `ravedude --help` to list supported boards)
* [ ] Update `Cargo.toml` to use the correct `avr-device` feature for your processor.