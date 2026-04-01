# nrf54l15-flpr examples

These examples must be used in combination with the corresponding app core example. 

NOTE: The nightly rustc toolchain is required to support the riscv32emc target

To build

```
cargo +nightly build --release --bin blinky
```

When built, write the example to the expected location:

```
rust-objcopy -O ihex target/riscv32emc-unknown-none-elf/release/blinky blinky.hex
probe-rs download blinky.hex --binary-format hex --chip nRF54L15
```
