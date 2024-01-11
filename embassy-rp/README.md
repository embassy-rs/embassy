# Embassy RP HAL

HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The Embassy RP HAL targets the Raspberry Pi 2040 family of hardware. The HAL implements both blocking and async APIs
for many peripherals. The benefit of using the async APIs is that the HAL takes care of waiting for peripherals to
complete operations in low power mod and handling interrupts, so that applications can focus on more important matters.

NOTE: The Embassy HALs can be used both for non-async and async operations. For async, you can choose which runtime you want to use.
