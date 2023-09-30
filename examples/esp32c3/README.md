# esp32-c3 example

this is an WIP signpost, to inform & guide to better examples. 

👋🏻 I'm concurrently proposing basic signpost for README-embassy.md this is in conjunction with my proposed uplift:
* https://github.com/esp-rs/esp-hal/pull/834
* https://github.com/drogue-iot/drogue-device/pull/388

## The Basics:
* embassy-rs uses esp-rs/esp-hal
* esp-hal: Hardware Abstraction Layer crates for the ESP32, ESP32-C2/C3/C6, ESP32-H2, and ESP32-S2/S3 from Espressif.
* The Minimum Supported Rust Version is 1.67.0 for all packages.
* 🔗 https://docs.rs/esp32c3-hal/latest/esp32c3_hal/

## Quickstart:
https://github.com/esp-rs/esp-hal/blob/main/esp32c3-hal/examples/embassy_hello_world.rs

```
gh repo clone esp-rs/esp-hal
rustup target add riscv32imc-unknown-none-elf
# when targeting RISC-V arch. necessary to set:
RUSTC_BOOTSTRAP=1
cd esp-hal/esp32c3-hal

```

## 🥰 more github.com/esp-rs/esp-hal/esp32c3-hal/examples
* embassy_hello_world.rs  
* embassy_i2s_read.rs   
* embassy_rmt_rx.rs  
* embassy_serial.rs  
* embassy_wait.rs
* embassy_i2c.rs          
* embassy_i2s_sound.rs
* embassy_rmt_tx.rs
* embassy_spi.rs

## coming soon:
* embassy_blink (esp32 c3 & s2)
* drogue-iot example with esp32


