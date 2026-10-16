# embassy-stm32-wpan

Async WPAN (short range wireless) on STM32WB families.

## Features

- Rust interface to the WPAN stack running on the STM32WB co-processor .
- [bt-hci](https://crates.io/crates/bt-hci) controller implementation, for use with the ST vendor commands
  and events of the [stm32wb-hci](https://crates.io/crates/stm32wb-hci) crate.

`stm32wb-hci` only provides the commands and events of the BLE stack it runs on, selected with
`wb-hci` by one release and one stack profile:

- STM32WB: the wireless binary flashed on CPU2, with one `wb-fw-*` feature (`wb-fw-1-15-0` to
  `wb-fw-1-24-0`) and one `wb-stack-*` feature (`wb-stack-full-extended`, `wb-stack-full`,
  `wb-stack-light`, `wb-stack-hci-layer-extended`, `wb-stack-hci-layer`, `wb-stack-hci-adv-scan`).
  The BLE + Thread/Zigbee/MAC combo binaries carry the full BLE stack: use `wb-stack-full`.
- STM32WBA: the STM32CubeWBA 1.10.0 library the bindings are generated from, with the profile of the
  `ble-stack-*` feature.
- Embassy-net driver implementation for 802.15.4 MAC.

## Examples

See the [stm32wb examples](https://github.com/embassy-rs/embassy/tree/main/examples/stm32wb).
