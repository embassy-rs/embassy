# SparkFun Artemis (Apollo3) Examples

Examples for the Ambiq Apollo3, configured specifically for SparkFun Artemis boards.

## Flashing & Bootloader
Artemis boards use the SparkFun SVL bootloader, which requires the application to boot from `0x10000`. 
- **Flashing:** Simply run `cargo run --release --bin <example_name>`.
- **How it works:** The `.cargo/config.toml` intercepts the runner and passes the binary to `tools/flash.sh`, which uses the SVL python script to flash over UART.

*(Note: If you want to flash a bare chip at `0x0` using a standard SWD debugger and `probe-rs`, you must change `memory.x`'s flash origin to `0x0` and remove the `svl-vtor` feature from `Cargo.toml`)*.

## Examples
- `sync_blinky`: Blocking blinky (no executor).
- `async_blinky`: Demonstrates the Embassy executor and STIMER backend.
- `async_button`: Using the `Input` driver's async `.wait_for_falling_edge()` to toggle an LED.
