# Low Power Examples for STM32WLEx family

Examples in this repo should work with [LoRa-E5 Dev Board](https://www.st.com/en/partner-products-and-services/lora-e5-development-kit.html) board.

## Prerequsits

- Connect a usb serial adapter to LPUart1 (this is where ALL logging will go)
- Optional: Connect an amp meter that ran measure down to 0.1uA to the power test pins
- `cargo install defmt-print` so you can print log messahes from LPUart1

## Example Notes

All examples will set all pins to analog mode before configuring pins for the example, if any. This saves about 500uA!!!!

- the `adc` example will sleep in STOP1 betwen samples and the chip will only draw about 13uA while sleeping
- the `blinky` example will sleep in STOP2 and the chip will only draw 1uA or less while sleeping
- the `button_exti` example will sleep in STOP2 and the chip will only draw 1uA or less while sleeping
- the `i2c` examples will sleep in STOP1 between reads and the chip only draw about 10uA while sleeping

For each example you will need to start `defmt-print` with the example binary and the correct serial port in a seperate terminal.  Example:
```
defmt-print -w -v -e target/thumbv7em-none-eabi/debug/<module-name> serial --path /dev/cu.usbserial-00000000 --baud 115200
```

Run individual examples with
```
cargo flash --chip STM32WLE5JCIx --connect-under-reset --bin <module-name>
```
for example
```
cargo flash --chip STM32WLE5JCIx --connect-under-reset --bin blinky
```

You can also run them with with `run`.  However in this case expect probe-rs to be disconnected as soon as flashing is done as all IO pins are set to analog input!
```
cargo run --bin blinky
```

## Checklist before running examples
You might need to adjust `.cargo/config.toml`, `Cargo.toml` and possibly update pin numbers or peripherals to match the specific MCU or board you are using.

* [ ] Update .cargo/config.toml with the correct probe-rs command to use your specific MCU. For example for L432KCU6 it should be `probe-rs run --chip STM32L432KCUx`. (use `probe-rs chip list` to find your chip)
* [ ] Update Cargo.toml to have the correct `embassy-stm32` feature. For example for L432KCU6 it should be `stm32l432kc`. Look in the `Cargo.toml` file of the `embassy-stm32` project to find the correct feature flag for your chip.
* [ ] If your board has a special clock or power configuration, make sure that it is set up appropriately.
* [ ] If your board has different pin mapping, update any pin numbers or peripherals in the given example code to match your schematic

If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:

* Which example you are trying to run
* Which chip and board you are using

Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
