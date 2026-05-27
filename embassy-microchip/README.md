# Embassy Microchip HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so
raw register manipulation is not needed.

The Embassy Microchip HAL targets the Microchip MEC and CEC Family of MCUs. The
HAL implements both blocking and async APIs for many peripherals. The benefit of
using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mode and handling of interrupts, so that
applications can focus on business logic.

NOTE: The Embassy HALs can be used both for non-async and async operations. For
async, you can choose which runtime you want to use.

For a complete list of available peripherals and features, see the
[embassy-microchip documentation](https://docs.embassy.dev/embassy-microchip).
