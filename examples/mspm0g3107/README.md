# Examples for MSPM0G3107

Run individual examples with
```
cargo run --bin <module-name>
```
for example
```
cargo run --bin can
```

## Checklist before running examples

The MSPM0G3107 does not have a Launchpad or official development board, so you likely need to modify examples to update pin numbers or peripherals to match the specific MCU or board you are using. Note that MSPM0G3507 has a very similar peripheral set and the examples here will likely be able to run on that platform with only small changes.

For a custom board with MSPM0G3107:

* [ ] Update Cargo.toml to have the correct `embassy-mspm0` feature reflecting the packaging option. Look in the `Cargo.toml` file of the `embassy-mspm0` project to find the correct feature flag for your chip.
* [ ] If your board has a special clock or power configuration, make sure that it is set up appropriately.
* [ ] If your board has different pin mapping, update any pin numbers or peripherals in the given example code to match your schematic

If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:

* Which example you are trying to run
* Which chip and board you are using

Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
