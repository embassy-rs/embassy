# Riverdi RVT50HQSNWC00-B Examples

This folder contains examples for the **Riverdi RVT50HQSNWC00-B** display module.

## Board Information

- **Display**: 5.0" TFT LCD with capacitive touch
- **Resolution**: 800x480 pixels
- **Microcontroller**: STM32U5A9NJH6Q (Cortex-M33)
- **Flash Memory**: 4MB (4096KB)
- **RAM**: 2.5MB (2560KB)
- **Interface**: RGB, I2C for touch controller

## Getting Started

### Prerequisites

1. Install [probe-rs](https://probe.rs/) for flashing and debugging:
   ```bash
   cargo install probe-rs --locked
   ```

2. Install the ARM toolchain:
   ```bash
   rustup target add thumbv8m.main-none-eabihf
   ```

### Building and Running

To build and run the hello world example:

```bash
cd examples/rvt50hqsnwc00-b
cargo run --bin hello_world
```

This will:
1. Compile the example
2. Flash it to the board using probe-rs
3. Start a debug session

### Available Examples

- `hello_world.rs` - Simple hello world that prints messages via RTT

## Configuration

The example is configured for:
- **Chip**: STM32U5A9NJH6Q
- **Target**: `thumbv8m.main-none-eabihf`
- **Runner**: `probe-rs run --chip STM32U5A9NJH6Q`

The `memory-x` feature in `embassy-stm32` automatically provides the correct memory layout for the STM32U5A9NJH6Q chip (4MB Flash, 2.5MB RAM).

## Adding New Examples

To add a new example:

1. Create a new file in `src/bin/` (e.g., `my_example.rs`)
2. Use the `#![no_std]` and `#![no_main]` attributes
3. Import necessary crates from embassy
4. Use `#[embassy_executor::main]` for the main function

Example template:

```rust
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let _p = embassy_stm32::init(config);
    
    info!("My example running!");
    
    loop {
        // Your code here
    }
}
```

## Hardware Connections

The RVT50HQSNWC00-B module typically connects to a host MCU via:
- **RGB Interface**: For display data
- **I2C**: For touch controller communication
- **Power**: 3.3V

Refer to the [RVT50HQSNWC00-B datasheet](https://download.riverdi.com/RVT50HQSNWC00-B/DS_RVT50HQSNWC00-B_Rev.1.1.pdf) for detailed pinout and connection information.

## Troubleshooting

### Chip Not Recognized

If `probe-rs` doesn't recognize the chip, try:
```bash
probe-rs chip list
```

And update the runner in `.cargo/config.toml` with the exact chip name.

### Build Errors

Ensure you have the latest dependencies:
```bash
cargo update
```

### Flashing Issues

Make sure your probe is properly connected and the board is powered.

## Resources

- [Riverdi RVT50HQSNWC00-B Datasheet](https://download.riverdi.com/RVT50HQSNWC00-B/DS_RVT50HQSNWC00-B_Rev.1.1.pdf)
- [STM32U5 Series Reference Manual](https://www.st.com/resource/en/reference_manual/dm00314054-stm32u5-series-advanced-arm-based-mcus-stmicroelectronics.pdf)
- [Embassy Documentation](https://docs.embassy.dev/)
- [probe-rs Documentation](https://probe.rs/docs/getting-started/installation/)

## License

All examples are licensed under either MIT or Apache-2.0 at your option.
