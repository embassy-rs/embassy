# Embassy MCXA276 HAL

A Hardware Abstraction Layer (HAL) for the NXP MCXA276 microcontroller using the Embassy async framework. This HAL provides safe, idiomatic Rust interfaces for GPIO, UART, and OSTIMER peripherals.

## Prerequisites

### Ubuntu/Debian Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add target for MCXA276 (ARM Cortex-M33)
rustup target add thumbv8m.main-none-eabihf

# Install required tools
sudo apt update
sudo apt install -y gdb-multiarch curl wget

# Install probe-rs for running and debugging
cargo install probe-rs --features cli
```

### Windows Setup

- Install Rust via https://rustup.rs (default options are fine)
- Add the MCXA276 target:
  ```powershell
  rustup target add thumbv8m.main-none-eabihf
  ```
- Install probe-rs CLI (we will use it directly; no GDB required):
  ```powershell
  cargo install probe-rs --features cli
  ```
- Install a serial terminal (e.g., Tera Term): https://ttssh2.osdn.jp/
- USB drivers: Windows 10/11 usually picks up the board as a USB CDC device automatically (COM port)



### Hardware Requirements

- NXP FRDM-MCXA276 development board
- Debug probe (CMSIS-DAP compatible)
- USB cable for power and programming

## Examples

This HAL includes several examples demonstrating different peripherals:

### GPIO Examples

#### `blink`
Blinks an LED connected to GPIO pin. Demonstrates basic GPIO output operations.

### UART Examples

#### `hello`
Interactive UART2 demo: prints a banner and supports `help`, `echo <text>`, `hex <byte>`.

### OSTIMER Examples

#### `ostimer_alarm`

Demonstrates setting and waiting for OSTIMER alarms.

#### `ostimer_async`
Shows asynchronous OSTIMER operations with Embassy's async runtime.

#### `ostimer_counter`
Demonstrates OSTIMER counter functionality.

#### `ostimer_race_test`
Advanced example testing OSTIMER race conditions and synchronization.

### RTC Example

#### `rtc_alarm`
Demonstrates how to enable and use the RTC to generate an interrupt after 10seconds.

## Build and Run

### Using probe-rs

All examples require specifying your debug probe. First, find your probe ID:

```bash
probe-rs list
```

Then run examples with your probe ID (replace `1fc9:0143:H3AYDQVQMTROB` with your actual probe):

```bash
# GPIO blink example
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "gpio ostimer0" --example blink



# UART hello example
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example hello

# OSTIMER examples
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_alarm
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_async
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_counter
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_race_test

# RTC example
PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 rtc0" --example rtc_alarm
```
**Note:** All examples run from RAM, not flash memory. They are loaded directly into RAM for faster development iteration.

**Important:** After pressing the RESET button on the board, the first `cargo run` attempt may fail with a connection error. This is expected - simply run the command again and it will work. The run.sh script now properly sets the Vector Table Offset Register (VTOR) to point to the RAM-based vector table, ensuring the correct stack pointer and reset vector are used.

smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$ PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --release --features "gpio ostimer0" --example blink
    Finished `release` profile [optimized + debuginfo] target(s) in 0.07s
     Running `/home/smw016108/Downloads/nxp/rust/uart/embassy-mcxa276/./run.sh target/thumbv8m.main-none-eabihf/release/examples/blink`
probe-rs gdb server failed to connect to target. Log:
----- probe-rs gdb log -----
  Error: Connecting to the chip was unsuccessful.

  Caused by:
      0: An ARM specific error occurred.
      1: Error using access port FullyQualifiedApAddress { dp: Default, ap: V1(0) }.
      2: Failed to read register DRW at address 0xd0c
      3: An error occurred in the communication with an access port or debug port.
      4: Target device responded with a FAULT response to the request.
smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$ PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --release --features "gpio ostimer0" --example blink
    Finished `release` profile [optimized + debuginfo] target(s) in 0.02s
     Running `/home/smw016108/Downloads/nxp/rust/uart/embassy-mcxa276/./run.sh target/thumbv8m.main-none-eabihf/release/examples/blink`

### Additional UART Examples

#### `uart_interrupt`
Interrupt-driven UART2 echo. Type in the serial terminal; each byte is echoed back from the IRQ handler path.

#### `lpuart_polling`
Blocking TX/RX echo over UART2 using the simple polling driver.

#### `lpuart_buffered`
Async buffered driver with separate TX/RX tasks; echoes typed characters in chunks.

Pins: UART2 TX=P2_2, RX=P2_3 (ALT3), 115200 8N1.

### ADC Examples

#### `adc_polling`
Configures ADC1 channel A8 (pin P1_10) and prints conversion values to UART2 periodically.

#### `adc_interrupt`
Triggers a conversion and signals completion via ADC1 interrupt, printing a notification on UART2.

0x20002040 in ?? ()
Supported Commands:

    info - print session information
    reset - reset target
    reset halt - reset target and halt afterwards

Loading section .vector_table, size 0x224 lma 0x20000000
Loading section .text, size 0x97e lma 0x20000224
Loading section .Reset, size 0x58 lma 0x20000ba4
Loading section .rodata, size 0x28 lma 0x20000bfc
Start address 0x20000ba4, load size 3106
Transfer rate: 13 KB/sec, 776 bytes/write.

then I see the LED blinking. I press CTRL+C to exit. It will show me ^C
Program received signal SIGINT, Interrupt.
0x20000880 in embassy_executor::arch::thread::Executor::run<blink::__cortex_m_rt_main::{closure_env#0}> (self=0x200027e8, init=...) at /home/smw016108/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/embassy-executor-0.9.1/src/arch/cortex_m.rs:106
106                         asm!("wfe");
[Inferior 1 (process 1) detached]
Program loaded and started (no reset)
smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$ \

Then I press RESET again and I want to run another example, like ostimer_alarm. I open the console using sudo picocom -b 115200 /dev/ttyACM0 and I start running the example:

smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$ PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_alarm
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.02s
     Running `/home/smw016108/Downloads/nxp/rust/uart/embassy-mcxa276/./run.sh target/thumbv8m.main-none-eabihf/debug/examples/ostimer_alarm`
probe-rs gdb server failed to connect to target. Log:
----- probe-rs gdb log -----
  Error: Connecting to the chip was unsuccessful.

  Caused by:
      0: An ARM specific error occurred.
      1: Error using access port FullyQualifiedApAddress { dp: Default, ap: V1(0) }.
      2: Failed to read register DRW at address 0xd0c
      3: An error occurred in the communication with an access port or debug port.
      4: Target device responded with a FAULT response to the request.
smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$ PROBE=1fc9:0143:H3AYDQVQMTROB cargo run --features "lpuart2 ostimer0" --example ostimer_alarm
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.02s
     Running `/home/smw016108/Downloads/nxp/rust/uart/embassy-mcxa276/./run.sh target/thumbv8m.main-none-eabihf/debug/examples/ostimer_alarm`
0x20002040 in core::panicking::panic_const::panic_const_mul_overflow () at library/core/src/panicking.rs:175
warning: 175    library/core/src/panicking.rs: No such file or directory
Supported Commands:

    info - print session information
    reset - reset target
    reset halt - reset target and halt afterwards

Loading section .vector_table, size 0x224 lma 0x20000000
Loading section .text, size 0x2226 lma 0x20000224
Loading section .Reset, size 0x58 lma 0x2000244c
Loading section .rodata, size 0x6dc lma 0x200024a4
Start address 0x2000244c, load size 11134
Transfer rate: 16 KB/sec, 1855 bytes/write.

I can see in the console
OSTIMER Alarm Example
Scheduling alarm for 2 seconds...
Alarm scheduled successfully
Alarm expired! Callback executed.
Scheduling another alarm for 3 seconds...
Alarm scheduled. Waiting 1 second then canceling...
Alarm canceled
Alarm was successfully canceled
Example complete

then I press CTRL+C to stop running

^C
Program received signal SIGINT, Interrupt.
0x20000e64 in embassy_executor::arch::thread::Executor::run<ostimer_alarm::__cortex_m_rt_main::{closure_env#0}> (self=0x200027e8, init=...) at /home/smw016108/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/embassy-executor-0.9.1/src/arch/cortex_m.rs:106
106                         asm!("wfe");
[Inferior 1 (process 1) detached]
Program loaded and started (no reset)
smw016108@smw016108:~/Downloads/nxp/rust/uart/embassy-mcxa276$


### Windows: Running examples (RAM, no RTT/defmt)

Important: On Windows, do not use `cargo run` because `.cargo/config.toml` sets a Linux-only runner (`./run.sh`). Instead, use `probe-rs run` directly.

1) Find your probe and COM port
- List probes:
  ```powershell
  probe-rs list
  ```
- If multiple probes are attached, set the specific one (replace with your ID):
  ```powershell
  $env:PROBE_RS_PROBE = "1366:0101:000600110607"
  ```
- Check Windows Device Manager → Ports (COM & LPT) for the board’s COM port.

2) Build the example
```powershell
cargo build --example hello --features "lpuart2"
```

3) Run from RAM with probe-rs
```powershell
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/hello
```
You will see a short probe-rs warning like "unknown variant, try to set watch point"; it’s harmless.

4) View output in Tera Term
- Open Tera Term, select the board’s COMx port, 115200 8N1
- Expected behavior per example:
  - hello: prints a banner; simple UART output
  - lpuart_polling / lpuart_buffered / uart_interrupt: echo typed characters
  - adc_polling: prints ADC values periodically (ADC1 channel A8 on P1_10)
  - adc_interrupt: prints "*** ADC interrupt TRIGGERED! ***" upon conversion completion
  - blink: LED on PIO3_18 blinks "SOS" pattern
  - rtc_alarm: schedules, cancels and reports alarm events on UART

Notes
- All examples run from RAM (not flashed). Reset clears the program.
- If the first attempt after a reset fails to connect, just run the command again.
- UART2 pins: TX=P2_2, RX=P2_3 (ALT3), 115200 8N1.

Quick commands for other examples (PowerShell)
```powershell
# Build
cargo build --example blink            --features "gpio ostimer0"
cargo build --example lpuart_polling   --features "lpuart2 ostimer0"
cargo build --example lpuart_buffered  --features "lpuart2 ostimer0"
cargo build --example uart_interrupt   --features "lpuart2 ostimer0"
cargo build --example rtc_alarm        --features "lpuart2 rtc0"
cargo build --example adc_polling      --features "adc1 lpuart2"
cargo build --example adc_interrupt    --features "adc1 lpuart2"

# Run (RAM)
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/blink
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/lpuart_polling
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/lpuart_buffered
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/uart_interrupt
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/rtc_alarm
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/adc_polling
probe-rs run --chip MCXA276 --protocol swd --speed 1000 target/thumbv8m.main-none-eabihf/debug/examples/adc_interrupt
```

How I tested on Windows
- Windows 11; Rust stable; probe-rs 0.29.x
- Built each example as above; ran with `probe-rs run` (RAM execution)
- Observed UART output in Tera Term at 115200 8N1; all examples behaved as expected
- No RTT/defmt used; purely UART or LED observation

### Build Only

To build without running:

```bash
cargo build --features "gpio ostimer0" --example blink
cargo build --features "lpuart2 ostimer0" --example hello
cargo build --features "lpuart2 ostimer0" --example ostimer_alarm
cargo build --features "lpuart2 rtc0" --example rtc_alarm
# etc.
```


## Development Notes

### Critical Fix: MCXA276 Interrupt Vector Table

**Problem:** The OSTIMER examples crashed during interrupt handling with a hardfault (SP=0x00000000). Investigation revealed the OS_EVENT interrupt vector was NULL in the vector table, causing the CPU to jump to address 0 when OSTIMER interrupts fired.

**Root Cause:** The `mcxa276-pac/src/lib.rs` file (generated from the SVD file) was missing the `WAKETIMER0` interrupt handler declaration. This caused the `__INTERRUPTS` array to have an off-by-one error, placing OS_EVENT at IRQ 58 instead of the correct IRQ 57 position.

**Solution:** Manually edited `mcxa276-pac/src/lib.rs` to add the missing WAKETIMER0 interrupt:

1. Added `fn WAKETIMER0()` to the `extern "C"` block
2. Fixed the `__INTERRUPTS: [Vector; 122]` array sequence:
   - Changed from: `LPTMR0, _reserved, _reserved, OS_EVENT, _reserved, UTICK0, ...`
   - Changed to: `LPTMR0, _reserved, OS_EVENT, WAKETIMER0, UTICK0, WWDT0, _reserved, ADC0, ...`
3. Added `WAKETIMER0 = 58` to the `Interrupt` enum

**Verification:** Binary analysis confirmed OS_EVENT is now at the correct position:
- IRQ 57 = word 73 = offset 0x124 in vector table
- OS_EVENT handler: 0x20000BB1 (verified with `arm-none-eabi-objdump`)

**Note:** This is likely an issue with the NXP MCXA276.svd file or svd2rust generation. The WAKETIMER0 peripheral exists in the PAC but the interrupt handler was missing. Future regeneration of the PAC from SVD may require reapplying this fix.

### Warning: Avoid `#[inline(always)]` in Performance-Critical Code

Using `#[inline(always)]` can cause the Rust compiler to generate incorrect assembly, leading to register corruption or unexpected behavior. For example, in tight polling loops like those in the OSTIMER driver, this attribute may result in invalid instructions that zero registers (e.g., `movs r1, r0` causing r1=0), triggering hardfaults.


## License

This project is licensed under MIT OR Apache-2.0.
