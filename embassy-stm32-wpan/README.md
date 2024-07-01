# embassy-stm32-wpan

Async WPAN (short range wireless) on STM32WB families.

## Features

- Rust interface to the WPAN stack running on the STM32WB co-processor .
- Controller trait implementation for the [stm32wb-hci](https://crates.io/crates/stm32wb-hci) crate.
- Embassy-net driver implementation for 802.15.4 MAC.

## Examples

See the [stm32wb examples](https://github.com/embassy-rs/embassy/tree/main/examples/stm32wb).
