# STM32N6 Flash Boot Example

Two-stage boot system (FSBL + application) for the STM32N6 using embassy-boot. Supports both the STM32N6570-DK and NUCLEO-N657X0-Q boards. The FSBL runs from SRAM, initializes external NOR flash in memory-mapped mode, and boots the application firmware with support for DFU updates.

## Prerequisites

- [cargo-binutils](https://github.com/rust-embedded/cargo-binutils): `cargo install cargo-binutils`
- [STM32CubeProgrammer](https://www.st.com/en/development-tools/stm32cubeprog.html) (provides the signing tool)
- [probe-rs](https://probe.rs/) **v0.32+** (not yet released — build from git master): `cargo install probe-rs-tools --git https://github.com/probe-rs/probe-rs`
- [just](https://github.com/casey/just): `cargo install just`

## Supported Boards

| Board | `board` value | External Flash | Size |
|-------|---------------|----------------|------|
| STM32N6570-DK | `dk` (default) | MX66UW1G45G | 128 MB |
| NUCLEO-N657X0-Q | `nucleo` | MX25UM51245G | 64 MB |

Set both BOOT switches to LOW for serial NOR boot (boot config 6, default when OTP is unfused).

## Project Structure

| Crate | Description |
|-------|-------------|
| `fsbl/` | First Stage Boot Loader — loaded by boot ROM into SRAM, enables debug access, configures clocks, initializes XSPI2 memory-mapped mode, then boots the app via embassy-boot |
| `app/` | Application firmware — runs from memory-mapped external flash at `0x70100400`, with DFU update support (hold PE0 at reset for UART DFU, hold PE0 3s to confirm boot via `mark_booted()`) |

## Quick Start

All recipes accept a `board` parameter (defaults to `dk`):

```console
# DK (default)
just flash-all

# Nucleo
just board=nucleo flash-all
```

This builds both crates, converts to raw binaries, signs them for the boot ROM, flashes everything to external flash, and erases the boot state partition. After flashing, set BOOT switches to LOW and reset the board.

Run `just --list` to see all available recipes.

## Memory Layout

```
External Flash (0x70000000, 128 MB):
  BOOTLOADER:  0x70000000 - 0x70100000 (1 MB)
  ACTIVE:      0x70100000 - 0x702FF000 (2044K)
  DFU:         0x70300000 - 0x70500000 (2048K)
  STATE:       0x70500000 - 0x70503000 (12K)

FSBL SRAM Layout:
  RAM:         0x34100000 - 0x34180400 (513K)
  FLASH:       0x34180400 - 0x341FF000 (507K)
```

Each signed image has a 0x400 header, so application code starts at `ACTIVE + 0x400 = 0x70100400`.

## Key Recipes

| Recipe | Description |
|--------|-------------|
| `just build` | Build both FSBL and app |
| `just flash-all` | Build, sign, flash everything, erase state |
| `just flash-fsbl` | Build, sign, flash FSBL only |
| `just flash-app` | Build, sign, flash app only |
| `just test-dfu` | Flash app to DFU partition and trigger swap |
| `just attach-fsbl` | Attach to running FSBL with defmt RTT output |
| `just debug-fsbl` | Flash FSBL and attach for debugging |
| `just erase-state` | Erase STATE partition (reset boot state) |
| `just trigger-swap` | Write SWAP_MAGIC to trigger DFU swap on next boot |
| `just uart-dfu <port>` | Send firmware via XMODEM over UART (requires lrzsz) |
| `just read-state` | Read STATE partition (debug) |
| `just mark-booted` | Write BOOT_MAGIC to confirm boot from host |
| `just clean` | Clean build artifacts |

All recipes work with the `board` parameter, e.g. `just board=nucleo flash-all`.

Recipes prefixed with `stm32-` (e.g. `just stm32-flash-all`) use STM32_Programmer_CLI instead of probe-rs, as a fallback if probe-rs cannot flash external flash.

## UART DFU

The application includes an XMODEM-CRC firmware receiver for cable-only DFU updates without a debug probe.

**Pins:** USART1 — PE5 (TX), PE6 (RX), 115200 baud, 8E1 (even parity)

**Workflow:**
1. Build and sign the new firmware: `just sign-app`
2. Start the XMODEM sender on the host **first**: `just uart-dfu /dev/tty.usbmodemXXXX`
3. Reset the board while holding PE0 (tamper button) — the app enters DFU mode
4. Firmware is received via XMODEM-CRC and written to the DFU partition
5. After transfer completes, SWAP_MAGIC is written automatically — reset to boot new firmware

Requires `lrzsz` (`sx` command): `brew install lrzsz` (macOS) or `apt install lrzsz` (Linux).

## Further Reading

For original boot flow notes and code snippets from the contributor who prototyped this approach, see [BOOT_NOTES.md](BOOT_NOTES.md).
