# Embassy NEORV32 HAL
HALs implement safe, idiomatic Rust APIs to use the hardware capabilities, so raw register manipulation is not needed.

The `embassy-neorv32` HAL targets the open-source [NEORV32](https://github.com/stnolting/neorv32)
RISC-V microcontroller and implements both blocking and async APIs/drivers for most of the peripherals.
Additionally, async and blocking traits from [embedded-hal](https://crates.io/crates/embedded-hal)
are implemented where appropriate.

## Support
The HAL currently supports the following peripherals and features:

### Peripherals
- UART
- GPIO
- DMA
- SYSINFO

### Additional Features
- Embassy time-driver via CLINT `mtimer`

## Usage
Please see the `examples/neorv32` folder for ideas on how to use this HAL in your own projects.  
To run these examples, follow these steps:

- Install [cargo-binutils](https://crates.io/crates/cargo-binutils)
- Modify build target in `examples/neorv32/.cargo/config.toml` to match your configuration
- Modify `examples/neorv32/memory.x` to match the size of your configured `DMEM` and `IMEM`
- Modify `examples/neorv32/Cargo.toml` features `sim` and `fpga` such that
the `tick-hz` feature for `embassy-time` matches your configuration
- Modify `UART_BAUD` in `examples/neorv32/src/lib.rs` to match your host UART
- Clone [neorv32 v1.12.6](https://github.com/stnolting/neorv32/tree/v1.12.6)
- Continue with one of the series of steps below depending on if running in simulation or on FPGA

### Simulation
- Modify `BASE` in `examples/neorv32/run-sim` to your `neorv32` repo path (default in `$HOME` folder)
- Install [GHDL](https://github.com/ghdl/ghdl) simulator
- From the `examples/neorv32` folder, run `cargo run-sim --release --bin hello-world`
- For more help, see [simulating the processor](https://stnolting.github.io/neorv32/ug/#_simulating_the_processor)

### FPGA over UART bootloader
- Modify `BASE` in `examples/neorv32/run-fpga` to your `neorv32` repo path (default in `$HOME` folder)
- Install `picocom` (or modify `run-fpga` to use your preferred tool)
- From the `examples/neorv32` folder, run `cargo run-fpga --release --bin hello-world`
- Press reset button on FPGA
- If using `picocom`, manually follow these steps within host terminal:
- Type: Any key
- Type: `u` (Upload)
- Type: `<Ctrl+A> <Ctrl+S>` (send file)
- Type: `hello-world-fpga <Enter>`
- Wait for upload to complete (should see `*** exit status: 0 *** OK`)
- Type: `e` (Execute)
- For more help, see [bootloader](https://stnolting.github.io/neorv32/#_bootloader)

## Version
This HAL targets NEORV32 [v1.12.6](https://github.com/stnolting/neorv32/tree/v1.12.6).
There is no guarantee it will work for different versions.

## References
- [NEORV32 Datasheet](https://stnolting.github.io/neorv32/)
- [NEORV32 User Guide](https://stnolting.github.io/neorv32/ug/)
