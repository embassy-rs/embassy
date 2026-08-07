# embassy-imxrt-examples

## Introduction

These examples illustrates how to use the embassy-imxrt HAL.

## Adding Examples
Add uniquely named example to `src/bin` like `adc.rs`

## Build
`cd` to examples folder
`cargo build --bin <example_name>` for example, `cargo build --bin adc`

## Run
Assuming RT685 is powered and connected to Jlink debug probe and the latest probe-rs is installed via  
  `$ cargo install probe-rs-tools --git https://github.com/probe-rs/probe-rs --locked`  
`cd` to examples folder  
`cargo run --bin <example_name>` for example, `cargo run --bin adc`