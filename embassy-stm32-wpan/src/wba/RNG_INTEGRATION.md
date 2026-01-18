# Hardware RNG Integration for STM32WBA BLE Stack

This document describes how to integrate the hardware RNG from `embassy-stm32` with the WBA BLE stack.

## Overview

The WBA link layer **requires** hardware random number generation for cryptographic operations and Bluetooth security features. The hardware RNG peripheral must be initialized and registered with the link layer before using any BLE/MAC functionality.

**Important**: Hardware RNG is mandatory. The link layer will panic with a clear error message if RNG is not initialized when random numbers are requested.

## Usage

### Step 1: Set up the RNG peripheral

First, initialize the hardware RNG peripheral using `embassy-stm32`:

```rust
use embassy_stm32::rng::Rng;
use embassy_stm32::peripherals;
use embassy_stm32::bind_interrupts;

bind_interrupts!(struct Irqs {
    RNG => embassy_stm32::rng::InterruptHandler<peripherals::RNG>;
});

// Initialize the RNG peripheral
let mut rng = Rng::new(p.RNG, Irqs);
```

### Step 2: Register the RNG with the link layer

Before initializing the BLE stack, register your RNG instance with the link layer:

```rust
use embassy_stm32_wpan::wba::linklayer_plat::set_rng_instance;

// Register the RNG instance
// SAFETY: The RNG instance must remain valid for the entire lifetime of BLE stack usage
unsafe {
    set_rng_instance(&mut rng as *mut _ as *mut ());
}
```

### Step 3: Initialize the BLE stack

Now you can initialize the BLE stack as usual. The link layer will automatically use the hardware RNG:

```rust
use embassy_stm32_wpan::bindings::mac::{ST_MAC_callbacks_t, ST_MAC_init};

// Your MAC callbacks
static MAC_CALLBACKS: ST_MAC_callbacks_t = ST_MAC_callbacks_t {
    // ... your callbacks
};

// Initialize the MAC layer
let status = unsafe { ST_MAC_init(&MAC_CALLBACKS as *const _ as *mut _) };
```

## Complete Example

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rng::Rng;
use embassy_stm32::bind_interrupts;
use embassy_stm32_wpan::wba::linklayer_plat::set_rng_instance;
use embassy_stm32_wpan::bindings::mac::{ST_MAC_callbacks_t, ST_MAC_init};

bind_interrupts!(struct Irqs {
    RNG => embassy_stm32::rng::InterruptHandler<embassy_stm32::peripherals::RNG>;
});

static MAC_CALLBACKS: ST_MAC_callbacks_t = ST_MAC_callbacks_t {
    // Initialize all callbacks to None
    mlmeAssociateCnfCb: None,
    mlmeAssociateIndCb: None,
    // ... (other callbacks)
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Configure system clocks
    let mut config = Config::default();
    config.rcc.mux.rngsel = embassy_stm32::rcc::mux::Rngsel::HSI;
    // ... (other clock configuration)
    
    let p = embassy_stm32::init(config);
    
    // Initialize hardware RNG
    let mut rng = Rng::new(p.RNG, Irqs);
    
    // Register RNG with the link layer
    unsafe {
        set_rng_instance(&mut rng as *mut _ as *mut ());
    }
    
    // Initialize the BLE stack
    let status = unsafe { ST_MAC_init(&MAC_CALLBACKS as *const _ as *mut _) };
    
    // Your application logic here
    loop {
        // ...
    }
}
```

## Important Notes

1. **Lifetime**: The RNG instance must remain valid for the entire lifetime of the BLE stack usage. Make sure not to drop the RNG instance while the link layer might still need it.

2. **Safety**: The `set_rng_instance` function performs proper synchronization. However, the caller must ensure the pointer remains valid for the entire lifetime of BLE stack usage.

3. **Mandatory RNG**: Hardware RNG is required. If `set_rng_instance` is not called before the BLE stack requests random numbers, the application will panic with the message: "Hardware RNG not initialized! Call embassy_stm32_wpan::wba::set_rng_instance() before using the BLE stack."

4. **Clock Configuration**: Make sure to properly configure the RNG clock source in your RCC configuration. For STM32WBA, this is typically done via `config.rcc.mux.rngsel`.

## Clearing the RNG Instance

If you need to clear the hardware RNG instance (e.g., to fall back to software PRNG), you can use:

```rust
use embassy_stm32_wpan::wba::linklayer_plat::clear_rng_instance;

clear_rng_instance();
```
