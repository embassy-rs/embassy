# ASR6601SE hardware verification

Register-level checks that write results to SRAM instead of requiring a UART.

## Run

```sh
cargo build --release
timeout 8s probe-rs run --chip ASR6601SE --protocol swd --non-interactive \
  target/thumbv7em-none-eabihf/release/embassy-asr6601-verify

addr=$(nm -n target/thumbv7em-none-eabihf/release/embassy-asr6601-verify \
  | awk '$3 == "VERIFY_RESULTS" { print $1 }')
probe-rs read --chip ASR6601SE --protocol swd --non-interactive b32 "0x$addr" 75
```

Result layout:

- word 0: magic `0x41535256` (`ASRV`)
- word 1: current test slot
- word 2: `0x444f4e45` (`DONE`) when finished
- words 3+: 4 words per test: `id`, `status`, `detail0`, `detail1`

Status values: `0` running, `1` pass, `2` fail, `3` skip.

## Tests

| ID | Test | Result on ASR6601SE |
|---:|---|---|
| 1 | RAM marker | pass |
| 2 | GPIO04 internal pull-up/down | pass |
| 3 | GPIO08 output latch | pass |
| 4 | GPTIM0 `OR` layout | pass |
| 5 | GPTIM1 `OR` layout | pass |
| 6 | GPTIM2 `OR` layout | pass |
| 7 | GPTIM3 `OR` layout | pass |
| 8 | GPTIM0 `SR` write-0-to-clear | pass |
| 9 | ADC single conversion | pass |
| 10 | ADC `ISR` write-1-to-clear | pass |
| 11 | ADC internal ref / VBAT field | pass |
| 12 | CRC byte feed stability | pass |
| 13 | DMA memory-to-memory | pass |
| 14 | DAC AFEC power/output fields | pass |
| 15 | RCC `RST_SR` write-1-to-clear | skip (`CPU_RESET_SR` not set after probe reset) |
| 16 | SYSCFG `CR5` user scratch | pass |
| 17 | WDG interrupt/clear | pass |
| 18 | IWDG `MAX`/`WIN` 12-bit write | pass |

The GPTIM `OR` layouts found on hardware match RM v1.5.0 but differ per
instance. The PAC now models each instance separately: non-`OR` registers
derive from GPTIM0, while `OR` has the instance-specific field layout.

## Additional loopback checks

- **GPIO04/GPIO08 loopback**: input level and rising-edge flag pass.
- **UART0 loopback** (`GPIO16` ↔ `GPIO17`): full 4-byte pattern passes in
  both TX/RX assignments.
- **SPI2 internal loopback** (PL022 `CR1.LBM`): byte pattern passes without
  external wiring.
- **I2C0 master ↔ I2C1 slave** (GPIO14/15 ↔ GPIO30/31): 2-byte write and read
  both pass.
- **RCC `RST_SR`**: a real power cycle reported `BOR_RESET_SR` (0x40); writing
  1 to the flag cleared it.
- **ADC ↔ DAC internal channel**: the DAC core output is visible on ADC
  sample channel 14 and tracks linearly.
- **ADC external GPIO05 input**: not working reliably on the current setup;
  the pin reads 0 or full scale instead of tracking the DAC. The ADC core,
  DAC, and channel 14 path are confirmed working.

At the end, the firmware leaves the DAC enabled at code `2048` on
**GPIO09** for an optional multimeter reading.

Measured on an ASR6601SE with VDD = 2.96 V:

| DAC code | Expected | Measured |
|---:|---:|---:|
| 0 | 0.000 V | 0.0088 V |
| 1024 | 0.740 V | 0.743 V |
| 2048 | 1.480 V | 1.480 V |
| 3072 | 2.221 V | 2.220 V |
| 4095 | 2.960 V | 2.960 V |

The converter is rail-to-rail with VDD as its reference; worst-case error in
this run was 8.8 mV at zero scale and 2.8 mV elsewhere.
