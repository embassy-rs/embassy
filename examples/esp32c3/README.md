# 👋🏻

This code was borrowed from [esp32c3-hal](https://github.com/esp-rs/esp-hal/tree/main/esp32c3-hal)

The Cargo.toml file has been modified to use the embassy repo in path

```
rustup target add riscv32imc-unknown-none-elf
```

For now

```
cargo run --example embassy_hello_world --features "default","embassy","embassy-time-timg0"
```  
