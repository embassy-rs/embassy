# Examples for STM32L4 family using Embassy with custom interrupt handlers
Examples in this project demonstrates how to create and use custom interrupt handlers.

Note that it is only possible to setup custom exti interrupt handlers when the feature flag `exti-with-custom-handlers` is enabled for `embassy-stm32`, otherwise a linker error will happen due to a duplicate definition of the exti interrupt value.

Note you can use the embassy pac api to manually 'discover' which exti groups are available, they are listed under `embassy_stm32::pac::interrupt::EXTI<_>`.

## How to run examples
Run individual examples with
```
cargo run --bin <module-name>
```
for example
```
cargo run --bin button_exti
```

## Checklist before running examples
You might need to adjust `.cargo/config.toml`, `Cargo.toml` and possibly update pin numbers or peripherals to match the specific MCU or board you are using.

* [ ] Update .cargo/config.toml with the correct probe-rs command to use your specific MCU. For example for L4R5ZI-P it should be `probe-rs run --chip STM32L4R5ZITxP`. (use `probe-rs chip list` to find your chip)
* [ ] Update Cargo.toml to have the correct `embassy-stm32` feature. For example for L4R5ZI-P it should be `stm32l4r5zi`. Look in the `Cargo.toml` file of the `embassy-stm32` project to find the correct feature flag for your chip.
* [ ] If your board has a special clock or power configuration, make sure that it is set up appropriately.
* [ ] If your board has different pin mapping, update any pin numbers or peripherals in the given example code to match your schematic

If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:

* Which example you are trying to run
* Which chip and board you are using

Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
