# ASR6601 examples

Examples for the `embassy-asr` HAL.

- `blinky` — toggles PA0 at 1 Hz and exercises the LPTIM0 time driver.
- `uart` — echoes bytes on UART0 (PB0 TX, PB1 RX, AF1).

Build from this directory:

```sh
cargo build --release
```

The runner uses `probe-rs run --chip ASR6601SE`; use `ASR6601CB` (128 KiB
flash / 16 KiB RAM) or the matching part name for other variants. The
`memory.x` file currently targets the ASR6601SE (256 KiB flash / 64 KiB RAM).
