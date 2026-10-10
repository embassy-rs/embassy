# Embassy NXP MCX-A MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-A family of
microcontrollers using the Embassy async framework. This HAL provides
safe, idiomatic Rust interfaces for GPIO, UART, and OSTIMER
peripherals.

## Host sleep recovery tests

Run these commands from the `embassy-mcxa` directory, selecting one chip family
at a time. Disable default features and leave `rt` disabled on the host; the
runtime, interrupt vectors, and inline assembly are for embedded ARM targets.

```powershell
cargo test --lib --no-default-features --features mcxa2xx clocks::sleep::tests
cargo test --lib --no-default-features --features mcxa5xx clocks::sleep::tests
```

The tests cover shared CMC/SPC control, FIRC readiness, and SPLL shutdown and
recovery using isolated register models. Analog startup timing and actual
low-power entry and wakeup still require hardware validation.
