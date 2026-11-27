# Examples for EFM32GG

The examples target the Bosch XDK110 (EFM32GG390F1024). They use the extension bus, because the pin
mapping of the on-board LEDs and buttons is not public.

| Example | Pins |
|---|---|
| `blinky` | LED on A12 (PA1) |
| `button` | button on A13 (PE2) to GND, LED on A12 (PA1) |
| `usart_blocking`, `usart_async` | UART1 TX on B1 (PB9), RX on B2 (PB10), 115200 8N1 |

## Running with a debug probe

```
cargo run --release --bin blinky
```

This overwrites the XDK110 USB bootloader in the first 64 KiB of flash.

## Running from the USB bootloader

The bootloader expects the application at `0x00010000`. Build with the `xdk110-bootloader` feature
and convert the ELF to a binary:

```
cargo build --release --no-default-features --features xdk110-bootloader --bin blinky
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/blinky blinky.bin
```

To enter the bootloader, hold button 1 while you switch the XDK110 on. Then send `u` on its USB
serial port and upload `blinky.bin` with XMODEM-CRC, for example with `sx` from lrzsz. Send `b` to
start the application. defmt output needs a debug probe.
